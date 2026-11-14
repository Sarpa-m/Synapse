pub mod db;
pub mod history_repo;
pub mod profile_repo;

#[cfg(test)]
mod tests {
    use super::{
        db::SqliteDatabase,
        history_repo::{SqliteConfigRepo, SqliteHistoryRepo},
        profile_repo::SqliteProfileRepo,
    };
    use crate::domain::{config::AppConfig, profile::Profile};
    use crate::ports::storage_ports::{ConfigRepoPort, HistoryRepoPort, ProfileRepoPort};

    #[test]
    fn test_migrations_and_initial_seed() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let repo = SqliteProfileRepo::new(db);

        let seeded = repo.seed_defaults_if_empty().unwrap();
        assert_eq!(seeded, 3);

        let profiles = repo.list_all().unwrap();
        assert_eq!(profiles.len(), 3);
        assert_eq!(profiles[0].name, "Correção Silenciosa");
        assert_eq!(profiles[1].name, "Reescrita Profissional");
        assert_eq!(profiles[2].name, "Tradução EN-PT");
    }

    #[test]
    fn test_seed_is_idempotent() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let repo = SqliteProfileRepo::new(db);

        assert_eq!(repo.seed_defaults_if_empty().unwrap(), 3);
        assert_eq!(repo.seed_defaults_if_empty().unwrap(), 0);
        assert_eq!(repo.list_all().unwrap().len(), 3);
    }

    #[test]
    fn test_profile_crud_lifecycle() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let repo = SqliteProfileRepo::new(db);
        repo.seed_defaults_if_empty().unwrap();

        let custom = Profile::new(
            "custom-resumo",
            "Resumir em Tópicos",
            "Resuma o texto em bullet points objetivos.",
            0.3,
            Some("4".to_string()),
            false,
        )
        .unwrap();

        repo.save(&custom).unwrap();
        assert_eq!(repo.list_all().unwrap().len(), 4);

        let mut updated = repo.get_by_id("custom-resumo").unwrap().unwrap();
        updated.temperature = 0.8;
        repo.save(&updated).unwrap();

        let fetched = repo.get_by_id("custom-resumo").unwrap().unwrap();
        assert!((fetched.temperature - 0.8).abs() < f32::EPSILON);

        repo.delete("custom-resumo").unwrap();
        assert!(repo.get_by_id("custom-resumo").unwrap().is_none());
        assert_eq!(repo.list_all().unwrap().len(), 3);
    }

    #[test]
    fn test_history_retention_purge() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let repo = SqliteHistoryRepo::new(db);

        // Registro antigo (40 dias atrás) e registro recente (hoje)
        repo.insert_with_timestamp(
            "Correção Silenciosa",
            "texto antigo",
            "texto antigo corrigido",
            "2020-01-01 12:00:00",
        )
        .unwrap();

        repo.record("Tradução EN-PT", "hello world", "olá mundo")
            .unwrap();

        assert_eq!(repo.list_recent(10).unwrap().len(), 2);

        let purged = repo.purge_older_than(30).unwrap();
        assert_eq!(purged, 1);

        let remaining = repo.list_recent(10).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].original_text, "hello world");
    }

    #[test]
    fn test_config_repo_persistence() {
        let db = SqliteDatabase::open_in_memory().unwrap();
        let repo = SqliteConfigRepo::new(db);

        let initial = repo.get_config().unwrap();
        assert_eq!(initial, AppConfig::default());

        let updated = AppConfig {
            model: "gpt-4o-mini".to_string(),
            retention_days: 15,
            ..initial
        };
        repo.save_config(&updated).unwrap();

        let loaded = repo.get_config().unwrap();
        assert_eq!(loaded.model, "gpt-4o-mini");
        assert_eq!(loaded.retention_days, 15);
    }
}
