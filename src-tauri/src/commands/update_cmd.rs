use crate::commands::AppState;
use crate::domain::{
    errors::SynapseResult,
    update::{UpdateCheckResult, UpdateReleaseInfo},
};
use crate::ports::update_port::ProgressCallback;

/// Retorna a versão atual compilada da aplicação (`CARGO_PKG_VERSION`).
#[must_use]
pub fn get_app_version_cmd(state: &AppState) -> String {
    state.update_service.current_version().to_string()
}

/// Retorna a atualização pendente detectada na sessão atual (se houver).
///
/// # Errors
/// Retorna erro caso o mutex de estado esteja corrompido.
pub fn get_pending_update_cmd(state: &AppState) -> SynapseResult<Option<UpdateReleaseInfo>> {
    state.get_pending_update()
}

/// Verifica no GitHub Releases se existe uma versão mais recente.
/// Se `manual` for `true`, ignora o filtro `skipped_update_version`.
///
/// # Errors
/// Retorna erro caso a consulta ao GitHub falhe.
pub async fn check_for_updates_cmd(
    state: &AppState,
    manual: bool,
) -> SynapseResult<UpdateCheckResult> {
    let result = state.update_service.check_for_updates(!manual).await?;
    if let UpdateCheckResult::UpdateAvailable { ref release, .. } = result {
        state.set_pending_update(Some(release.clone()))?;
    } else if matches!(result, UpdateCheckResult::UpToDate { .. }) {
        state.set_pending_update(None)?;
    }
    Ok(result)
}

/// Baixa o instalador da release autorizada pelo usuário e inicia sua instalação.
///
/// # Errors
/// Retorna erro caso o download ou a inicialização do instalador falhem.
pub async fn install_update_cmd(
    state: &AppState,
    release: UpdateReleaseInfo,
    on_progress: ProgressCallback,
) -> SynapseResult<()> {
    state
        .update_service
        .download_and_launch_update(release, on_progress)
        .await
}

/// Marca uma versão específica como ignorada nas checagens automáticas de inicialização.
///
/// # Errors
/// Retorna erro caso a versão seja inválida ou falhe ao salvar no banco SQLite.
pub fn skip_update_version_cmd(state: &AppState, version: &str) -> SynapseResult<()> {
    state.update_service.skip_version(version)?;
    state.set_pending_update(None)?;
    Ok(())
}

/// Descarta o aviso de atualização atual ("Mais tarde") sem ignorar permanentemente a versão.
///
/// # Errors
/// Retorna erro caso o lock de estado falhe.
pub fn dismiss_update_prompt_cmd(state: &AppState) -> SynapseResult<()> {
    state.set_pending_update(None)
}
