# Auditoria Shift-Left de Seguranca (DAST) e Modelo STRIDE

O modulo de **Seguranca** do Relay atua como uma camada defensiva integrada ao ciclo de vida de desenvolvimento de APIs, permitindo identificar vulnerabilidades tecnicas e estruturais antes do envio do codigo para homologacao ou producao.

---

## 1. Auditoria DAST em Tempo Real (Analise Passiva)

O motor de inspecao passiva (`security::linter`) avalia continuamente todos os pacotes HTTP que transitam pelo proxy reverso sem interferir na latencia ou na entrega original dos dados.

### Verificacoes Passivas Automatizadas:
1. **CORS Inseguro:** Detecta utilizacao indevida de curinga `Access-Control-Allow-Origin: *` combinado com `Access-Control-Allow-Credentials: true`.
2. **Security Headers Ausentes:** Verifica presenca de cabecalhos essenciais como `X-Content-Type-Options: nosniff`, `X-Frame-Options` e politicas de protecao contra clickjacking e MIME-sniffing.
3. **Exposicao de Dados Sensiveis:** Inspeciona corpos de requisicao e resposta em busca de campos restritos expostos indevidamente (senhas em texto plano, hashes Bcrypt, chaves privadas e tokens de infraestrutura).
4. **Vazamento Tecnologico (Tech Leak):** Notifica presenca de cabecalhos reveladores de runtime e infraestrutura como `Server`, `X-Powered-By`, `X-Runtime` e `X-AspNet-Version`.
5. **Vulnerabilidades em Tokens JWT:** Valida se tokens capturados estao configurados com algoritmo inseguro (`none`), ausencia de claims de expiracao (`exp`) ou validade expirada.

---

## 2. Testes Ativos de Seguranca (Active Probes)

Alem da auditoria passiva, o modulo de testes ativos (`security::probes`) permite disparar simulacoes direcionadas contra endpoints especificos:

* **BOLA / IDOR (Broken Object Level Authorization):** Simula requisicoes com mutacao automatica de identificadores numericos ou UUIDs em rotas parametrizadas para testar controle de acesso a nivel de objeto.
* **Mass Assignment:** Injeta atributos privilegiados (ex: `role: admin`, `is_admin: true`, `verified: true`) em cargas uteis `POST`, `PUT` e `PATCH` para detectar absorcao automatica de parametros por ORMs.
* **Auth Bypass:** Dispara a rota removendo cabecalhos de autenticacao (`Authorization`, cookies de sessao) para certificar que rotas privadas rejeitam chamadas anonimas com status `401 Unauthorized` ou `403 Forbidden`.

---

## 3. Modelagem de Ameacas STRIDE

O motor arquitetural (`security::stride`) analisa as fronteiras de confianca e a topologia de conexao entre o Cliente HTTP, o Proxy Relay e os servicos Upstream de backend:

| Categoria STRIDE | Significado Arquitetural | Mitigacao Recomendada |
| :--- | :--- | :--- |
| **Spoofing** | Falsificacao de Identidade | Assinatura criptografica de tokens JWT e validacao mutua TLS (mTLS) |
| **Tampering** | Adulteracao de Dados em Transito | Uso estrito de HTTPS/TLS e integridade de payload com HMAC/SHA-256 |
| **Repudiation** | Repudio ou Falta de Auditoria | Logs auditaveis estruturados e rastreabilidade de transacoes (Correlation IDs) |
| **Info Disclosure** | Vazamento ou Exposicao de Dados | Sanitizacao de cabecalhos de tecnologia e mascaramento de dados sensiveis |
| **Denial of Service** | Negacao de Servico (DoS) | Rate limiting, timeouts agressivos de upstream e quotas por cliente |
| **Elevation of Privilege** | Elevacao Indevida de Privilegios | Validacao de escopos e autorizacao RBAC/ABAC em todos os controladores de dominio |

---

## 4. Painel de Controle e Usabilidade

* **Filtro de Severidades Integrado:** Navegacao rapida por niveis de criticidade (`Todos`, `Criticos`, `Altos`, `Medios`, `Baixos`, `Info`).
* **Score Consolidado de Conformidade:** Calculo dinamico de conformidade (0 a 100) com classificacao visual imediata.
* **Layout Adaptavel em Tres Zonas:** Seletor de modo a esquerda, filtros centralizados e acoes contextuais a direita sem espacos mortos.
* **Barra Lateral com Ajuste de Largura:** Redimensionamento livre com persistencia automatica no armazenamento local (`localStorage`).
* **Guia de Remediacao Didatico:** Cada achado acompanha explicacao tecnica aprofundada, rota afetada e codigo pronto para copiar e colar para correcao no backend.
