use bytes::Bytes;
use hyper::header::{HeaderName, HeaderValue};
use hyper::Request;
use hyper_util::rt::TokioIo;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::proxy::ca::generate_root_ca;
use crate::proxy::engine::ProxyServer;
use crate::proxy::export::{export_to_har, export_to_openapi};
use crate::proxy::recorder::{
    HeaderEntry, HttpExchange, InterceptedRequest, InterceptedResponse, ProxyConfig,
};
use crate::proxy::scanner::{scan_local_targets, DiscoveredTarget};
use crate::proxy::GeneratedCa;
use crate::state::{
    extract_jwts_from_body, extract_jwts_from_headers, ExtractedJwt, SessionState,
};

use crate::security::SecurityFinding;

pub struct AppState {
    pub proxy_server: Mutex<Option<ProxyServer>>,
    pub session: SessionState,
    pub exchanges: Mutex<Vec<HttpExchange>>,
    pub config: Mutex<ProxyConfig>,
    pub security_findings: Mutex<Vec<SecurityFinding>>,
    pub silenced_traffic_count: std::sync::atomic::AtomicU64,
}

#[tauri::command]
pub async fn start_proxy(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    config: Option<ProxyConfig>,
) -> Result<(), String> {
    let mut server = {
        let server_lock = state.proxy_server.lock();
        if server_lock.is_some() {
            return Err("O servidor proxy já está em execução.".to_string());
        }

        if let Some(cfg) = config {
            *state.config.lock() = cfg;
        }

        let current_config = state.config.lock().clone();
        ProxyServer::new(current_config)
    };

    server.start(app).await?;
    *state.proxy_server.lock() = Some(server);

    Ok(())
}

