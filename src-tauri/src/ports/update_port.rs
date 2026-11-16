use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use crate::domain::{
    errors::SynapseResult,
    update::{UpdateProgress, UpdateReleaseInfo},
};

/// Callback thread-safe invocado durante o download do instalador.
pub type ProgressCallback = Arc<dyn Fn(UpdateProgress) + Send + Sync + 'static>;

/// Porta para consulta de releases no GitHub e download seguro do artefato de atualização.
#[cfg_attr(test, mockall::automock)]
pub trait ReleaseProviderPort: Send + Sync {
    /// Consulta a última release publicada no repositório remoto.
    fn fetch_latest_release(
        &self,
    ) -> Pin<Box<dyn Future<Output = SynapseResult<UpdateReleaseInfo>> + Send + 'static>>;

    /// Baixa o instalador da release para um diretório temporário reportando progresso.
    fn download_installer(
        &self,
        release: UpdateReleaseInfo,
        on_progress: ProgressCallback,
    ) -> Pin<Box<dyn Future<Output = SynapseResult<PathBuf>> + Send + 'static>>;
}

/// Porta para execução do instalador baixado no sistema operacional.
#[cfg_attr(test, mockall::automock)]
pub trait UpdateInstallerPort: Send + Sync {
    /// Inicia o instalador (NSIS/MSI ou script de substituição portátil) de forma desacoplada.
    ///
    /// # Errors
    /// Retorna [`crate::domain::errors::SynapseError::UpdateInstallFailed`] caso o processo não possa ser iniciado.
    fn launch_installer(&self, installer_path: &Path) -> SynapseResult<()>;
}
