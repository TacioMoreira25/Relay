use super::recorder::HttpExchange;
use serde_json::json;

/// Converte um vetor de HttpExchange para a especificação oficial HAR (HTTP Archive 1.2)
pub fn export_to_har(exchanges: &[HttpExchange]) -> serde_json::Value {
    let mut entries = Vec::new();

    for exchange in exchanges {
        let req = &exchange.request;
        let mut req_headers = Vec::new();
        for h in &req.headers {
            req_headers.push(json!({
                "name": h.key,
                "value": h.value
            }));
        }

        let post_data = req.body.as_ref().map(|b| {
            json!({
                "mimeType": "application/json",
                "text": b
            })
        });

        let (res_status, res_status_text, res_headers, res_content, duration_ms) =
            if let Some(ref res) = exchange.response {
                let mut hdrs = Vec::new();
                for h in &res.headers {
                    hdrs.push(json!({
                        "name": h.key,
                        "value": h.value
                    }));
                }

                let content = json!({
                    "size": res.size_bytes,
                    "mimeType": "application/json",
                    "text": res.body.clone().unwrap_or_default()
                });

                (res.status_code, "OK", hdrs, content, res.duration_ms)
            } else {
                (0, "Pending / Failed", Vec::new(), json!({"size": 0, "text": ""}), 0)
            };

        entries.push(json!({
            "startedDateTime": chrono::DateTime::from_timestamp_millis(req.timestamp)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            "time": duration_ms,
            "request": {
                "method": req.method,
                "url": req.uri,
                "httpVersion": "HTTP/1.1",
                "headers": req_headers,
                "queryString": [],
                "postData": post_data,
                "headersSize": -1,
                "bodySize": req.size_bytes
            },
            "response": {
                "status": res_status,
                "statusText": res_status_text,
                "httpVersion": "HTTP/1.1",
                "headers": res_headers,
                "content": res_content,
                "redirectURL": "",
                "headersSize": -1,
                "bodySize": exchange.response.as_ref().map(|r| r.size_bytes).unwrap_or(0)
            },
            "cache": {},
            "timings": {
                "send": 0,
                "wait": duration_ms,
                "receive": 0
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
            "entries": entries
        }
    })
}

/// Gera uma especificação OpenAPI 3.0.3 a partir das rotas capturadas
pub fn export_to_openapi(exchanges: &[HttpExchange], target_host: &str, target_port: u16) -> serde_json::Value {
    let mut paths_obj = serde_json::Map::new();

    for exchange in exchanges {
        let raw_path = &exchange.request.uri;
        let clean_path = raw_path.split('?').next().unwrap_or("/");
        let method = exchange.request.method.to_lowercase();

        let path_item = paths_obj
            .entry(clean_path.to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

        if let Some(path_map) = path_item.as_object_mut() {
            let status_str = exchange
                .response
                .as_ref()
                .map(|r| r.status_code.to_string())
                .unwrap_or_else(|| "200".to_string());

            let responses_obj = json!({
                status_str: {
                    "description": "Intercepted response from Relay Proxy",
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object"
                            }
                        }
                    }
                }
            });

            path_map.insert(
                method,
                json!({
                    "summary": format!("Auto-generated endpoint for {}", clean_path),
                    "responses": responses_obj
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
    use crate::proxy::recorder::InterceptedRequest;

    #[test]
    fn test_export_to_har() {
        let exchange = HttpExchange {
            id: "test-1".to_string(),
            request: InterceptedRequest {
                id: "test-1".to_string(),
                timestamp: 1600000000000,
                method: "GET".to_string(),
                uri: "/api/v1/users".to_string(),
                headers: vec![],
                body: None,
                size_bytes: 0,
            },
            response: None,
            status: "pending".to_string(),
            error: None,
        };

        let har = export_to_har(&[exchange]);
        assert_eq!(har["log"]["version"], "1.2");
        assert_eq!(har["log"]["entries"][0]["request"]["method"], "GET");
    }

    #[test]
    fn test_export_to_openapi() {
        let exchange = HttpExchange {
            id: "test-2".to_string(),
            request: InterceptedRequest {
                id: "test-2".to_string(),
                timestamp: 1600000000000,
                method: "POST".to_string(),
                uri: "/api/v1/login?ref=app".to_string(),
                headers: vec![],
                body: Some("{}".to_string()),
                size_bytes: 2,
            },
            response: None,
            status: "completed".to_string(),
            error: None,
        };

        let openapi = export_to_openapi(&[exchange], "127.0.0.1", 3000);
        assert_eq!(openapi["openapi"], "3.0.3");
        assert!(openapi["paths"]["/api/v1/login"]["post"].is_object());
    }
}
