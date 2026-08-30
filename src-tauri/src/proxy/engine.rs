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
use tauri::{AppHandle, Emitter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::recorder::{HeaderEntry, HttpExchange, InterceptedRequest, InterceptedResponse, ProxyConfig};

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

        let config = Arc::new(self.config.clone());
        let app = Arc::new(app_handle);

        info!("Proxy iniciado em http://{}", addr);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_res = listener.accept() => {
                        match accept_res {
                            Ok((stream, _remote_addr)) => {
                                let io = TokioIo::new(stream);
                                let app_clone = Arc::clone(&app);
                                let config_clone = Arc::clone(&config);

                                tokio::spawn(async move {
                                    let service = service_fn(move |req| {
                                        let app = Arc::clone(&app_clone);
                                        let cfg = Arc::clone(&config_clone);
                                        async move {
                                            handle_proxy_request(req, app, cfg).await
                                        }
                                    });

                                    if let Err(err) = http1::Builder::new()
                                        .preserve_header_case(true)
                                        .title_case_headers(true)
                                        .serve_connection(io, service)
                                        .await
                                    {
                                        warn!("Erro na conexao de proxy: {:?}", err);
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

async fn handle_proxy_request(
    req: Request<hyper::body::Incoming>,
    app: Arc<AppHandle>,
    config: Arc<ProxyConfig>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
    let req_id = Uuid::new_v4().to_string();
    let start_time = Instant::now();
    let timestamp = chrono::Utc::now().timestamp_millis();

    let (parts, incoming_body) = req.into_parts();

    // Leitura assíncrona do payload sem reter bloqueios
    let body_bytes = match incoming_body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };

    let mut headers_vec = Vec::new();
    for (k, v) in parts.headers.iter() {
        headers_vec.push(HeaderEntry {
            key: k.as_str().to_string(),
            value: v.to_str().unwrap_or_default().to_string(),
        });
    }

    let body_str = String::from_utf8(body_bytes.to_vec()).ok();

    let intercepted_req = InterceptedRequest {
        id: req_id.clone(),
        timestamp,
        method: parts.method.to_string(),
        uri: parts.uri.to_string(),
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

    let _ = app.emit("relay:request", exchange);

    // Injeção de latência simulada
    if config.latency_ms > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(config.latency_ms)).await;
    }

    // Encaminha requisição para o alvo (Upstream)
    let target_uri = format!("http://{}:{}{}", config.target_host, config.target_port, parts.uri);
    
    let forward_res = forward_to_upstream(
        &parts.method,
        &target_uri,
        &headers_vec,
        body_bytes,
        &config,
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
                body: res_body_str,
                size_bytes: res_bytes.len(),
                duration_ms,
            };

            let _ = app.emit("relay:response", intercepted_res);

            let mut resp = Response::builder().status(res_status);
            for h in res_headers {
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
            let _ = app.emit("relay:error", serde_json::json!({
                "requestId": req_id,
                "error": err_msg,
            }));

            let full_body = Full::new(Bytes::from(format!("Relay Proxy Error: {}", err_msg)))
                .map_err(|never| match never {});

            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(BoxBody::new(full_body))
                .unwrap())
        }
    }
}

async fn forward_to_upstream(
    method: &hyper::Method,
    target_url: &str,
    headers: &[HeaderEntry],
    body_bytes: Bytes,
    config: &ProxyConfig,
) -> Result<(StatusCode, Vec<HeaderEntry>, Bytes), String> {
    let target_addr = format!("{}:{}", config.target_host, config.target_port);
    let stream = TcpStream::connect(&target_addr).await.map_err(|e| format!("Falha de conexao com upstream: {}", e))?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| format!("Erro no handshake com upstream: {}", e))?;

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            warn!("Conexao com upstream fechou com erro: {:?}", err);
        }
    });

    let mut builder = Request::builder()
        .method(method)
        .uri(target_url);

    for h in headers {
        if !h.key.eq_ignore_ascii_case("host") {
            if let Ok(name) = hyper::header::HeaderName::from_bytes(h.key.as_bytes()) {
                if let Ok(val) = hyper::header::HeaderValue::from_str(&h.value) {
                    builder = builder.header(name, val);
                }
            }
        }
    }
    builder = builder.header("host", target_addr);

    let req = builder
        .body(Full::new(body_bytes))
        .map_err(|e| format!("Falha ao construir requisicao upstream: {}", e))?;

    let res = sender.send_request(req).await.map_err(|e| format!("Falha ao enviar requisicao upstream: {}", e))?;
    let status = res.status();

    let mut res_headers = Vec::new();
    for (k, v) in res.headers().iter() {
        res_headers.push(HeaderEntry {
            key: k.as_str().to_string(),
            value: v.to_str().unwrap_or_default().to_string(),
        });
    }

    let res_body_bytes = res.into_body().collect().await.map_err(|e| format!("Falha ao coletar body upstream: {}", e))?.to_bytes();

    Ok((status, res_headers, res_body_bytes))
}
