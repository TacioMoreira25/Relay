pub mod commands;
pub mod proxy;
pub mod state;

use commands::AppState;
use parking_lot::Mutex;
use proxy::ProxyConfig;
use state::SessionState;
use std::sync::Arc;
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
        .invoke_handler(tauri::generate_handler![
            commands::start_proxy,
            commands::stop_proxy,
            commands::update_proxy_config,
            commands::get_session_jwts,
            commands::get_exchanges,
            commands::clear_exchanges,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
