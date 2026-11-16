#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::sync::Arc;

fn open_log_file() -> Option<Arc<fs::File>> {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::temp_dir().into())?;
    let dir = base.join("com.synapse.desktop");
    fs::create_dir_all(&dir).ok()?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("synapse.log"))
        .ok()?;
    Some(Arc::new(file))
}

fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    if let Some(log_file) = open_log_file() {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(false)
            .with_writer(log_file)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "iniciando Synapse Desktop"
    );

    #[cfg(windows)]
    synapse_lib::desktop::run();

    #[cfg(not(windows))]
    tracing::warn!("Synapse Desktop é Windows-first: o runtime Tauri só é compilado em Windows. Use `cargo test` para validar o core.");
}
