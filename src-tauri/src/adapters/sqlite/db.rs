use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;

use crate::domain::errors::{SynapseError, SynapseResult};

const INITIAL_SCHEMA_SQL: &str = include_str!("../../../migrations/001_initial_schema.sql");

/// Gerenciador de conexão SQLite thread-safe (`rusqlite` com feature `bundled`).
/// Suporta tanto banco em disco (`open`) quanto banco em memória (`open_in_memory`) para testes.
#[derive(Clone)]
pub struct SqliteDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteDatabase {
    /// Abre (ou cria) o banco SQLite em um arquivo local e executa as migrações.
    ///
    /// # Errors
    /// Retorna [`SynapseError::DatabaseError`] caso falhe ao abrir ou aplicar o schema.
    pub fn open(path: impl AsRef<Path>) -> SynapseResult<Self> {
        let conn = Connection::open(path.as_ref())
            .map_err(|e| SynapseError::DatabaseError(e.to_string()))?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations()?;
        Ok(db)
    }

    /// Abre um banco SQLite 100% em memória e aplica as migrações (ideal para testes unitários).
    ///
    /// # Errors
    /// Retorna [`SynapseError::DatabaseError`] caso falhe ao inicializar em memória.
    pub fn open_in_memory() -> SynapseResult<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| SynapseError::DatabaseError(e.to_string()))?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations()?;
        Ok(db)
    }

    /// Adquire o lock da conexão SQLite sem usar `.unwrap()`.
    pub(crate) fn lock_conn(&self) -> SynapseResult<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| SynapseError::DatabaseError("mutex da conexão SQLite envenenado".into()))
    }

    fn run_migrations(&self) -> SynapseResult<()> {
        let conn = self.lock_conn()?;
        conn.execute_batch(INITIAL_SCHEMA_SQL)
            .map_err(|e| SynapseError::DatabaseError(format!("falha na migração SQLite: {e}")))?;

        let mut has_compose_col = false;
        let mut has_delivery_col = false;
        conn.prepare("PRAGMA table_info(app_config)")
            .and_then(|mut stmt| {
                let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
                for col in rows {
                    match col?.as_str() {
                        "compose_shortcut" => has_compose_col = true,
                        "selection_delivery_mode" => has_delivery_col = true,
                        _ => {}
                    }
                }
                Ok(())
            })
            .map_err(|e| {
                SynapseError::DatabaseError(format!("falha ao inspecionar app_config: {e}"))
            })?;

        if !has_compose_col {
            conn.execute(
                "ALTER TABLE app_config ADD COLUMN compose_shortcut TEXT NOT NULL DEFAULT 'Ctrl+Alt+Space'",
                [],
            )
            .map_err(|e| SynapseError::DatabaseError(format!("falha ao adicionar compose_shortcut: {e}")))?;
        }
        if !has_delivery_col {
            conn.execute(
                "ALTER TABLE app_config ADD COLUMN selection_delivery_mode TEXT NOT NULL DEFAULT 'clipboard'",
                [],
            )
            .map_err(|e| {
                SynapseError::DatabaseError(format!(
                    "falha ao adicionar selection_delivery_mode: {e}"
                ))
            })?;
        }
        Ok(())
    }
}
