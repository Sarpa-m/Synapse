use std::sync::Arc;

use crate::domain::{
    errors::SynapseResult,
    update::{AppVersion, UpdateCheckResult, UpdateReleaseInfo},
};
use crate::ports::{
    storage_ports::ConfigRepoPort,
    update_port::{ProgressCallback, ReleaseProviderPort, UpdateInstallerPort},
};

/// Serviço de orquestração do fluxo de atualização automática (GitHub Releases -> Permissão -> Instalação).
#[derive(Clone)]
pub struct UpdateService {
    release_provider: Arc<dyn ReleaseProviderPort>,
    installer: Arc<dyn UpdateInstallerPort>,
    config_repo: Arc<dyn ConfigRepoPort>,
    current_version: String,
}

impl UpdateService {
    #[must_use]
    pub fn new(
        release_provider: Arc<dyn ReleaseProviderPort>,
        installer: Arc<dyn UpdateInstallerPort>,
        config_repo: Arc<dyn ConfigRepoPort>,
    ) -> Self {
        Self {
            release_provider,
            installer,
            config_repo,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    #[must_use]
    pub fn with_current_version(mut self, version: impl Into<String>) -> Self {
        self.current_version = version.into();
        self
    }

    #[must_use]
    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    /// Verifica no GitHub se há uma versão superior à atual.
    /// Quando `respect_skipped` é `true` (checagem automática no boot), versões que o usuário
    /// marcou como "Ignorar esta versão" retornam [`UpdateCheckResult::Skipped`].
    ///
    /// # Errors
    /// Retorna erro caso a versão seja inválida ou a consulta ao GitHub falhe.
    pub async fn check_for_updates(
        &self,
        respect_skipped: bool,
    ) -> SynapseResult<UpdateCheckResult> {
        let current = AppVersion::parse(&self.current_version)?;
        let release = self.release_provider.fetch_latest_release().await?;
        let remote = AppVersion::parse(&release.version)?;

        if remote <= current {
            return Ok(UpdateCheckResult::UpToDate {
                current_version: current.to_string(),
            });
        }

        if respect_skipped {
            let cfg = self.config_repo.get_config()?;
            if let Some(skipped_raw) = cfg.skipped_update_version {
                if let Ok(skipped_ver) = AppVersion::parse(&skipped_raw) {
                    if skipped_ver == remote {
                        return Ok(UpdateCheckResult::Skipped {
                            current_version: current.to_string(),
                            skipped_version: remote.to_string(),
                        });
                    }
                }
            }
        }

        Ok(UpdateCheckResult::UpdateAvailable {
            current_version: current.to_string(),
            release: UpdateReleaseInfo {
                version: remote.to_string(),
                ..release
            },
        })
    }

    /// Executa o download do instalador autorizado pelo usuário e inicia sua execução.
    ///
    /// # Errors
    /// Retorna erro caso o download falhe ou o instalador não possa ser iniciado.
    pub async fn download_and_launch_update(
        &self,
        release: UpdateReleaseInfo,
        on_progress: ProgressCallback,
    ) -> SynapseResult<()> {
        let installer_path = self
            .release_provider
            .download_installer(release, on_progress)
            .await?;
        self.installer.launch_installer(&installer_path)?;
        Ok(())
    }

    /// Registra que o usuário optou por ignorar notificações automáticas para uma versão específica.
    ///
    /// # Errors
    /// Retorna erro caso o formato da versão seja inválido ou o banco SQLite falhe.
    pub fn skip_version(&self, version: &str) -> SynapseResult<()> {
        let parsed = AppVersion::parse(version)?;
        let mut cfg = self.config_repo.get_config()?;
        cfg.skipped_update_version = Some(parsed.to_string());
        self.config_repo.save_config(&cfg)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::domain::{config::AppConfig, errors::SynapseError};
    use crate::ports::{
        storage_ports::MockConfigRepoPort,
        update_port::{MockReleaseProviderPort, MockUpdateInstallerPort},
    };

    use super::*;

    fn sample_release(version: &str) -> UpdateReleaseInfo {
        UpdateReleaseInfo {
            version: version.to_string(),
            tag_name: format!("v{version}"),
            release_notes: "Correções e melhorias".to_string(),
            published_at: "2026-11-16T12:00:00Z".to_string(),
            download_url: "https://example.com/Synapse-setup.exe".to_string(),
            asset_name: "Synapse-setup.exe".to_string(),
            asset_size_bytes: 1024,
        }
    }

    #[tokio::test]
    async fn test_check_detects_newer_version_available() {
        let mut provider = MockReleaseProviderPort::new();
        provider
            .expect_fetch_latest_release()
            .times(1)
            .returning(|| Box::pin(async { Ok(sample_release("0.2.0")) }));

        let installer = MockUpdateInstallerPort::new();
        let mut cfg_repo = MockConfigRepoPort::new();
        cfg_repo
            .expect_get_config()
            .times(1)
            .returning(|| Ok(AppConfig::default()));

        let svc = UpdateService::new(Arc::new(provider), Arc::new(installer), Arc::new(cfg_repo))
            .with_current_version("0.1.0");

        let res = svc.check_for_updates(true).await.unwrap();
        assert_eq!(
            res,
            UpdateCheckResult::UpdateAvailable {
                current_version: "0.1.0".to_string(),
                release: sample_release("0.2.0"),
            }
        );
    }

    #[tokio::test]
    async fn test_check_returns_up_to_date_when_remote_equal_or_older() {
        let mut provider = MockReleaseProviderPort::new();
        provider
            .expect_fetch_latest_release()
            .times(1)
            .returning(|| Box::pin(async { Ok(sample_release("0.1.0")) }));

        let installer = MockUpdateInstallerPort::new();
        let cfg_repo = MockConfigRepoPort::new();

        let svc = UpdateService::new(Arc::new(provider), Arc::new(installer), Arc::new(cfg_repo))
            .with_current_version("0.1.0");

        let res = svc.check_for_updates(true).await.unwrap();
        assert_eq!(
            res,
            UpdateCheckResult::UpToDate {
                current_version: "0.1.0".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_check_respects_skipped_version_on_boot_and_shows_on_manual() {
        let mut provider = MockReleaseProviderPort::new();
        provider
            .expect_fetch_latest_release()
            .times(2)
            .returning(|| Box::pin(async { Ok(sample_release("0.2.0")) }));

        let installer = MockUpdateInstallerPort::new();
        let mut cfg_repo = MockConfigRepoPort::new();
        cfg_repo.expect_get_config().times(1).returning(|| {
            Ok(AppConfig {
                skipped_update_version: Some("0.2.0".to_string()),
                ..AppConfig::default()
            })
        });

        let svc = UpdateService::new(Arc::new(provider), Arc::new(installer), Arc::new(cfg_repo))
            .with_current_version("0.1.0");

        let boot_res = svc.check_for_updates(true).await.unwrap();
        assert_eq!(
            boot_res,
            UpdateCheckResult::Skipped {
                current_version: "0.1.0".to_string(),
                skipped_version: "0.2.0".to_string(),
            }
        );

        let manual_res = svc.check_for_updates(false).await.unwrap();
        assert!(matches!(
            manual_res,
            UpdateCheckResult::UpdateAvailable { .. }
        ));
    }

    #[tokio::test]
    async fn test_download_and_launch_aborts_when_download_fails() {
        let mut provider = MockReleaseProviderPort::new();
        provider
            .expect_download_installer()
            .times(1)
            .returning(|_, _| {
                Box::pin(async { Err(SynapseError::UpdateDownloadFailed("erro de rede".into())) })
            });

        let mut installer = MockUpdateInstallerPort::new();
        installer.expect_launch_installer().times(0);

        let cfg_repo = MockConfigRepoPort::new();
        let svc = UpdateService::new(Arc::new(provider), Arc::new(installer), Arc::new(cfg_repo));

        let err = svc
            .download_and_launch_update(sample_release("0.2.0"), Arc::new(|_| {}))
            .await
            .unwrap_err();
        assert!(matches!(err, SynapseError::UpdateDownloadFailed(_)));
    }

    #[tokio::test]
    async fn test_download_and_launch_invokes_installer_on_success() {
        let expected_path = PathBuf::from("C:/temp/Synapse-setup.exe");
        let path_clone = expected_path.clone();

        let mut provider = MockReleaseProviderPort::new();
        provider
            .expect_download_installer()
            .times(1)
            .returning(move |_, _| {
                let p = path_clone.clone();
                Box::pin(async move { Ok(p) })
            });

        let mut installer = MockUpdateInstallerPort::new();
        installer
            .expect_launch_installer()
            .withf(move |p| p == expected_path.as_path())
            .times(1)
            .returning(|_| Ok(()));

        let cfg_repo = MockConfigRepoPort::new();
        let svc = UpdateService::new(Arc::new(provider), Arc::new(installer), Arc::new(cfg_repo));

        svc.download_and_launch_update(sample_release("0.2.0"), Arc::new(|_| {}))
            .await
            .unwrap();
    }
}
