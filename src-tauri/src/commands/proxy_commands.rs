use tauri::{AppHandle, State};
use parking_lot::Mutex;
use std::sync::Arc;

use crate::proxy::{ProxyConfig, ProxyServer};
use crate::state::SessionState;

pub struct AppState {
    pub proxy_server: Mutex<Option<ProxyServer>>,
    pub session: SessionState,
}

#[tauri::command]
pub async fn start_proxy(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    config: ProxyConfig,
) -> Result<(), String> {
    let mut server = ProxyServer::new(config);
    server.start(app).await?;
    
    let mut current_server = state.proxy_server.lock();
    if let Some(mut old) = current_server.take() {
        old.stop();
    }
    *current_server = Some(server);
    Ok(())
}

#[tauri::command]
pub async fn stop_proxy(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut current_server = state.proxy_server.lock();
    if let Some(mut server) = current_server.take() {
        server.stop();
    }
    Ok(())
}

#[tauri::command]
pub async fn get_session_jwts(state: State<'_, Arc<AppState>>) -> Result<Vec<crate::state::ExtractedJwt>, String> {
    Ok(state.session.list_jwts())
}
