use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::Request;
use hyper_util::rt::TokioIo;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::proxy::{
    export_to_har, export_to_openapi, generate_root_ca, resolve_route_target, GeneratedCa,
    HeaderEntry, HttpExchange, InterceptedRequest, InterceptedResponse, ProxyConfig, ProxyServer,
};
use crate::state::{extract_jwts_from_body, extract_jwts_from_headers, ExtractedJwt, SessionState};

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

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SavedTemplateInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub tag: Option<String>,
    pub method: String,
    pub uri: String,
    #[serde(default)]
    pub headers: Vec<HeaderEntry>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub requires_auth: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SavedTemplateOutput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tag: Option<String>,
    pub method: String,
    pub uri: String,
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
    pub requires_auth: bool,
}

/// Importa uma coleção de endpoints estruturados (JSON de Templates)
#[tauri::command]
pub async fn parse_collection_json(
    json_content: String,
) -> Result<Vec<SavedTemplateOutput>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(&json_content).map_err(|e| format!("JSON inválido: {}", e))?;

    let mut result = Vec::new();

    if let Some(arr) = parsed.as_array() {
        for (i, item) in arr.iter().enumerate() {
            if let Ok(tpl) = serde_json::from_value::<SavedTemplateInput>(item.clone()) {
                let id = tpl.id.unwrap_or_else(|| format!("tpl-{}", i + 1));
                result.push(SavedTemplateOutput {
                    id,
                    name: tpl.name,
                    description: tpl.description,
                    tag: tpl.tag,
                    method: tpl.method.to_uppercase(),
                    uri: tpl.uri,
                    headers: tpl.headers,
                    body: tpl.body,
                    requires_auth: tpl.requires_auth,
                });
            }
        }
        return Ok(result);
    }

    if let Some(arr) = parsed.get("requests").and_then(|r| r.as_array()) {
        for (i, item) in arr.iter().enumerate() {
            if let Ok(tpl) = serde_json::from_value::<SavedTemplateInput>(item.clone()) {
                let id = tpl.id.unwrap_or_else(|| format!("tpl-{}", i + 1));
                result.push(SavedTemplateOutput {
                    id,
                    name: tpl.name,
                    description: tpl.description,
                    tag: tpl.tag,
                    method: tpl.method.to_uppercase(),
                    uri: tpl.uri,
                    headers: tpl.headers,
                    body: tpl.body,
                    requires_auth: tpl.requires_auth,
                });
            }
        }
        return Ok(result);
    }

    Err("Formato de coleção desconhecido. Envie um JSON array com os endpoints.".to_string())
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayRequestPayload {
    pub method: String,
    pub uri: String,
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
}

