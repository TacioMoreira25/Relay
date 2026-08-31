use serde_json::json;

use super::recorder::HttpExchange;

/// Exporta uma lista de HttpExchange para a especificação oficial HAR 1.2 (HTTP Archive)
pub fn export_to_har(exchanges: &[HttpExchange]) -> serde_json::Value {
    let mut entries = Vec::new();

    for exchange in exchanges {
        let req = &exchange.request;

        let req_headers: Vec<serde_json::Value> = req
            .headers
            .iter()
            .map(|h| {
                json!({
                    "name": h.key,
                    "value": h.value,
                })
            })
            .collect();

        let (res_status, res_status_text, res_headers, res_content, duration) =
            if let Some(ref res) = exchange.response {
                let hdrs: Vec<serde_json::Value> = res
                    .headers
                    .iter()
                    .map(|h| {
                        json!({
                            "name": h.key,
                            "value": h.value,
                        })
                    })
                    .collect();

                let content = json!({
                    "size": res.size_bytes,
                    "mimeType": "application/json",
                    "text": res.body.clone().unwrap_or_default(),
                });

                (res.status_code, "OK", hdrs, content, res.duration_ms)
            } else {
                (
                    0,
                    "Pending / Failed",
                    Vec::new(),
                    json!({"size": 0, "text": ""}),
                    0,
                )
            };

        entries.push(json!({
            "startedDateTime": chrono::DateTime::from_timestamp_millis(req.timestamp)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            "time": duration,
            "request": {
                "method": req.method,
                "url": req.uri,
                "httpVersion": "HTTP/1.1",
                "headers": req_headers,
                "queryString": [],
                "postData": {
                    "mimeType": "application/json",
                    "text": req.body.clone().unwrap_or_default(),
                },
                "headersSize": -1,
                "bodySize": req.size_bytes,
            },
            "response": {
                "status": res_status,
                "statusText": res_status_text,
                "httpVersion": "HTTP/1.1",
                "headers": res_headers,
                "content": res_content,
                "redirectURL": "",
                "headersSize": -1,
                "bodySize": exchange.response.as_ref().map(|r| r.size_bytes).unwrap_or(0),
            },
            "cache": {},
            "timings": {
                "send": 0,
                "wait": duration,
                "receive": 0,
            }
        }));
    }

    json!({
        "log": {
            "version": "1.2",
            "creator": {
                "name": "Relay",
                "version": "1.0.0"
            },
            "entries": entries,
        }
    })
}

/// Gera uma especificação OpenAPI 3.0.3 a partir das rotas capturadas
pub fn export_to_openapi(
    exchanges: &[HttpExchange],
    target_host: &str,
    target_port: u16,
) -> serde_json::Value {
    let mut paths_obj = serde_json::Map::new();

    for exchange in exchanges {
        let clean_path = exchange
            .request
            .uri
            .split('?')
            .next()
            .unwrap_or(&exchange.request.uri);
        let method = exchange.request.method.to_lowercase();
        let status = exchange
            .response
            .as_ref()
            .map(|r| r.status_code.to_string())
            .unwrap_or_else(|| "200".to_string());

        let path_entry = paths_obj
            .entry(clean_path.to_string())
            .or_insert_with(|| json!({}));

        if let Some(path_map) = path_entry.as_object_mut() {
            path_map.insert(
                method.clone(),
                json!({
                    "summary": format!("Auto-generated endpoint for {}", clean_path),
                    "responses": {
                        status: {
                            "description": "Intercepted response from Relay Proxy",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object"
                                    }
                                }
                            }
                        }
                    }
                }),
            );
        }
    }

    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Relay Intercepted API Specification",
            "version": "1.0.0",
            "description": "Especificação OpenAPI 3.0 gerada automaticamente a partir do tráfego capturado pelo Relay."
        },
        "servers": [
            {
                "url": format!("http://{}:{}", target_host, target_port),
                "description": "Upstream Target Server"
            }
        ],
        "paths": paths_obj
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::recorder::{HeaderEntry, InterceptedRequest, InterceptedResponse};

    #[test]
    fn test_export_to_har() {
        let exchange = HttpExchange {
            id: "123".to_string(),
            request: InterceptedRequest {
                id: "123".to_string(),
                timestamp: 1700000000000,
                method: "GET".to_string(),
                uri: "/api/v1/users".to_string(),
                headers: vec![HeaderEntry {
                    key: "accept".to_string(),
                    value: "application/json".to_string(),
                }],
                body: None,
                size_bytes: 0,
            },
            response: Some(InterceptedResponse {
                id: "456".to_string(),
                request_id: "123".to_string(),
                timestamp: 1700000000100,
                status_code: 200,
                headers: vec![],
                body: Some("{\"users\": []}".to_string()),
                size_bytes: 13,
                duration_ms: 100,
            }),
            status: "completed".to_string(),
            error: None,
        };

        let har = export_to_har(&[exchange]);
        assert_eq!(har["log"]["version"], "1.2");
        assert_eq!(har["log"]["entries"][0]["request"]["method"], "GET");
        assert_eq!(har["log"]["entries"][0]["response"]["status"], 200);
    }

    #[test]
    fn test_export_to_openapi() {
        let exchange = HttpExchange {
            id: "123".to_string(),
            request: InterceptedRequest {
                id: "123".to_string(),
                timestamp: 1700000000000,
                method: "POST".to_string(),
                uri: "/api/v1/auth/login".to_string(),
                headers: vec![],
                body: Some("{}".to_string()),
                size_bytes: 2,
            },
            response: Some(InterceptedResponse {
                id: "456".to_string(),
                request_id: "123".to_string(),
                timestamp: 1700000000100,
                status_code: 201,
                headers: vec![],
                body: Some("{\"token\": \"abc\"}".to_string()),
                size_bytes: 16,
                duration_ms: 50,
            }),
            status: "completed".to_string(),
            error: None,
        };

        let openapi = export_to_openapi(&[exchange], "127.0.0.1", 3000);
        assert_eq!(openapi["openapi"], "3.0.3");
        assert!(openapi["paths"]["/api/v1/auth/login"]["post"].is_object());
        assert!(openapi["paths"]["/api/v1/auth/login"]["post"]["responses"]["201"].is_object());
    }
}
