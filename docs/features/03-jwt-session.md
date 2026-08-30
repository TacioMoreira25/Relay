# 🛡️ Funcionalidade: Auto-Captura de JWT & Gestão de Sessão

## 1. Visão Geral
Durante o fluxo de desenvolvimento de APIs, copiar e colar tokens JWT para testar requisições autenticadas é um processo repetitivo. O Relay intercepta automaticamente cabeçalhos e respostas de login para extrair e armazenar os tokens válidos.

## 2. Mecanismo de Detecção
1. **Request Headers:** Inspeção de `Authorization: Bearer <token>`.
2. **Response Headers:** Verificação de headers `Set-Cookie` ou customizados `X-Auth-Token`.
3. **Response Body:** Parse de campos JSON comuns de autenticação (`access_token`, `token`, `jwt`, `id_token`).

## 3. Decodificação de Claims
O estado interno decodifica o payload base64 do JWT sem validar assinaturas localmente, extraindo campos como:
* `sub` (Subject / ID do Usuário)
* `exp` (Data de Expiração com contagem regressiva)
* `roles` / `permissions`
