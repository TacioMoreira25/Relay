use crate::proxy::recorder::{HttpExchange, InterceptedResponse};
use crate::security::types::{FindingCategory, FindingSeverity, SecurityFinding};
use crate::state::session::decode_jwt_token;
use serde_json::Value;

pub fn audit_exchange(exchange: &HttpExchange) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();
    let path = exchange
        .request
        .uri
        .split('?')
        .next()
        .unwrap_or(&exchange.request.uri);
    let resource = format!("{} {}", exchange.request.method, path);
    let now = chrono::Utc::now().timestamp_millis();

    if let Some(ref response) = exchange.response {
        check_cors(exchange, response, &resource, now, &mut findings);
        check_security_headers(response, &resource, now, &mut findings);
        check_tech_leaks(response, &resource, now, &mut findings);

        if let Some(ref body_text) = response.body {
            check_data_exposure(body_text, &resource, now, exchange, &mut findings);
        }
    }

    check_jwt_vulnerabilities(exchange, &resource, now, &mut findings);

    findings
}

fn check_cors(
    exchange: &HttpExchange,
    response: &InterceptedResponse,
    resource: &str,
    now: i64,
    findings: &mut Vec<SecurityFinding>,
) {
    let mut allow_origin = None;
    let mut allow_credentials = false;

    for h in &response.headers {
        let key_lower = h.key.to_lowercase();
        if key_lower == "access-control-allow-origin" {
            allow_origin = Some(h.value.trim());
        } else if key_lower == "access-control-allow-credentials" {
            allow_credentials = h.value.trim().eq_ignore_ascii_case("true");
        }
    }

    if let Some(origin) = allow_origin {
        if origin == "*" && allow_credentials {
            findings.push(SecurityFinding {
                id: format!("cors-wildcard-cred-{}", exchange.id),
                exchange_id: exchange.id.clone(),
                title: "CORS Permissivo Crítico (Wildcard com Credenciais)".to_string(),
                description: "O servidor configurou Access-Control-Allow-Origin: * em conjunto com Access-Control-Allow-Credentials: true. Isso permite que qualquer site malicioso faça requisições autenticadas e leia respostas confidenciais.".to_string(),
                severity: FindingSeverity::Critical,
                category: FindingCategory::Cors,
                remediation: "Especifique explicitamente a lista de origens autorizadas ou desative Allow-Credentials quando utilizar wildcard.".to_string(),
                affected_resource: resource.to_string(),
                timestamp: now,
            });
        }

        if origin.eq_ignore_ascii_case("null") {
            findings.push(SecurityFinding {
                id: format!("cors-origin-null-{}", exchange.id),
                exchange_id: exchange.id.clone(),
                title: "CORS com Origem 'null' Permitida".to_string(),
                description: "A origem 'null' é aceita pelo servidor. Ataques via iframes sandbox ou esquemas data: utilizam Origin: null para contornar políticas de mesma origem.".to_string(),
                severity: FindingSeverity::High,
                category: FindingCategory::Cors,
                remediation: "Nunca confie ou adicione 'null' à lista de origens permitidas nas configurações de CORS.".to_string(),
                affected_resource: resource.to_string(),
                timestamp: now,
            });
        }

        // Verifica reflexão cega do Origin enviado no Request
        if let Some(req_origin) = exchange.request.headers.iter().find(|h| h.key.eq_ignore_ascii_case("origin")) {
            let req_origin_val = req_origin.value.trim();
            if origin == req_origin_val && !origin.contains("localhost") && !origin.contains("127.0.0.1") {
                findings.push(SecurityFinding {
                    id: format!("cors-origin-reflected-{}", exchange.id),
                    exchange_id: exchange.id.clone(),
                    title: "CORS Refletido Dinamicamente".to_string(),
                    description: format!("O servidor reflete dinamicamente a origem enviada pelo cliente ('{}') sem uma validação estrita de domínios confiáveis.", req_origin_val),
                    severity: FindingSeverity::Medium,
                    category: FindingCategory::Cors,
                    remediation: "Valide o cabeçalho Origin contra uma lista estática (whitelist) segura de domínios antes de refleti-lo no cabeçalho de resposta.".to_string(),
                    affected_resource: resource.to_string(),
                    timestamp: now,
                });
            }
        }
    }
}

