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
    pub name: Option<String>,
    pub description: Option<String>,
    pub tag: Option<String>,
    pub method: Option<String>,
    pub uri: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
    pub endpoint: Option<String>,
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

/// Parser inteligente e universal de coleções (OpenAPI 3.0 / Swagger, Postman Collection v2.1 ou Array Relay)
#[tauri::command]
pub async fn parse_collection_json(
    json_content: String,
) -> Result<Vec<SavedTemplateOutput>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(&json_content).map_err(|e| format!("JSON inválido: {}", e))?;

    let mut result = Vec::new();

    // 1. Suporte Nativo a OpenAPI 3.0 / Swagger (openapi: "3.0.x" ou swagger: "2.0")
    if let Some(paths) = parsed.get("paths").and_then(|p| p.as_object()) {
        let mut idx = 1;
        for (path_key, path_item) in paths {
            if let Some(methods_map) = path_item.as_object() {
                for (method_key, op_val) in methods_map {
                    let method_upper = method_key.to_uppercase();
                    if !["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]
                        .contains(&method_upper.as_str())
                    {
                        continue;
                    }

                    let summary = op_val
                        .get("summary")
                        .and_then(|s| s.as_str())
                        .or_else(|| op_val.get("operationId").and_then(|o| o.as_str()))
                        .unwrap_or(path_key.as_str())
                        .to_string();

                    let description = op_val
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());

                    let tag = op_val
                        .get("tags")
                        .and_then(|t| t.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|first_tag| first_tag.as_str())
                        .map(|s| s.to_string());

                    let headers = vec![HeaderEntry {
                        key: "Content-Type".to_string(),
                        value: "application/json".to_string(),
                    }];

                    let mut body = None;
                    if let Some(req_body) = op_val.get("requestBody") {
                        if let Some(content) = req_body.get("content") {
                            if let Some(json_content) = content.get("application/json") {
                                if let Some(schema) = json_content.get("schema") {
                                    if let Some(example) = json_content
                                        .get("example")
                                        .or_else(|| schema.get("example"))
                                    {
                                        body = Some(serde_json::to_string_pretty(example).unwrap_or_default());
                                    } else {
                                        body = Some("{\n  \"example\": \"data\"\n}".to_string());
                                    }
                                }
                            }
                        }
                    }

                    let requires_auth = op_val.get("security").is_some() || parsed.get("security").is_some();

                    result.push(SavedTemplateOutput {
                        id: format!("openapi-{}", idx),
                        name: summary,
                        description,
                        tag,
                        method: method_upper,
                        uri: path_key.clone(),
                        headers,
                        body,
                        requires_auth,
                    });
                    idx += 1;
                }
            }
        }

        if !result.is_empty() {
            return Ok(result);
        }
    }

    // 2. Suporte a Coleções Postman (item: [...])
    if let Some(items) = parsed.get("item").and_then(|i| i.as_array()) {
        fn parse_postman_items(
            items: &[serde_json::Value],
            current_tag: Option<String>,
            out: &mut Vec<SavedTemplateOutput>,
            counter: &mut usize,
        ) {
            for it in items {
                if let Some(sub_items) = it.get("item").and_then(|sub| sub.as_array()) {
                    let folder_name = it.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());
                    parse_postman_items(sub_items, folder_name, out, counter);
                } else if let Some(req) = it.get("request") {
                    *counter += 1;
                    let name = it
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Requisição Postman")
                        .to_string();

                    let method = req
                        .get("method")
                        .and_then(|m| m.as_str())
                        .unwrap_or("GET")
                        .to_uppercase();

                    let mut uri = "/".to_string();
                    if let Some(url_obj) = req.get("url") {
                        if let Some(raw) = url_obj.get("raw").and_then(|r| r.as_str()) {
                            if raw.starts_with('/') {
                                uri = raw.to_string();
                            } else if let Some(pos) = raw.find("://") {
                                if let Some(slash_pos) = raw[pos + 3..].find('/') {
                                    uri = raw[pos + 3 + slash_pos..].to_string();
                                }
                            } else if let Some(slash_pos) = raw.find('/') {
                                uri = raw[slash_pos..].to_string();
                            }
                        }
                    }

                    let mut headers = Vec::new();
                    if let Some(h_arr) = req.get("header").and_then(|h| h.as_array()) {
                        for h in h_arr {
                            if let (Some(k), Some(v)) = (
                                h.get("key").and_then(|k| k.as_str()),
                                h.get("value").and_then(|v| v.as_str()),
                            ) {
                                headers.push(HeaderEntry {
                                    key: k.to_string(),
                                    value: v.to_string(),
                                });
                            }
                        }
                    }

                    let body = req
                        .get("body")
                        .and_then(|b| b.get("raw"))
                        .and_then(|r| r.as_str())
                        .map(|s| s.to_string());

                    let requires_auth = req.get("auth").is_some();

                    out.push(SavedTemplateOutput {
                        id: format!("postman-{}", counter),
                        name,
                        description: it.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()),
                        tag: current_tag.clone(),
                        method,
                        uri,
                        headers,
                        body,
                        requires_auth,
                    });
                }
            }
        }

        let mut counter = 0;
        parse_postman_items(items, None, &mut result, &mut counter);
        if !result.is_empty() {
            return Ok(result);
        }
    }

    // 3. Array Padrão Relay / JSON Universal
    let raw_array = if let Some(arr) = parsed.as_array() {
        Some(arr)
    } else if let Some(arr) = parsed.get("requests").and_then(|r| r.as_array()) {
        Some(arr)
    } else if let Some(arr) = parsed.get("endpoints").and_then(|r| r.as_array()) {
        Some(arr)
    } else {
        None
    };

    if let Some(arr) = raw_array {
        for (i, item) in arr.iter().enumerate() {
            if let Ok(tpl) = serde_json::from_value::<SavedTemplateInput>(item.clone()) {
                let id = tpl.id.unwrap_or_else(|| format!("tpl-{}", i + 1));
                let uri = tpl
                    .uri
                    .or(tpl.url)
                    .or(tpl.path)
                    .or(tpl.endpoint)
                    .unwrap_or_else(|| "/".to_string());
                let method = tpl.method.unwrap_or_else(|| "GET".to_string()).to_uppercase();
                let name = tpl.name.unwrap_or_else(|| format!("{} {}", method, uri));

                result.push(SavedTemplateOutput {
                    id,
                    name,
                    description: tpl.description,
                    tag: tpl.tag,
                    method,
                    uri,
                    headers: tpl.headers,
                    body: tpl.body,
                    requires_auth: tpl.requires_auth,
                });
            }
        }
        if !result.is_empty() {
            return Ok(result);
        }
    }

    Err("Formato de coleção não reconhecido. Formatos suportados: OpenAPI 3.0 / Swagger JSON, Postman Collection v2.1 ou JSON Array Relay.".to_string())
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
        if let (Ok(hn), Ok(hv)) = (
            HeaderName::from_bytes(h.key.as_bytes()),
            HeaderValue::from_str(&h.value),
        ) {
            req_builder = req_builder.header(hn, hv);
        }
    }

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

    state.exchanges.lock().push(exchange.clone());
    let _ = app.emit("relay:response", &intercepted_res);

    Ok(exchange)
}

#[tauri::command]
pub async fn create_ca_certificate() -> Result<GeneratedCa, String> {
    generate_root_ca("Relay Local Root CA")
}

#[tauri::command]
pub async fn export_har(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let exchanges = state.exchanges.lock().clone();
    let json_val = export_to_har(&exchanges);
    serde_json::to_string_pretty(&json_val).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_openapi(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let exchanges = state.exchanges.lock().clone();
    let config = state.config.lock().clone();
    let json_val = export_to_openapi(&exchanges, &config.target_host, config.target_port);
    serde_json::to_string_pretty(&json_val).map_err(|e| e.to_string())
}
