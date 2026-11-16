//! Camada de apresentação Tauri v2 (somente Windows).
//! Registra atalho global, System Tray, janelas pré-carregadas e handlers IPC finos
//! que apenas delegam para `crate::commands::*`.

use std::sync::Arc;
use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::adapters::sqlite::db::SqliteDatabase;
use crate::commands::{history_cmd, popup_cmd, profile_cmd, settings_cmd, update_cmd, AppState};
use crate::domain::{
    config::{AppConfig, AppConfigView},
    errors::{SynapseError, SynapseResult},
    history::HistoryEntry,
    profile::Profile,
    update::{UpdateCheckResult, UpdateProgress, UpdateReleaseInfo},
};
use crate::ports::os_ports::{center_window_on_focused_monitor, MonitorBounds};

const POPUP_LABEL: &str = "popup";
const SETTINGS_LABEL: &str = "settings";
const UPDATE_LABEL: &str = "update";
pub const EVENT_SESSION_STARTED: &str = "synapse://session-started";
pub const EVENT_SETTINGS_OPENED: &str = "synapse://settings-opened";
pub const EVENT_UPDATE_AVAILABLE: &str = "synapse://update-available";
pub const EVENT_UPDATE_PROGRESS: &str = "synapse://update-progress";
const RETENTION_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const STARTUP_UPDATE_CHECK_DELAY: Duration = Duration::from_secs(3);

fn hide_popup(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(POPUP_LABEL) {
        if let Err(e) = win.hide() {
            tracing::warn!(error = %e, "falha ao esconder popup");
        }
    }
}

fn hide_update_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(UPDATE_LABEL) {
        let _ = win.hide();
    }
}

fn current_focus_point(app: &AppHandle) -> Option<(i32, i32)> {
    app.try_state::<AppState>()
        .and_then(|state| state.capture_service.get_focus_screen_point(None).ok())
}

fn show_update_window(app: &AppHandle, release: &UpdateReleaseInfo) {
    let _ = app.emit(EVENT_UPDATE_AVAILABLE, release);
    if let Some(win) = app.get_webview_window(UPDATE_LABEL) {
        center_on_focused_monitor(&win, current_focus_point(app));
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn show_settings(app: &AppHandle) {
    let _ = app.emit(EVENT_SETTINGS_OPENED, ());
    if let Some(win) = app.get_webview_window(SETTINGS_LABEL) {
        if !win.is_visible().unwrap_or(false) {
            center_on_focused_monitor(&win, current_focus_point(app));
        }
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn center_on_focused_monitor(win: &tauri::WebviewWindow, focus_point: Option<(i32, i32)>) {
    let Some(pt) = focus_point else {
        let _ = win.center();
        return;
    };

    let Ok(raw_monitors) = win.available_monitors() else {
        let _ = win.center();
        return;
    };

    let monitors: Vec<MonitorBounds> = raw_monitors
        .iter()
        .map(|m| MonitorBounds {
            x: m.position().x,
            y: m.position().y,
            width: m.size().width,
            height: m.size().height,
        })
        .collect();

    let win_size = win
        .outer_size()
        .map(|s| (s.width, s.height))
        .unwrap_or((620, 340));

    if let Some((x, y)) = center_window_on_focused_monitor(&monitors, pt, win_size) {
        let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
            x, y,
        )));
        // Recalcula após mover caso o monitor de destino tenha escala DPI diferente do anterior
        if let Ok(updated_size) = win.outer_size() {
            if (updated_size.width, updated_size.height) != win_size {
                if let Some((nx, ny)) = center_window_on_focused_monitor(
                    &monitors,
                    pt,
                    (updated_size.width, updated_size.height),
                ) {
                    let _ = win.set_position(tauri::Position::Physical(
                        tauri::PhysicalPosition::new(nx, ny),
                    ));
                }
            }
        }
    } else {
        let _ = win.center();
    }
}

fn show_popup_with_session(
    app: &AppHandle,
    view: &popup_cmd::PopupSessionView,
    focus_point: Option<(i32, i32)>,
) {
    if let Some(win) = app.get_webview_window(POPUP_LABEL) {
        let _ = app.emit_to(POPUP_LABEL, EVENT_SESSION_STARTED, view);
        center_on_focused_monitor(&win, focus_point);
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
            Ok(view) => {
                let source_hwnd = state
                    .get_active_session()
                    .ok()
                    .flatten()
                    .map(|s| s.source_hwnd);
                let focus_point = state
                    .capture_service
                    .get_focus_screen_point(source_hwnd)
                    .ok();
                show_popup_with_session(&app, &view, focus_point);
            }
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
    let focus_point = state.capture_service.get_focus_screen_point(None).ok();
    match popup_cmd::trigger_compose_popup(&state) {
        Ok(view) => show_popup_with_session(&app, &view, focus_point),
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

#[tauri::command]
fn get_app_version_cmd(state: State<'_, AppState>) -> String {
    update_cmd::get_app_version_cmd(&state)
}

#[tauri::command]
fn get_pending_update_cmd(state: State<'_, AppState>) -> SynapseResult<Option<UpdateReleaseInfo>> {
    update_cmd::get_pending_update_cmd(&state)
}

#[tauri::command]
async fn check_for_updates_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    manual: bool,
) -> SynapseResult<UpdateCheckResult> {
    let result = update_cmd::check_for_updates_cmd(&state, manual).await?;
    if let UpdateCheckResult::UpdateAvailable { ref release, .. } = result {
        show_update_window(&app, release);
    }
    Ok(result)
}

#[tauri::command]
async fn install_update_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    release: UpdateReleaseInfo,
) -> SynapseResult<()> {
    let app_for_progress = app.clone();
    let on_progress = Arc::new(move |progress: UpdateProgress| {
        let _ = app_for_progress.emit(EVENT_UPDATE_PROGRESS, &progress);
    });

    update_cmd::install_update_cmd(&state, release, on_progress).await?;

    let app_for_exit = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(400)).await;
        app_for_exit.exit(0);
    });
    Ok(())
}