fn check_security_headers(
    response: &InterceptedResponse,
    resource: &str,
    now: i64,
    findings: &mut Vec<SecurityFinding>,
) {
    if response.status_code >= 400 {
        return;
    }

    let mut has_nosniff = false;
    let mut has_frame_options = false;
    let mut has_csp = false;

    for h in &response.headers {
        let key_lower = h.key.to_lowercase();
        if key_lower == "x-content-type-options" && h.value.to_lowercase().contains("nosniff") {
            has_nosniff = true;
        } else if key_lower == "x-frame-options" {
            has_frame_options = true;
        } else if key_lower == "content-security-policy" {
            has_csp = true;
            if h.value.to_lowercase().contains("frame-ancestors") {
                has_frame_options = true;
            }
        }
    }

    if !has_nosniff {
        findings.push(SecurityFinding {
            id: format!("sec-header-nosniff-{}", response.request_id),
            exchange_id: response.request_id.clone(),
            title: "Ausência do Cabeçalho X-Content-Type-Options".to_string(),
            description: "A resposta não define 'X-Content-Type-Options: nosniff'. Isso expõe a aplicação a ataques de MIME Sniffing, onde o navegador interpreta arquivos como scripts executáveis.".to_string(),
            severity: FindingSeverity::Low,
            category: FindingCategory::SecurityHeaders,
            remediation: "Adicione o cabeçalho 'X-Content-Type-Options: nosniff' a todas as respostas HTTP da sua API.".to_string(),
            affected_resource: resource.to_string(),
            timestamp: now,
        });
    }

    if !has_frame_options && !has_csp {
        findings.push(SecurityFinding {
            id: format!("sec-header-clickjacking-{}", response.request_id),
            exchange_id: response.request_id.clone(),
            title: "Proteção contra Clickjacking Ausente".to_string(),
            description: "A resposta não inclui 'X-Frame-Options' nem 'Content-Security-Policy: frame-ancestors', permitindo que o endpoint seja embutido em iframes por terceiros.".to_string(),
            severity: FindingSeverity::Low,
            category: FindingCategory::SecurityHeaders,
            remediation: "Defina 'X-Frame-Options: DENY' ou utilize a diretiva 'frame-ancestors 'none'' no Content-Security-Policy.".to_string(),
            affected_resource: resource.to_string(),
            timestamp: now,
        });
    }
}

fn check_tech_leaks(
    response: &InterceptedResponse,
    resource: &str,
    now: i64,
    findings: &mut Vec<SecurityFinding>,
) {
    for h in &response.headers {
        let key_lower = h.key.to_lowercase();
        let val_lower = h.value.to_lowercase();

        if key_lower == "x-powered-by" {
            findings.push(SecurityFinding {
                id: format!("tech-leak-x-powered-{}", response.request_id),
                exchange_id: response.request_id.clone(),
                title: format!("Vazamento de Tecnologia no Header ('{}')", h.value),
                description: format!("O cabeçalho X-Powered-By expõe a stack tecnológica do backend ('{}'), facilitando a identificação de exploits específicos por atacantes.", h.value),
                severity: FindingSeverity::Low,
                category: FindingCategory::TechLeak,
                remediation: "Desabilite ou remova o cabeçalho X-Powered-By nas configurações do framework ou servidor web.".to_string(),
                affected_resource: resource.to_string(),
                timestamp: now,
            });
        } else if key_lower == "server" && (val_lower.contains('/') || val_lower.contains("apache") || val_lower.contains("nginx")) {
            findings.push(SecurityFinding {
                id: format!("tech-leak-server-{}", response.request_id),
                exchange_id: response.request_id.clone(),
                title: format!("Versão do Servidor Exposta ('{}')", h.value),
                description: format!("O cabeçalho Server detalha a versão do software de servidor ('{}').", h.value),
                severity: FindingSeverity::Info,
                category: FindingCategory::TechLeak,
                remediation: "Oculte a versão detalhada do servidor web (ex: 'server_tokens off' no Nginx ou 'ServerTokens Prod' no Apache).".to_string(),
                affected_resource: resource.to_string(),
                timestamp: now,
            });
        } else if key_lower.starts_with("x-aspnet") || key_lower == "x-runtime" {
            findings.push(SecurityFinding {
                id: format!("tech-leak-extra-{}-{}", key_lower, response.request_id),
                exchange_id: response.request_id.clone(),
                title: format!("Metadado Interno Exposto no Header ('{}')", h.key),
                description: format!("O cabeçalho '{}' com valor '{}' expõe detalhes internos de execução do ambiente.", h.key, h.value),
                severity: FindingSeverity::Info,
                category: FindingCategory::TechLeak,
                remediation: "Remova cabeçalhos de depuração e metadados de runtime em ambientes de produção.".to_string(),
                affected_resource: resource.to_string(),
                timestamp: now,
            });
        }
    }
}

