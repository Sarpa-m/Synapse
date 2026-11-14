use crate::commands::AppState;
use crate::domain::{errors::SynapseResult, history::HistoryEntry};

pub fn list_history_cmd(
    state: &AppState,
    limit: Option<usize>,
) -> SynapseResult<Vec<HistoryEntry>> {
    state.history_service.list_recent(limit.unwrap_or(100))
}

pub fn clear_history_cmd(state: &AppState) -> SynapseResult<()> {
    state.history_service.clear_all()
}

pub fn get_last_undo_entry_cmd(state: &AppState) -> SynapseResult<HistoryEntry> {
    state.history_service.get_last_for_undo()
}
