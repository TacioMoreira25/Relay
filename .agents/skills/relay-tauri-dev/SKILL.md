---
name: relay-tauri-dev
description: Guidelines and architectural best practices for developing the Relay project (Tauri v2 + Rust + Svelte 5 + Hyper).
---

# Relay Development & Architecture Skill

When working on the Relay codebase, always follow these rules and design patterns:

## 1. Rust Backend Standards
- **Zero unsafe:** Never use `unsafe` blocks unless strictly interacting with FFI.
- **Zero unwrap in production:** Always propagate errors with `Result<T, E>` and `thiserror` / `anyhow`.
- **Async Tokio:** Never block Tokio worker threads. Use `tokio::time::sleep`, `tokio::sync`, or `parking_lot::RwLock` for fast in-memory read locks.
- **Zero-Copy First:** Use `bytes::Bytes` and slices (`&[u8]`) when forwarding HTTP payloads between proxy connections and Tauri IPC.

## 2. Frontend Standards (Svelte 5)
- **Runes Only:** Always use `$state`, `$derived`, `$props()`, `$bindable()` and `$effect`. Do not use classic Svelte store `.subscribe()` or writable stores in components.
- **Strict TypeScript:** No `any`. All IPC events and payloads must match types defined in `$lib/types/index.ts`.
- **Event Listeners:** Always clean up Tauri event listeners (`listen(...)`) in `onDestroy` or the teardown return of `onMount`.
