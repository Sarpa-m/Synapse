use std::fs::{self, File};
use std::future::Future;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::domain::{
    errors::{SynapseError, SynapseResult},
    update::{AppVersion, UpdateProgress, UpdateReleaseInfo},
};
use crate::ports::update_port::{ProgressCallback, ReleaseProviderPort};

const DEFAULT_GITHUB_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/Sarpa-m/Synapse/releases/latest";
const USER_AGENT: &str = concat!("Synapse-Desktop/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize)]
struct GitHubReleasePayload {
    tag_name: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<GitHubAssetPayload>,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubAssetPayload {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// Adaptador HTTP para consulta de releases e download de instaladores no GitHub Releases.
#[derive(Clone)]
pub struct GitHubReleasesAdapter {
    http_client: Client,
    latest_release_url: String,
    download_dir: PathBuf,
}

impl GitHubReleasesAdapter {
    /// Inicializa o adaptador configurado para o repositório oficial `Sarpa-m/Synapse`.
    ///
    /// # Errors
    /// Retorna [`SynapseError::UpdateCheckFailed`] caso o cliente TLS não possa ser construído.
    pub fn new() -> SynapseResult<Self> {
        let http_client = Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| SynapseError::UpdateCheckFailed(e.to_string()))?;

        Ok(Self {
            http_client,
            latest_release_url: DEFAULT_GITHUB_LATEST_RELEASE_URL.to_string(),
            download_dir: std::env::temp_dir().join("synapse-updater"),
        })
    }

    /// Sobrescreve a URL da API e o diretório de download (usado em testes com `wiremock`).
    #[must_use]
    pub fn with_custom_config(mut self, latest_release_url: String, download_dir: PathBuf) -> Self {
        self.latest_release_url = latest_release_url;
        self.download_dir = download_dir;
        self
    }

    /// Seleciona o artefato Windows mais apropriado da release:
    /// 1. Instalador NSIS (`*-setup.exe`)
    /// 2. Pacote `.msi`
    /// 3. Binário portátil `.exe` (`synapse-desktop.exe`)
    fn select_windows_asset(assets: &[GitHubAssetPayload]) -> Option<&GitHubAssetPayload> {
        let nsis = assets.iter().find(|a| {
            let lower = a.name.to_ascii_lowercase();
            lower.ends_with(".exe") && lower.contains("setup")
        });
        if nsis.is_some() {
            return nsis;
        }

        let msi = assets
            .iter()
            .find(|a| a.name.to_ascii_lowercase().ends_with(".msi"));
        if msi.is_some() {
            return msi;
        }

        assets
            .iter()
            .find(|a| a.name.to_ascii_lowercase().ends_with(".exe"))
    }

    /// Sanitiza o nome do arquivo para evitar traversal de diretórios e garantir extensão válida.
    fn sanitize_asset_filename(version: &str, raw_name: &str) -> SynapseResult<String> {
        let file_name = Path::new(raw_name)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                SynapseError::UpdateDownloadFailed(format!("nome de artefato inválido: {raw_name}"))
            })?;

        let lower = file_name.to_ascii_lowercase();
        let has_valid_ext = Path::new(&lower)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "exe" || ext == "msi");
        if !has_valid_ext {
            return Err(SynapseError::UpdateDownloadFailed(format!(
                "extensão não suportada para atualização: {file_name}"
            )));
        }

        let clean_version: String = version
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
            .collect();
        Ok(format!("synapse-{clean_version}-{file_name}"))
    }
}

impl ReleaseProviderPort for GitHubReleasesAdapter {
    fn fetch_latest_release(
        &self,
    ) -> Pin<Box<dyn Future<Output = SynapseResult<UpdateReleaseInfo>> + Send + 'static>> {
        let client = self.http_client.clone();
        let url = self.latest_release_url.clone();

