# 🔌 Funcionalidade: Motor de Proxy Reverso Local & Interceptação

## 1. Visão Geral
O motor de proxy do Relay é implementado em **Rust nativo com Tokio e Hyper 1.x**, desenhado especificamente para desenvolvedores que necessitam inspecionar chamadas HTTP/REST sem o overhead de ferramentas pesadas.

## 2. Arquitetura e Funcionamento
* **TCP Listener Assíncrono:** Abre um socket TCP não-bloqueante na porta configurada (padrão `127.0.0.1:8080`).
* **Handshake HTTP/1.1 Upstream:** Cada requisição recebida é desmembrada em memória usando fatias `bytes::Bytes` (*Zero-Copy*) e reencaminhada para o servidor de destino (ex: `127.0.0.1:3000`).
* **Shutdown Gracioso:** O proxy pode ser iniciado e pausado dinamicamente através de canais `tokio::sync::watch` acionados pelos comandos Tauri IPC da interface.

## 3. Injeção de Falhas e Caos de Rede
* **Latência Artificial (Jitter):** Possibilidade de injetar atrasos configuráveis (`latencyMs`) antes do repasse ao upstream, permitindo testar spinners de carregamento e timeouts no frontend.
* **Taxa de Falha Simulada:** Permite forçar respostas HTTP 500/502 controladas para testar a resiliência de clientes HTTP.

## 4. Tipos e Contratos (Rust & TypeScript)
```typescript
export interface ProxyConfig {
  listenPort: number;
  targetHost: string;
  targetPort: number;
  latencyMs: number;
  simulateFailureRate: number;
  autoExtractJwt: boolean;
}
```
