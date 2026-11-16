-- Schema inicial do Synapse Desktop (SQLite)

CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    system_prompt TEXT NOT NULL,
    temperature REAL NOT NULL CHECK (temperature >= 0.0 AND temperature <= 2.0),
    shortcut TEXT,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS app_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    global_shortcut TEXT NOT NULL,
    compose_shortcut TEXT NOT NULL DEFAULT 'Ctrl+Alt+Space',
    undo_shortcut TEXT NOT NULL,
    launch_on_startup INTEGER NOT NULL DEFAULT 0,
    base_url TEXT NOT NULL,
    model TEXT NOT NULL,
    timeout_secs INTEGER NOT NULL DEFAULT 10,
    retention_days INTEGER NOT NULL DEFAULT 30,
    selection_delivery_mode TEXT NOT NULL DEFAULT 'clipboard',
    auto_check_updates INTEGER NOT NULL DEFAULT 1,
    skipped_update_version TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    profile_name TEXT NOT NULL,
    original_text TEXT NOT NULL,
    generated_text TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at DESC);
