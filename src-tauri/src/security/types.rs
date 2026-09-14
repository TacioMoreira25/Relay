use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    Cors,
    SecurityHeaders,
    DataExposure,
    TechLeak,
    JwtVulnerability,
    MissingAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityFinding {
    pub id: String,
    pub exchange_id: String,
    pub title: String,
    pub description: String,
    pub severity: FindingSeverity,
    pub category: FindingCategory,
    pub remediation: String,
    pub affected_resource: String,
    pub timestamp: i64,
}