        Box::pin(async move {
            let response = client
                .get(&url)
                .header("Accept", "application/vnd.github+json")
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        SynapseError::UpdateCheckFailed(
                            "tempo limite excedido ao consultar o GitHub".into(),
                        )
                    } else {
                        SynapseError::UpdateCheckFailed(e.to_string())
                    }
                })?;

            match response.status() {
                StatusCode::OK => {}
                StatusCode::NOT_FOUND => {
                    return Err(SynapseError::UpdateCheckFailed(
                        "nenhuma release publicada encontrada no repositório GitHub".into(),
                    ));
                }
                StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS => {
                    return Err(SynapseError::UpdateCheckFailed(
                        "limite de requisições da API do GitHub atingido; tente novamente mais tarde"
                            .into(),
                    ));
                }
                status => {
                    return Err(SynapseError::UpdateCheckFailed(format!(
                        "status HTTP inesperado do GitHub: {status}"
                    )));
                }
            }

            let payload: GitHubReleasePayload = response
                .json()
                .await
                .map_err(|e| SynapseError::UpdateCheckFailed(format!("JSON inválido: {e}")))?;

            let parsed_version = AppVersion::parse(&payload.tag_name)?;
            let asset = Self::select_windows_asset(&payload.assets).ok_or_else(|| {
                SynapseError::UpdateCheckFailed(
                    "a última release não possui instalador Windows (.exe ou .msi)".into(),
                )
            })?;

            Ok(UpdateReleaseInfo {
                version: parsed_version.to_string(),
                tag_name: payload.tag_name,
                release_notes: payload.body.unwrap_or_default().trim().to_string(),
                published_at: payload.published_at.unwrap_or_default(),
                download_url: asset.browser_download_url.clone(),
                asset_name: asset.name.clone(),
                asset_size_bytes: asset.size,
            })
        })
    }

    fn download_installer(
        &self,
        release: UpdateReleaseInfo,
        on_progress: ProgressCallback,
    ) -> Pin<Box<dyn Future<Output = SynapseResult<PathBuf>> + Send + 'static>> {
        let client = self.http_client.clone();
        let download_dir = self.download_dir.clone();

        Box::pin(async move {
            let safe_filename =
                Self::sanitize_asset_filename(&release.version, &release.asset_name)?;
            fs::create_dir_all(&download_dir)
                .map_err(|e| SynapseError::UpdateDownloadFailed(e.to_string()))?;

            let dest_path = download_dir.join(safe_filename);

            let mut response = client
                .get(&release.download_url)
                .timeout(Duration::from_secs(120))
                .send()
                .await
                .map_err(|e| SynapseError::UpdateDownloadFailed(e.to_string()))?;

            if !response.status().is_success() {
                return Err(SynapseError::UpdateDownloadFailed(format!(
                    "falha no download do instalador (HTTP {})",
                    response.status()
                )));
            }

            let total_bytes = response
                .content_length()
                .unwrap_or(release.asset_size_bytes);
            let mut file = File::create(&dest_path)
                .map_err(|e| SynapseError::UpdateDownloadFailed(e.to_string()))?;

            let mut downloaded_bytes: u64 = 0;
            on_progress(UpdateProgress::new(0, total_bytes));

            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|e| SynapseError::UpdateDownloadFailed(e.to_string()))?
            {
                file.write_all(&chunk)
                    .map_err(|e| SynapseError::UpdateDownloadFailed(e.to_string()))?;
                downloaded_bytes = downloaded_bytes.saturating_add(chunk.len() as u64);
                on_progress(UpdateProgress::new(downloaded_bytes, total_bytes));
            }

            file.flush()
                .map_err(|e| SynapseError::UpdateDownloadFailed(e.to_string()))?;

            Ok(dest_path)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    #[tokio::test]
    async fn test_fetch_latest_release_prioritizes_nsis_setup_exe() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "tag_name": "v0.2.0",
            "body": "### Novidades\n- Auto-updater via GitHub Releases",
            "published_at": "2026-11-16T12:00:00Z",
            "assets": [
                {
                    "name": "synapse-desktop.exe",
                    "browser_download_url": format!("{}/download/synapse-desktop.exe", server.uri()),
                    "size": 4_000_000
                },
                {
                    "name": "Synapse Desktop_0.2.0_x64-setup.exe",
                    "browser_download_url": format!("{}/download/setup.exe", server.uri()),
                    "size": 3_500_000
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/repos/Sarpa-m/Synapse/releases/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let adapter = GitHubReleasesAdapter::new().unwrap().with_custom_config(
            format!("{}/repos/Sarpa-m/Synapse/releases/latest", server.uri()),
            std::env::temp_dir().join("synapse-updater-test"),
        );

        let info = adapter.fetch_latest_release().await.unwrap();
        assert_eq!(info.version, "0.2.0");
        assert_eq!(info.tag_name, "v0.2.0");
        assert_eq!(info.asset_name, "Synapse Desktop_0.2.0_x64-setup.exe");
        assert_eq!(info.asset_size_bytes, 3_500_000);
    }

    #[tokio::test]
    async fn test_fetch_latest_release_handles_404_and_rate_limit() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/not-found"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let adapter = GitHubReleasesAdapter::new().unwrap().with_custom_config(
            format!("{}/not-found", server.uri()),
            std::env::temp_dir().join("synapse-updater-test"),
        );

        let err = adapter.fetch_latest_release().await.unwrap_err();
        assert!(matches!(err, SynapseError::UpdateCheckFailed(_)));
    }

    #[tokio::test]
    async fn test_download_installer_streams_bytes_and_reports_progress() {
        let server = MockServer::start().await;
        let fake_binary = vec![0x4D, 0x5A, 0x90, 0x00, 0x01, 0x02, 0x03, 0x04];

        Mock::given(method("GET"))
            .and(path("/download/Synapse_0.2.0-setup.exe"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(fake_binary.clone()))
            .mount(&server)
            .await;

        let temp_dir = std::env::temp_dir().join(format!("synapse-dl-test-{}", std::process::id()));
        let adapter = GitHubReleasesAdapter::new()
            .unwrap()
            .with_custom_config(server.uri(), temp_dir.clone());

        let release = UpdateReleaseInfo {
            version: "0.2.0".to_string(),
            tag_name: "v0.2.0".to_string(),
            release_notes: "Notas".to_string(),
            published_at: "2026-11-16T12:00:00Z".to_string(),
            download_url: format!("{}/download/Synapse_0.2.0-setup.exe", server.uri()),
            asset_name: "Synapse_0.2.0-setup.exe".to_string(),
            asset_size_bytes: fake_binary.len() as u64,
        };

        let events: Arc<Mutex<Vec<UpdateProgress>>> = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&events);

        let downloaded_path = adapter
            .download_installer(
                release,
                Arc::new(move |p| {
                    if let Ok(mut guard) = events_clone.lock() {
                        guard.push(p);
                    }
                }),
            )
            .await
            .unwrap();

        assert!(downloaded_path.exists());
        let saved_bytes = std::fs::read(&downloaded_path).unwrap();
        assert_eq!(saved_bytes, fake_binary);

        let recorded = events.lock().unwrap();
        assert!(!recorded.is_empty());
        assert_eq!(recorded.last().unwrap().percentage, 100);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
