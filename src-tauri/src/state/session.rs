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
        tokens.values().cloned().collect()
    }
}
