use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::recorder::{
    HeaderEntry, HttpExchange, InterceptedRequest, InterceptedResponse, ProxyConfig,
};
use crate::commands::AppState;
use crate::state::{extract_jwts_from_body, extract_jwts_from_headers};

pub struct ProxyServer {
    config: ProxyConfig,
    shutdown_tx: Option<watch::Sender<bool>>,
}

impl ProxyServer {
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config,
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self, app_handle: AppHandle) -> Result<(), String> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.listen_port));
        let listener = TcpListener::bind(addr).await.map_err(|e| e.to_string())?;

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        self.shutdown_tx = Some(shutdown_tx);

        let app = Arc::new(app_handle);

        info!("Proxy iniciado com sucesso em http://{}", addr);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_res = listener.accept() => {
                        match accept_res {
                            Ok((stream, _remote_addr)) => {
                                let io = TokioIo::new(stream);
                                let app_clone = Arc::clone(&app);

                                tokio::spawn(async move {
                                    let service = service_fn(move |req| {
                                        let app = Arc::clone(&app_clone);
                                        async move {
                                            handle_proxy_request(req, app).await
                                        }
                                    });

                                    if let Err(err) = http1::Builder::new()
                                        .serve_connection(io, service)
                                        .await
                                    {
                                        warn!("Conexão HTTP encerrada com erro: {:?}", err);
                                    }
                                });
                            }
                            Err(err) => {
                                error!("Erro no listener accept: {:?}", err);
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            info!("Encerrando servidor proxy listener.");
                            break;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }
    }
}

pub struct RouteTargetResolution {
    pub host: String,
    pub port: u16,
    pub latency_ms: u64,
    pub is_mock: bool,
    pub mock_status: u16,
    pub mock_body: Option<String>,
}

pub fn resolve_route_target_full(uri_path: &str, config: &ProxyConfig) -> RouteTargetResolution {
    let clean_path = uri_path.split('?').next().unwrap_or(uri_path);

    for rule in &config.routes {
        let prefix = rule.path_prefix.trim_end_matches('*');
        if clean_path.starts_with(prefix) {
            let host = rule
                .target_host
                .as_ref()
                .cloned()
                .unwrap_or_else(|| config.target_host.clone());
            let latency = rule.latency_ms.unwrap_or(config.latency_ms);
            return RouteTargetResolution {
                host,
                port: rule.target_port,
                latency_ms: latency,
                is_mock: rule.is_mock,
                mock_status: rule.mock_status_code.unwrap_or(200),
                mock_body: rule.mock_body.clone(),
            };
        }
    }

    RouteTargetResolution {
        host: config.target_host.clone(),
        port: config.target_port,
        latency_ms: config.latency_ms,
        is_mock: false,
        mock_status: 200,
        mock_body: None,
    }
}

pub fn resolve_route_target(uri_path: &str, config: &ProxyConfig) -> (String, u16, u64) {
    let res = resolve_route_target_full(uri_path, config);
    (res.host, res.port, res.latency_ms)
}

