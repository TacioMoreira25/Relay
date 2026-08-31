# Mocks Locais, Gerenciamento de Ambientes e Auto-Descoberta de Portas

Esta funcionalidade expande o Relay para atuar como um API Gateway local e cliente HTTP inteligente durante o desenvolvimento frontend e backend.

---

## 1. Roteamento por Prefixo e Mocks Locais (Sem Backend)

O Relay permite definir regras flexíveis de roteamento multisserviço por prefixo de caminho (`pathPrefix`).

### Casos de Uso:
1. **Unificação de Portas para o Frontend:**
   * Frontend aponta apenas para `http://localhost:8080`.
   * `/auth` encaminhado para `:3000`.
   * `/users` encaminhado para `:3001`.
   * `/payments` encaminhado para `:3002`.
2. **Mocking Local:**
   * Quando uma rota for marcada como `isMock: true`, o Relay responde imediatamente com o Status Code e JSON configurados, sem tentar conectar a nenhum servidor externo.

---

## 2. Auto-Descoberta de Portas no Linux

O Relay possui uma rotina de escaneamento assíncrono e não-bloqueante no Rust (`src-tauri/src/proxy/scanner.rs`) que varre portas de desenvolvimento comuns (`3000`, `3001`, `4200`, `5000`, `5173`, `8000`, `8081`).

* **Inferência de Tipo:** Identifica automaticamente se a porta aberta pertence a um servidor Vite/Svelte, Angular, Node/Fastify ou Python.
* **Conexão com 1 Clique:** Ao clicar no serviço detectado no menu suspenso, o proxy atualiza o alvo imediatamente.

---

## 3. Gerenciamento de Ambientes (CRUD)

* **Persistência Local:** Ambientes salvos (ex: `Staging AWS`, `API Local 4000`) são persistidos no navegador.
* **Ações Rápidas:** Suporte a criação, edição (nome, host, porta, SSL/HTTPS) e exclusão.
* **Status em Tempo Real:** Indicador visual de porta ativa (verde pulsante) ou desconectada (cinza).

---

## 4. Encadeamento Dinâmico de Variáveis de Teste (Postman-Like)

* **Auto-Captura:** Ao executar chamadas de cadastro ou login, o Relay extrai automaticamente IDs (`customerId`, `id`, `accountId`, `token`) dos JSONs de resposta.
* **Substituição de Variáveis:** Nos templates salvos ou no editor de Replay, campos com `{{customerId}}` ou `{{token}}` são substituídos em tempo de execução pelos valores capturados.
