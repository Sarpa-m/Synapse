use std::sync::Arc;

use crate::domain::{
    errors::{SynapseError, SynapseResult},
    history::HistoryEntry,
};
use crate::ports::storage_ports::HistoryRepoPort;

/// Serviço de aplicação responsável pelo cofre de histórico, Undo rápido e expurgo automático (RN-07).
#[derive(Clone)]
pub struct HistoryService {
    repo: Arc<dyn HistoryRepoPort>,
}

impl HistoryService {
    #[must_use]
    pub fn new(repo: Arc<dyn HistoryRepoPort>) -> Self {
        Self { repo }
    }

    pub fn record_operation(
        &self,
        profile_name: &str,
        original_text: &str,
        generated_text: &str,
    ) -> SynapseResult<HistoryEntry> {
        self.repo
            .record(profile_name, original_text, generated_text)
    }

    pub fn list_recent(&self, limit: usize) -> SynapseResult<Vec<HistoryEntry>> {
        self.repo.list_recent(limit)
    }

    pub fn get_last_for_undo(&self) -> SynapseResult<HistoryEntry> {
        self.repo.get_latest()?.ok_or(SynapseError::NothingToUndo)
    }

    pub fn clear_all(&self) -> SynapseResult<()> {
        self.repo.clear_all()
    }

    pub fn run_retention_cleanup(&self, retention_days: u32) -> SynapseResult<usize> {
        self.repo.purge_older_than(retention_days)
    }
}
