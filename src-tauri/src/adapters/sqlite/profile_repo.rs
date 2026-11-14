use rusqlite::{params, OptionalExtension};

use crate::adapters::sqlite::db::SqliteDatabase;
use crate::domain::{
    errors::{SynapseError, SynapseResult},
    profile::Profile,
};
use crate::ports::storage_ports::ProfileRepoPort;

#[derive(Clone)]
pub struct SqliteProfileRepo {
    db: SqliteDatabase,
}

impl SqliteProfileRepo {
    #[must_use]
    pub fn new(db: SqliteDatabase) -> Self {
        Self { db }
    }
}

impl ProfileRepoPort for SqliteProfileRepo {
    fn list_all(&self) -> SynapseResult<Vec<Profile>> {
        let conn = self.db.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, system_prompt, temperature, shortcut, is_default
                 FROM profiles
                 ORDER BY is_default DESC, created_at ASC, name ASC",
            )
            .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let is_default_int: i64 = row.get(5)?;
                Ok(Profile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    system_prompt: row.get(2)?,
                    temperature: row.get(3)?,
                    shortcut: row.get(4)?,
                    is_default: is_default_int != 0,
                })
            })
            .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;

        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(row.map_err(|e| SynapseError::DatabaseError(e.to_string()))?);
        }
        Ok(profiles)
    }

    fn get_by_id(&self, id: &str) -> SynapseResult<Option<Profile>> {
        let conn = self.db.lock_conn()?;
        conn.query_row(
            "SELECT id, name, system_prompt, temperature, shortcut, is_default
             FROM profiles
             WHERE id = ?1",
            params![id],
            |row| {
                let is_default_int: i64 = row.get(5)?;
                Ok(Profile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    system_prompt: row.get(2)?,
                    temperature: row.get(3)?,
                    shortcut: row.get(4)?,
                    is_default: is_default_int != 0,
                })
            },
        )
        .optional()
        .map_err(|e| SynapseError::DatabaseError(e.to_string()))
    }

    fn save(&self, profile: &Profile) -> SynapseResult<()> {
        Profile::validate_fields(&profile.name, &profile.system_prompt, profile.temperature)?;

        let conn = self.db.lock_conn()?;
        conn.execute(
            "INSERT INTO profiles (id, name, system_prompt, temperature, shortcut, is_default)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                system_prompt = excluded.system_prompt,
                temperature = excluded.temperature,
                shortcut = excluded.shortcut,
                is_default = excluded.is_default",
            params![
                profile.id,
                profile.name,
                profile.system_prompt,
                profile.temperature,
                profile.shortcut,
                i64::from(profile.is_default),
            ],
        )
        .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, id: &str) -> SynapseResult<()> {
        let conn = self.db.lock_conn()?;
        let affected = conn
            .execute("DELETE FROM profiles WHERE id = ?1", params![id])
            .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;

        if affected == 0 {
            return Err(SynapseError::ProfileNotFound(id.to_string()));
        }
        Ok(())
    }

    fn seed_defaults_if_empty(&self) -> SynapseResult<usize> {
        let count: i64 = {
            let conn = self.db.lock_conn()?;
            conn.query_row("SELECT COUNT(*) FROM profiles", [], |row| row.get(0))
                .map_err(|e| SynapseError::DatabaseError(e.to_string()))?
        };

        if count > 0 {
            return Ok(0);
        }

        let seeds = Profile::default_seeds();
        let inserted = seeds.len();
        for seed in &seeds {
            self.save(seed)?;
        }
        Ok(inserted)
    }
}
