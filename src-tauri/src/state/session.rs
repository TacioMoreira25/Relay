use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedJwt {
    pub token: String,
    pub source: String,
    pub detected_at: i64,
    pub claims: Option<serde_json::Value>,
    pub header: Option<serde_json::Value>,
    pub subject: Option<String>,
    pub issuer: Option<String>,
    pub expires_at: Option<i64>,
}

#[derive(Default)]
pub struct SessionState {
    pub tokens: RwLock<HashMap<String, ExtractedJwt>>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert_jwt(&self, jwt: ExtractedJwt) {
        let mut tokens = self.tokens.write();
        tokens.insert(jwt.token.clone(), jwt);
    }

    pub fn list_jwts(&self) -> Vec<ExtractedJwt> {
        let tokens = self.tokens.read();
        let mut list: Vec<ExtractedJwt> = tokens.values().cloned().collect();
        // Ordena do mais recente para o mais antigo
        list.sort_by(|a, b| b.detected_at.cmp(&a.detected_at));
        list
    }

    pub fn clear(&self) {
        let mut tokens = self.tokens.write();
        tokens.clear();
    }
}

/// Tenta decodificar um token JWT (header + payload claims) sem validar assinatura
pub fn decode_jwt_token(raw_token: &str, source: &str) -> Option<ExtractedJwt> {
    let token = raw_token.trim();
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Decodifica Header
    let header_json = decode_base64_url_segment(parts[0]);

    // Decodifica Claims / Payload
    let claims_json = decode_base64_url_segment(parts[1])?;

    let subject = claims_json
        .get("sub")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let issuer = claims_json
        .get("iss")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let expires_at = claims_json.get("exp").and_then(|v| v.as_i64());

    Some(ExtractedJwt {
        token: token.to_string(),
        source: source.to_string(),
        detected_at: chrono::Utc::now().timestamp_millis(),
        claims: Some(claims_json),
        header: header_json,
        subject,
        issuer,
        expires_at,
    })
}

fn decode_base64_url_segment(segment: &str) -> Option<serde_json::Value> {
    // Normaliza padding base64 URL-safe se necessário
    let clean = segment.trim_end_matches('=');
    let decoded_bytes = URL_SAFE_NO_PAD.decode(clean).ok()?;
    let json_str = String::from_utf8(decoded_bytes).ok()?;
    serde_json::from_str(&json_str).ok()
}

/// Extrai tokens JWT presentes em cabeçalhos HTTP
pub fn extract_jwts_from_headers(
    headers: &[(String, String)],
    source_prefix: &str,
) -> Vec<ExtractedJwt> {
    let mut tokens = Vec::new();

    for (k, v) in headers {
        if k.eq_ignore_ascii_case("authorization") {
            if let Some(token_part) = v
                .strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
            {
                if let Some(jwt) = decode_jwt_token(
                    token_part.trim(),
                    &format!("{}_header_auth", source_prefix),
                ) {
                    tokens.push(jwt);
                }
            }
        } else if k.eq_ignore_ascii_case("x-access-token")
            || k.eq_ignore_ascii_case("x-auth-token")
            || k.eq_ignore_ascii_case("token")
        {
            if let Some(jwt) = decode_jwt_token(
                v.trim(),
                &format!("{}_header_{}", source_prefix, k.to_lowercase()),
            ) {
                tokens.push(jwt);
            }
        }
    }

    tokens
}

/// Extrai tokens JWT em payloads JSON de resposta (ex: login / refresh responses)
pub fn extract_jwts_from_body(body_str: &str, source: &str) -> Vec<ExtractedJwt> {
    let mut tokens = Vec::new();
    let body_trimmed = body_str.trim();

    if !body_trimmed.starts_with('{') {
        return tokens;
    }

    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(body_trimmed) {
        if let Some(obj) = json_val.as_object() {
            let candidate_keys = [
                "token",
                "accessToken",
                "access_token",
                "idToken",
                "id_token",
                "jwt",
                "refreshToken",
                "refresh_token",
            ];

            for key in candidate_keys {
                if let Some(val) = obj.get(key).and_then(|v| v.as_str()) {
                    if let Some(jwt) = decode_jwt_token(val, &format!("{}_{}", source, key)) {
                        tokens.push(jwt);
                    }
                }
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_jwt_token() {
        // Token JWT padrão de teste com payload {"sub": "1234567890", "name": "Tacio", "iat": 1516239022, "exp": 1999999999}
        let raw_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IlRhY2lvIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE5OTk5OTk5OTl9.4Xx_testing";
        let jwt = decode_jwt_token(raw_token, "test").unwrap();

        assert_eq!(jwt.subject.as_deref(), Some("1234567890"));
        assert_eq!(jwt.expires_at, Some(1999999999));
        assert!(jwt.claims.is_some());
    }

    #[test]
    fn test_extract_jwts_from_headers() {
        let headers = vec![(
            "Authorization".to_string(),
            "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyMTIzIn0.sig".to_string(),
        )];
        let jwts = extract_jwts_from_headers(&headers, "req");
        assert_eq!(jwts.len(), 1);
        assert_eq!(jwts[0].subject.as_deref(), Some("user123"));
    }
}
