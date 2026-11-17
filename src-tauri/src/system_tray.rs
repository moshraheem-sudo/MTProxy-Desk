use crate::proxy_manager::ProxyManager;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub fn setup_system_tray(
    app: &AppHandle,
    manager: Arc<ProxyManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(
        app,
        "show",
        "إظهار التطبيق (Show Window)",
        true,
        None::<&str>,
    )?;
    let connect_fastest_item = MenuItem::with_id(
        app,
        "connect_fastest",
        "⚡ اتصال بأسرع بروكسي (Connect Fastest)",
        true,
        None::<&str>,
    )?;
    let retest_all_item = MenuItem::with_id(
        app,
        "retest_all",
        "🔄 فحص جميع البروكسيات (Retest All)",
        true,
        None::<&str>,
    )?;
    let launch_tg_item = MenuItem::with_id(
        app,
        "launch_telegram",
        "✈️ تشغيل تليجرام (Launch Telegram)",
        true,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(app, "quit", "❌ خروج (Exit)", true, None::<&str>)?;

    let tray_menu = Menu::with_items(
        app,
        &[
            &show_item,
            &connect_fastest_item,
            &retest_all_item,
            &launch_tg_item,
            &quit_item,
        ],
    )?;

    let mgr_for_events = manager.clone();

    let _tray = TrayIconBuilder::new()
        .menu(&tray_menu)
        .tooltip("MTProxy Desk – Telegram Proxy Manager")
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(move |app_handle, event| {
            let id = event.id.as_ref();
            match id {
                "show" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "connect_fastest" => {
                    if let Some(fastest) = mgr_for_events.get_fastest_proxy() {
                        let _ = mgr_for_events.launch_telegram_proxy(&fastest.id);
                    }
                }
                "retest_all" => {
                    let mgr = mgr_for_events.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = mgr.validate_all().await;
                    });
                }
                "launch_telegram" => {
                    let custom_path = mgr_for_events
                        .settings
                        .lock()
                        .ok()
                        .and_then(|s| s.custom_telegram_path.clone());
                    let _ = mgr_for_events.launcher.launch_proxy(
                        "127.0.0.1",
                        1080,
                        "secret",
                        custom_path.as_deref(),
                    );
                }
                "quit" => {
                    app_handle.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
