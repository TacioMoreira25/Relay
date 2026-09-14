use bytes::Bytes;
use hyper::header::{HeaderName, HeaderValue};
use hyper::Request;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::proxy::recorder::{HeaderEntry, HttpExchange};
use crate::security::types::{FindingCategory, FindingSeverity, SecurityFinding};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProbeType {
    MassAssignment,
    AuthBypass,
    BolaAb,
    HiddenVerbs,
    OutdatedAssets,
    StackTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveProbeResult {
    pub id: String,
    pub probe_type: ProbeType,
    pub target_uri: String,
    pub target_method: String,
    pub vulnerable: bool,
    pub title: String,
    pub details: String,
    pub status_code: Option<u16>,
    pub evidence: Option<String>,
    pub remediation: String,
    pub timestamp: i64,
}

impl ActiveProbeResult {
    pub fn to_finding(&self, exchange_id: &str) -> Option<SecurityFinding> {
        if !self.vulnerable {
            return None;
        }

        let (category, severity) = match self.probe_type {
            ProbeType::MassAssignment => (FindingCategory::DataExposure, FindingSeverity::High),
            ProbeType::AuthBypass => (FindingCategory::MissingAuth, FindingSeverity::Critical),
            ProbeType::BolaAb => (FindingCategory::MissingAuth, FindingSeverity::Critical),
            ProbeType::HiddenVerbs => (FindingCategory::MissingAuth, FindingSeverity::Medium),
            ProbeType::OutdatedAssets => (FindingCategory::TechLeak, FindingSeverity::Low),
            ProbeType::StackTrace => (FindingCategory::TechLeak, FindingSeverity::Medium),
        };

        Some(SecurityFinding {
            id: format!("finding-probe-{}", Uuid::new_v4()),
            exchange_id: exchange_id.to_string(),
            title: self.title.clone(),
            description: self.details.clone(),
            severity,
            category,
            remediation: self.remediation.clone(),
            affected_resource: format!("{} {}", self.target_method, self.target_uri),
            timestamp: self.timestamp,
        })
    }
}

/// Helper para disparo de requisição HTTP direta ao upstream
pub async fn send_raw_probe_request(
    target_host: &str,
    target_port: u16,
    method: &str,
    uri: &str,
    headers: &[HeaderEntry],
    body: Option<&str>,
) -> Result<(u16, Vec<HeaderEntry>, String), String> {
    let upstream_addr = format!("{}:{}", target_host, target_port);

    let tcp_stream = match tokio::time::timeout(
        Duration::from_millis(4000),
        TcpStream::connect(&upstream_addr),
    )
    .await
    {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => return Err(format!("Falha de conexão com {}: {}", upstream_addr, e)),
        Err(_) => return Err(format!("Timeout de conexão com {}", upstream_addr)),
    };

    let io = TokioIo::new(tcp_stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e| format!("Falha no handshake HTTP: {}", e))?;

    tokio::spawn(async move {
        let _ = conn.await;
    });

    let body_bytes = body.map(|b| Bytes::from(b.to_string())).unwrap_or_default();

    let mut req_builder = Request::builder().method(method).uri(uri);

    for h in headers {
        if !h.key.eq_ignore_ascii_case("host")
            && !h.key.eq_ignore_ascii_case("connection")
            && !h.key.eq_ignore_ascii_case("transfer-encoding")
            && !h.key.eq_ignore_ascii_case("content-length")
        {
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_bytes(h.key.as_bytes()),
                HeaderValue::from_str(&h.value),
            ) {
                req_builder = req_builder.header(hn, hv);
            }
        }
    }

    req_builder = req_builder.header("host", &upstream_addr);
    req_builder = req_builder.header("content-length", body_bytes.len().to_string());

    let req_body = http_body_util::Full::new(body_bytes);
    let req = req_builder
        .body(req_body)
        .map_err(|e| format!("Falha ao construir requisição: {}", e))?;

    let resp = sender
        .send_request(req)
        .await
        .map_err(|e| format!("Erro ao enviar requisição de probe: {}", e))?;

    let status_code = resp.status().as_u16();
    let res_headers: Vec<HeaderEntry> = resp
        .headers()
        .iter()
        .map(|(k, v)| HeaderEntry {
            key: k.to_string(),
            value: v.to_str().unwrap_or("").to_string(),
        })
        .collect();

    use http_body_util::BodyExt;
    let resp_bytes = resp
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("Erro ao coletar resposta do probe: {}", e))?
        .to_bytes();

    let body_str = String::from_utf8_lossy(&resp_bytes).to_string();

    Ok((status_code, res_headers, body_str))
}

