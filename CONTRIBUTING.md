# 🤝 Guia de Contribuição & Boas Práticas (Relay)

Obrigado pelo interesse em contribuir com o **Relay**! Este documento orienta sobre o fluxo de trabalho, padrões arquiteturais e como submeter alterações com qualidade.

---

## 🛠️ 1. Ambiente Local de Desenvolvimento

Certifique-se de ter instalado os pré-requisitos descritos no [Guia de Instalação Linux](docs/installation/linux.md).

### Comandos Essenciais do Dia a Dia

| Tarefa | Comando |
| :--- | :--- |
| **Iniciar App com Hot-Reload** | `npm run tauri dev` |
| **Verificar Tipagem do Frontend** | `npm run check` |
| **Formatar Código Rust** | `cargo fmt --all --manifest-path src-tauri/Cargo.toml` |
| **Linters e Boas Práticas Rust** | `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` |
| **Build de Produção Nativo** | `npm run tauri build` |

---

## 📐 2. Padrões de Código Mandatórios

### 🦀 Backend (Rust)
* **Zero `unsafe`:** Exceto quando estritamente indispensável em FFI.
* **Sem `.unwrap()` em código de produção:** Trate explicitamente erros com `Result<T, E>`, `thiserror` ou `anyhow`.
* **Async Tokio Não-Bloqueante:** Nunca use `std::thread::sleep`. Utilize `tokio::time::sleep` ou canais assíncronos (`tokio::sync`).
* **Zero-Copy First:** Favoreça o uso de fatias e `bytes::Bytes` para duplicação e repasse de pacotes de rede para o IPC.
* **Formatação:** Sempre execute `cargo fmt` antes de abrir um Pull Request.

### ⚡ Frontend (Svelte 5 & TypeScript)
* **Svelte 5 Runes:** Utilize exclusivamente `$state`, `$derived`, `$props()` e `$bindable()`. Evite stores clássicas com assinaturas manuais (`.subscribe()`).
* **Strict TypeScript:** Proibido o uso de `any`. Todos os tipos trafegados via IPC devem constar em `src/lib/types/index.ts`.
* **Limpeza de Eventos:** Garanta o unlisten de eventos Tauri (`listen(...)`) no retorno de limpeza do `onMount`.

---

## 🔀 3. Fluxo de Git e Branches

1. Crie uma branch a partir da `main`:
   ```bash
   git checkout -b feature/nome-da-feature
   # ou
   git checkout -b fix/descricao-do-bug
   ```
2. Escreva commits semânticos seguindo a convenção [Conventional Commits](https://www.conventionalcommits.org/):
   * `feat: add jwt decoding in session store`
   * `fix: handle connection drop on upstream handshake`
   * `style: apply rustfmt`
   * `docs: update proxy engine technical specs`
3. Abra um Pull Request referenciando a issue correspondente (ex: `Closes #2`).
