---
title: "[Fase 1] Fundação e Setup da Arquitetura do Relay"
labels: ["fase-1", "enhancement", "architecture"]
---

### 📋 Descrição
Estruturação inicial da arquitetura híbrida do Relay (Tauri v2 + Rust Tokio/Hyper + Svelte 5 com Runes + TailwindCSS).

### 🎯 Tarefas & Entregáveis
- [x] Inicialização do workspace Tauri v2 com empacotamento nativo Linux.
- [x] Configuração de tipos estritos TypeScript sincronizados com structs Rust Serde.
- [x] Gerenciamento de estado global no frontend com Svelte 5 Runes (`$state`, `$derived`).
- [x] Criação da skill de workspace para diretrizes de desenvolvimento (`.agents/skills/relay-tauri-dev/SKILL.md`).
- [x] Documentação inicial (ADRs, guias por funcionalidade e setup multi-distro Linux).