fn check_data_exposure(
    body_text: &str,
    resource: &str,
    now: i64,
    exchange: &HttpExchange,
    findings: &mut Vec<SecurityFinding>,
) {
    if let Ok(val) = serde_json::from_str::<Value>(body_text) {
        scan_json_exposure(&val, resource, now, exchange, findings);
    }

    // Varredura por hashes Bcrypt ou chaves privadas no texto
    if body_text.contains("$2a$") || body_text.contains("$2b$") || body_text.contains("$2y$") {
        findings.push(SecurityFinding {
            id: format!("data-leak-bcrypt-{}", exchange.id),
            exchange_id: exchange.id.clone(),
            title: "Possível Hash Bcrypt Exposto no Payload".to_string(),
            description: "O corpo da resposta contém padrões característicos de hashes de senha Bcrypt ($2a$/$2b$). Hashes de credenciais nunca devem ser transmitidos ao cliente.".to_string(),
            severity: FindingSeverity::Critical,
            category: FindingCategory::DataExposure,
            remediation: "Certifique-se de expurgar o campo de senha/hash nos DTOs ou serializadores do backend antes de responder à requisição.".to_string(),
            affected_resource: resource.to_string(),
            timestamp: now,
        });
    }

    if body_text.contains("BEGIN RSA PRIVATE KEY") || body_text.contains("BEGIN PRIVATE KEY") {
        findings.push(SecurityFinding {
            id: format!("data-leak-privkey-{}", exchange.id),
            exchange_id: exchange.id.clone(),
            title: "Chave Privada Exposta no Payload da Resposta".to_string(),
            description: "Detectada estrutura de chave privada criptográfica exposta no corpo da resposta.".to_string(),
            severity: FindingSeverity::Critical,
            category: FindingCategory::DataExposure,
            remediation: "Remova imediatamente chaves privadas de rotas públicas e rotacione os certificados/chaves comprometidos.".to_string(),
            affected_resource: resource.to_string(),
            timestamp: now,
        });
    }
}