#[tauri::command]
pub async fn stop_proxy(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut server_lock = state.proxy_server.lock();
    if let Some(mut server) = server_lock.take() {
        server.stop();
        Ok(())
    } else {
        Err("O servidor proxy não está rodando.".to_string())
    }
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
pub async fn get_proxy_config(state: State<'_, Arc<AppState>>) -> Result<ProxyConfig, String> {
    Ok(state.config.lock().clone())
}

#[tauri::command]
pub async fn load_config_from_json(
    state: State<'_, Arc<AppState>>,
    json_content: String,
) -> Result<ProxyConfig, String> {
    let config: ProxyConfig = serde_json::from_str(&json_content)
        .map_err(|e| format!("Formato de arquivo JSON inválido: {}", e))?;
    *state.config.lock() = config.clone();
    Ok(config)
}

/// Escaneia portas locais de desenvolvimento ativas
#[tauri::command]
pub async fn scan_active_targets() -> Result<Vec<DiscoveredTarget>, String> {
    Ok(scan_local_targets().await)
}

/// Testa se um host e porta específicos estão ativos
#[tauri::command]
pub async fn check_target_active(host: String, port: u16) -> Result<bool, String> {
    let addrs = if host == "127.0.0.1" || host == "localhost" {
        vec![format!("127.0.0.1:{}", port), format!("[::1]:{}", port)]
    } else {
        vec![format!("{}:{}", host, port)]
    };

    for addr in addrs {
        if let Ok(Ok(_)) = tokio::time::timeout(std::time::Duration::from_millis(150), tokio::net::TcpStream::connect(&addr)).await {
            return Ok(true);
        }
    }
    Ok(false)
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
    state.silenced_traffic_count.store(0, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn delete_exchange(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    let mut exchs = state.exchanges.lock();
    exchs.retain(|e| e.id != id);
    Ok(())
}

#[tauri::command]
pub async fn delete_exchanges(state: State<'_, Arc<AppState>>, ids: Vec<String>) -> Result<(), String> {
    let id_set: std::collections::HashSet<String> = ids.into_iter().collect();
    let mut exchs = state.exchanges.lock();
    exchs.retain(|e| !id_set.contains(&e.id));
    Ok(())
}

#[tauri::command]
pub async fn get_security_findings(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<SecurityFinding>, String> {
    Ok(state.security_findings.lock().clone())
}

#[tauri::command]
pub async fn clear_security_findings(
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.security_findings.lock().clear();
    Ok(())
}

pub use crate::proxy::importer::{SavedTemplateInput, SavedTemplateOutput};


/// Parser inteligente e universal de coleções (OpenAPI 3.0 / Swagger, Postman Collection v2.1 ou Array Relay)
#[tauri::command]
pub async fn parse_collection_json(
    json_content: String,
) -> Result<Vec<SavedTemplateOutput>, String> {
    crate::proxy::importer::parse_collection_content(&json_content)
}


#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayRequestPayload {
    pub method: String,
    pub uri: String,
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
}

/// Executa um replay direto para o servidor alvo de forma segura com timeout ou mock
#[tauri::command]
pub async fn execute_replay(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    payload: ReplayRequestPayload,
) -> Result<HttpExchange, String> {
    let config = state.config.lock().clone();
    let req_id = format!("replay-{}", Uuid::new_v4());
    let timestamp = chrono::Utc::now().timestamp_millis();
    let start_time = Instant::now();

    let method = match hyper::Method::from_bytes(payload.method.to_uppercase().as_bytes()) {
        Ok(m) => m,
        Err(_) => hyper::Method::GET,
    };

    let body_bytes = payload
        .body
        .as_ref()
        .map(|b| Bytes::from(b.clone()))
        .unwrap_or_default();

    let intercepted_req = InterceptedRequest {
        id: req_id.clone(),
        timestamp,
        method: method.to_string(),
        uri: payload.uri.clone(),
        headers: payload.headers.clone(),
        body: payload.body.clone(),
        size_bytes: body_bytes.len(),
    };

    let mut exchange = HttpExchange {
        id: req_id.clone(),
        request: intercepted_req,
        response: None,
        status: "pending".to_string(),
        error: None,
    };

    let _ = app.emit("relay:request", &exchange);

    // Verifica se a rota é um Mock configurado
    for route in &config.routes {
        if route.is_mock && payload.uri.starts_with(&route.path_prefix) {
            let status_code = route.mock_status_code.unwrap_or(200);
            let mock_body = route
                .mock_body
                .clone()
                .unwrap_or_else(|| "{\"mock\": true}".to_string());
            let duration_ms = start_time.elapsed().as_millis() as u64;

            let intercepted_res = InterceptedResponse {
                id: format!("res-{}", Uuid::new_v4()),
                request_id: req_id.clone(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                status_code,
                headers: vec![
                    HeaderEntry {
                        key: "content-type".to_string(),
                        value: "application/json".to_string(),
                    },
                    HeaderEntry {
                        key: "x-relay-mock".to_string(),
                        value: "true".to_string(),
                    },
                ],
                body: Some(mock_body),
                size_bytes: route.mock_body.as_ref().map(|b| b.len()).unwrap_or(0),
                duration_ms,
            };

            exchange.response = Some(intercepted_res.clone());
            exchange.status = "completed".to_string();

            state.exchanges.lock().push(exchange.clone());
            let _ = app.emit("relay:response", &intercepted_res);
            return Ok(exchange);
        }
    }

    // Identifica target da rota ou default
    let mut target_host = config.target_host.clone();
    let mut target_port = config.target_port;

    for route in &config.routes {
        if payload.uri.starts_with(&route.path_prefix) {
            target_port = route.target_port;
            if let Some(ref h) = route.target_host {
                target_host = h.clone();
            }
            break;
        }
    }

    let upstream_addr = format!("{}:{}", target_host, target_port);
    let tcp_stream = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        TcpStream::connect(&upstream_addr),
    )
    .await
    {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            let err_msg = format!("Falha de conexão com {}: {}", upstream_addr, e);
            exchange.status = "failed".to_string();
            exchange.error = Some(err_msg.clone());
            state.exchanges.lock().push(exchange.clone());
            let _ = app.emit(
                "relay:error",
                serde_json::json!({ "requestId": req_id, "error": err_msg }),
            );
            return Ok(exchange);
        }
        Err(_) => {
            let err_msg = format!("Timeout de conexão ao contactar {}", upstream_addr);
            exchange.status = "failed".to_string();
            exchange.error = Some(err_msg.clone());
            state.exchanges.lock().push(exchange.clone());
            let _ = app.emit(
                "relay:error",
                serde_json::json!({ "requestId": req_id, "error": err_msg }),
            );
            return Ok(exchange);
        }
    };

    let io = TokioIo::new(tcp_stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| format!("Falha no handshake HTTP: {}", e))?;

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            tracing::warn!("Conexão upstream encerrada: {:?}", err);
        }
    });

    let mut req_builder = Request::builder()
        .method(method)
        .uri(payload.uri.as_str());

    for h in &payload.headers {
        if !h.key.eq_ignore_ascii_case("host")
            && !h.key.eq_ignore_ascii_case("connection")
            && !h.key.eq_ignore_ascii_case("transfer-encoding")
        {
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_bytes(h.key.as_bytes()),
                HeaderValue::from_str(&h.value),
            ) {
                req_builder = req_builder.header(hn, hv);
            }
        }
    }
    // Sempre injeta o Host correto do upstream
    req_builder = req_builder.header("host", &upstream_addr);

    let req_body = http_body_util::Full::new(body_bytes);
    let req = req_builder
        .body(req_body)
        .map_err(|e| format!("Falha ao construir requisição: {}", e))?;

    let resp = match sender.send_request(req).await {
        Ok(r) => r,
        Err(e) => {
            let err_msg = format!("Erro ao enviar requisição para upstream: {}", e);
            exchange.status = "failed".to_string();
            exchange.error = Some(err_msg.clone());
            state.exchanges.lock().push(exchange.clone());
            let _ = app.emit(
                "relay:error",
                serde_json::json!({ "requestId": req_id, "error": err_msg }),
            );
            return Ok(exchange);
        }
    };

    let status_code = resp.status().as_u16();
    let res_headers: Vec<HeaderEntry> = resp
        .headers()
        .iter()
        .map(|(k, v)| HeaderEntry {
            key: k.to_string(),
            value: v.to_str().unwrap_or("").to_string(),
        })
        .collect();

    use http_body_util::BodyExt;
    let resp_bytes = resp
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("Erro ao ler corpo da resposta: {}", e))?
        .to_bytes();

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let body_str = String::from_utf8(resp_bytes.to_vec()).ok();

    let intercepted_res = InterceptedResponse {
        id: format!("res-{}", Uuid::new_v4()),
        request_id: req_id.clone(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        status_code,
        headers: res_headers,
        body: body_str,
        size_bytes: resp_bytes.len(),
        duration_ms,
    };

    exchange.response = Some(intercepted_res.clone());
    exchange.status = "completed".to_string();

    // Auto-extração de JWT na requisição e na resposta do Replay
    if config.auto_extract_jwt {
        let req_header_tuples: Vec<(String, String)> = payload
            .headers
            .iter()
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect();
        let mut replay_jwts = extract_jwts_from_headers(&req_header_tuples, "replay_request");

        let res_header_tuples: Vec<(String, String)> = intercepted_res
            .headers
            .iter()
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect();
        replay_jwts.extend(extract_jwts_from_headers(&res_header_tuples, "replay_response_header"));

        if let Some(ref b) = intercepted_res.body {
            replay_jwts.extend(extract_jwts_from_body(b, "replay_response_body"));
        }

        for jwt in replay_jwts {
            state.session.insert_jwt(jwt.clone());
            let _ = app.emit("relay:jwt", &jwt);
        }
    }

    {
        let mut exchs = state.exchanges.lock();
        if exchs.len() >= 150 {
            exchs.remove(0);
        }
        exchs.push(exchange.clone());
    }
    let _ = app.emit("relay:request", &exchange);
    let _ = app.emit("relay:response", &intercepted_res);

    // Auditoria Passiva de Segurança
    let findings = crate::security::audit_exchange(&exchange);
    if !findings.is_empty() {
        let mut sec_lock = state.security_findings.lock();
        for f in findings {
            let _ = app.emit("relay:security_finding", &f);
            if sec_lock.len() >= 200 {
                sec_lock.remove(0);
            }
            sec_lock.push(f);
        }
    }

    Ok(exchange)
}

