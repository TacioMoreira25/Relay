pub mod commands;
pub mod proxy;
pub mod state;

use commands::AppState;
use parking_lot::Mutex;
use proxy::ProxyConfig;
use state::SessionState;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "relay=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = Arc::new(AppState {
        proxy_server: Mutex::new(None),
        session: SessionState::new(),
        exchanges: Mutex::new(Vec::new()),
        config: Mutex::new(ProxyConfig::default()),
    });

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            let toggle_i = MenuItem::with_id(
                app,
                "toggle_window",
                "Mostrar / Ocultar Relay",
                true,
                None::<&str>,
            )?;
            let quit_i = MenuItem::with_id(app, "quit", "Encerrar", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("Relay - HTTP Interceptor & Replay")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle_window" => {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
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
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_proxy,
            commands::stop_proxy,
            commands::update_proxy_config,
            commands::get_proxy_config,
            commands::load_config_from_json,
            commands::get_session_jwts,
            commands::clear_session_jwts,
            commands::get_exchanges,
            commands::clear_exchanges,
            commands::execute_replay,
            commands::create_ca_certificate,
            commands::export_har,
            commands::export_openapi,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