/// Sonda 1: Mass Assignment Probe (Injeção de Parâmetros de Alto Privilégio)
pub async fn run_mass_assignment_probe(
    target_host: &str,
    target_port: u16,
    exchange: &HttpExchange,
) -> ActiveProbeResult {
    let now = chrono::Utc::now().timestamp_millis();
    let original_req = &exchange.request;
    let uri = &original_req.uri;
    let method = &original_req.method;

    if method != "POST" && method != "PUT" && method != "PATCH" {
        return ActiveProbeResult {
            id: format!("probe-{}", Uuid::new_v4()),
            probe_type: ProbeType::MassAssignment,
            target_uri: uri.clone(),
            target_method: method.clone(),
            vulnerable: false,
            title: "Mass Assignment Inaplicável".to_string(),
            details: format!(
                "O método HTTP {} não costuma transportar carga mutável de dados (esperado POST/PUT/PATCH).",
                method
            ),
            status_code: None,
            evidence: None,
            remediation: "Aplique DTOs específicos em rotas mutáveis.".to_string(),
            timestamp: now,
        };
    }

    // Injeta atributos privilegiados no payload JSON
    let mut mutated_body = serde_json::json!({
        "isAdmin": true,
        "role": "admin",
        "is_admin": true,
        "permissions": ["*"]
    });

    if let Some(ref b) = original_req.body {
        if let Ok(mut original_json) = serde_json::from_str::<serde_json::Value>(b) {
            if let Some(obj) = original_json.as_object_mut() {
                obj.insert("isAdmin".to_string(), serde_json::Value::Bool(true));
                obj.insert("role".to_string(), serde_json::Value::String("admin".to_string()));
                obj.insert("is_admin".to_string(), serde_json::Value::Bool(true));
            }
            mutated_body = original_json;
        }
    }

    let payload_str = mutated_body.to_string();
    let mut probe_headers = original_req.headers.clone();
    if !probe_headers.iter().any(|h| h.key.eq_ignore_ascii_case("content-type")) {
        probe_headers.push(HeaderEntry {
            key: "content-type".to_string(),
            value: "application/json".to_string(),
        });
    }

    match send_raw_probe_request(
        target_host,
        target_port,
        method,
        uri,
        &probe_headers,
        Some(&payload_str),
    )
    .await
    {
        Ok((status, _, body)) => {
            let is_success = status >= 200 && status < 300;
            let reflected_privilege = body.contains("\"isAdmin\":true")
                || body.contains("\"role\":\"admin\"")
                || body.contains("\"is_admin\":true");

            let vulnerable = is_success && reflected_privilege;

            ActiveProbeResult {
                id: format!("probe-{}", Uuid::new_v4()),
                probe_type: ProbeType::MassAssignment,
                target_uri: uri.clone(),
                target_method: method.clone(),
                vulnerable,
                title: if vulnerable {
                    "Vulnerabilidade Crítica: Mass Assignment Confirmada".to_string()
                } else {
                    "Mass Assignment Não Detectado".to_string()
                },
                details: if vulnerable {
                    format!(
                        "O backend aceitou e refletiu propriedades de alta permissão ('role': 'admin' / 'isAdmin': true) com status HTTP {}. Isso indica ausência de DTOs restritivos.",
                        status
                    )
                } else {
                    format!(
                        "O backend rejeitou ou descartou os campos administrativos injetados (Status HTTP: {}).",
                        status
                    )
                },
                status_code: Some(status),
                evidence: if vulnerable {
                    Some(format!("Payload enviado: {}\nResposta refletida: {}", payload_str, body))
                } else {
                    None
                },
                remediation: "Utilize DTOs (Data Transfer Objects) estritos com bibliotecas como Zod, Pydantic, class-validator ou serde com 'deny_unknown_fields' para proibir vinculação de atributos arbitrários.".to_string(),
                timestamp: now,
            }
        }
        Err(err) => ActiveProbeResult {
            id: format!("probe-{}", Uuid::new_v4()),
            probe_type: ProbeType::MassAssignment,
            target_uri: uri.clone(),
            target_method: method.clone(),
            vulnerable: false,
            title: "Erro ao Executar Sonda de Mass Assignment".to_string(),
            details: format!("Falha de comunicação: {}", err),
            status_code: None,
            evidence: None,
            remediation: "Certifique-se de que a API alvo está online e acessível.".to_string(),
            timestamp: now,
        },
    }
}

