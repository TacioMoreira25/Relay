# Funcionalidade: Auto-Captura de JWT & Gestão de Sessão

## 1. Visão Geral
Durante o fluxo de desenvolvimento de APIs, copiar e colar tokens JWT para testar requisições autenticadas é um processo repetitivo e propenso a erros. O Relay intercepta automaticamente cabeçalhos e respostas de login para extrair, decodificar e disponibilizar os tokens no ecossistema da aplicação.

## 2. Mecanismo de Detecção Abrangente
A extração opera de forma contínua tanto no tráfego interceptado pelo Proxy quanto nas chamadas disparadas manualmente pelo Replay:
1. **Request Headers:** Inspeção de `Authorization` (com ou sem prefixo `Bearer`), `Authentication`, `x-access-token`, `x-auth-token`, `jwt`, `token`, `access-token`, `id-token` e cookies de requisição.
2. **Response Headers:** Verificação de cabeçalhos `Set-Cookie` (extraindo tokens contidos em cookies com atributos), `Authorization` ou cabeçalhos de resposta customizados.
3. **Response Body:**
   * **JSON Estruturado:** Varredura recursiva de objetos em busca de propriedades comuns (`access_token`, `token`, `jwt`, `id_token`, `data.token`, etc.), incluindo tokens que iniciam com `Bearer ey...`.
   * **Corpos em Texto / Raw:** Varredura tolerante de palavras que atendem ao padrão JWT de três segmentos delimitados por ponto (`header.payload.signature`).
4. **Decodificação Resiliente:** Suporta base64 URL-safe e standard com ou sem padding (`=`).

## 3. Decodificação de Claims e Injeção Dinâmica
O estado decodifica o payload base64 do JWT e extrai:
* `sub` (Subject / ID do Usuário, suportando strings ou números)
* `iss` (Issuer / Emissor)
* `exp` (Data de Expiração com cálculo de validade em tempo real)
* Variáveis dinâmicas automáticas injetadas na store: `{{token}}`, `{{sub}}`, `{{customerId}}` para reuso direto no Replay.
