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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RouteRule {
    pub path_prefix: String,         // Ex: "/api/v1/auth", "/auth", "/users", "/gestao"
    pub target_host: Option<String>, // Opcional, se ausente herda target_host global
    pub target_port: u16,            // Porta específica deste serviço (ex: 4000)
    #[serde(default)]
    pub latency_ms: Option<u64>,     // Latência específica da rota (opcional)
    #[serde(default)]
    pub strip_prefix: bool,          // Se true, remove o path_prefix da URI enviada ao upstream
    #[serde(default)]
    pub is_mock: bool,               // Se true, responde diretamente sem ir ao upstream
    #[serde(default)]
    pub mock_status_code: Option<u16>, // 200, 201, 400, etc.
    #[serde(default)]
    pub mock_body: Option<String>,   // JSON ou string de mock
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,
    #[serde(default = "default_target_host")]
    pub target_host: String,
    #[serde(default = "default_target_port")]
    pub target_port: u16,
    #[serde(default)]
    pub latency_ms: u64,
    #[serde(default)]
    pub jitter_ms: u64,
    #[serde(default)]
    pub simulate_failure_rate: f32, // 0.0 a 1.0 (ex: 0.25 = 25% de falhas)
    #[serde(default = "default_failure_status")]
    pub failure_status_code: u16,   // 500, 502, 503, 504
    #[serde(default = "default_true")]
    pub auto_extract_jwt: bool,
    #[serde(default)]
    pub routes: Vec<RouteRule>, // Regras flexíveis de roteamento multisserviço
}

fn default_listen_port() -> u16 { 8080 }
fn default_target_host() -> String { "127.0.0.1".to_string() }
fn default_target_port() -> u16 { 3000 }
fn default_failure_status() -> u16 { 500 }
fn default_true() -> bool { true }

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_port: default_listen_port(),
            target_host: default_target_host(),
            target_port: default_target_port(),
            latency_ms: 0,
            jitter_ms: 0,
            simulate_failure_rate: 0.0,
            failure_status_code: default_failure_status(),
            auto_extract_jwt: default_true(),
            routes: Vec::new(),
        }
    }
}
