use rcgen::{
    BasicConstraints, CertificateParams, DnType, IsCa, KeyPair, KeyUsagePurpose,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedCa {
    pub cert_pem: String,
    pub key_pem: String,
    pub common_name: String,
    pub created_at: i64,
}

/// Gera uma autoridade de certificação raiz (Root CA) autoassinada para interceptação HTTPS / MITM
pub fn generate_root_ca(common_name: &str) -> Result<GeneratedCa, String> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params
        .distinguished_name
        .push(DnType::CommonName, common_name);
    params
        .distinguished_name
        .push(DnType::OrganizationName, "Relay HTTP Proxy & Inspector");
    params
        .distinguished_name
        .push(DnType::OrganizationalUnitName, "Security & Debugging CA");

    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);

    let keypair = KeyPair::generate()
        .map_err(|e| format!("Falha ao gerar par de chaves RSA/ECDSA: {}", e))?;
    let cert = params
        .self_signed(&keypair)
        .map_err(|e| format!("Falha ao assinar certificado raiz CA: {}", e))?;

    let cert_pem = cert.pem();
    let key_pem = keypair.serialize_pem();

    Ok(GeneratedCa {
        cert_pem,
        key_pem,
        common_name: common_name.to_string(),
        created_at: chrono::Utc::now().timestamp_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_root_ca() {
        let ca = generate_root_ca("Relay Root CA Local").unwrap();
        assert!(ca.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(ca.cert_pem.contains("END CERTIFICATE"));
        assert!(ca.key_pem.contains("BEGIN PRIVATE KEY"));
        assert_eq!(ca.common_name, "Relay Root CA Local");
    }
}