/// Sonda 2: Auth Bypass Probe (Acesso sem cabeçalho Authorization)
pub async fn run_auth_bypass_probe(
    target_host: &str,
    target_port: u16,
    exchange: &HttpExchange,
) -> ActiveProbeResult {
    let now = chrono::Utc::now().timestamp_millis();
    let original_req = &exchange.request;
    let uri = &original_req.uri;
    let method = &original_req.method;

    let has_auth = original_req.headers.iter().any(|h| {
        let k = h.key.to_lowercase();
        k == "authorization" || k == "cookie" || k == "x-access-token" || k == "x-api-key"
    });

    if !has_auth {
        return ActiveProbeResult {
            id: format!("probe-{}", Uuid::new_v4()),
            probe_type: ProbeType::AuthBypass,
            target_uri: uri.clone(),
            target_method: method.clone(),
            vulnerable: false,
            title: "Endpoint Aparentemente Público".to_string(),
            details: "A requisição original não continha credenciais de autorização.".to_string(),
            status_code: None,
            evidence: None,
            remediation: "Se este endpoint contiver dados confidenciais, proteja-o com middleware de autenticação.".to_string(),
            timestamp: now,
        };
    }

    // Cria headers sem NENHUMA credencial
    let stripped_headers: Vec<HeaderEntry> = original_req
        .headers
        .iter()
        .filter(|h| {
            let k = h.key.to_lowercase();
            k != "authorization" && k != "cookie" && k != "x-access-token" && k != "x-api-key"
        })
        .cloned()
        .collect();

    match send_raw_probe_request(
        target_host,
        target_port,
        method,
        uri,
        &stripped_headers,
        original_req.body.as_deref(),
    )
    .await
    {
        Ok((status, _, body)) => {
            let vulnerable = status >= 200 && status < 300;

            ActiveProbeResult {
                id: format!("probe-{}", Uuid::new_v4()),
                probe_type: ProbeType::AuthBypass,
                target_uri: uri.clone(),
                target_method: method.clone(),
                vulnerable,
                title: if vulnerable {
                    "Vulnerabilidade Crítica: Ausência de Autenticação (Auth Bypass)".to_string()
                } else {
                    "Autenticação Validada com Sucesso".to_string()
                },
                details: if vulnerable {
                    format!(
                        "O recurso '{}' retornou sucesso (HTTP {}) mesmo quando a requisição foi despachada sem qualquer token ou credencial.",
                        uri, status
                    )
                } else {
                    format!(
                        "O backend bloqueou adequadamente a requisição não autenticada (HTTP {}).",
                        status
                    )
                },
                status_code: Some(status),
                evidence: if vulnerable {
                    Some(format!("Resposta recebida sem autenticação:\n{}", body))
                } else {
                    None
                },
                remediation: "Verifique a cadeia de middlewares de rota do seu framework e certifique-se de que a validação de JWT/Session bloqueia o acesso com 401 Unauthorized.".to_string(),
                timestamp: now,
            }
        }
        Err(err) => ActiveProbeResult {
            id: format!("probe-{}", Uuid::new_v4()),
            probe_type: ProbeType::AuthBypass,
            target_uri: uri.clone(),
            target_method: method.clone(),
            vulnerable: false,
            title: "Erro ao Testar Auth Bypass".to_string(),
            details: format!("Falha de comunicação: {}", err),
            status_code: None,
            evidence: None,
            remediation: "Certifique-se de que a API alvo está online.".to_string(),
            timestamp: now,
        },
    }
}

