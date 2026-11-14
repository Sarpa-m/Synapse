#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    #[cfg(windows)]
    synapse_lib::desktop::run();

    #[cfg(not(windows))]
    tracing::warn!("Synapse Desktop é Windows-first: o runtime Tauri só é compilado em Windows. Use `cargo test` para validar o core.");
}
