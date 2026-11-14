use std::sync::Arc;

use crate::domain::{
    errors::{SynapseError, SynapseResult},
    profile::Profile,
};
use crate::ports::storage_ports::ProfileRepoPort;

/// Serviço de aplicação responsável pelo CRUD e validação de Perfis de IA.
#[derive(Clone)]
pub struct ProfileService {
    repo: Arc<dyn ProfileRepoPort>,
}

impl ProfileService {
    #[must_use]
    pub fn new(repo: Arc<dyn ProfileRepoPort>) -> Self {
        Self { repo }
    }

    pub fn ensure_defaults(&self) -> SynapseResult<usize> {
        self.repo.seed_defaults_if_empty()
    }

    pub fn list_profiles(&self) -> SynapseResult<Vec<Profile>> {
        self.repo.list_all()
    }

    pub fn get_profile(&self, id: &str) -> SynapseResult<Profile> {
        self.repo
            .get_by_id(id)?
            .ok_or_else(|| SynapseError::ProfileNotFound(id.to_string()))
    }

    pub fn upsert_profile(&self, profile: Profile) -> SynapseResult<Profile> {
        let validated = Profile::new(
            profile.id,
            profile.name,
            profile.system_prompt,
            profile.temperature,
            profile.shortcut,
            profile.is_default,
        )?;
        self.repo.save(&validated)?;
        Ok(validated)
    }

    pub fn delete_profile(&self, id: &str) -> SynapseResult<()> {
        self.repo.delete(id)
    }

    pub fn restore_defaults(&self) -> SynapseResult<Vec<Profile>> {
        for seed in Profile::default_seeds() {
            self.repo.save(&seed)?;
        }
        self.repo.list_all()
    }
}
