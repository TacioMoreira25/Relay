use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::proxy::{HttpExchange, ProxyConfig, ProxyServer};
use crate::state::{ExtractedJwt, SessionState};

pub struct AppState {
    pub proxy_server: Mutex<Option<ProxyServer>>,
    pub session: SessionState,
    pub exchanges: Mutex<Vec<HttpExchange>>,
    pub config: Mutex<ProxyConfig>,
}

#[tauri::command]
pub async fn start_proxy(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    config: ProxyConfig,
) -> Result<(), String> {
    *state.config.lock() = config.clone();

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
pub async fn update_proxy_config(
    state: State<'_, Arc<AppState>>,
    config: ProxyConfig,
) -> Result<(), String> {
    *state.config.lock() = config;
    Ok(())
}

#[tauri::command]
pub async fn get_session_jwts(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ExtractedJwt>, String> {
    Ok(state.session.list_jwts())
}

#[tauri::command]
pub async fn clear_session_jwts(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.session.clear();
    Ok(())
}

#[tauri::command]
pub async fn get_exchanges(state: State<'_, Arc<AppState>>) -> Result<Vec<HttpExchange>, String> {
    Ok(state.exchanges.lock().clone())
}

#[tauri::command]
pub async fn clear_exchanges(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.exchanges.lock().clear();
    Ok(())
}