#[tauri::command]
fn skip_update_version_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    version: String,
) -> SynapseResult<()> {
    hide_update_window(&app);
    update_cmd::skip_update_version_cmd(&state, &version)
}

#[tauri::command]
fn dismiss_update_prompt_cmd(app: AppHandle, state: State<'_, AppState>) -> SynapseResult<()> {
    hide_update_window(&app);
    update_cmd::dismiss_update_prompt_cmd(&state)
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
    let check_updates = MenuItem::with_id(
        app,
        "check_updates",
        "Verificar atualizações...",
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Sair do Synapse", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &check_updates, &quit])?;

    let mut builder = TrayIconBuilder::with_id("synapse-tray")
        .tooltip("Synapse Desktop")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => show_settings(app),
            "check_updates" => {
                let app_handle = app.clone();
                let state = app.state::<AppState>().inner().clone();
                tauri::async_runtime::spawn(async move {
                    match update_cmd::check_for_updates_cmd(&state, true).await {
                        Ok(UpdateCheckResult::UpdateAvailable { release, .. }) => {
                            show_update_window(&app_handle, &release);
                        }
                        Ok(_) => show_settings(&app_handle),
                        Err(e) => {
                            tracing::warn!(error = %e, "falha ao verificar atualizações pelo tray");
                            show_settings(&app_handle);
                        }
                    }
                });
            }
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

    // Fechar as janelas de configurações e atualização apenas as esconde (o app vive no tray)
    for label in [SETTINGS_LABEL, UPDATE_LABEL] {
        if let Some(window) = handle.get_webview_window(label) {
            let win = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win.hide();
                }
            });
        }
    }

    // Verificação automática de atualização no boot (assíncrona e silenciosa em caso de erro)
    if config.auto_check_updates {
        let update_handle = handle.clone();
        let update_state = state.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(STARTUP_UPDATE_CHECK_DELAY).await;
            match update_cmd::check_for_updates_cmd(&update_state, false).await {
                Ok(UpdateCheckResult::UpdateAvailable { release, .. }) => {
                    tracing::info!(
                        new_version = %release.version,
                        "nova versão encontrada no GitHub; solicitando permissão ao usuário"
                    );
                    show_update_window(&update_handle, &release);
                }
                Ok(UpdateCheckResult::UpToDate { current_version }) => {
                    tracing::debug!(version = %current_version, "Synapse está na versão mais recente");
                }
                Ok(UpdateCheckResult::Skipped {
                    skipped_version, ..
                }) => {
                    tracing::debug!(skipped = %skipped_version, "atualização disponível foi ignorada pelo usuário");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "não foi possível verificar atualizações na inicialização");
                }
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
            get_app_version_cmd,
            get_pending_update_cmd,
            check_for_updates_cmd,
            install_update_cmd,
            skip_update_version_cmd,
            dismiss_update_prompt_cmd,
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        tracing::error!(error = %e, "falha fatal ao executar o Synapse");
    }
}
