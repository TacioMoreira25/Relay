# Roadmap de Desenvolvimento do Relay

Este documento define o plano de entrega do Relay, partindo do MVP ate a maturidade completa do produto.

---

### Fase 1: Fundacao & Setup da Arquitetura (MVP Core)
- [x] Estruturacao do workspace Tauri v2 + Rust + Svelte 5 + TailwindCSS.
- [x] Configuracao de tipagem estrita no TypeScript e structs serializaveis no Rust.
- [x] Implementacao do esqueleto de comando IPC para inicializacao/parada do Proxy.

### Fase 2: Motor de Proxy HTTP/1.1 Assincrono
- [x] Implementacao do listener TCP assincrono com Hyper 1.x e Tokio.
- [x] Encaminhamento transparente de requisicoes e respostas para o upstream configurado com sanitizacao de headers hop-by-hop.
- [x] Mecanismo de shutdown gracioso do servidor proxy via canais tokio::sync::watch.
- [x] Modal de configuracao interativa de portas no frontend Svelte 5.
- [x] Teste unitario automatizado do repasse HTTP assincrono com mock listener Tokio.

### Fase 3: Interceptador & Streaming de Trafego em Tempo Real
- [x] Captura de headers, metodo, URI e corpo da requisicao sem bloqueio do stream assincrono.
- [x] Streaming de eventos `relay:request`, `relay:response` e `relay:error` para a interface desktop via IPC.
- [x] Renderizacao reativa de alto desempenho no `RequestList.svelte` com filtros por metodo HTTP.
- [x] Filtros em tempo real por faixa de Status Code (2xx, 3xx, 4xx, 5xx, ERR) e busca global de texto.
- [x] Auto-formatacao e identacao inteligente de JSON no `Inspector.svelte` com botoes de copia.

### Fase 4: Auto-Deteccao & Gerenciador de Sessao JWT
- [x] Parser regex/heuristico de tokens JWT em cabecalhos e JSON bodies de resposta.
- [x] Armazenamento seguro de sessao em memoria com decodificacao base64 URL-safe e extracao de claims.
- [x] Painel visual dedicado na UI (`JwtManager.svelte`) com calculo de exp e visualizacao de claims.
- [x] Testes unitarios no Rust para decodificacao de JWT e extracao de headers.

### Fase 5: Simulador de Falhas de Rede & Chaos Engineering
- [x] Injecao de latencia base com suporte a Jitter aleatorio configuravel.
- [x] Injecao de taxa de falhas percentuais controladas (0% a 100%) com retorno imediato de erro simulado.
- [x] Suporte a selecao de codigos de erro HTTP customizados com cabecalho `x-relay-chaos`.
- [x] Testes unitarios de calculo de jitter e probabilidade de falhas.

### Fase 6: Cliente HTTP, Colecoes & Replay de Chamadas
- [x] Acao de Replay no inspetor para carregar qualquer requisicao capturada no editor.
- [x] Aba dedicada de Colecao com importacao de JSON e organizacao de endpoints prontos para teste.
- [x] Auto-injecao do Bearer Token JWT mais recente em rotas protegidas com 1 clique.
- [x] Auto-fechamento de chaves {}, aspas "" e colchetes [], suporte a Tab e formatador de JSON.
- [x] Auto-captura de variaveis de resposta (IDs, tokens) e encadeamento automatico em templates.

### Fase 7: Roteamento Multisservico, Mocks Locais & Ambientes
- [x] Roteamento dinamico por prefixo de caminho (API Gateway local).
- [x] Resposta de Mock local sem necessidade de backend ativo.
- [x] Auto-descoberta assincrona de portas abertas de desenvolvimento no Linux.
- [x] Gerenciador CRUD de ambientes persistidos no localStorage com edicao e exclusao.
- [x] Visualizador Diff lado a lado entre respostas e exportacao instantanea para cURL.

### Fase 8: Suporte a HTTPS / MITM & Exportacao de Trafego
- [x] Geracao dinamica de certificados raiz locais (CA) para interceptacao transparente de HTTPS.
- [x] Exportacao de sessoes completas para HAR 1.2 (DevTools, Postman, Insomnia).
- [x] Geracao automatica de especificacoes OpenAPI 3.0 (Swagger) baseadas no trafego observado.
- [x] Pipeline de CI rapido com testes automatizados e typecheck estrito.
