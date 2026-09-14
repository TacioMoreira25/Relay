use crate::proxy::recorder::{HeaderEntry, HttpExchange, InterceptedRequest, InterceptedResponse};

/// Sanitiza headers mascarando dados sensíveis
pub fn sanitize_headers(headers: &[HeaderEntry]) -> Vec<HeaderEntry> {
    let sensitive_keys = [
        "authorization",
        "cookie",
        "set-cookie",
        "x-api-key",
        "proxy-authorization",
        "x-auth-token",
        "api-key",
    ];

    headers
        .iter()
        .map(|h| {
            let k_lower = h.key.to_lowercase();
            if sensitive_keys.contains(&k_lower.as_str()) {
                let masked_val = if k_lower == "authorization" && h.value.starts_with("Bearer ") {
                    "Bearer [REDACTED_TOKEN]".to_string()
                } else {
                    "[REDACTED]".to_string()
                };
                HeaderEntry {
                    key: h.key.clone(),
                    value: masked_val,
                }
            } else {
                h.clone()
            }
        })
        .collect()
}

/// Sanitiza parâmetros de query confidenciais em uma URI
pub fn sanitize_uri(uri: &str) -> String {
    if !uri.contains('?') {
        return uri.to_string();
    }

    let parts: Vec<&str> = uri.splitn(2, '?').collect();
    let base = parts[0];
    let query = parts[1];

    let sensitive_params = [
        "token", "secret", "password", "api_key", "key", "auth", "access_token",
    ];

    let sanitized_pairs: Vec<String> = query
        .split('&')
        .map(|pair| {
            let mut kv = pair.splitn(2, '=');
            let key = kv.next().unwrap_or("");
            let val = kv.next();

            if sensitive_params.contains(&key.to_lowercase().as_str()) {
                format!("{}=[REDACTED]", key)
            } else if let Some(v) = val {
                format!("{}={}", key, v)
            } else {
                key.to_string()
            }
        })
        .collect();

    format!("{}?{}", base, sanitized_pairs.join("&"))
}

/// Sanitiza recursivamente valores JSON mascarando campos confidenciais
pub fn sanitize_json_value(val: &mut serde_json::Value) {
    let sensitive_fields = [
        "password",
        "senha",
        "secret",
        "token",
        "access_token",
        "refresh_token",
        "private_key",
        "credit_card",
        "card_number",
        "cvv",
        "cpf",
    ];

    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if sensitive_fields.contains(&k.to_lowercase().as_str()) {
                    *v = serde_json::Value::String("[REDACTED]".to_string());
                } else {
                    sanitize_json_value(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                sanitize_json_value(item);
            }
        }
        _ => {}
    }
}

/// Sanitiza o corpo textual caso seja JSON válido
pub fn sanitize_body(body: Option<&str>) -> Option<String> {
    let raw = body?;
    if let Ok(mut json_val) = serde_json::from_str::<serde_json::Value>(raw) {
        sanitize_json_value(&mut json_val);
        serde_json::to_string_pretty(&json_val).ok()
    } else {
        Some(raw.to_string())
    }
}

/// Sanitiza uma troca HTTP completa para exportação segura em HAR ou OpenAPI
pub fn sanitize_exchange(exchange: &HttpExchange) -> HttpExchange {
    let sanitized_req = InterceptedRequest {
        id: exchange.request.id.clone(),
        timestamp: exchange.request.timestamp,
        method: exchange.request.method.clone(),
        uri: sanitize_uri(&exchange.request.uri),
        headers: sanitize_headers(&exchange.request.headers),
        body: sanitize_body(exchange.request.body.as_deref()),
        size_bytes: exchange.request.size_bytes,
    };

    let sanitized_res = exchange.response.as_ref().map(|res| InterceptedResponse {
        id: res.id.clone(),
        request_id: res.request_id.clone(),
        timestamp: res.timestamp,
        status_code: res.status_code,
        headers: sanitize_headers(&res.headers),
        body: sanitize_body(res.body.as_deref()),
        size_bytes: res.size_bytes,
        duration_ms: res.duration_ms,
    });

    HttpExchange {
        id: exchange.id.clone(),
        request: sanitized_req,
        response: sanitized_res,
        status: exchange.status.clone(),
        error: exchange.error.clone(),
    }
}

/// Sanitiza um lote de trocas HTTP
pub fn sanitize_exchanges(exchanges: &[HttpExchange]) -> Vec<HttpExchange> {
    exchanges.iter().map(sanitize_exchange).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_headers() {
        let headers = vec![
            HeaderEntry {
                key: "Authorization".to_string(),
                value: "Bearer eyJhbGciOi...".to_string(),
            },
            HeaderEntry {
                key: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
        ];

        let sanitized = sanitize_headers(&headers);
        assert_eq!(sanitized[0].value, "Bearer [REDACTED_TOKEN]");
        assert_eq!(sanitized[1].value, "application/json");
    }

    #[test]
    fn test_sanitize_uri_query() {
        let uri = "/api/v1/auth?token=supersecret123&type=client";
        let sanitized = sanitize_uri(uri);
        assert_eq!(sanitized, "/api/v1/auth?token=[REDACTED]&type=client");
    }

    #[test]
    fn test_sanitize_json_body() {
        let body = "{\"email\":\"user@test.com\",\"password\":\"p@ssw0rd\"}";
        let sanitized = sanitize_body(Some(body)).unwrap();
        assert!(sanitized.contains("\"password\": \"[REDACTED]\""));
        assert!(sanitized.contains("\"email\": \"user@test.com\""));
    }
}
