use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredTarget {
    pub id: String,
    pub label: String,
    pub host: String,
    pub port: u16,
    pub is_active: bool,
    pub source: String, // "auto_discovered", "manual", "remote"
}

/// Formata a identificação do serviço de forma limpa e neutra
fn format_target_label(_host: &str, port: u16) -> String {
    format!("Localhost :{}", port)
}

/// Testa se uma porta TCP está ativa de forma não-bloqueante com timeout curto (100ms)
async fn probe_port(host: &str, port: u16, listen_port: u16) -> bool {
    // Nunca escaneia a si mesmo
    if port == listen_port {
        return false;
    }
    let addr = format!("{}:{}", host, port);
    match timeout(Duration::from_millis(100), TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => true,
        _ => false,
    }
}

/// Realiza a varredura concorrente e RETORNA APENAS PORTAS QUE ESTÃO REALMENTE ATIVAS (is_active == true)
pub async fn scan_local_targets_with_listen_port(listen_port: u16) -> Vec<DiscoveredTarget> {
    let dev_ports: &[u16] = &[
        3000, 3001, 3002, 3003, 3004, 3005,
        4000, 4200, 4201, 4202,
        5000, 5173, 5174, 5175,
        8000, 8001, 8081, 8082, 8083, 8084, 8085,
    ];
    let mut tasks = Vec::new();

    for &port in dev_ports {
        let lp = listen_port;
        tasks.push(tokio::spawn(async move {
            let is_active = probe_port("127.0.0.1", port, lp).await;
            if is_active {
                Some(DiscoveredTarget {
                    id: format!("auto-127.0.0.1-{}", port),
                    label: format_target_label("127.0.0.1", port),
                    host: "127.0.0.1".to_string(),
                    port,
                    is_active: true,
                    source: "auto_discovered".to_string(),
                })
            } else {
                None
            }
        }));
    }

    let mut results = Vec::new();
    for task in tasks {
        if let Ok(Some(target)) = task.await {
            results.push(target);
        }
    }

    // Ordena as portas ativas por número de porta
    results.sort_by(|a, b| a.port.cmp(&b.port));
    results
}

pub async fn scan_local_targets() -> Vec<DiscoveredTarget> {
    scan_local_targets_with_listen_port(8080).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_target_label() {
        assert_eq!(format_target_label("127.0.0.1", 3000), "Localhost :3000");
        assert_eq!(format_target_label("127.0.0.1", 8081), "Localhost :8081");
    }
}
