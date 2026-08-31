use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedJwt {
    pub token: String,
    pub source: String, // "request_header_authorization", "response_header_set_cookie", "response_body"
    pub detected_at: i64,
    pub claims: Option<serde_json::Value>,
    pub header: Option<serde_json::Value>,
    pub subject: Option<String>,
    pub issuer: Option<String>,
    pub expires_at: Option<i64>,
}

#[derive(Clone, Default)]
pub struct SessionState {
    tokens: Arc<RwLock<HashMap<String, ExtractedJwt>>>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert_jwt(&self, jwt: ExtractedJwt) {
        let mut map = self.tokens.write();
        map.insert(jwt.token.clone(), jwt);
    }

    pub fn list_jwts(&self) -> Vec<ExtractedJwt> {
        let map = self.tokens.read();
        let mut list: Vec<ExtractedJwt> = map.values().cloned().collect();
        list.sort_by(|a, b| b.detected_at.cmp(&a.detected_at));
        list
    }

    pub fn clear(&self) {
        self.tokens.write().clear();
    }
}

/// Decodifica o payload de um token JWT sem validar assinatura criptográfica (para inspeção)
pub fn decode_jwt_token(token_str: &str, source: &str) -> Option<ExtractedJwt> {
    let parts: Vec<&str> = token_str.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Decodifica Header
    let header_json = decode_base64_json(parts[0]);

    // Decodifica Payload (Claims)
    let claims_json = decode_base64_json(parts[1])?;

    let subject = claims_json
        .get("sub")
        .and_then(|v| v.as_str())
        .map(String::from);
    let issuer = claims_json
        .get("iss")
        .and_then(|v| v.as_str())
        .map(String::from);
    let expires_at = claims_json.get("exp").and_then(|v| v.as_i64());

    Some(ExtractedJwt {
        token: token_str.to_string(),
        source: source.to_string(),
        detected_at: chrono::Utc::now().timestamp_millis(),
        claims: Some(claims_json),
        header: header_json,
        subject,
        issuer,
        expires_at,
    })
}

fn decode_base64_json(b64_str: &str) -> Option<serde_json::Value> {
    let unpadded = b64_str.trim_end_matches('=');
    let decoded_bytes = URL_SAFE_NO_PAD.decode(unpadded).ok()?;
    let json_str = String::from_utf8(decoded_bytes).ok()?;
    serde_json::from_str(&json_str).ok()
}

/// Varre uma lista de tuplas de cabeçalhos procurando tokens Bearer ou JWTs
pub fn extract_jwts_from_headers(
    headers: &[(String, String)],
    source_prefix: &str,
) -> Vec<ExtractedJwt> {
    let mut tokens = Vec::new();

    for (k, v) in headers {
        let key_lower = k.to_lowercase();
        if key_lower == "authorization" {
            if let Some(token_part) = v
                .strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
            {
                if let Some(jwt) =
                    decode_jwt_token(token_part.trim(), &format!("{}_header_auth", source_prefix))
                {
                    tokens.push(jwt);
                }
            }
        } else if key_lower == "x-access-token"
            || key_lower == "x-auth-token"
            || key_lower == "jwt"
            || key_lower == "token"
        {
            if let Some(jwt) =
                decode_jwt_token(v.trim(), &format!("{}_header_{}", source_prefix, key_lower))
            {
                tokens.push(jwt);
            }
        }
    }

    tokens
}

/// Varre campos comuns de resposta JSON (ex: login) procurando tokens JWT
pub fn extract_jwts_from_body(body_str: &str, source_prefix: &str) -> Vec<ExtractedJwt> {
    let mut tokens = Vec::new();

    if let Ok(val) = serde_json::from_str::<serde_json::Value>(body_str) {
        scan_json_for_jwts(&val, source_prefix, &mut tokens);
    }

    tokens
}

fn scan_json_for_jwts(
    val: &serde_json::Value,
    source_prefix: &str,
    tokens: &mut Vec<ExtractedJwt>,
) {
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                if let serde_json::Value::String(s) = v {
                    if s.starts_with("ey") && s.contains('.') {
                        if let Some(jwt) =
                            decode_jwt_token(s, &format!("{}_field_{}", source_prefix, k))
                        {
                            tokens.push(jwt);
                        }
                    }
                } else {
                    scan_json_for_jwts(v, source_prefix, tokens);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                scan_json_for_jwts(item, source_prefix, tokens);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_jwt_token() {
        let test_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyXzEyMzQ1IiwibmFtZSI6IlRhY2lvIiwicm9sZSI6ImFkbWluIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE5OTk5OTk5OTl9.signature_aqui";

        let decoded = decode_jwt_token(test_token, "test_source");
        assert!(decoded.is_some());

        let jwt = decoded.unwrap();
        assert_eq!(jwt.subject, Some("user_12345".to_string()));
        assert_eq!(jwt.expires_at, Some(1999999999));
        assert!(jwt.claims.is_some());
    }

    #[test]
    fn test_extract_jwts_from_headers() {
        let test_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyXzEyMzQ1IiwibmFtZSI6IlRhY2lvIiwicm9sZSI6ImFkbWluIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE5OTk5OTk5OTl9.signature_aqui";

        let headers = vec![
            (
                "authorization".to_string(),
                format!("Bearer {}", test_token),
            ),
            ("content-type".to_string(), "application/json".to_string()),
        ];

        let extracted = extract_jwts_from_headers(&headers, "request");
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].subject, Some("user_12345".to_string()));
    }
}
