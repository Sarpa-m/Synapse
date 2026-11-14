use crate::commands::AppState;
use crate::domain::{errors::SynapseResult, profile::Profile};

pub fn list_profiles_cmd(state: &AppState) -> SynapseResult<Vec<Profile>> {
    state.profile_service.list_profiles()
}

pub fn save_profile_cmd(state: &AppState, profile: Profile) -> SynapseResult<Profile> {
    state.profile_service.upsert_profile(profile)
}

pub fn delete_profile_cmd(state: &AppState, id: &str) -> SynapseResult<()> {
    state.profile_service.delete_profile(id)
}

pub fn restore_default_profiles_cmd(state: &AppState) -> SynapseResult<Vec<Profile>> {
    state.profile_service.restore_defaults()
}
