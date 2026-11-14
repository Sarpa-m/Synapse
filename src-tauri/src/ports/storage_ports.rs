use crate::domain::{
    config::AppConfig, errors::SynapseResult, history::HistoryEntry, profile::Profile,
};

/// Porta de repositório para persistência de Perfis de IA no SQLite.
#[cfg_attr(test, mockall::automock)]
pub trait ProfileRepoPort: Send + Sync {
    fn list_all(&self) -> SynapseResult<Vec<Profile>>;
    fn get_by_id(&self, id: &str) -> SynapseResult<Option<Profile>>;
    fn save(&self, profile: &Profile) -> SynapseResult<()>;
    fn delete(&self, id: &str) -> SynapseResult<()>;
    fn seed_defaults_if_empty(&self) -> SynapseResult<usize>;
}

/// Porta de repositório para persistência de Histórico de Operações no SQLite.
#[cfg_attr(test, mockall::automock)]
pub trait HistoryRepoPort: Send + Sync {
    fn record(
        &self,
        profile_name: &str,
        original_text: &str,
        generated_text: &str,
    ) -> SynapseResult<HistoryEntry>;
    fn list_recent(&self, limit: usize) -> SynapseResult<Vec<HistoryEntry>>;
    fn get_latest(&self) -> SynapseResult<Option<HistoryEntry>>;
    fn clear_all(&self) -> SynapseResult<()>;
    fn purge_older_than(&self, retention_days: u32) -> SynapseResult<usize>;
}

/// Porta de repositório para leitura e gravação das configurações gerais (`AppConfig`).
#[cfg_attr(test, mockall::automock)]
pub trait ConfigRepoPort: Send + Sync {
    fn get_config(&self) -> SynapseResult<AppConfig>;
    fn save_config(&self, config: &AppConfig) -> SynapseResult<()>;
}

/// Porta de cofre seguro de credenciais do SO (Windows Credential Manager via `keyring`).
/// Segue estritamente a regra RN-01.
#[cfg_attr(test, mockall::automock)]
pub trait SecretStorePort: Send + Sync {
    fn get_api_key(&self) -> SynapseResult<Option<String>>;
    fn set_api_key(&self, api_key: &str) -> SynapseResult<()>;
    fn delete_api_key(&self) -> SynapseResult<()>;

    fn has_api_key(&self) -> SynapseResult<bool> {
        Ok(self
            .get_api_key()?
            .is_some_and(|key| !key.trim().is_empty()))
    }
}
