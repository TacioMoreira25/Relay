# Roteamento Declarativo por Prefixo & Arquivo de Configuração

O Relay permite automatizar e versionar configurações de projeto através de arquivos `relay.config.json` e roteamento multisserviço.

---

## 1. Arquivo `relay.config.json`

Exemplo de estrutura suportada na raiz do projeto:

```json
{
  "listenPort": 8080,
  "targetHost": "127.0.0.1",
  "targetPort": 3000,
  "latencyMs": 0,
  "jitterMs": 0,
  "simulateFailureRate": 0.0,
  "failureStatusCode": 500,
  "autoExtractJwt": true,
  "routes": [
    {
      "pathPrefix": "/api/v1/auth",
      "targetPort": 4000
    },
    {
      "pathPrefix": "/api/v1/billing",
      "targetPort": 5000
    }
  ]
}
```

---

## 2. Roteamento Multisserviço por Prefixo (`resolve_route_target`)

- Requisições que casam com o prefixo `/api/v1/auth` são direcionadas automaticamente para a porta `4000` (Microsserviço de Autenticação).
- Requisições que casam com `/api/v1/billing` vão para a porta `5000` (Microsserviço Financeiro).
- Todas as demais rotas são encaminhadas para o Host Padrão (`targetHost:targetPort`).

---

## 3. Importação e Exportação pela Interface

No modal de configurações:
- **Importar JSON**: Carrega instantaneamente o arquivo de configuração do projeto.
- **Salvar JSON**: Exporta as configurações atuais para download imediato.
