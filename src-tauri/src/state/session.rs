use base64::{
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD},
    Engine as _,
};
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
        list.sort_by_key(|a| std::cmp::Reverse(a.detected_at));
        list
    }

    pub fn clear(&self) {
        self.tokens.write().clear();
    }
}

/// Decodifica o payload de um token JWT sem validar assinatura criptográfica (para inspeção)
pub fn decode_jwt_token(token_str: &str, source: &str) -> Option<ExtractedJwt> {
    let clean_token = token_str
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_matches('`');

    let stripped = if let Some(t) = clean_token.strip_prefix("Bearer ") {
        t.trim()
    } else if let Some(t) = clean_token.strip_prefix("bearer ") {
        t.trim()
    } else if let Some(t) = clean_token.strip_prefix("BEARER ") {
        t.trim()
    } else {
        clean_token
    };

    let parts: Vec<&str> = stripped.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Decodifica Header
    let header_json = decode_base64_json(parts[0]);

    // Decodifica Payload (Claims)
    let claims_json = decode_base64_json(parts[1])?;

    let subject = claims_json
        .get("sub")
        .and_then(|v| v.as_str().map(String::from).or_else(|| v.as_i64().map(|n| n.to_string())));
    let issuer = claims_json
        .get("iss")
        .and_then(|v| v.as_str())
        .map(String::from);
    let expires_at = claims_json.get("exp").and_then(|v| v.as_i64());

    Some(ExtractedJwt {
        token: stripped.to_string(),
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
    let clean = b64_str.trim().trim_matches('"').trim_matches('\'');
    let unpadded = clean.trim_end_matches('=');
    let decoded_bytes = URL_SAFE_NO_PAD
        .decode(unpadded)
        .or_else(|_| STANDARD_NO_PAD.decode(unpadded))
        .or_else(|_| URL_SAFE.decode(clean))
        .or_else(|_| STANDARD.decode(clean))
        .ok()?;
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
        let val_trimmed = v.trim();

        if key_lower == "authorization" || key_lower == "authentication" {
            if let Some(jwt) = decode_jwt_token(val_trimmed, &format!("{}_header_auth", source_prefix)) {
                tokens.push(jwt);
            }
        } else if key_lower == "set-cookie" || key_lower == "cookie" {
            for cookie_part in val_trimmed.split(';') {
                let cookie_trimmed = cookie_part.trim();
                if let Some((cookie_name, cookie_val)) = cookie_trimmed.split_once('=') {
                    let c_val = cookie_val.trim();
                    if (c_val.starts_with("ey") || c_val.contains('.')) && c_val.len() > 20 {
                        if let Some(jwt) = decode_jwt_token(
                            c_val,
                            &format!("{}_cookie_{}", source_prefix, cookie_name.trim().to_lowercase()),
                        ) {
                            tokens.push(jwt);
                        }
                    }
                }
            }
        } else if key_lower == "x-access-token"
            || key_lower == "x-auth-token"
            || key_lower == "jwt"
            || key_lower == "token"
            || key_lower == "access-token"
            || key_lower == "id-token"
        {
            if let Some(jwt) = decode_jwt_token(val_trimmed, &format!("{}_header_{}", source_prefix, key_lower)) {
                tokens.push(jwt);
            }
        } else if val_trimmed.starts_with("eyJ") && val_trimmed.contains('.') {
            if let Some(jwt) = decode_jwt_token(val_trimmed, &format!("{}_header_{}", source_prefix, key_lower)) {
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

    if tokens.is_empty() && body_str.contains("ey") {
        scan_raw_text_for_jwts(body_str, source_prefix, &mut tokens);
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
                    let s_trimmed = s.trim();
                    if (s_trimmed.starts_with("ey")
                        || s_trimmed.starts_with("Bearer ey")
                        || s_trimmed.starts_with("bearer ey")
                        || s_trimmed.starts_with("BEARER ey"))
                        && s_trimmed.contains('.')
                    {
                        if let Some(jwt) =
                            decode_jwt_token(s_trimmed, &format!("{}_field_{}", source_prefix, k))
                        {
                            if !tokens.iter().any(|t| t.token == jwt.token) {
                                tokens.push(jwt);
                            }
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

fn scan_raw_text_for_jwts(
    text: &str,
    source_prefix: &str,
    tokens: &mut Vec<ExtractedJwt>,
) {
    for word in text.split(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ';' || c == ',') {
        let w = word.trim();
        if w.starts_with("eyJ") && w.split('.').count() == 3 {
            if let Some(jwt) = decode_jwt_token(w, &format!("{}_raw_body", source_prefix)) {
                if !tokens.iter().any(|t| t.token == jwt.token) {
                    tokens.push(jwt);
                }
            }
        }
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

    #[test]
    fn test_extract_jwts_from_cookies_and_body() {
        let test_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyXzEyMzQ1IiwibmFtZSI6IlRhY2lvIiwicm9sZSI6ImFkbWluIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE5OTk5OTk5OTl9.signature_aqui";

        // Cookie
        let headers = vec![
            ("set-cookie".to_string(), format!("access_token={}; Path=/; HttpOnly", test_token)),
        ];
        let extracted_cookie = extract_jwts_from_headers(&headers, "response");
        assert_eq!(extracted_cookie.len(), 1);
        assert_eq!(extracted_cookie[0].subject, Some("user_12345".to_string()));

        // Body com Bearer
        let body_json = format!(r#"{{"data": {{"token": "Bearer {}"}}}}"#, test_token);
        let extracted_body = extract_jwts_from_body(&body_json, "response");
        assert_eq!(extracted_body.len(), 1);
        assert_eq!(extracted_body[0].subject, Some("user_12345".to_string()));
    }
}
