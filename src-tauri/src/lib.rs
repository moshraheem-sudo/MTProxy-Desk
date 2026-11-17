pub mod auto_updater;
pub mod app_updater;
pub mod database;
pub mod diagnostics;
pub mod proxy_fetcher;
pub mod proxy_manager;
pub mod proxy_parser;
pub mod proxy_validator;
pub mod settings;
pub mod system_tray;
pub mod telegram_launcher;
pub mod telegram_links;

use auto_updater::AutoUpdater;
use database::Database;
use diagnostics::DiagnosticsLogger;
use proxy_manager::{ManagerStats, ProxyManager};
use proxy_parser::ProxyItem;
use settings::{AppSettings, set_windows_autostart};
use std::sync::{Arc, Mutex};
use tauri::{Manager, State, WindowEvent};
use telegram_launcher::TelegramLaunchResult;
use telegram_links::{ProxyLinks, generate_links};

pub struct AppState {
    pub manager: Arc<ProxyManager>,
}

#[tauri::command]
async fn fetch_proxies(state: State<'_, AppState>) -> Result<Vec<ProxyItem>, String> {
    state.manager.refresh_proxies().await
}

#[tauri::command]
fn get_proxies(state: State<'_, AppState>) -> Result<Vec<ProxyItem>, String> {
    state
        .manager
        .db
        .get_all_proxies()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn validate_all_proxies(state: State<'_, AppState>) -> Result<(), String> {
    state.manager.validate_all().await
}

#[tauri::command]
async fn validate_single_proxy(
    id: String,
    state: State<'_, AppState>,
) -> Result<ProxyItem, String> {
    state.manager.validate_single(&id).await
}

#[tauri::command]
fn connect_telegram(
    id: String,
    state: State<'_, AppState>,
) -> Result<TelegramLaunchResult, String> {
    Ok(state.manager.launch_telegram_proxy(&id))
}

#[tauri::command]
fn connect_fastest_telegram(state: State<'_, AppState>) -> Result<TelegramLaunchResult, String> {
    if let Some(fastest) = state.manager.get_fastest_proxy() {
        Ok(state.manager.launch_telegram_proxy(&fastest.id))
    } else {
        Err("No reachable proxies found. Please run Retest All first.".to_string())
    }
}

#[tauri::command]
fn toggle_favorite(id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let current_fav = state
        .manager
        .db
        .get_all_proxies()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|p| p.id == id)
        .map(|p| p.is_favorite)
        .unwrap_or(false);
    let new_fav = !current_fav;
    state
        .manager
        .db
        .set_favorite(&id, new_fav)
        .map_err(|e| e.to_string())?;
    Ok(new_fav)
}

