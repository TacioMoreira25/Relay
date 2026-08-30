# Arquitetura do Relay

## Visão Geral

O Relay opera através de uma arquitetura híbrida dividida em dois domínios:
1. **Core Engine (Rust):** Responsável pelo socket listening, proxy forwarding, injeção de latência, parsers de HTTP e armazenamento de tokens.
2. **Interface de Usuário (Svelte/WebKit):** Responsável por exibir o histórico de tráfego, permitir edição de headers, replay e configuração de parâmetros.

```
┌────────────────────────────────────────────────────────┐
│               Svelte 5 Frontend (UI)                   │
│   RequestList  │  Inspector  │  Replay  │  Settings    │
└──────────────────────────┬─────────────────────────────┘
                           │ (Tauri IPC Events & Commands)
┌──────────────────────────▼─────────────────────────────┐
│                 Tauri v2 Core Bridge                   │
└──────────────────────────┬─────────────────────────────┘
                           │
┌──────────────────────────▼─────────────────────────────┐
│                 Rust Backend Engine                    │
│  ┌───────────────────────┐   ┌───────────────────────┐ │
│  │   Proxy Server        │   │   Session / JWT Store │ │
│  │   (Tokio + Hyper)     │   │   (RwLock State)      │ │
│  └───────────────────────┘   └───────────────────────┘ │
└────────────────────────────────────────────────────────┘
```
