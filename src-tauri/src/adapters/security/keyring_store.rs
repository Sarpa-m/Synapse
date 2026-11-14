use std::sync::{Arc, Mutex};

use crate::domain::errors::{SynapseError, SynapseResult};
use crate::ports::storage_ports::SecretStorePort;

pub const KEYRING_SERVICE: &str = "com.synapse.desktop";
pub const KEYRING_USER: &str = "llm_api_key";

/// Cofre de credenciais integrado ao Windows Credential Manager (RN-01).
/// Em ambientes de desenvolvimento sem serviço D-Bus/Keyring ou em testes de integração,
/// disponibiliza também `InMemorySecretStore`.
pub struct KeyringSecretStore {
    #[cfg(not(target_os = "windows"))]
    fallback: InMemorySecretStore,
}

impl KeyringSecretStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_os = "windows"))]
            fallback: InMemorySecretStore::new(),
        }
    }
}

impl Default for KeyringSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStorePort for KeyringSecretStore {
    fn get_api_key(&self) -> SynapseResult<Option<String>> {
        #[cfg(target_os = "windows")]
        {
            let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
                .map_err(|e| SynapseError::SecretStoreError(e.to_string()))?;
            match entry.get_password() {
                Ok(secret) if !secret.trim().is_empty() => Ok(Some(secret)),
                Ok(_) | Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(SynapseError::SecretStoreError(e.to_string())),
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            self.fallback.get_api_key()
        }
    }

    fn set_api_key(&self, api_key: &str) -> SynapseResult<()> {
        let trimmed = api_key.trim();
        if trimmed.is_empty() {
            return self.delete_api_key();
        }

        #[cfg(target_os = "windows")]
        {
            let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
                .map_err(|e| SynapseError::SecretStoreError(e.to_string()))?;
            entry
                .set_password(trimmed)
                .map_err(|e| SynapseError::SecretStoreError(e.to_string()))
        }

        #[cfg(not(target_os = "windows"))]
        {
            self.fallback.set_api_key(trimmed)
        }
    }

    fn delete_api_key(&self) -> SynapseResult<()> {
        #[cfg(target_os = "windows")]
        {
            let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
                .map_err(|e| SynapseError::SecretStoreError(e.to_string()))?;
            match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(SynapseError::SecretStoreError(e.to_string())),
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            self.fallback.delete_api_key()
        }
    }
}

/// Implementação volátil em memória (nunca grava em disco) para testes e ambientes headless.
#[derive(Clone, Default)]
pub struct InMemorySecretStore {
    secret: Arc<Mutex<Option<String>>>,
}

impl InMemorySecretStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            secret: Arc::new(Mutex::new(None)),
        }
    }
}

impl SecretStorePort for InMemorySecretStore {
    fn get_api_key(&self) -> SynapseResult<Option<String>> {
        let guard = self
            .secret
            .lock()
            .map_err(|_| SynapseError::SecretStoreError("lock de segredo corrompido".into()))?;
        Ok(guard.clone())
    }

    fn set_api_key(&self, api_key: &str) -> SynapseResult<()> {
        let mut guard = self
            .secret
            .lock()
            .map_err(|_| SynapseError::SecretStoreError("lock de segredo corrompido".into()))?;
        let trimmed = api_key.trim();
        if trimmed.is_empty() {
            *guard = None;
        } else {
            *guard = Some(trimmed.to_string());
        }
        Ok(())
    }

    fn delete_api_key(&self) -> SynapseResult<()> {
        let mut guard = self
            .secret
            .lock()
            .map_err(|_| SynapseError::SecretStoreError("lock de segredo corrompido".into()))?;
        *guard = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::{AppConfig, AppConfigView};

    #[test]
    fn test_config_view_never_leaks_api_key() {
        let store = InMemorySecretStore::new();
        let sensitive_key = "sk-secret-deepseek-999999";
        store.set_api_key(sensitive_key).unwrap();

        assert!(store.has_api_key().unwrap());

        let view = AppConfigView::from_config(AppConfig::default(), store.has_api_key().unwrap());
        let serialized_json = serde_json::to_string(&view).unwrap();

        assert!(serialized_json.contains("\"has_api_key\":true"));
        assert!(!serialized_json.contains(sensitive_key));
    }
}
