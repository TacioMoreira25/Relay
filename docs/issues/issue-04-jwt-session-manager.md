---
title: "[Fase 4] Auto-Detecção de JWT & Gerenciador de Sessão"
labels: ["fase-4", "security", "jwt"]
---

### 📋 Descrição
Detecção heurística automática de tokens JWT que transitam pelo proxy e painel de inspeção de claims.

### 🎯 Tarefas & Entregáveis
- [ ] Scanner regex de tokens Bearer em cabeçalhos de requisição e resposta.
- [ ] Parser de JSON responses para detecção de campos de autenticação (`access_token`, `token`).
- [ ] Decodificação de claims JWT (sub, exp, roles) e indicador de tempo de validade.
- [ ] Ação de 1 clique para copiar token decodificado para o clipboard do SO.
