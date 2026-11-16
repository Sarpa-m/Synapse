use crate::commands::AppState;
use crate::domain::{
    config::{AppConfig, AppConfigView},
    errors::SynapseResult,
};

/// Retorna a configuração para a UI garantindo que a API Key NUNCA seja exposta (RN-01).
pub fn get_settings_cmd(state: &AppState) -> SynapseResult<AppConfigView> {
    let config = state.config_repo.get_config()?;
    let has_api_key = match state.secret_store.has_api_key() {
        Ok(val) => val,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "falha ao consultar cofre de credenciais do Windows; assumindo has_api_key = false"
            );
            false
        }
    };
    Ok(AppConfigView::from_config(config, has_api_key))
}

/// Atualiza configurações gerais (e executa expurgo caso o limite de retenção tenha mudado).
pub fn save_settings_cmd(state: &AppState, config: AppConfig) -> SynapseResult<AppConfigView> {
    state.config_repo.save_config(&config)?;
    state
        .history_service
        .run_retention_cleanup(config.retention_days)?;
    let has_api_key = state.secret_store.has_api_key().unwrap_or(false);
    Ok(AppConfigView::from_config(config, has_api_key))
}

/// Salva a chave de API diretamente no cofre nativo (Windows Credential Manager via `keyring`).
pub fn save_api_key_cmd(state: &AppState, api_key: &str) -> SynapseResult<AppConfigView> {
    state.secret_store.set_api_key(api_key)?;
    get_settings_cmd(state)
}

/// Remove a chave de API do cofre nativo.
pub fn clear_api_key_cmd(state: &AppState) -> SynapseResult<AppConfigView> {
    state.secret_store.delete_api_key()?;
    get_settings_cmd(state)
}
