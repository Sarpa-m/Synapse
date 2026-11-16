use rusqlite::{params, OptionalExtension};

use crate::adapters::sqlite::db::SqliteDatabase;
use crate::domain::{
    config::{AppConfig, SelectionDeliveryMode},
    errors::{SynapseError, SynapseResult},
    history::HistoryEntry,
};
use crate::ports::storage_ports::{ConfigRepoPort, HistoryRepoPort};

#[derive(Clone)]
pub struct SqliteHistoryRepo {
    db: SqliteDatabase,
}

impl SqliteHistoryRepo {
    #[must_use]
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }

    /// Método auxiliar usado em testes para inserir registro com data específica.
    #[cfg(test)]
    pub fn insert_with_timestamp(
        &self,
        profile_name: &str,
        original_text: &str,
        generated_text: &str,
        created_at: &str,
    ) -> SynapseResult<()> {
        let conn = self.db.lock_conn()?;
        conn.execute(
            "INSERT INTO history (profile_name, original_text, generated_text, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![profile_name, original_text, generated_text, created_at],
        )
        .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

impl HistoryRepoPort for SqliteHistoryRepo {
    fn record(
        &self,
        profile_name: &str,
        original_text: &str,
        generated_text: &str,
    ) -> SynapseResult<HistoryEntry> {
        let conn = self.db.lock_conn()?;
        conn.execute(
            "INSERT INTO history (profile_name, original_text, generated_text)
             VALUES (?1, ?2, ?3)",
            params![profile_name, original_text, generated_text],
        )
        .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;

        let row_id = conn.last_insert_rowid();
        conn.query_row(
            "SELECT id, profile_name, original_text, generated_text, created_at
             FROM history
             WHERE id = ?1",
            params![row_id],
            |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    profile_name: row.get(1)?,
                    original_text: row.get(2)?,
                    generated_text: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| SynapseError::DatabaseError(e.to_string()))
    }

    fn list_recent(&self, limit: usize) -> SynapseResult<Vec<HistoryEntry>> {
        let conn = self.db.lock_conn()?;
        let limit_i64 = i64::try_from(limit).unwrap_or(100);
        let mut stmt = conn
            .prepare(
                "SELECT id, profile_name, original_text, generated_text, created_at
                 FROM history
                 ORDER BY id DESC
                 LIMIT ?1",
            )
            .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map(params![limit_i64], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    profile_name: row.get(1)?,
                    original_text: row.get(2)?,
                    generated_text: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| SynapseError::DatabaseError(e.to_string()))?);
        }
        Ok(items)
    }

    fn get_latest(&self) -> SynapseResult<Option<HistoryEntry>> {
        let conn = self.db.lock_conn()?;
        conn.query_row(
            "SELECT id, profile_name, original_text, generated_text, created_at
             FROM history
             ORDER BY id DESC
             LIMIT 1",
            [],
            |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    profile_name: row.get(1)?,
                    original_text: row.get(2)?,
                    generated_text: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(|e| SynapseError::DatabaseError(e.to_string()))
    }

    fn clear_all(&self) -> SynapseResult<()> {
        let conn = self.db.lock_conn()?;
        conn.execute("DELETE FROM history", [])
            .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    fn purge_older_than(&self, retention_days: u32) -> SynapseResult<usize> {
        let conn = self.db.lock_conn()?;
        let modifier = format!("-{retention_days} days");
        let deleted = conn
            .execute(
                "DELETE FROM history WHERE datetime(created_at) < datetime('now', ?1)",
                params![modifier],
            )
            .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;
        Ok(deleted)
    }
}

/// Repositório SQLite para `AppConfig` (linha única `id = 1`).
#[derive(Clone)]
pub struct SqliteConfigRepo {
    db: SqliteDatabase,
}

impl SqliteConfigRepo {
    #[must_use]
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }
}

impl ConfigRepoPort for SqliteConfigRepo {
    fn get_config(&self) -> SynapseResult<AppConfig> {
        let conn = self.db.lock_conn()?;
        let maybe_cfg = conn
            .query_row(
                "SELECT global_shortcut, compose_shortcut, undo_shortcut, launch_on_startup, base_url, model, timeout_secs, retention_days, selection_delivery_mode
                 FROM app_config
                 WHERE id = 1",
                [],
                |row| {
                    let launch_int: i64 = row.get(3)?;
                    let timeout_i64: i64 = row.get(6)?;
                    let retention_i64: i64 = row.get(7)?;
                    let delivery_raw: String = row.get(8)?;
                    Ok(AppConfig {
                        global_shortcut: row.get(0)?,
                        compose_shortcut: row.get(1)?,
                        undo_shortcut: row.get(2)?,
                        launch_on_startup: launch_int != 0,
                        base_url: row.get(4)?,
                        model: row.get(5)?,
                        timeout_secs: u64::try_from(timeout_i64).unwrap_or(10),
                        retention_days: u32::try_from(retention_i64).unwrap_or(30),
                        selection_delivery_mode: SelectionDeliveryMode::from_db(&delivery_raw),
                    })
                },
            )
            .optional()
            .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;

        if let Some(cfg) = maybe_cfg {
            Ok(cfg)
        } else {
            let default_cfg = AppConfig::default();
            drop(conn);
            self.save_config(&default_cfg)?;
            Ok(default_cfg)
        }
    }

    fn save_config(&self, config: &AppConfig) -> SynapseResult<()> {
        config.validate()?;
        let conn = self.db.lock_conn()?;
        conn.execute(
            "INSERT INTO app_config (id, global_shortcut, compose_shortcut, undo_shortcut, launch_on_startup, base_url, model, timeout_secs, retention_days, selection_delivery_mode, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                global_shortcut = excluded.global_shortcut,
                compose_shortcut = excluded.compose_shortcut,
                undo_shortcut = excluded.undo_shortcut,
                launch_on_startup = excluded.launch_on_startup,
                base_url = excluded.base_url,
                model = excluded.model,
                timeout_secs = excluded.timeout_secs,
                retention_days = excluded.retention_days,
                selection_delivery_mode = excluded.selection_delivery_mode,
                updated_at = datetime('now')",
            params![
                config.global_shortcut,
                config.compose_shortcut,
                config.undo_shortcut,
                i64::from(config.launch_on_startup),
                config.base_url,
                config.model,
                i64::try_from(config.timeout_secs).unwrap_or(10),
                i64::from(config.retention_days),
                config.selection_delivery_mode.as_str(),
            ],
        )
        .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