/// Executa um replay direto para o servidor alvo de forma segura com timeout
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

    // Resolve o host e porta de destino conforme as regras de rotas
    let (target_host, target_port, _) = resolve_route_target(&payload.uri, &config);
    let target_addr = format!("{}:{}", target_host, target_port);

    // Conexão com timeout de 5 segundos
    let connect_res = timeout(Duration::from_secs(5), TcpStream::connect(&target_addr)).await;
    let stream = match connect_res {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            let err_msg = format!("Falha ao conectar com o upstream ({}): {}", target_addr, e);
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
            let err_msg = format!("Timeout de conexão com o upstream ({})", target_addr);
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

    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| format!("Erro no handshake com upstream: {}", e))?;

    tokio::spawn(async move {
        let _ = conn.await;
    });

    let uri_path = if payload.uri.starts_with('/') {
        payload.uri.clone()
    } else {
        format!("/{}", payload.uri)
    };

    let mut builder = Request::builder().method(&method).uri(&uri_path);

    for h in &payload.headers {
        if !h.key.eq_ignore_ascii_case("host")
            && !h.key.eq_ignore_ascii_case("connection")
            && !h.key.eq_ignore_ascii_case("transfer-encoding")
        {
            if let Ok(name) = hyper::header::HeaderName::from_bytes(h.key.as_bytes()) {
                if let Ok(val) = hyper::header::HeaderValue::from_str(&h.value) {
                    builder = builder.header(name, val);
                }
            }
        }
    }
    builder = builder.header("host", &target_addr);

    let req = builder
        .body(Full::new(body_bytes))
        .map_err(|e| format!("Falha ao construir requisição: {}", e))?;

    let forward_res = timeout(Duration::from_secs(10), sender.send_request(req)).await;
    let duration_ms = start_time.elapsed().as_millis() as u64;

    match forward_res {
        Ok(Ok(res)) => {
            let status_code = res.status().as_u16();
            let mut res_headers = Vec::new();
            for (k, v) in res.headers().iter() {
                res_headers.push(HeaderEntry {
                    key: k.as_str().to_string(),
                    value: v.to_str().unwrap_or_default().to_string(),
                });
            }

            let res_bytes = match res.into_body().collect().await {
                Ok(c) => c.to_bytes(),
                Err(_) => Bytes::new(),
            };

            let res_body_str = String::from_utf8(res_bytes.to_vec()).ok();

            let intercepted_res = InterceptedResponse {
                id: Uuid::new_v4().to_string(),
                request_id: req_id.clone(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                status_code,
                headers: res_headers.clone(),
                body: res_body_str.clone(),
                size_bytes: res_bytes.len(),
                duration_ms,
            };

            // Auto-captura JWT em respostas do replay
            if config.auto_extract_jwt {
                let res_header_tuples: Vec<(String, String)> = res_headers
                    .iter()
                    .map(|h| (h.key.clone(), h.value.clone()))
                    .collect();

                let mut res_jwts = extract_jwts_from_headers(&res_header_tuples, "replay_response");
                if let Some(ref body_text) = res_body_str {
                    let body_jwts = extract_jwts_from_body(body_text, "replay_response_body");
                    res_jwts.extend(body_jwts);
                }

                for jwt in res_jwts {
                    state.session.insert_jwt(jwt.clone());
                    let _ = app.emit("relay:jwt", &jwt);
                }
            }

            exchange.response = Some(intercepted_res.clone());
            exchange.status = "completed".to_string();

            state.exchanges.lock().push(exchange.clone());
            let _ = app.emit("relay:response", &intercepted_res);

            Ok(exchange)
        }
        Ok(Err(err)) => {
            let err_msg = err.to_string();
            exchange.status = "failed".to_string();
            exchange.error = Some(err_msg.clone());

            state.exchanges.lock().push(exchange.clone());
            let _ = app.emit(
                "relay:error",
                serde_json::json!({
                    "requestId": req_id,
                    "error": err_msg,
                }),
            );

            Ok(exchange)
        }
        Err(_) => {
            let err_msg = "Timeout aguardando resposta do upstream (10s)".to_string();
            exchange.status = "failed".to_string();
            exchange.error = Some(err_msg.clone());

            state.exchanges.lock().push(exchange.clone());
            let _ = app.emit(
                "relay:error",
                serde_json::json!({
                    "requestId": req_id,
                    "error": err_msg,
                }),
            );

            Ok(exchange)
        }
    }
}

/// Gera um novo certificado raiz CA para inspeção HTTPS / MITM
#[tauri::command]
pub async fn create_ca_certificate(common_name: Option<String>) -> Result<GeneratedCa, String> {
    let name = common_name.unwrap_or_else(|| "Relay Root CA Local".to_string());
    generate_root_ca(&name)
}

/// Exporta a sessão atual de tráfego para formato HAR 1.2
#[tauri::command]
pub async fn export_har(state: State<'_, Arc<AppState>>) -> Result<serde_json::Value, String> {
    let list = state.exchanges.lock();
    Ok(export_to_har(&list))
}

/// Exporta os endpoints interceptados para especificação OpenAPI 3.0
#[tauri::command]
pub async fn export_openapi(state: State<'_, Arc<AppState>>) -> Result<serde_json::Value, String> {
    let list = state.exchanges.lock();
    let config = state.config.lock();
    Ok(export_to_openapi(
        &list,
        &config.target_host,
        config.target_port,
    ))
}
