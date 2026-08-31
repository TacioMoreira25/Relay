use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterceptedRequest {
    pub id: String,
    pub timestamp: i64,
    pub method: String,
    pub uri: String,
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterceptedResponse {
    pub id: String,
    pub request_id: String,
    pub timestamp: i64,
    pub status_code: u16,
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
    pub size_bytes: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpExchange {
    pub id: String,
    pub request: InterceptedRequest,
    pub response: Option<InterceptedResponse>,
    pub status: String, // "pending", "completed", "failed"
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    pub listen_port: u16,
    pub target_host: String,
    pub target_port: u16,
    pub latency_ms: u64,
    pub jitter_ms: u64,
    pub simulate_failure_rate: f32, // 0.0 a 1.0 (ex: 0.25 = 25% de falhas)
    pub failure_status_code: u16,   // 500, 502, 503, 504
    pub auto_extract_jwt: bool,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_port: 8080,
            target_host: "127.0.0.1".to_string(),
            target_port: 3000,
            latency_ms: 0,
            jitter_ms: 0,
            simulate_failure_rate: 0.0,
            failure_status_code: 500,
            auto_extract_jwt: true,
        }
    }
}
