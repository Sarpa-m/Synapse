use std::sync::{Arc, Mutex};

use crate::adapters::{
    llm::openai_compat::OpenAiCompatClient,
    security::keyring_store::KeyringSecretStore,
    sqlite::{
        db::SqliteDatabase,
        history_repo::{SqliteConfigRepo, SqliteHistoryRepo},
        profile_repo::SqliteProfileRepo,
    },
    windows_os::{
        clipboard::SystemClipboardAdapter, keyboard::WindowsKeyboardAdapter,
        window_focus::WindowsWindowFocusAdapter,
    },
};
use crate::domain::errors::{SynapseError, SynapseResult};
use crate::ports::{
    llm_port::LlmProviderPort,
    os_ports::{ClipboardPort, KeyboardSimulatorPort, WindowFocusPort},
    storage_ports::{ConfigRepoPort, HistoryRepoPort, ProfileRepoPort, SecretStorePort},
};
use crate::services::{
    capture_service::{CaptureService, CaptureSession},
    execution_service::ExecutionService,
    history_service::HistoryService,
    profile_service::ProfileService,
};

pub mod history_cmd;
pub mod popup_cmd;
pub mod profile_cmd;
pub mod settings_cmd;

/// Estado compartilhado da aplicação gerenciado pelo Tauri (`AppState`).
#[derive(Clone)]
pub struct AppState {
    pub profile_service: ProfileService,
    pub history_service: HistoryService,
    pub capture_service: CaptureService,
    pub execution_service: ExecutionService,
    pub config_repo: Arc<dyn ConfigRepoPort>,
    pub secret_store: Arc<dyn SecretStorePort>,
    active_session: Arc<Mutex<Option<CaptureSession>>>,
}

impl AppState {
    /// Inicializa o estado com o banco SQLite fornecido e os adaptadores do sistema operacional.
    ///
    /// # Errors
    /// Retorna erro caso o seed inicial de perfis, limpeza de retenção ou inicialização HTTP falhe.
    pub fn bootstrap(db: SqliteDatabase) -> SynapseResult<Self> {
        let profile_repo: Arc<dyn ProfileRepoPort> = Arc::new(SqliteProfileRepo::new(db.clone()));
        let history_repo: Arc<dyn HistoryRepoPort> = Arc::new(SqliteHistoryRepo::new(db.clone()));
        let config_repo: Arc<dyn ConfigRepoPort> = Arc::new(SqliteConfigRepo::new(db));
        let secret_store: Arc<dyn SecretStorePort> = Arc::new(KeyringSecretStore::new());

        let clipboard: Arc<dyn ClipboardPort> = Arc::new(SystemClipboardAdapter::new());
        let keyboard: Arc<dyn KeyboardSimulatorPort> = Arc::new(WindowsKeyboardAdapter::new());
        let window_focus: Arc<dyn WindowFocusPort> = Arc::new(WindowsWindowFocusAdapter::new());
        let llm_client: Arc<dyn LlmProviderPort> = Arc::new(OpenAiCompatClient::new()?);

        let profile_service = ProfileService::new(Arc::clone(&profile_repo));
        profile_service.ensure_defaults()?;

        let history_service = HistoryService::new(Arc::clone(&history_repo));
        let cfg = config_repo.get_config()?;
        history_service.run_retention_cleanup(cfg.retention_days)?;

        let capture_service = CaptureService::new(
            Arc::clone(&clipboard),
            Arc::clone(&keyboard),
            Arc::clone(&window_focus),
        );

        let execution_service = ExecutionService::new(
            llm_client,
            clipboard,
            keyboard,
            window_focus,
            history_repo,
            Arc::clone(&secret_store),
        );

        Ok(Self {
            profile_service,
            history_service,
            capture_service,
            execution_service,
            config_repo,
            secret_store,
            active_session: Arc::new(Mutex::new(None)),
        })
    }

    pub fn set_active_session(&self, session: Option<CaptureSession>) -> SynapseResult<()> {
        let mut guard = self
            .active_session
            .lock()
            .map_err(|_| SynapseError::DatabaseError("lock de sessão ativa corrompido".into()))?;
        *guard = session;
        Ok(())
    }

    pub fn get_active_session(&self) -> SynapseResult<Option<CaptureSession>> {
        let guard = self
            .active_session
            .lock()
            .map_err(|_| SynapseError::DatabaseError("lock de sessão ativa corrompido".into()))?;
        Ok(guard.clone())
    }
}
