use serde::{Deserialize, Serialize};

use crate::commands::AppState;
use crate::domain::errors::{SynapseError, SynapseResult};
use crate::domain::profile::Profile;
use crate::services::capture_service::CaptureSession;
use crate::services::execution_service::ExecutionService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PopupMode {
    Selection,
    Compose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupSessionView {
    pub selected_text: String,
    pub char_count: usize,
    pub mode: PopupMode,
}

impl From<&CaptureSession> for PopupSessionView {
    fn from(session: &CaptureSession) -> Self {
        Self {
            char_count: session.selected_text.chars().count(),
            selected_text: session.selected_text.clone(),
            mode: PopupMode::Selection,
        }
    }
}

/// Abre o Popup diretamente em modo Composição (sem capturar seleção da janela ativa).
pub fn trigger_compose_popup(state: &AppState) -> SynapseResult<PopupSessionView> {
    state.set_active_session(None)?;
    Ok(PopupSessionView {
        selected_text: String::new(),
        char_count: 0,
        mode: PopupMode::Compose,
    })
}

/// Aciona a captura segura na janela ativa e armazena a sessão atual para uso pelo Popup.
pub async fn trigger_popup_capture(state: &AppState) -> SynapseResult<PopupSessionView> {
    let session = state.capture_service.capture_selection().await?;
    let view = PopupSessionView::from(&session);
    state.set_active_session(Some(session))?;
    Ok(view)
}

/// Retorna a sessão atualmente carregada no Popup Contextual.
pub fn get_current_popup_session(state: &AppState) -> SynapseResult<Option<PopupSessionView>> {
    Ok(state
        .get_active_session()?
        .as_ref()
        .map(PopupSessionView::from))
}

/// Fluxo completo: LLM -> esconde popup (callback da camada Tauri) -> injeção -> histórico.
/// Em caso de erro na LLM, o popup permanece aberto e a sessão é mantida para "tentar novamente" (RN-04).
async fn run_profile(
    state: &AppState,
    profile: &Profile,
    hide_popup: &(dyn Fn() + Send + Sync),
) -> SynapseResult<String> {
    let session = state
        .get_active_session()?
        .ok_or(SynapseError::EmptySelection)?;
    let config = state.config_repo.get_config()?;

    let text = state
        .execution_service
        .generate(&session, profile, &config)
        .await?;

    hide_popup();
    state.set_active_session(None)?;

    state
        .execution_service
        .finalize(
            &session,
            &profile.name,
            &text,
            config.selection_delivery_mode,
        )
        .await?;
    Ok(text)
}

/// Executa um perfil selecionado pelo usuário sobre o texto capturado.
pub async fn execute_profile_cmd(
    state: &AppState,
    profile_id: &str,
    hide_popup: &(dyn Fn() + Send + Sync),
) -> SynapseResult<String> {
    let profile = state.profile_service.get_profile(profile_id)?;
    run_profile(state, &profile, hide_popup).await
}

/// Executa uma instrução avulsa digitada pelo usuário no Popup Contextual.
pub async fn execute_custom_prompt_cmd(
    state: &AppState,
    instruction: &str,
    hide_popup: &(dyn Fn() + Send + Sync),
) -> SynapseResult<String> {
    let profile = ExecutionService::build_custom_profile(instruction)?;
    run_profile(state, &profile, hide_popup).await
}

/// Modo Composição: envia o texto digitado no popup para o perfil escolhido
/// e copia a saída diretamente para o clipboard (`Ctrl+V`).
pub async fn execute_compose_profile_cmd(
    state: &AppState,
    profile_id: &str,
    input_text: &str,
    hide_popup: &(dyn Fn() + Send + Sync),
) -> SynapseResult<String> {
    let profile = state.profile_service.get_profile(profile_id)?;
    let config = state.config_repo.get_config()?;
    let text = state
        .execution_service
        .execute_to_clipboard(input_text, &profile, &config)
        .await?;
    hide_popup();
    state.set_active_session(None)?;
    Ok(text)
}

/// Cancela a sessão atual do Popup Contextual (`Esc` ou `onBlur`).
/// O clipboard do usuário já foi restaurado na captura, então nada mais precisa ser feito.
pub fn cancel_popup_session_cmd(state: &AppState) -> SynapseResult<()> {
    state.set_active_session(None)
}
