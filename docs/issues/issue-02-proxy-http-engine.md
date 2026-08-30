---
title: "[Fase 2] Motor de Proxy HTTP/1.1 Assíncrono com Hyper"
labels: ["fase-2", "rust", "networking"]
---

### 📋 Descrição
Desenvolvimento do motor central de proxy TCP/HTTP com repasse assíncrono para servidores upstream.

### 🎯 Tarefas & Entregáveis
- [ ] Implementar TCP Listener com `tokio::net::TcpListener` e handshake Hyper 1.x.
- [ ] Implementar encaminhamento transparente de conexões para o alvo upstream (`target_host` / `target_port`).
- [ ] Suporte a shutdown gracioso e reinício de porta via `tokio::sync::watch`.
- [ ] Preservação de headers sensíveis (ex: Host header handling) e conexões keep-alive.
