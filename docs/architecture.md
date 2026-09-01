# Arquitetura do Relay

## Visão Geral

O Relay opera através de uma arquitetura híbrida dividida em dois domínios principais:
1. **Core Engine (Rust / Hyper / Tokio):** Responsável pelo socket listening TCP, encaminhamento de requisições, injeção de Chaos (latência com jitter e falhas controladas), extração de tokens JWT, emissão de certificados CA e exportação HAR/OpenAPI.
2. **Interface de Usuário (Svelte 5 / TailwindCSS / WebKit):** Responsável pelo streaming reativo de tráfego, inspeção de headers/corpos com auto-identação JSON, editor de Replay e gerenciamento visual de rotas declarativas e projetos.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Svelte 5 Frontend (UI)                          │
│  ProjectSelector │ RequestList │ Inspector │ ReplayModal │ JwtManager  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ (Tauri v2 IPC Events & Commands)
┌───────────────────────────────────▼────────────────────────────────────┐
│                          Tauri v2 Core Bridge                          │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────┐
│                          Rust Backend Engine                           │
│  ┌─────────────────────────┐  ┌─────────────────────────────────────┐  │
│  │   Hyper 1.x Proxy Server│  │   Session / JWT Scanner & Decod.    │  │
│  │   (Tokio Async Streams) │  │   (RwLock Thread-Safe Memory)       │  │
│  ├─────────────────────────┤  ├─────────────────────────────────────┤  │
│  │   Chaos Simulator       │  │   OpenAPI / Postman / HAR Parser    │  │
│  │   (Jitter & Failures)   │  │   (Dynamic Collection Deserializer) │  │
│  ├─────────────────────────┤  ├─────────────────────────────────────┤  │
│  │   Prefix Route Resolver │  │   Root CA Certificate Generator     │  │
│  │   (Multi-target Proxy)  │  │   (rcgen X.509 Cryptography)        │  │
│  └─────────────────────────┘  └─────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Módulos Documentados

1. [Motor de Proxy HTTP/1.1](features/01-proxy-engine.md)
2. [Inspetor de Tráfego & Filtros](features/02-traffic-inspector.md)
3. [Auto-Detecção & Gerenciador JWT](features/03-jwt-session.md)
4. [Cliente HTTP & Replay de Chamadas](features/04-http-replay.md)
5. [Chaos Engineering & Simulador de Resiliência](features/05-chaos-engineering.md)
6. [Exportação HAR, OpenAPI 3.0 & HTTPS CA](features/06-export-and-https.md)
7. [Roteamento Declarativo por Prefixo & Arquivo de Configuração](features/07-declarative-routing-and-config.md)
8. [Otimizações Linux & Atalhos de Teclado](features/08-linux-and-keyboard-shortcuts.md)
9. [Gerenciamento de Projetos, Importação de Coleções e Pastas](features/11-projects-and-collections.md)