/// Sonda 3: BOLA / IDOR A-B Test (Substituição de Token por Credencial Cruzada)
pub async fn run_bola_ab_probe(
    target_host: &str,
    target_port: u16,
    exchange: &HttpExchange,
    token_b: &str,
) -> ActiveProbeResult {
    let now = chrono::Utc::now().timestamp_millis();
    let original_req = &exchange.request;
    let uri = &original_req.uri;
    let method = &original_req.method;

    let clean_token_b = token_b.trim().replace("Bearer ", "");
    if clean_token_b.is_empty() {
        return ActiveProbeResult {
            id: format!("probe-{}", Uuid::new_v4()),
            probe_type: ProbeType::BolaAb,
            target_uri: uri.clone(),
            target_method: method.clone(),
            vulnerable: false,
            title: "Token B Inválido ou Ausente".to_string(),
            details: "Para realizar o teste de BOLA (IDOR), informe um token JWT de outro usuário válido.".to_string(),
            status_code: None,
            evidence: None,
            remediation: "Forneça o token B para simulação de acesso cruzado a recursos.".to_string(),
            timestamp: now,
        };
    }

    // Substitui o Authorization Header pelo Token B
    let mut probe_headers: Vec<HeaderEntry> = original_req
        .headers
        .iter()
        .filter(|h| !h.key.eq_ignore_ascii_case("authorization"))
        .cloned()
        .collect();

    probe_headers.push(HeaderEntry {
        key: "authorization".to_string(),
        value: format!("Bearer {}", clean_token_b),
    });

    match send_raw_probe_request(
        target_host,
        target_port,
        method,
        uri,
        &probe_headers,
        original_req.body.as_deref(),
    )
    .await
    {
        Ok((status, _, body)) => {
            let original_status = exchange.response.as_ref().map(|r| r.status_code).unwrap_or(200);
            let vulnerable = (status >= 200 && status < 300) && (original_status >= 200 && original_status < 300);

            ActiveProbeResult {
                id: format!("probe-{}", Uuid::new_v4()),
                probe_type: ProbeType::BolaAb,
                target_uri: uri.clone(),
                target_method: method.clone(),
                vulnerable,
                title: if vulnerable {
                    "Vulnerabilidade Crítica: Possível BOLA / IDOR Detectado".to_string()
                } else {
                    "Controle de Acesso em Nível de Objeto Validado".to_string()
                },
                details: if vulnerable {
                    format!(
                        "O Usuário B conseguiu acessar o recurso '{}' originalmente solicitado pelo Usuário A com status HTTP {}. Verifique se há checagem de posse do registro no banco de dados.",
                        uri, status
                    )
                } else {
                    format!(
                        "O backend bloqueou adequadamente o acesso cruzado com Token B (HTTP {}).",
                        status
                    )
                },
                status_code: Some(status),
                evidence: if vulnerable {
                    Some(format!("Resposta retornada para o Usuário B:\n{}", body))
                } else {
                    None
                },
                remediation: "Valide sempre que o registro pertence ao usuário autenticado (ex: 'WHERE id = :id AND user_id = :session_user_id') em vez de confiar apenas no identificador passado na rota.".to_string(),
                timestamp: now,
            }
        }
        Err(err) => ActiveProbeResult {
            id: format!("probe-{}", Uuid::new_v4()),
            probe_type: ProbeType::BolaAb,
            target_uri: uri.clone(),
            target_method: method.clone(),
            vulnerable: false,
            title: "Erro ao Testar BOLA A-B".to_string(),
            details: format!("Falha de comunicação: {}", err),
            status_code: None,
            evidence: None,
            remediation: "Certifique-se de que a API alvo está online.".to_string(),
            timestamp: now,
        },
    }
}

/// Sonda 4: Hidden HTTP Verbs Fuzzer (Teste de Verbos HTTP Alternativos)
pub async fn run_hidden_verbs_probe(
    target_host: &str,
    target_port: u16,
    exchange: &HttpExchange,
) -> ActiveProbeResult {
    let now = chrono::Utc::now().timestamp_millis();
    let original_req = &exchange.request;
    let uri = &original_req.uri;
    let original_method = &original_req.method;

    let candidate_verbs = ["OPTIONS", "HEAD", "PUT", "DELETE", "PATCH"];
    let mut exposed_verbs: Vec<String> = Vec::new();

    for verb in candidate_verbs {
        if verb.eq_ignore_ascii_case(original_method) {
            continue;
        }

        if let Ok((status, _, _)) = send_raw_probe_request(
            target_host,
            target_port,
            verb,
            uri,
            &original_req.headers,
            None,
        )
        .await
        {
            if status >= 200 && status < 300 {
                exposed_verbs.push(format!("{} (HTTP {})", verb, status));
            }
        }
    }

    let vulnerable = !exposed_verbs.is_empty();

    ActiveProbeResult {
        id: format!("probe-{}", Uuid::new_v4()),
        probe_type: ProbeType::HiddenVerbs,
        target_uri: uri.clone(),
        target_method: original_method.clone(),
        vulnerable,
        title: if vulnerable {
            "Métodos HTTP Não Mapeados Ativos".to_string()
        } else {
            "Verbos HTTP Restritos Corretamente".to_string()
        },
        details: if vulnerable {
            format!(
                "O endpoint '{}' respondeu com sucesso aos seguintes verbos não intencionais: {}.",
                uri,
                exposed_verbs.join(", ")
            )
        } else {
            format!(
                "Nenhum verbo HTTP oculto foi aceito pelo endpoint '{}'.",
                uri
            )
        },
        status_code: None,
        evidence: if vulnerable {
            Some(format!("Verbos ativos: {:?}", exposed_verbs))
        } else {
            None
        },
        remediation: "Configure o roteador da sua API para rejeitar métodos não declarados com '405 Method Not Allowed' ou restrinja os métodos nos middlewares de segurança.".to_string(),
        timestamp: now,
    }
}

