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

/// Infere o nome típico do serviço dev com base na porta
fn infer_service_label(port: u16) -> &'static str {
    match port {
        3000 => "Node / Fastify / Next API",
        3001 => "Secondary API / NestJS",
        4200 => "Angular Dev Server",
        5000 => "Flask / Python Web",
        5173 => "Vite / Svelte / Vue",
        8000 => "Django / FastAPI / PHP",
        8081 => "Metro Bundler (React Native)",
        _ => "Local Service",
    }
}

/// Testa se uma porta TCP está ativa de forma não-bloqueante com timeout curto (120ms)
async fn probe_port(host: &str, port: u16, listen_port: u16) -> bool {
    // Nunca escaneia a si mesmo
    if port == listen_port {
        return false;
    }
    let addr = format!("{}:{}", host, port);
    match timeout(Duration::from_millis(120), TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => true,
        _ => false,
    }
}

/// Realiza a varredura concorrente de portas locais de desenvolvimento comuns excluindo a própria porta do proxy
pub async fn scan_local_targets_with_listen_port(listen_port: u16) -> Vec<DiscoveredTarget> {
    let dev_ports: &[u16] = &[3000, 3001, 4200, 5000, 5173, 8000, 8081];
    let mut tasks = Vec::new();

    for &port in dev_ports {
        let lp = listen_port;
        tasks.push(tokio::spawn(async move {
            let is_active = probe_port("127.0.0.1", port, lp).await;
            DiscoveredTarget {
                id: format!("auto-127.0.0.1-{}", port),
                label: infer_service_label(port).to_string(),
                host: "127.0.0.1".to_string(),
                port,
                is_active,
                source: "auto_discovered".to_string(),
            }
        }));
    }

    let mut results = Vec::new();
    for task in tasks {
        if let Ok(target) = task.await {
            results.push(target);
        }
    }

    // Ordena colocando as portas ativas no topo
    results.sort_by(|a, b| b.is_active.cmp(&a.is_active).then_with(|| a.port.cmp(&b.port)));
    results
}

pub async fn scan_local_targets() -> Vec<DiscoveredTarget> {
    scan_local_targets_with_listen_port(8080).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_service_label() {
        assert_eq!(infer_service_label(3000), "Node / Fastify / Next API");
        assert_eq!(infer_service_label(5173), "Vite / Svelte / Vue");
        assert_eq!(infer_service_label(4200), "Angular Dev Server");
        assert_eq!(infer_service_label(9999), "Local Service");
    }
}