#[tauri::command]
pub async fn create_ca_certificate() -> Result<GeneratedCa, String> {
    generate_root_ca("Relay Local Root CA")
}

#[tauri::command]
pub async fn export_har(
    state: State<'_, Arc<AppState>>,
    sanitize: Option<bool>,
) -> Result<String, String> {
    let mut exchanges = state.exchanges.lock().clone();
    if sanitize.unwrap_or(true) {
        exchanges = crate::security::sanitize_exchanges(&exchanges);
    }
    let json_val = export_to_har(&exchanges);
    serde_json::to_string_pretty(&json_val).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_openapi(
    state: State<'_, Arc<AppState>>,
    sanitize: Option<bool>,
) -> Result<String, String> {
    let mut exchanges = state.exchanges.lock().clone();
    if sanitize.unwrap_or(true) {
        exchanges = crate::security::sanitize_exchanges(&exchanges);
    }
    let config = state.config.lock().clone();
    let json_val = export_to_openapi(&exchanges, &config.target_host, config.target_port);
    serde_json::to_string_pretty(&json_val).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_active_probe(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    exchange_id: String,
    exchange_fallback: Option<crate::proxy::recorder::HttpExchange>,
    probe_type: crate::security::ProbeType,
    token_b: Option<String>,
) -> Result<crate::security::ActiveProbeResult, String> {
    let (target_host, target_port) = {
        let cfg = state.config.lock();
        (cfg.target_host.clone(), cfg.target_port)
    };

    let exchange = {
        let lock = state.exchanges.lock();
        lock.iter()
            .find(|e| e.id == exchange_id)
            .cloned()
            .or(exchange_fallback)
            .ok_or_else(|| format!("Requisição '{}' não encontrada no histórico.", exchange_id))?
    };

    let result = match probe_type {
        crate::security::ProbeType::MassAssignment => {
            crate::security::run_mass_assignment_probe(&target_host, target_port, &exchange).await
        }
        crate::security::ProbeType::AuthBypass => {
            crate::security::run_auth_bypass_probe(&target_host, target_port, &exchange).await
        }
        crate::security::ProbeType::BolaAb => {
            let tb = token_b.unwrap_or_default();
            crate::security::run_bola_ab_probe(&target_host, target_port, &exchange, &tb).await
        }
        crate::security::ProbeType::HiddenVerbs => {
            crate::security::run_hidden_verbs_probe(&target_host, target_port, &exchange).await
        }
        crate::security::ProbeType::OutdatedAssets => {
            crate::security::run_hidden_verbs_probe(&target_host, target_port, &exchange).await
        }
        crate::security::ProbeType::StackTrace => {
            crate::security::run_stack_trace_probe(&target_host, target_port, &exchange).await
        }
    };

    // Se vulnerável, gera finding e emite para o frontend
    if let Some(finding) = result.to_finding(&exchange_id) {
        let mut sec_lock = state.security_findings.lock();
        let _ = app.emit("relay:security_finding", &finding);
        if sec_lock.len() >= 200 {
            sec_lock.remove(0);
        }
        sec_lock.push(finding);
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_stride_report(
    state: State<'_, Arc<AppState>>,
) -> Result<crate::security::StrideReport, String> {
    let exchanges = state.exchanges.lock().clone();
    let findings = state.security_findings.lock().clone();
    Ok(crate::security::generate_stride_report(&exchanges, &findings))
}