/// Sonda 5: Stack Trace / Verbose Error Leak Probe
pub async fn run_stack_trace_probe(
    target_host: &str,
    target_port: u16,
    exchange: &HttpExchange,
) -> ActiveProbeResult {
    let now = chrono::Utc::now().timestamp_millis();
    let original_req = &exchange.request;
    let uri = &original_req.uri;
    let method = &original_req.method;

    // Injeta payload que induz erro de parsing ou tipo
    let malicious_payload = "{\"id\": null, \"malformed_json': true";

    match send_raw_probe_request(
        target_host,
        target_port,
        method,
        uri,
        &original_req.headers,
        Some(malicious_payload),
    )
    .await
    {
        Ok((status, _, body)) => {
            let indicators = [
                "at Object.<anonymous>",
                "at Module._compile",
                "Traceback (most recent call last):",
                "NullPointerException",
                "SyntaxError:",
                "System.Exception",
                "node_modules",
                "vendor/bundle",
                "src/main",
                "jakarta.",
                "org.springframework.",
                "django.core.",
            ];

            let mut matched_indicators = Vec::new();
            for ind in indicators {
                if body.contains(ind) {
                    matched_indicators.push(ind.to_string());
                }
            }

            let vulnerable = !matched_indicators.is_empty();

            ActiveProbeResult {
                id: format!("probe-{}", Uuid::new_v4()),
                probe_type: ProbeType::StackTrace,
                target_uri: uri.clone(),
                target_method: method.clone(),
                vulnerable,
                title: if vulnerable {
                    "Vazamento de Stack Trace / Erro Verboso".to_string()
                } else {
                    "Tratamento de Erros Seguro".to_string()
                },
                details: if vulnerable {
                    format!(
                        "O backend retornou detalhes internos de execução e rastreamento de pilha (HTTP {}). Indicadores encontrados: {}.",
                        status,
                        matched_indicators.join(", ")
                    )
                } else {
                    format!(
                        "O backend tratou a requisição anômala sem expor rastreamento de pilha (HTTP {}).",
                        status
                    )
                },
                status_code: Some(status),
                evidence: if vulnerable {
                    Some(body.chars().take(500).collect())
                } else {
                    None
                },
                remediation: "Desative mensagens detalhadas de erro e stack traces em ambientes de produção. Configure um manipulador global de erros (Error Middleware) que retorne apenas uma mensagem amigável com código 400 ou 500 padronizado.".to_string(),
                timestamp: now,
            }
        }
        Err(err) => ActiveProbeResult {
            id: format!("probe-{}", Uuid::new_v4()),
            probe_type: ProbeType::StackTrace,
            target_uri: uri.clone(),
            target_method: method.clone(),
            vulnerable: false,
            title: "Erro ao Testar Stack Trace".to_string(),
            details: format!("Falha de comunicação: {}", err),
            status_code: None,
            evidence: None,
            remediation: "Certifique-se de que a API alvo está online.".to_string(),
            timestamp: now,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mass_assignment_inapplicable_for_get() {
        let exchange = HttpExchange {
            id: "req-1".to_string(),
            request: crate::proxy::recorder::InterceptedRequest {
                id: "req-1".to_string(),
                timestamp: 0,
                method: "GET".to_string(),
                uri: "/api/users".to_string(),
                headers: vec![],
                body: None,
                size_bytes: 0,
            },
            response: None,
            status: "completed".to_string(),
            error: None,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt.block_on(run_mass_assignment_probe("127.0.0.1", 3000, &exchange));
        assert!(!res.vulnerable);
        assert_eq!(res.probe_type, ProbeType::MassAssignment);
    }

    #[test]
    fn test_auth_bypass_inapplicable_for_public_route() {
        let exchange = HttpExchange {
            id: "req-2".to_string(),
            request: crate::proxy::recorder::InterceptedRequest {
                id: "req-2".to_string(),
                timestamp: 0,
                method: "GET".to_string(),
                uri: "/public/health".to_string(),
                headers: vec![],
                body: None,
                size_bytes: 0,
            },
            response: None,
            status: "completed".to_string(),
            error: None,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt.block_on(run_auth_bypass_probe("127.0.0.1", 3000, &exchange));
        assert!(!res.vulnerable);
        assert_eq!(res.probe_type, ProbeType::AuthBypass);
    }
}