async fn handle_proxy_request(
    req: Request<hyper::body::Incoming>,
    app: Arc<AppHandle>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
    let req_id = Uuid::new_v4().to_string();
    let start_time = Instant::now();
    let timestamp = chrono::Utc::now().timestamp_millis();

    // Lê a configuração dinâmica em runtime
    let config = if let Some(state) = app.try_state::<Arc<AppState>>() {
        state.config.lock().clone()
    } else {
        ProxyConfig::default()
    };

    let (parts, incoming_body) = req.into_parts();

    // Leitura assíncrona do payload sem bloquear o stream de rede
    let body_bytes = match incoming_body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };

    let mut headers_vec = Vec::new();
    let mut header_tuples = Vec::new();
    for (k, v) in parts.headers.iter() {
        let key_str = k.as_str().to_string();
        let val_str = v.to_str().unwrap_or_default().to_string();
        headers_vec.push(HeaderEntry {
            key: key_str.clone(),
            value: val_str.clone(),
        });
        header_tuples.push((key_str, val_str));
    }

    let body_str = String::from_utf8(body_bytes.to_vec()).ok();

    let uri_string = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or_else(|| parts.uri.path())
        .to_string();

    let intercepted_req = InterceptedRequest {
        id: req_id.clone(),
        timestamp,
        method: parts.method.to_string(),
        uri: uri_string.clone(),
        headers: headers_vec.clone(),
        body: body_str.clone(),
        size_bytes: body_bytes.len(),
    };

    let exchange = HttpExchange {
        id: req_id.clone(),
        request: intercepted_req,
        response: None,
        status: "pending".to_string(),
        error: None,
    };

    // Auto-extração de JWT nos cabeçalhos da Requisição
    if config.auto_extract_jwt {
        let req_jwts = extract_jwts_from_headers(&header_tuples, "request");
        if let Some(state) = app.try_state::<Arc<AppState>>() {
            for jwt in req_jwts {
                state.session.insert_jwt(jwt.clone());
                let _ = app.emit("relay:jwt", &jwt);
            }
        }
    }

    // Armazena no estado compartilhado em memória
    if let Some(state) = app.try_state::<Arc<AppState>>() {
        state.exchanges.lock().push(exchange.clone());
    }

    let _ = app.emit("relay:request", &exchange);

    // Resolução de Rota Dinâmica
    let route_res = resolve_route_target_full(&uri_string, &config);

    // Injeção de latência dinâmica configurada com Jitter
    let total_delay_ms = calculate_delay(route_res.latency_ms, config.jitter_ms);
    if total_delay_ms > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(total_delay_ms)).await;
    }

    // Rota de Mock Local
    if route_res.is_mock {
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let mock_body_str = route_res
            .mock_body
            .unwrap_or_else(|| r#"{"mock": true, "message": "Relay Mock Engine"}"#.to_string());
        let mock_bytes = Bytes::from(mock_body_str.clone());
        let status = StatusCode::from_u16(route_res.mock_status).unwrap_or(StatusCode::OK);

        let intercepted_res = InterceptedResponse {
            id: Uuid::new_v4().to_string(),
            request_id: req_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            status_code: status.as_u16(),
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
            body: Some(mock_body_str),
            size_bytes: mock_bytes.len(),
            duration_ms,
        };

        if let Some(state) = app.try_state::<Arc<AppState>>() {
            let mut exchs = state.exchanges.lock();
            if let Some(item) = exchs.iter_mut().find(|e| e.id == req_id) {
                item.response = Some(intercepted_res.clone());
                item.status = "completed".to_string();
            }
        }

        let _ = app.emit("relay:response", &intercepted_res);

        let full_body = Full::new(mock_bytes).map_err(|never| match never {});
        return Ok(Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .header("x-relay-mock", "true")
            .body(BoxBody::new(full_body))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(BoxBody::new(Empty::new().map_err(|never| match never {})))
                    .unwrap()
            }));
    }

    // Simulação de Falhas Controladas (Chaos Failure Injection)
    if should_simulate_failure(config.simulate_failure_rate) {
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let fail_status = StatusCode::from_u16(config.failure_status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let simulated_body_str = format!(
            r#"{{"error": "Simulated Chaos Failure", "statusCode": {}, "simulated": true, "durationMs": {}}}"#,
            fail_status.as_u16(),
            duration_ms
        );
        let simulated_bytes = Bytes::from(simulated_body_str.clone());

        let simulated_res = InterceptedResponse {
            id: Uuid::new_v4().to_string(),
            request_id: req_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            status_code: fail_status.as_u16(),
            headers: vec![
                HeaderEntry {
                    key: "content-type".to_string(),
                    value: "application/json".to_string(),
                },
                HeaderEntry {
                    key: "x-relay-chaos".to_string(),
                    value: "simulated-failure".to_string(),
                },
            ],
            body: Some(simulated_body_str),
            size_bytes: simulated_bytes.len(),
            duration_ms,
        };

        if let Some(state) = app.try_state::<Arc<AppState>>() {
            let mut exchs = state.exchanges.lock();
            if let Some(item) = exchs.iter_mut().find(|e| e.id == req_id) {
                item.response = Some(simulated_res.clone());
                item.status = "completed".to_string();
            }
        }

        let _ = app.emit("relay:response", &simulated_res);

        let full_body = Full::new(simulated_bytes).map_err(|never| match never {});
        return Ok(Response::builder()
            .status(fail_status)
            .header("content-type", "application/json")
            .header("x-relay-chaos", "simulated-failure")
            .body(BoxBody::new(full_body))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(BoxBody::new(Empty::new().map_err(|never| match never {})))
                    .unwrap()
            }));
    }

    // Repasse transparente para o upstream real
    match forward_to_upstream(
        &route_res.host,
        route_res.port,
        &parts.method,
        &uri_string,
        &headers_vec,
        body_bytes,
    )
    .await
    {
        Ok((status, res_headers, res_bytes)) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let res_body_str = String::from_utf8(res_bytes.to_vec()).ok();

            let intercepted_res = InterceptedResponse {
                id: Uuid::new_v4().to_string(),
                request_id: req_id.clone(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                status_code: status.as_u16(),
                headers: res_headers.clone(),
                body: res_body_str.clone(),
                size_bytes: res_bytes.len(),
                duration_ms,
            };

            // Auto-captura JWT em respostas
            if config.auto_extract_jwt {
                let res_header_tuples: Vec<(String, String)> = res_headers
                    .iter()
                    .map(|h| (h.key.clone(), h.value.clone()))
                    .collect();

                let mut res_jwts = extract_jwts_from_headers(&res_header_tuples, "response_header");
                if let Some(ref body_text) = res_body_str {
                    let body_jwts = extract_jwts_from_body(body_text, "response_body");
                    res_jwts.extend(body_jwts);
                }

                if let Some(state) = app.try_state::<Arc<AppState>>() {
                    for jwt in res_jwts {
                        state.session.insert_jwt(jwt.clone());
                        let _ = app.emit("relay:jwt", &jwt);
                    }
                }
            }

            if let Some(state) = app.try_state::<Arc<AppState>>() {
                let mut exchs = state.exchanges.lock();
                if let Some(item) = exchs.iter_mut().find(|e| e.id == req_id) {
                    item.response = Some(intercepted_res.clone());
                    item.status = "completed".to_string();
                }
            }

            let _ = app.emit("relay:response", &intercepted_res);

            let mut builder = Response::builder().status(status);
            for h in &res_headers {
                if !h.key.eq_ignore_ascii_case("transfer-encoding")
                    && !h.key.eq_ignore_ascii_case("connection")
                {
                    if let Ok(name) = hyper::header::HeaderName::from_bytes(h.key.as_bytes()) {
                        if let Ok(val) = hyper::header::HeaderValue::from_str(&h.value) {
                            builder = builder.header(name, val);
                        }
                    }
                }
            }

            let full_body = Full::new(res_bytes).map_err(|never| match never {});
            Ok(builder.body(BoxBody::new(full_body)).unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(BoxBody::new(Empty::new().map_err(|never| match never {})))
                    .unwrap()
            }))
        }
        Err(err_msg) => {
            if let Some(state) = app.try_state::<Arc<AppState>>() {
                let mut exchs = state.exchanges.lock();
                if let Some(item) = exchs.iter_mut().find(|e| e.id == req_id) {
                    item.status = "failed".to_string();
                    item.error = Some(err_msg.clone());
                }
            }

            let _ = app.emit(
                "relay:error",
                serde_json::json!({
                    "requestId": req_id,
                    "error": err_msg,
                }),
            );

            let error_payload = format!(
                r#"{{"error": "Relay Proxy Error: Falha ao conectar com o upstream: {}"}}"#,
                err_msg
            );
            let full_body = Full::new(Bytes::from(error_payload)).map_err(|never| match never {});

            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("content-type", "application/json")
                .body(BoxBody::new(full_body))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(BoxBody::new(Empty::new().map_err(|never| match never {})))
                        .unwrap()
                }))
        }
    }
}

