//! Camada de apresentação Tauri v2 (somente Windows).
//! Registra atalho global, System Tray, janelas pré-carregadas e handlers IPC finos
//! que apenas delegam para `crate::commands::*`.

use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::adapters::sqlite::db::SqliteDatabase;
use crate::commands::{history_cmd, popup_cmd, profile_cmd, settings_cmd, AppState};
use crate::domain::{
    config::{AppConfig, AppConfigView},
    errors::{SynapseError, SynapseResult},
    history::HistoryEntry,
    profile::Profile,
};

const POPUP_LABEL: &str = "popup";
const SETTINGS_LABEL: &str = "settings";
pub const EVENT_SESSION_STARTED: &str = "synapse://session-started";
const RETENTION_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

fn hide_popup(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(POPUP_LABEL) {
        if let Err(e) = win.hide() {
            tracing::warn!(error = %e, "falha ao esconder popup");
        }
    }
}

fn show_settings(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn show_popup_with_session(app: &AppHandle, view: &popup_cmd::PopupSessionView) {
    if let Some(win) = app.get_webview_window(POPUP_LABEL) {
        let _ = app.emit_to(POPUP_LABEL, EVENT_SESSION_STARTED, view);
        let _ = win.center();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

/// Disparado pelo atalho global principal: captura segura -> mostra popup pré-carregado (zero cold-start).
fn on_main_shortcut(app: AppHandle) {
    if let Some(win) = app.get_webview_window(POPUP_LABEL) {
        if win.is_visible().unwrap_or(false) {
            return; // evita capturas concorrentes enquanto o popup já está aberto
        }
    }

    let state = app.state::<AppState>().inner().clone();
    tauri::async_runtime::spawn(async move {
        match popup_cmd::trigger_popup_capture(&state).await {
            Ok(view) => show_popup_with_session(&app, &view),
            // RN-03: seleção vazia / arquivo / imagem -> aborta silenciosamente
            Err(SynapseError::EmptySelection) => {
                tracing::debug!("atalho acionado sem texto selecionado; abortando silenciosamente");
            }
            Err(e) => tracing::warn!(error = %e, "falha na captura"),
        }
    });
}

/// Disparado pelo atalho de Composição: abre o popup para digitar texto livre e copiar o resultado ao clipboard.
fn on_compose_shortcut(app: AppHandle) {
    if let Some(win) = app.get_webview_window(POPUP_LABEL) {
        if win.is_visible().unwrap_or(false) {
            return;
        }
    }

    let state = app.state::<AppState>().inner().clone();
    match popup_cmd::trigger_compose_popup(&state) {
        Ok(view) => show_popup_with_session(&app, &view),
        Err(e) => tracing::warn!(error = %e, "falha ao abrir modo composição"),
    }
}

// ---------------------------------------------------------------------------
// Comandos IPC (nomes espelhados em src/services/tauriClient.ts)
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_current_popup_session(
    state: State<'_, AppState>,
) -> SynapseResult<Option<popup_cmd::PopupSessionView>> {
    popup_cmd::get_current_popup_session(&state)
}

#[tauri::command]
async fn execute_profile_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> SynapseResult<String> {
    let hide = move || hide_popup(&app);
    popup_cmd::execute_profile_cmd(&state, &profile_id, &hide).await
}

#[tauri::command]
async fn execute_custom_prompt_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    instruction: String,
) -> SynapseResult<String> {
    let hide = move || hide_popup(&app);
    popup_cmd::execute_custom_prompt_cmd(&state, &instruction, &hide).await
}

#[tauri::command]
async fn execute_compose_profile_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    input_text: String,
) -> SynapseResult<String> {
    let hide = move || hide_popup(&app);
    popup_cmd::execute_compose_profile_cmd(&state, &profile_id, &input_text, &hide).await
}

#[tauri::command]
fn cancel_popup_session_cmd(app: AppHandle, state: State<'_, AppState>) -> SynapseResult<()> {
    hide_popup(&app);
    popup_cmd::cancel_popup_session_cmd(&state)
}

#[tauri::command]
fn list_profiles_cmd(state: State<'_, AppState>) -> SynapseResult<Vec<Profile>> {
    profile_cmd::list_profiles_cmd(&state)
}

#[tauri::command]
fn save_profile_cmd(state: State<'_, AppState>, profile: Profile) -> SynapseResult<Profile> {
    profile_cmd::save_profile_cmd(&state, profile)
}

#[tauri::command]
fn delete_profile_cmd(state: State<'_, AppState>, id: String) -> SynapseResult<()> {
    profile_cmd::delete_profile_cmd(&state, &id)
}

#[tauri::command]
fn restore_default_profiles_cmd(state: State<'_, AppState>) -> SynapseResult<Vec<Profile>> {
    profile_cmd::restore_default_profiles_cmd(&state)
}

#[tauri::command]
fn get_settings_cmd(state: State<'_, AppState>) -> SynapseResult<AppConfigView> {
    settings_cmd::get_settings_cmd(&state)
}

#[tauri::command]
fn save_settings_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> SynapseResult<AppConfigView> {
    let previous = state.config_repo.get_config()?;
    let view = settings_cmd::save_settings_cmd(&state, config)?;
    if previous.global_shortcut != view.global_shortcut {
        register_shortcut(
            &app,
            Some(&previous.global_shortcut),
            &view.global_shortcut,
            on_main_shortcut,
        )?;
    }
    if previous.compose_shortcut != view.compose_shortcut {
        register_shortcut(
            &app,
            Some(&previous.compose_shortcut),
            &view.compose_shortcut,
            on_compose_shortcut,
        )?;
    }
    Ok(view)
}

#[tauri::command]
fn save_api_key_cmd(state: State<'_, AppState>, api_key: String) -> SynapseResult<AppConfigView> {
    settings_cmd::save_api_key_cmd(&state, &api_key)
}

#[tauri::command]
fn clear_api_key_cmd(state: State<'_, AppState>) -> SynapseResult<AppConfigView> {
    settings_cmd::clear_api_key_cmd(&state)
}

#[tauri::command]
fn list_history_cmd(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> SynapseResult<Vec<HistoryEntry>> {
    history_cmd::list_history_cmd(&state, limit)
}

#[tauri::command]
fn clear_history_cmd(state: State<'_, AppState>) -> SynapseResult<()> {
    history_cmd::clear_history_cmd(&state)
}

// ---------------------------------------------------------------------------
// Bootstrap
// ---------------------------------------------------------------------------

fn parse_shortcut(raw: &str) -> SynapseResult<Shortcut> {
    raw.parse::<Shortcut>()
        .map_err(|e| SynapseError::KeyboardSimulationError(format!("atalho inválido '{raw}': {e}")))
}

fn register_shortcut(
    app: &AppHandle,
    old: Option<&str>,
    new: &str,
    handler: fn(AppHandle),
) -> SynapseResult<()> {
    let gs = app.global_shortcut();
    if let Some(old) = old {
        if let Ok(old_sc) = parse_shortcut(old) {
            let _ = gs.unregister(old_sc);
        }
    }
    let sc = parse_shortcut(new)?;
    gs.on_shortcut(sc, move |app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            handler(app.clone());
        }
    })
    .map_err(|e| SynapseError::KeyboardSimulationError(e.to_string()))
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, "settings", "Configurações", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair do Synapse", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &quit])?;

    let mut builder = TrayIconBuilder::with_id("synapse-tray")
        .tooltip("Synapse Desktop")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => show_settings(app),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    let data_dir = handle.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    let db = SqliteDatabase::open(data_dir.join("synapse.db"))?;
    let state = AppState::bootstrap(db)?;
    let config = state.config_repo.get_config()?;
    app.manage(state.clone());

    if let Err(e) = register_shortcut(&handle, None, &config.global_shortcut, on_main_shortcut) {
        tracing::warn!(
            shortcut = %config.global_shortcut,
            error = %e,
            "não foi possível registrar o atalho global no boot; altere-o nas Configurações pelo System Tray"
        );
    }
    if let Err(e) = register_shortcut(&handle, None, &config.compose_shortcut, on_compose_shortcut)
    {
        tracing::warn!(
            shortcut = %config.compose_shortcut,
            error = %e,
            "não foi possível registrar o atalho de composição no boot; altere-o nas Configurações pelo System Tray"
        );
    }
    build_tray(&handle)?;

    // Fechar a janela de configurações apenas a esconde (o app vive no tray)
    if let Some(settings) = handle.get_webview_window(SETTINGS_LABEL) {
        let win = settings.clone();
        settings.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = win.hide();
            }
        });
    }

    // RN-07: expurgo a cada 24h (o primeiro já rodou no bootstrap)
    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(RETENTION_INTERVAL);
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let days = state
                .config_repo
                .get_config()
                .map(|c| c.retention_days)
                .unwrap_or(30);
            match state.history_service.run_retention_cleanup(days) {
                Ok(n) => tracing::debug!(purged = n, "expurgo de histórico executado"),
                Err(e) => tracing::warn!(error = %e, "falha no expurgo de histórico"),
            }
        }
    });

    Ok(())
}

/// Ponto de entrada do aplicativo desktop.
pub fn run() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            get_current_popup_session,
            execute_profile_cmd,
            execute_custom_prompt_cmd,
            execute_compose_profile_cmd,
            cancel_popup_session_cmd,
            list_profiles_cmd,
            save_profile_cmd,
            delete_profile_cmd,
            restore_default_profiles_cmd,
            get_settings_cmd,
            save_settings_cmd,
            save_api_key_cmd,
            clear_api_key_cmd,
            list_history_cmd,
            clear_history_cmd,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        tracing::error!(error = %e, "falha fatal ao executar o Synapse");
    }
}
