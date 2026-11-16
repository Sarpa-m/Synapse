use serde::{Deserialize, Serialize};

use crate::domain::errors::{SynapseError, SynapseResult};

/// Define como o Synapse entrega o resultado após processar um texto selecionado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionDeliveryMode {
    #[default]
    Clipboard,
    PasteAndKeep,
    PasteAndRestore,
}

impl SelectionDeliveryMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clipboard => "clipboard",
            Self::PasteAndKeep => "paste_and_keep",
            Self::PasteAndRestore => "paste_and_restore",
        }
    }

    #[must_use]
    pub fn from_db(raw: &str) -> Self {
        match raw {
            "paste_and_keep" => Self::PasteAndKeep,
            "paste_and_restore" => Self::PasteAndRestore,
            _ => Self::Clipboard,
        }
    }
}

/// Configurações persistidas da aplicação (sem dados secretos).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub global_shortcut: String,
    #[serde(default = "default_compose_shortcut")]
    pub compose_shortcut: String,
    pub undo_shortcut: String,
    pub launch_on_startup: bool,
    pub base_url: String,
    pub model: String,
    pub timeout_secs: u64,
    pub retention_days: u32,
    #[serde(default)]
    pub selection_delivery_mode: SelectionDeliveryMode,
    #[serde(default = "default_auto_check_updates")]
    pub auto_check_updates: bool,
    #[serde(default)]
    pub skipped_update_version: Option<String>,
}

fn default_compose_shortcut() -> String {
    "Ctrl+Alt+Space".to_string()
}

const fn default_auto_check_updates() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            global_shortcut: "Alt+Space".to_string(),
            compose_shortcut: default_compose_shortcut(),
            undo_shortcut: "Ctrl+Alt+Z".to_string(),
            launch_on_startup: false,
            base_url: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            timeout_secs: 10,
            retention_days: 30,
            selection_delivery_mode: SelectionDeliveryMode::Clipboard,
            auto_check_updates: true,
            skipped_update_version: None,
        }
    }
}

impl AppConfig {
    /// Valida configurações antes de persistir.
    ///
    /// # Errors
    /// Retorna [`SynapseError::InvalidRetentionDays`] ou [`SynapseError::InvalidTimeout`]
    /// caso os limites estejam fora do intervalo seguro.
    pub fn validate(&self) -> SynapseResult<()> {
        if !(1..=365).contains(&self.retention_days) {
            return Err(SynapseError::InvalidRetentionDays);
        }
        if !(1..=60).contains(&self.timeout_secs) {
            return Err(SynapseError::InvalidTimeout);
        }
        Ok(())
    }
}

/// DTO seguro enviado para o Frontend (RN-01).
/// Jamais contém a chave de API real, apenas a flag booleana `has_api_key`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfigView {
    pub global_shortcut: String,
    pub compose_shortcut: String,
    pub undo_shortcut: String,
    pub launch_on_startup: bool,
    pub base_url: String,
    pub model: String,
    pub timeout_secs: u64,
    pub retention_days: u32,
    pub selection_delivery_mode: SelectionDeliveryMode,
    pub auto_check_updates: bool,
    pub skipped_update_version: Option<String>,
    pub has_api_key: bool,
}

impl AppConfigView {
    #[must_use]
    pub fn from_config(config: AppConfig, has_api_key: bool) -> Self {
        Self {
            global_shortcut: config.global_shortcut,
            compose_shortcut: config.compose_shortcut,
            undo_shortcut: config.undo_shortcut,
            launch_on_startup: config.launch_on_startup,
            base_url: config.base_url,
            model: config.model,
            timeout_secs: config.timeout_secs,
            retention_days: config.retention_days,
            selection_delivery_mode: config.selection_delivery_mode,
            auto_check_updates: config.auto_check_updates,
            skipped_update_version: config.skipped_update_version,
            has_api_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let cfg = AppConfig::default();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.timeout_secs, 10);
        assert_eq!(cfg.retention_days, 30);
    }

    #[test]
    fn test_config_validation_rejects_invalid_bounds() {
        let bad_retention = AppConfig {
            retention_days: 0,
            ..AppConfig::default()
        };
        assert_eq!(
            bad_retention.validate().unwrap_err(),
            SynapseError::InvalidRetentionDays
        );

        let bad_timeout = AppConfig {
            timeout_secs: 0,
            ..AppConfig::default()
        };
        assert_eq!(
            bad_timeout.validate().unwrap_err(),
            SynapseError::InvalidTimeout
        );
    }
}
