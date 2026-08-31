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
                                        .preserve_header_case(true)
                                        .title_case_headers(true)
                                        .serve_connection(io, service)
                                        .await
                                    {
                                        warn!("Conexao de cliente proxy finalizada: {:?}", err);
                                    }
                                });
                            }
                            Err(e) => {
                                error!("Erro no listener TCP do proxy: {:?}", e);
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            info!("Sinal de encerramento do proxy recebido.");
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

pub fn resolve_route_target(uri_path: &str, config: &ProxyConfig) -> (String, u16, u64) {
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
            return (host, rule.target_port, latency);
        }
    }

    (config.target_host.clone(), config.target_port, config.latency_ms)
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

    // Resolução de Rota Dinâmica (Suporta qualquer rota e múltiplos microservices)
    let (target_host, target_port, route_latency_ms) =
        resolve_route_target(&uri_string, &config);

    // Injeção de latência dinâmica configurada com Jitter (Chaos Engineering)
    let total_delay_ms = calculate_delay(route_latency_ms, config.jitter_ms);
    if total_delay_ms > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(total_delay_ms)).await;
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
            .unwrap());
    }

    // Encaminhamento transparente para o servidor Upstream
    let path_and_query = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");

    let forward_res = forward_to_upstream(
        &parts.method,
        path_and_query,
        &headers_vec,
        body_bytes,
        &target_host,
        target_port,
    )
    .await;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    match forward_res {
        Ok((res_status, res_headers, res_bytes)) => {
            let res_body_str = String::from_utf8(res_bytes.to_vec()).ok();
            let intercepted_res = InterceptedResponse {
                id: Uuid::new_v4().to_string(),
                request_id: req_id.clone(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                status_code: res_status.as_u16(),
                headers: res_headers.clone(),
                body: res_body_str.clone(),
                size_bytes: res_bytes.len(),
                duration_ms,
            };

            // Auto-extração de JWT nos cabeçalhos e body da Resposta
            if config.auto_extract_jwt {
                let res_header_tuples: Vec<(String, String)> = res_headers
                    .iter()
                    .map(|h| (h.key.clone(), h.value.clone()))
                    .collect();

                let mut res_jwts = extract_jwts_from_headers(&res_header_tuples, "response");
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

            // Atualiza no estado compartilhado
            if let Some(state) = app.try_state::<Arc<AppState>>() {
                let mut exchs = state.exchanges.lock();
                if let Some(item) = exchs.iter_mut().find(|e| e.id == req_id) {
                    item.response = Some(intercepted_res.clone());
                    item.status = "completed".to_string();
                }
            }

            let _ = app.emit("relay:response", &intercepted_res);

            let mut resp = Response::builder().status(res_status);
            for h in res_headers {
                if h.key.eq_ignore_ascii_case("transfer-encoding")
                    || h.key.eq_ignore_ascii_case("connection")
                {
                    continue;
                }
                if let Ok(name) = hyper::header::HeaderName::from_bytes(h.key.as_bytes()) {
                    if let Ok(val) = hyper::header::HeaderValue::from_str(&h.value) {
                        resp = resp.header(name, val);
                    }
                }
            }

            let full_body = Full::new(res_bytes).map_err(|never| match never {});
            Ok(resp.body(BoxBody::new(full_body)).unwrap_or_else(|_| {
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

            let full_body = Full::new(Bytes::from(format!("Relay Proxy Error: {}", err_msg)))
                .map_err(|never| match never {});

            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(BoxBody::new(full_body))
                .unwrap())
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
    let random_jitter = fastrand::u64(0..=jitter_ms);
    base_latency_ms.saturating_add(random_jitter)
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
    method: &hyper::Method,
    path_and_query: &str,
    headers: &[HeaderEntry],
    body_bytes: Bytes,
    target_host: &str,
    target_port: u16,
) -> Result<(StatusCode, Vec<HeaderEntry>, Bytes), String> {
    let target_addr = format!("{}:{}", target_host, target_port);
    let stream = TcpStream::connect(&target_addr)
        .await
        .map_err(|e| format!("Falha ao conectar com o upstream ({}): {}", target_addr, e))?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| format!("Erro no handshake HTTP/1.1 com upstream: {}", e))?;

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            warn!("Conexao com upstream finalizada: {:?}", err);
        }
    });

    let mut builder = Request::builder().method(method).uri(path_and_query);

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
    fn test_resolve_route_target() {
        let mut config = ProxyConfig::default();
        config.target_host = "127.0.0.1".to_string();
        config.target_port = 3000;
        config.latency_ms = 10;

        config.routes.push(RouteRule {
            path_prefix: "/api/v1/auth".to_string(),
            target_host: Some("127.0.0.1".to_string()),
            target_port: 4000,
            latency_ms: Some(50),
        });

        let (_host, port, latency) = resolve_route_target("/api/v1/auth/login", &config);
        assert_eq!(port, 4000);
        assert_eq!(latency, 50);

        let (_def_host, def_port, def_lat) = resolve_route_target("/api/v1/users", &config);
        assert_eq!(def_port, 3000);
        assert_eq!(def_lat, 10);
    }

    #[test]
    fn test_calculate_delay_jitter() {
        let delay = calculate_delay(100, 50);
        assert!(delay >= 100 && delay <= 150);

        assert_eq!(calculate_delay(0, 0), 0);
        assert_eq!(calculate_delay(200, 0), 200);
    }

    #[test]
    fn test_should_simulate_failure() {
        assert!(!should_simulate_failure(0.0));
        assert!(should_simulate_failure(1.0));
    }

    #[tokio::test]
    async fn test_forward_to_upstream_success() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        let port = local_addr.port();

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let io = TokioIo::new(stream);
                let service = service_fn(|_req| async {
                    Ok::<_, Infallible>(
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("x-mock-test", "passed")
                            .body(Full::new(Bytes::from("hello from test upstream")))
                            .unwrap(),
                    )
                });
                let _ = hyper::server::conn::http1::Builder::new()
                    .serve_connection(io, service)
                    .await;
            }
        });

        let method = hyper::Method::GET;
        let headers = vec![HeaderEntry {
            key: "user-agent".to_string(),
            value: "relay-unit-test".to_string(),
        }];

        let result = forward_to_upstream(
            &method,
            "/api/test",
            &headers,
            Bytes::new(),
            "127.0.0.1",
            port,
        )
        .await;

        assert!(result.is_ok());
        let (status, resp_headers, body) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(resp_headers
            .iter()
            .any(|h| h.key == "x-mock-test" && h.value == "passed"));
        assert_eq!(body, Bytes::from("hello from test upstream"));
    }
}
