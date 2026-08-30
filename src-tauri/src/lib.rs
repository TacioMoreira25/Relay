pub mod commands;
pub mod proxy;
pub mod state;

use std::sync::Arc;
use commands::AppState;
use parking_lot::Mutex;
use state::SessionState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(AppState {
        proxy_server: Mutex::new(None),
        session: SessionState::new(),
    });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::start_proxy,
            commands::stop_proxy,
            commands::get_session_jwts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
