use serde::{Deserialize, Serialize};

use crate::proxy::recorder::HttpExchange;
use crate::security::types::{FindingCategory, SecurityFinding};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StrideCategory {
    Spoofing,
    Tampering,
    Repudiation,
    InformationDisclosure,
    DenialOfService,
    ElevationOfPrivilege,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrideThreat {
    pub category: StrideCategory,
    pub title: String,
    pub description: String,
    pub affected_endpoints: Vec<String>,
    pub risk_level: String, // "critical", "high", "medium", "low"
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrideReport {
    pub total_threats: usize,
    pub high_risk_count: usize,
    pub threats: Vec<StrideThreat>,
    pub trust_boundaries: Vec<String>,
    pub timestamp: i64,
}

/// Motor de Modelagem de Ameaças STRIDE baseado em Tráfego Real e Achados de Segurança
pub fn generate_stride_report(
    exchanges: &[HttpExchange],
    findings: &[SecurityFinding],
) -> StrideReport {
    let mut threats: Vec<StrideThreat> = Vec::new();
    let now = chrono::Utc::now().timestamp_millis();

    let mut endpoints_with_missing_auth = Vec::new();
    let mut endpoints_with_disclosure = Vec::new();
    let mut endpoints_with_mutations = Vec::new();
    let mut endpoints_with_unpaged_queries = Vec::new();
    let mut endpoints_with_privilege_risks = Vec::new();

    for f in findings {
        match f.category {
            FindingCategory::MissingAuth => {
                endpoints_with_missing_auth.push(f.affected_resource.clone());
            }
            FindingCategory::DataExposure | FindingCategory::TechLeak => {
                endpoints_with_disclosure.push(f.affected_resource.clone());
            }
            FindingCategory::JwtVulnerability => {
                endpoints_with_missing_auth.push(f.affected_resource.clone());
            }
            FindingCategory::Cors => {
                endpoints_with_privilege_risks.push(f.affected_resource.clone());
            }
            _ => {}
        }
    }

    // Varre exchanges identificando padrões de tráfego
    for e in exchanges {
        let uri = &e.request.uri;
        let method = &e.request.method;

        // Tampering: Métodos mutáveis sem validação
        if method == "PUT" || method == "POST" || method == "PATCH" {
            if !endpoints_with_mutations.contains(uri) {
                endpoints_with_mutations.push(uri.clone());
            }
        }

        // Denial of Service: GET em coleções sem limit/offset/page
        if method == "GET" && (uri.contains("/api/") || uri.contains("/v1/")) && !uri.contains('?') {
            if !endpoints_with_unpaged_queries.contains(uri) {
                endpoints_with_unpaged_queries.push(uri.clone());
            }
        }

        // Elevation of Privilege: Rotas administrativas ou com parâmetros de role
        if uri.contains("/admin") || uri.contains("/gerente") || uri.contains("/roles") {
            if !endpoints_with_privilege_risks.contains(uri) {
                endpoints_with_privilege_risks.push(uri.clone());
            }
        }
    }

    if exchanges.is_empty() && findings.is_empty() {
        return StrideReport {
            total_threats: 0,
            high_risk_count: 0,
            threats: vec![],
            trust_boundaries: vec![
                "Cliente HTTP <-> Proxy Relay (Fronteira Externa)".to_string(),
                "Proxy Relay <-> Backend Upstream (Fronteira Interna)".to_string(),
                "Middleware de Autenticação <-> Controladores de Domínio".to_string(),
            ],
            timestamp: now,
        };
    }

    // 1. S - Spoofing (Falsificação de Identidade)
    if !endpoints_with_missing_auth.is_empty() {
        threats.push(StrideThreat {
            category: StrideCategory::Spoofing,
            title: "Falsificação de Identidade (Spoofing) em Rotas Privadas".to_string(),
            description: "Endpoints que processam dados sensíveis responderam sem exigir assinatura JWT válida ou autenticação multifator.".to_string(),
            affected_endpoints: endpoints_with_missing_auth.clone(),
            risk_level: "critical".to_string(),
            mitigation: "Exija Bearer JWT com assinatura criptográfica robusta e tempo de expiração curto.".to_string(),
        });
    }

    // 2. T - Tampering (Adulteração de Dados)
    if !endpoints_with_mutations.is_empty() {
        threats.push(StrideThreat {
            category: StrideCategory::Tampering,
            title: "Adulteração de Dados em Operações Mutáveis".to_string(),
            description: "Rotas POST/PUT/PATCH precisam de validação estrita de payload para impedir injeção de parâmetros arbitrários (Mass Assignment).".to_string(),
            affected_endpoints: endpoints_with_mutations.iter().take(5).cloned().collect(),
            risk_level: "high".to_string(),
            mitigation: "Implemente validação com schemas estritos (ex: DTOs no backend com deny_unknown_fields).".to_string(),
        });
    }

    // 3. R - Repudiation (Repúdio / Falta de Rastreabilidade)
    if !endpoints_with_mutations.is_empty() {
        threats.push(StrideThreat {
            category: StrideCategory::Repudiation,
            title: "Risco de Não-Repúdio em Ações Críticas".to_string(),
            description: "Operações mutáveis e exclusões devem gerar registros imutáveis de auditoria contendo ID do usuário, timestamp e IP de origem.".to_string(),
            affected_endpoints: endpoints_with_mutations.iter().take(4).cloned().collect(),
            risk_level: "medium".to_string(),
            mitigation: "Grave logs de auditoria estruturados com correlationId/requestId em cada operação que altera registros.".to_string(),
        });
    }

    // 4. I - Information Disclosure (Vazamento de Informações)
    if !endpoints_with_disclosure.is_empty() {
        threats.push(StrideThreat {
            category: StrideCategory::InformationDisclosure,
            title: "Vazamento de Informações Sensíveis / PII".to_string(),
            description: "Foram detectadas exposições de senhas, chaves privadas, stack traces ou headers de servidor nos corpos das respostas HTTP.".to_string(),
            affected_endpoints: endpoints_with_disclosure.clone(),
            risk_level: "high".to_string(),
            mitigation: "Remova headers 'X-Powered-By' e 'Server', e serialize apenas campos de saída públicos (Response View Models).".to_string(),
        });
    }

    // 5. D - Denial of Service (Negação de Serviço)
    if !endpoints_with_unpaged_queries.is_empty() {
        threats.push(StrideThreat {
            category: StrideCategory::DenialOfService,
            title: "Risco de Exaustão de Recursos por Listagem Sem Paginação".to_string(),
            description: "Consultas a listas que não utilizam limit/offset obrigatórios podem induzir exaustão de memória sob alta concorrência.".to_string(),
            affected_endpoints: endpoints_with_unpaged_queries.iter().take(4).cloned().collect(),
            risk_level: "medium".to_string(),
            mitigation: "Aplique paginação obrigatória com limite máximo de registros (ex: limit <= 50) e Rate Limiting.".to_string(),
        });
    }

    // 6. E - Elevation of Privilege (Elevação de Privilégio)
    if !endpoints_with_privilege_risks.is_empty() {
        threats.push(StrideThreat {
            category: StrideCategory::ElevationOfPrivilege,
            title: "Possível Acesso Não Autorizado a Recursos Administrativos (BOLA / Privilégios)".to_string(),
            description: "Endpoints contendo prefixos ou parâmetros de privilégio devem validar se a role/permissão do usuário permite a execução da ação.".to_string(),
            affected_endpoints: endpoints_with_privilege_risks.clone(),
            risk_level: "critical".to_string(),
            mitigation: "Implemente RBAC (Role-Based Access Control) e valide sempre que a entidade pertence ao tenant/usuário conectado.".to_string(),
        });
    }

    // Deduplica lista de endpoints por categoria
    fn deduplicate(endpoints: &mut Vec<String>) {
        endpoints.sort();
        endpoints.dedup();
    }

    deduplicate(&mut endpoints_with_missing_auth);
    deduplicate(&mut endpoints_with_disclosure);
    deduplicate(&mut endpoints_with_mutations);
    deduplicate(&mut endpoints_with_unpaged_queries);
    deduplicate(&mut endpoints_with_privilege_risks);

    let high_risk_count = threats
        .iter()
        .filter(|t| t.risk_level == "critical" || t.risk_level == "high")
        .count();
    let total_threats = threats.len();

    let trust_boundaries = vec![
        "Cliente HTTP <-> Proxy Relay (Fronteira Externa)".to_string(),
        "Proxy Relay <-> Backend Upstream (Fronteira Interna)".to_string(),
        "Middleware de Autenticação <-> Controladores de Domínio".to_string(),
    ];

    StrideReport {
        total_threats,
        high_risk_count,
        threats,
        trust_boundaries,
        timestamp: now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_stride_report() {
        let exchanges = vec![HttpExchange {
            id: "req-stride-1".to_string(),
            request: crate::proxy::recorder::InterceptedRequest {
                id: "req-stride-1".to_string(),
                timestamp: 0,
                method: "POST".to_string(),
                uri: "/api/users".to_string(),
                headers: vec![],
                body: Some("{}".to_string()),
                size_bytes: 2,
            },
            response: None,
            status: "completed".to_string(),
            error: None,
        }];

        let findings = vec![SecurityFinding {
            id: "f-1".to_string(),
            exchange_id: "req-stride-1".to_string(),
            title: "Vazamento de Senha".to_string(),
            description: "Exposição".to_string(),
            severity: crate::security::types::FindingSeverity::Critical,
            category: FindingCategory::DataExposure,
            remediation: "Corrigir".to_string(),
            affected_resource: "POST /api/users".to_string(),
            timestamp: 0,
        }];

        let report = generate_stride_report(&exchanges, &findings);
        assert!(report.total_threats >= 2);
        assert_eq!(report.trust_boundaries.len(), 3);
    }

    #[test]
    fn test_empty_stride_report_when_no_traffic() {
        let exchanges = vec![];
        let findings = vec![];
        let report = generate_stride_report(&exchanges, &findings);
        assert_eq!(report.total_threats, 0);
        assert_eq!(report.high_risk_count, 0);
        assert!(report.threats.is_empty());
    }
}
