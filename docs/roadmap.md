# 🗺️ Roadmap de Desenvolvimento do Relay (8 Fases)

Este documento define o plano de entrega do Relay, partindo do MVP até a maturidade completa do produto.

---

### 🔹 Fase 1: Fundação & Setup da Arquitetura (MVP Core)
- [x] Estruturação do workspace Tauri v2 + Rust + Svelte 5 + TailwindCSS.
- [x] Configuração de tipagem estrita no TypeScript e structs serializáveis no Rust.
- [x] Implementação do esqueleto de comando IPC para inicialização/parada do Proxy.

### 🔹 Fase 2: Motor de Proxy HTTP/1.1 Assíncrono
- [x] Implementação do listener TCP assíncrono com Hyper 1.x e Tokio.
- [x] Encaminhamento transparente de requisições e respostas para o upstream configurado com sanitização de headers hop-by-hop.
- [x] Mecanismo de shutdown gracioso do servidor proxy via canais `tokio::sync::watch`.
- [x] Modal de configuração interativa de portas (`listenPort`, `targetHost`, `targetPort`, `latencyMs`) no frontend Svelte 5.
- [x] Teste unitário automatizado do repasse HTTP assíncrono com mock listener Tokio.

### 🔹 Fase 3: Interceptador & Streaming de Tráfego em Tempo Real
- [x] Captura de headers, método, URI e corpo da requisição sem bloqueio do stream assíncrono.
- [x] Streaming de eventos `relay:request`, `relay:response` e `relay:error` para a interface desktop via IPC.
- [x] Renderização reativa de alto desempenho no `RequestList.svelte` com filtros por método HTTP (GET, POST, PUT, DELETE, PATCH).
- [x] Filtros em tempo real por faixa de Status Code (2xx, 3xx, 4xx, 5xx, ERR) e busca global de texto em URIs e bodies.
- [x] Auto-formatação e identação inteligente de JSON no `Inspector.svelte` com botões rápidos de cópia para a área de transferência.

### 🔹 Fase 4: Auto-Detecção & Gerenciador de Sessão JWT
- [x] Parser regex/heurístico de tokens JWT em cabeçalhos (`Authorization: Bearer ...`, `x-access-token`, `token`).
- [x] Scanner automático de tokens JWT em payloads de resposta JSON (campos `token`, `accessToken`, `jwt`, `id_token`).
- [x] Armazenamento seguro de sessão em memória com decodificação base64 URL-safe e extração de claims (`sub`, `iss`, `exp`).
- [x] Painel visual dedicado na UI (`JwtManager.svelte`) com cálculo de tempo de expiração (`exp`), visualização de claims formatados e botões de cópia rápida para `Bearer <token>`.
- [x] Testes unitários no Rust para decodificação de JWT e extração de headers (`test_decode_jwt_token`, `test_extract_jwts_from_headers`).

### 🔹 Fase 5: Simulador de Falhas de Rede & Chaos Engineering
- [x] Injeção de latência base com suporte a Jitter aleatório configurável (`latency_ms` + `jitter_ms`).
- [x] Injeção de taxa de falhas percentuais controladas (`simulate_failure_rate` de 0% a 100%) com retorno imediato de erro simulado sem onerar o upstream.
- [x] Suporte a seleção de códigos de erro HTTP customizados (500 Internal Server Error, 502 Bad Gateway, 503 Service Unavailable, 504 Gateway Timeout, 429 Too Many Requests) com cabeçalho de rastreio `x-relay-chaos: simulated-failure`.
- [x] Painel visual e badges de status em tempo real na barra de ferramentas e no modal de configurações.
- [x] Testes unitários de cálculo de jitter e probabilidade de falhas (`test_calculate_delay_jitter`, `test_should_simulate_failure`).

### 🔹 Fase 6: Cliente HTTP & Replay de Chamadas
- [x] Ação de "Replay Chamada" no inspetor para carregar qualquer requisição capturada no editor.
- [x] Editor completo de método HTTP, URI/query parameters, tabela de cabeçalhos dinâmicos e textarea de payload body.
- [x] Botão rápido de "Auto-Injetar JWT da Sessão" para preencher automaticamente o cabeçalho `Authorization: Bearer <token>` com o token ativo mais recente.
- [x] Execução nativa no backend Rust via comando `execute_replay` com streaming direto do resultado para a lista de tráfego e inspector.

### 🔹 Fase 7: Otimizações para Linux & KDE Plasma
- [x] Suporte a System Tray com menu de contexto nativo ("Mostrar / Ocultar Relay", "Encerrar") e clique rápido na bandeja.
- [x] Integração de atalhos de teclado no desktop (`Ctrl+K` para busca global, `Ctrl+L` para limpar tráfego, `Ctrl+P` para ligar/desligar proxy).
- [x] Pipeline de CI no GitHub Actions configurado com job automatizado de empacotamento Linux (AppImage & Debian/Ubuntu `.deb`).

### 🔹 Fase 8: Suporte a HTTPS / MITM & Exportação de Tráfego
- [x] Geração dinâmica de certificados raiz locais (CA) para interceptação transparente de HTTPS (`rcgen`, chaves RSA/ECDSA e download de arquivos `.crt` e `.key`).
- [x] Exportação de sessões de tráfego completas para a especificação oficial HAR 1.2 (compatível com DevTools, Postman, Insomnia e Charles).
- [x] Geração automática de especificações OpenAPI 3.0 (Swagger) baseadas no tráfego HTTP observado em tempo de execução.
- [x] Modal unificado de exportação e HTTPS (`ExportModal.svelte`) com atalho rápido de teclado `Ctrl+E`.
- [x] Testes unitários no Rust para geração de CA (`test_generate_root_ca`), exportação HAR (`test_export_to_har`) e OpenAPI (`test_export_to_openapi`).