fn scan_json_exposure(
    val: &Value,
    resource: &str,
    now: i64,
    exchange: &HttpExchange,
    findings: &mut Vec<SecurityFinding>,
) {
    match val {
        Value::Object(map) => {
            for (key, v) in map {
                let key_lower = key.to_lowercase();
                if key_lower == "password"
                    || key_lower == "password_hash"
                    || key_lower == "passwd"
                    || key_lower == "senha"
                    || key_lower == "salt"
                {
                    findings.push(SecurityFinding {
                        id: format!("data-exposure-{}-{}", key_lower, exchange.id),
                        exchange_id: exchange.id.clone(),
                        title: format!("Exposição Excessiva de Dados: Campo '{}' Retornado", key),
                        description: format!("A API retornou a propriedade '{}' no JSON da resposta. Dados de autenticação não devem ser devolvidos nas consultas da aplicação.", key),
                        severity: FindingSeverity::High,
                        category: FindingCategory::DataExposure,
                        remediation: format!("Exclua o atributo '{}' na camada de apresentação/serialização do seu modelo de dados.", key),
                        affected_resource: resource.to_string(),
                        timestamp: now,
                    });
                } else if key_lower == "secret" || key_lower == "api_secret" || key_lower == "private_key" {
                    findings.push(SecurityFinding {
                        id: format!("data-exposure-secret-{}-{}", key_lower, exchange.id),
                        exchange_id: exchange.id.clone(),
                        title: format!("Credencial Interna Exposta: '{}'", key),
                        description: format!("O atributo sensível '{}' foi identificado no payload retornado pela API.", key),
                        severity: FindingSeverity::Critical,
                        category: FindingCategory::DataExposure,
                        remediation: "Remova segredos e chaves de API privadas de respostas destinadas ao frontend ou clientes públicos.".to_string(),
                        affected_resource: resource.to_string(),
                        timestamp: now,
                    });
                }

                scan_json_exposure(v, resource, now, exchange, findings);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter().take(5) {
                scan_json_exposure(item, resource, now, exchange, findings);
            }
        }
        _ => {}
    }
}

fn check_jwt_vulnerabilities(
    exchange: &HttpExchange,
    resource: &str,
    now: i64,
    findings: &mut Vec<SecurityFinding>,
) {
    let mut candidate_tokens = Vec::new();

    // Coleta tokens dos headers da requisição
    for h in &exchange.request.headers {
        if h.key.eq_ignore_ascii_case("authorization") {
            let val = h.value.trim();
            let raw = val
                .strip_prefix("Bearer ")
                .or_else(|| val.strip_prefix("bearer "))
                .unwrap_or(val);
            if raw.starts_with("ey") {
                candidate_tokens.push(raw);
            }
        }
    }

    // Coleta tokens do response body se houver
    if let Some(ref res) = exchange.response {
        if let Some(ref body) = res.body {
            if body.contains("eyJ") {
                for word in body.split(|c: char| c.is_whitespace() || c == '"' || c == '\'') {
                    let w = word.trim();
                    if w.starts_with("eyJ") && w.split('.').count() == 3 {
                        candidate_tokens.push(w);
                    }
                }
            }
        }
    }

    for token in candidate_tokens {
        if let Some(jwt) = decode_jwt_token(token, "linter") {
            if let Some(ref header_val) = jwt.header {
                if let Some(alg) = header_val.get("alg").and_then(|v| v.as_str()) {
                    if alg.eq_ignore_ascii_case("none") {
                        findings.push(SecurityFinding {
                            id: format!("jwt-alg-none-{}", exchange.id),
                            exchange_id: exchange.id.clone(),
                            title: "Token JWT com Algoritmo Inseguro 'none'".to_string(),
                            description: "O token JWT utiliza 'alg: none', o que significa que qualquer atacante pode forjar assinaturas e adulterar claims sem ser detectado.".to_string(),
                            severity: FindingSeverity::Critical,
                            category: FindingCategory::JwtVulnerability,
                            remediation: "Rejeite incondicionalmente tokens com 'alg: none' no backend e force algoritmos criptográficos robustos como HS256, RS256 ou EdDSA.".to_string(),
                            affected_resource: resource.to_string(),
                            timestamp: now,
                        });
                    }
                }
            }

            if jwt.expires_at.is_none() {
                findings.push(SecurityFinding {
                    id: format!("jwt-no-exp-{}", exchange.id),
                    exchange_id: exchange.id.clone(),
                    title: "Token JWT Sem Data de Expiração ('exp')".to_string(),
                    description: "O token não possui a claim 'exp'. Tokens sem expiração permanecem válidos indefinidamente caso sejam vazados ou interceptados.".to_string(),
                    severity: FindingSeverity::Medium,
                    category: FindingCategory::JwtVulnerability,
                    remediation: "Adicione a claim 'exp' ao emitir tokens JWT, estipulando um tempo de vida curto (ex: 15 a 60 minutos).".to_string(),
                    affected_resource: resource.to_string(),
                    timestamp: now,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::recorder::{HeaderEntry, InterceptedRequest, InterceptedResponse};

    #[test]
    fn test_cors_wildcard_with_credentials() {
        let exchange = HttpExchange {
            id: "test-1".to_string(),
            request: InterceptedRequest {
                id: "test-1".to_string(),
                method: "GET".to_string(),
                uri: "/api/users".to_string(),
                headers: vec![],
                body: None,
                size_bytes: 0,
                timestamp: 0,
            },
            response: Some(InterceptedResponse {
                id: "res-1".to_string(),
                request_id: "test-1".to_string(),
                status_code: 200,
                headers: vec![
                    HeaderEntry {
                        key: "access-control-allow-origin".to_string(),
                        value: "*".to_string(),
                    },
                    HeaderEntry {
                        key: "access-control-allow-credentials".to_string(),
                        value: "true".to_string(),
                    },
                ],
                body: None,
                duration_ms: 10,
                size_bytes: 0,
                timestamp: 0,
            }),
            status: "completed".to_string(),
            error: None,
        };

        let findings = audit_exchange(&exchange);
        assert!(findings.iter().any(|f| f.category == FindingCategory::Cors && f.severity == FindingSeverity::Critical));
    }

    #[test]
    fn test_excessive_data_exposure_password() {
        let exchange = HttpExchange {
            id: "test-2".to_string(),
            request: InterceptedRequest {
                id: "test-2".to_string(),
                method: "GET".to_string(),
                uri: "/api/profile".to_string(),
                headers: vec![],
                body: None,
                size_bytes: 0,
                timestamp: 0,
            },
            response: Some(InterceptedResponse {
                id: "res-2".to_string(),
                request_id: "test-2".to_string(),
                status_code: 200,
                headers: vec![],
                body: Some(r#"{"user": {"id": 1, "name": "Admin", "password_hash": "$2a$12$e8ZbJ1..."}}"#.to_string()),
                duration_ms: 15,
                size_bytes: 100,
                timestamp: 0,
            }),
            status: "completed".to_string(),
            error: None,
        };

        let findings = audit_exchange(&exchange);
        assert!(findings.iter().any(|f| f.category == FindingCategory::DataExposure));
    }
}
