# 🗺️ Roadmap de Desenvolvimento do Relay (8 Fases)

Este documento define o plano de entrega do Relay, partindo do MVP até a maturidade completa do produto.

---

### 🔹 Fase 1: Fundação & Setup da Arquitetura (MVP Core)
- [x] Estruturação do workspace Tauri v2 + Rust + Svelte 5 + TailwindCSS.
- [x] Configuração de tipagem estrita no TypeScript e structs serializáveis no Rust.
- [x] Implementação do esqueleto de comando IPC para inicialização/parada do Proxy.

### 🔹 Fase 2: Motor de Proxy HTTP/1.1 Assíncrono
- [ ] Implementação do listener TCP assíncrono com Hyper 1.x e Tokio.
- [ ] Encaminhamento transparente de requisições e respostas para o upstream configurado.
- [ ] Mecanismo de shutdown gracioso do servidor proxy via canais `tokio::sync::watch`.

### 🔹 Fase 3: Interceptador & Streaming de Tráfego em Tempo Real
- [ ] Captura de headers, método, URI e corpo da requisição sem bloqueio do stream.
- [ ] Streaming de eventos `relay:request` e `relay:response` para a interface desktop.
- [ ] Renderização virtualizada da lista de tráfego na UI com filtros por método HTTP e busca por texto.

### 🔹 Fase 4: Auto-Detecção & Gerenciador de Sessão JWT
- [ ] Parser regex/heurístico de tokens JWT em headers (`Authorization: Bearer ...`) e payloads de resposta.
- [ ] Armazenamento em memória com descriptografia e decodificação automática de claims.
- [ ] Painel lateral na UI para visualização e cópia rápida de tokens detectados.

### 🔹 Fase 5: Simulador de Falhas de Rede & Chaos Engineering
- [ ] Suporte à injeção de latência configurável (milissegundos com jitter opcional).
- [ ] Configuração de taxa percentual de erros simulados (ex: responder 500/503 aleatoriamente).
- [ ] Painel de controle na barra superior para ajuste dinâmico em runtime.

### 🔹 Fase 6: Cliente HTTP & Replay de Chamadas
- [ ] Ação de "Replay" no inspetor para reenviar qualquer chamada gravada.
- [ ] Editor integrado de parâmetros, query strings e headers para repetição personalizada.
- [ ] Auto-injeção do último token JWT válido durante o replay.

### 🔹 Fase 7: Otimizações para Linux & KDE Plasma
- [ ] Suporte a System Tray com menu de contexto para controle rápido do proxy.
- [ ] Integração de atalhos globais de teclado no desktop.
- [ ] Empacotamento para Fedora (.rpm), Flatpak e AppImage com pipeline CI automatizado.

### 🔹 Fase 8: Suporte a HTTPS / MITM & Exportação de Tráfego
- [ ] Geração dinâmica de certificados raiz locais (CA) para interceptação transparente de HTTPS.
- [ ] Exportação e importação de sessões de tráfego nos formatos HAR e OpenAPI 3.0.
- [ ] Documentação completa de uso e release da versão 1.0 estável.