pub fn calculate_delay(base_latency_ms: u64, jitter_ms: u64) -> u64 {
    if base_latency_ms == 0 && jitter_ms == 0 {
        return 0;
    }
    if jitter_ms == 0 {
        return base_latency_ms;
    }
    let jitter_offset = fastrand::i64(-(jitter_ms as i64)..=(jitter_ms as i64));
    let calculated = (base_latency_ms as i64) + jitter_offset;
    if calculated < 0 {
        0
    } else {
        calculated as u64
    }
}

pub fn should_simulate_failure(failure_rate: f32) -> bool {
    if failure_rate <= 0.0 {
        return false;
    }
    if failure_rate >= 1.0 {
        return true;
    }
    fastrand::f32() < failure_rate
}

async fn forward_to_upstream(
    target_host: &str,
    target_port: u16,
    method: &hyper::Method,
    uri_path: &str,
    headers: &[HeaderEntry],
    body_bytes: Bytes,
) -> Result<(StatusCode, Vec<HeaderEntry>, Bytes), String> {
    let target_addr = format!("{}:{}", target_host, target_port);

    let stream = TcpStream::connect(&target_addr)
        .await
        .map_err(|e| format!("Falha ao conectar com o upstream ({}): {}", target_addr, e))?;

    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| format!("Erro no handshake com upstream: {}", e))?;

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            warn!("Conexão com upstream finalizada: {:?}", err);
        }
    });

    let mut builder = Request::builder().method(method).uri(uri_path);

    for h in headers {
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
        .map_err(|e| format!("Falha ao construir requisicao upstream: {}", e))?;

    let res = sender
        .send_request(req)
        .await
        .map_err(|e| format!("Falha ao enviar requisicao upstream: {}", e))?;
    let status = res.status();

    let mut res_headers = Vec::new();
    for (k, v) in res.headers().iter() {
        res_headers.push(HeaderEntry {
            key: k.as_str().to_string(),
            value: v.to_str().unwrap_or_default().to_string(),
        });
    }

    let res_body_bytes = res
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("Falha ao coletar resposta do upstream: {}", e))?
        .to_bytes();

    Ok((status, res_headers, res_body_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::recorder::RouteRule;
    use http_body_util::Full;
    use hyper::service::service_fn;
    use hyper::{Response, StatusCode};
    use hyper_util::rt::TokioIo;
    use std::convert::Infallible;
    use tokio::net::TcpListener;

    #[test]
    fn test_calculate_delay_jitter() {
        let base = 100;
        let jitter = 20;
        for _ in 0..50 {
            let d = calculate_delay(base, jitter);
            assert!(d >= 80 && d <= 120);
        }
    }

    #[test]
    fn test_should_simulate_failure() {
        assert!(!should_simulate_failure(0.0));
        assert!(should_simulate_failure(1.0));
    }

    #[test]
    fn test_resolve_route_target() {
        let config = ProxyConfig {
            listen_port: 8080,
            target_host: "127.0.0.1".to_string(),
            target_port: 3000,
            latency_ms: 50,
            jitter_ms: 0,
            simulate_failure_rate: 0.0,
            failure_status_code: 500,
            auto_extract_jwt: true,
            routes: vec![
                RouteRule {
                    path_prefix: "/api/v1/auth".to_string(),
                    target_host: Some("auth-service".to_string()),
                    target_port: 4000,
                    latency_ms: Some(10),
                    is_mock: false,
                    mock_status_code: None,
                    mock_body: None,
                },
                RouteRule {
                    path_prefix: "/mock/users".to_string(),
                    target_host: None,
                    target_port: 3000,
                    latency_ms: None,
                    is_mock: true,
                    mock_status_code: Some(201),
                    mock_body: Some("{}".to_string()),
                },
            ],
        };

        let (host, port, lat) = resolve_route_target("/api/v1/auth/login", &config);
        assert_eq!(host, "auth-service");
        assert_eq!(port, 4000);
        assert_eq!(lat, 10);

        let res = resolve_route_target_full("/mock/users", &config);
        assert!(res.is_mock);
        assert_eq!(res.mock_status, 201);
    }

    #[tokio::test]
    async fn test_forward_to_upstream_success() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);
            http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|_req| async {
                        Ok::<_, Infallible>(
                            Response::builder()
                                .status(StatusCode::OK)
                                .header("x-custom", "relay-test")
                                .body(Full::new(Bytes::from("hello from upstream")))
                                .unwrap(),
                        )
                    }),
                )
                .await
                .unwrap();
        });

        let (status, headers, body) = forward_to_upstream(
            "127.0.0.1",
            port,
            &hyper::Method::GET,
            "/test",
            &[],
            Bytes::new(),
        )
        .await
        .unwrap();

        assert_eq!(status, StatusCode::OK);
        assert!(headers
            .iter()
            .any(|h| h.key == "x-custom" && h.value == "relay-test"));
        assert_eq!(body.as_ref(), b"hello from upstream");
    }
}