#[tauri::command]
fn delete_proxy(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .manager
        .db
        .delete_proxy(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_stats(state: State<'_, AppState>) -> Result<ManagerStats, String> {
    Ok(state.manager.get_stats())
}

#[tauri::command]
fn get_proxy_share_info(server: String, port: u16, secret: String) -> Result<ProxyLinks, String> {
    generate_links(&server, port, &secret)
}

#[tauri::command]
fn generate_qr(id: String, state: State<'_, AppState>) -> Result<String, String> {
    let proxies = state
        .manager
        .db
        .get_all_proxies()
        .map_err(|e| e.to_string())?;
    if let Some(p) = proxies.into_iter().find(|p| p.id == id) {
        let links = generate_links(&p.server, p.port, &p.secret)?;
        Ok(links.qr_svg)
    } else {
        Err("Proxy not found".to_string())
    }
}

#[tauri::command]
fn export_proxy_txt(server: String, port: u16, secret: String) -> Result<String, String> {
    Ok(telegram_links::format_proxy_txt(&server, port, &secret))
}

#[tauri::command]
fn export_txt(id: String, state: State<'_, AppState>) -> Result<String, String> {
    let proxies = state
        .manager
        .db
        .get_all_proxies()
        .map_err(|e| e.to_string())?;
    if let Some(p) = proxies.into_iter().find(|p| p.id == id) {
        Ok(telegram_links::format_proxy_txt(&p.server, p.port, &p.secret))
    } else {
        Err("Proxy not found".to_string())
    }
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let s = state.manager.settings.lock().map_err(|e| e.to_string())?;
    Ok(s.clone())
}

#[tauri::command]
fn update_settings(settings: AppSettings, state: State<'_, AppState>) -> Result<(), String> {
    if let Ok(json) = serde_json::to_string(&settings) {
        let _ = state.manager.db.set_setting("app_settings", &json);
    }

    let _ = set_windows_autostart(settings.autostart);

    let mut current = state.manager.settings.lock().map_err(|e| e.to_string())?;
    *current = settings;
    Ok(())
}

#[tauri::command]
fn get_diagnostics_report(state: State<'_, AppState>) -> Result<String, String> {
    let stats = state.manager.get_stats();
    Ok(state
        .manager
        .logger
        .export_report(stats.total, stats.tcp_reachable, &stats.last_update))
}

#[tauri::command]
fn get_diagnostics_logs(state: State<'_, AppState>) -> Result<String, String> {
    let logs = state.manager.logger.get_recent_logs();
    Ok(logs.join("\n"))
}

#[tauri::command]
fn clear_diagnostics_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.manager.logger.clear_logs();
    Ok(())
}

#[tauri::command]
fn open_diagnostics_file(state: State<'_, AppState>) -> Result<(), String> {
    let path = state.manager.logger.get_log_path();
    open::that_detached(&path).map_err(|e| format!("Failed to open log file: {}", e))
}

#[tauri::command]
fn get_sources(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.manager.db.get_sources().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_source(name: Option<String>, url: String, state: State<'_, AppState>) -> Result<(), String> {
    let _ = name;
    state.manager.db.add_source(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_source(url: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .manager
        .db
        .remove_source(&url)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_app_updates() -> Result<app_updater::UpdateCheckResult, String> {
    app_updater::check_for_updates().await
}

#[tauri::command]
async fn start_app_update(
    app: tauri::AppHandle,
    download_url: String,
    file_name: String,
) -> Result<String, String> {
    app_updater::download_and_install_update(app, download_url, file_name).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap());

            let logger = DiagnosticsLogger::new(&app_data_dir);
            logger.log(
                "INFO",
                "CORE",
                &format!("MTProxy Desk starting in: {}", app_data_dir.display()),
            );

            let db = Arc::new(Database::new(&app_data_dir).expect("Failed to initialize database"));

            let mut settings = AppSettings::default();
            if let Ok(Some(saved_json)) = db.get_setting("app_settings")
                && let Ok(parsed) = serde_json::from_str::<AppSettings>(&saved_json)
            {
                settings = parsed;
            }
            let settings_arc = Arc::new(Mutex::new(settings));

            let manager = Arc::new(ProxyManager::new(db, logger.clone(), settings_arc.clone()));

            let updater = AutoUpdater::new(manager.clone());
            updater.start_background_task();

            if let Err(e) = system_tray::setup_system_tray(&app_handle, manager.clone()) {
                logger.log(
                    "ERROR",
                    "TRAY",
                    &format!("Failed to setup system tray: {}", e),
                );
            }

            app.manage(AppState {
                manager: manager.clone(),
            });

            if let Some(main_window) = app.get_webview_window("main") {
                let settings_for_close = settings_arc.clone();
                let logger_for_close = logger.clone();
                let win_clone = main_window.clone();

                main_window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        let minimize_to_tray = settings_for_close
                            .lock()
                            .map(|s| s.minimize_to_tray)
                            .unwrap_or(true);

                        if minimize_to_tray {
                            api.prevent_close();
                            let _ = win_clone.hide();
                            logger_for_close.log(
                                "INFO",
                                "WINDOW",
                                "Window close requested. Minimized to system tray.",
                            );
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            fetch_proxies,
            get_proxies,
            validate_all_proxies,
            validate_single_proxy,
            connect_telegram,
            connect_fastest_telegram,
            toggle_favorite,
            delete_proxy,
            get_stats,
            get_proxy_share_info,
            generate_qr,
            export_proxy_txt,
            export_txt,
            get_settings,
            update_settings,
            get_diagnostics_report,
            get_diagnostics_logs,
            clear_diagnostics_logs,
            open_diagnostics_file,
            get_sources,
            add_source,
            remove_source,
            check_app_updates,
            start_app_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MTProxy Desk application");
}
