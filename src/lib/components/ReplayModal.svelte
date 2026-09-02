<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { HeaderEntry, HttpExchange, HttpMethod, SavedRequestTemplate } from "$lib/types";
  import { IconPlay, IconKey, IconWand2, IconSparkles } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let {
    isOpen = $bindable(false),
    exchange = null,
    template = null
  }: {
    isOpen: boolean;
    exchange: HttpExchange | null;
    template?: SavedRequestTemplate | null;
  } = $props();

  let method = $state<HttpMethod>("POST");
  let uri = $state<string>("/");
  let headers = $state<HeaderEntry[]>([]);
  let body = $state<string>("");
  let isSending = $state<boolean>(false);
  let statusMessage = $state<string | null>(null);

  // Variáveis disponíveis para auto-complete
  let showVariableDropdown = $state<boolean>(false);
  let availableVars = $derived(Object.keys(relayState.activeVariables));

  // Inicializa uma única vez na montagem do modal
  onMount(() => {
    if (template) {
      method = template.method;
      uri = template.uri;
      headers = template.headers ? template.headers.map(h => ({ ...h })) : [];
      body = template.body || "";

      // Aplica substituição de variáveis pré-existentes (ex: {{customerId}}, {{token}})
      uri = relayState.replaceVariables(uri);
      body = relayState.replaceVariables(body);

      if (template.requiresAuth && relayState.jwts.length > 0) {
        const authIdx = headers.findIndex(h => h.key.toLowerCase() === "authorization");
        const bearer = `Bearer ${relayState.jwts[0].token}`;
        if (authIdx >= 0) {
          headers[authIdx].value = bearer;
        } else {
          headers.push({ key: "Authorization", value: bearer });
        }
      }
    } else if (exchange) {
      method = exchange.request.method;
      uri = exchange.request.uri;
      headers = exchange.request.headers.map(h => ({ ...h }));
      body = exchange.request.body || "";
    } else {
      method = "POST";
      uri = "/";
      headers = [
        { key: "Content-Type", value: "application/json" },
        { key: "Accept", value: "application/json" }
      ];
      body = "{\n  \n}";
    }
  });

  function closeModal(): void {
    isOpen = false;
  }

  function addHeader(): void {
    headers.push({ key: "", value: "" });
  }

  function removeHeader(index: number): void {
    headers.splice(index, 1);
  }

  function formatJsonBody(): void {
    if (!body.trim()) return;
    try {
      const parsed = JSON.parse(body);
      body = JSON.stringify(parsed, null, 2);
      statusMessage = "JSON formatado!";
      setTimeout(() => (statusMessage = null), 2000);
    } catch {
      statusMessage = "JSON inválido. Verifique vírgulas e aspas.";
      setTimeout(() => (statusMessage = null), 3000);
    }
  }

  function insertVariable(varName: string): void {
    const target = document.querySelector<HTMLTextAreaElement>('#replay-textarea');
    if (!target) {
      body += `{{${varName}}}`;
      showVariableDropdown = false;
      return;
    }

    const start = target.selectionStart;
    const end = target.selectionEnd;
    const varValue = relayState.activeVariables[varName] || `{{${varName}}}`;
    body = body.substring(0, start) + `"${varValue}"` + body.substring(end);
    showVariableDropdown = false;

    setTimeout(() => {
      target.focus();
      target.selectionStart = target.selectionEnd = start + varValue.length + 2;
    }, 0);
  }

  function handleBodyKeyDown(e: KeyboardEvent): void {
    const target = e.target as HTMLTextAreaElement;
    const start = target.selectionStart;
    const end = target.selectionEnd;

    // Suporte a Tab
    if (e.key === "Tab") {
      e.preventDefault();
      body = body.substring(0, start) + "  " + body.substring(end);
      setTimeout(() => {
        target.selectionStart = target.selectionEnd = start + 2;
      }, 0);
      return;
    }

    // Auto-fechamento de Chaves { -> {}
    if (e.key === "{") {
      if (start > 0 && body[start - 1] === "{") {
        showVariableDropdown = true;
      }
      e.preventDefault();
      body = body.substring(0, start) + "{}" + body.substring(end);
      setTimeout(() => {
        target.selectionStart = target.selectionEnd = start + 1;
      }, 0);
      return;
    }

    // Auto-fechamento de Aspas " -> ""
    if (e.key === '"') {
      if (body[start] === '"') {
        e.preventDefault();
        target.selectionStart = target.selectionEnd = start + 1;
        return;
      }
      e.preventDefault();
      body = body.substring(0, start) + '""' + body.substring(end);
      setTimeout(() => {
        target.selectionStart = target.selectionEnd = start + 1;
      }, 0);
      return;
    }

    // Auto-fechamento de Colchetes [ -> []
    if (e.key === "[") {
      e.preventDefault();
      body = body.substring(0, start) + "[]" + body.substring(end);
      setTimeout(() => {
        target.selectionStart = target.selectionEnd = start + 1;
      }, 0);
      return;
    }

    // Enter inteligente entre chaves { | }
    if (e.key === "Enter") {
      if (start > 0 && body[start - 1] === "{" && body[start] === "}") {
        e.preventDefault();
        body = body.substring(0, start) + "\n  \n" + body.substring(end);
        setTimeout(() => {
          target.selectionStart = target.selectionEnd = start + 3;
        }, 0);
        return;
      }
    }
  }

  function injectLatestJwt(): void {
    if (relayState.jwts.length === 0) {
      statusMessage = "Nenhum token JWT disponível na sessão.";
      setTimeout(() => (statusMessage = null), 2500);
      return;
    }

    const latestJwt = relayState.jwts[0];
    const bearerVal = `Bearer ${latestJwt.token}`;

    const authIndex = headers.findIndex(h => h.key.toLowerCase() === "authorization");
    if (authIndex >= 0) {
      headers[authIndex].value = bearerVal;
    } else {
      headers.push({ key: "Authorization", value: bearerVal });
    }

    statusMessage = `JWT injetado (${latestJwt.subject ? 'sub: ' + latestJwt.subject : 'token'})!`;
    setTimeout(() => (statusMessage = null), 2500);
  }

  async function sendReplay(): Promise<void> {
    isSending = true;
    statusMessage = null;

    try {
      const activeHeaders = headers.filter(h => h.key.trim() !== "");
      
      const hasContentType = activeHeaders.some(h => h.key.toLowerCase() === "content-type");
      if (!hasContentType && body.trim().startsWith("{")) {
        activeHeaders.push({ key: "Content-Type", value: "application/json" });
      }

      // Substitui variáveis ativas antes de enviar
      const finalUri = relayState.replaceVariables(uri);
      const finalBody = body.trim() ? relayState.replaceVariables(body) : null;

      const res = await invoke<HttpExchange>("execute_replay", {
        payload: {
          method,
          uri: finalUri,
          headers: activeHeaders,
          body: finalBody,
        }
      });

      if (res) {
        relayState.select(res);
        relayState.sidebarTab = "history";
      }
      closeModal();
    } catch (err) {
      statusMessage = `Erro ao disparar chamada: ${err}`;
    } finally {
      isSending = false;
    }
  }
</script>

<!-- Backdrop com clique para fechar -->
<div
  role="presentation"
  class="fixed inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center z-50 p-4"
  onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
  onkeydown={(e) => { if (e.key === "Escape") closeModal(); }}
>
  <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-2xl w-full p-5 shadow-2xl space-y-4 max-h-[90vh] flex flex-col">
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-zinc-800 pb-3 select-none">
      <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-100 flex items-center space-x-2">
        <span>{template ? `Testar: ${template.name}` : 'HTTP Client & Replay'}</span>
      </h3>
      <button
        onclick={closeModal}
        class="text-zinc-500 hover:text-zinc-300 text-xs p-1 cursor-pointer"
      >
        ✕
      </button>
    </div>

    <!-- Content Form -->
    <div class="flex-1 overflow-y-auto space-y-4 pr-1 text-xs">
      <!-- Target Info & Quick Actions Bar -->
      <div class="flex items-center justify-between bg-zinc-950 p-2.5 rounded-lg border border-zinc-800 font-mono">
        <div class="flex items-center space-x-2 text-[11px]">
          <span class="text-zinc-500">Destino:</span>
          <span class="text-indigo-400 font-semibold">{relayState.config.targetHost}:{relayState.config.targetPort}</span>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Seletor de Variáveis Dinâmicas -->
          {#if availableVars.length > 0}
            <div class="relative">
              <button
                onclick={() => (showVariableDropdown = !showVariableDropdown)}
                class="text-[11px] px-2 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-indigo-300 border border-indigo-500/30 transition-all flex items-center space-x-1 cursor-pointer"
                title="Inserir variáveis capturadas automaticamente"
              >
                <IconSparkles size={11} class="text-indigo-400" />
                <span>Inserir Variável ({availableVars.length})</span>
              </button>

              {#if showVariableDropdown}
                <div class="absolute right-0 top-8 w-56 bg-zinc-900 border border-zinc-800 rounded-lg shadow-xl p-1 z-50 space-y-1 font-mono text-[11px] max-h-48 overflow-y-auto">
                  {#each availableVars as vName}
                    <button
                      onclick={() => insertVariable(vName)}
                      class="w-full text-left px-2 py-1.5 rounded hover:bg-zinc-800 flex items-center justify-between text-zinc-300 transition-colors cursor-pointer"
                    >
                      <span class="text-indigo-400 font-bold">{`{{${vName}}}`}</span>
                      <span class="text-zinc-500 truncate max-w-[80px] text-[10px]">{relayState.activeVariables[vName]}</span>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}

          <button
            onclick={injectLatestJwt}
            class="text-[11px] px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition-all flex items-center space-x-1.5 cursor-pointer"
            title="Injeta o token JWT mais recente no header Authorization"
          >
            <IconKey size={12} class="text-amber-400" />
            <span>Auto-Injetar JWT</span>
          </button>
        </div>
      </div>

      {#if statusMessage}
        <div class="p-2 rounded bg-indigo-500/10 border border-indigo-500/30 text-indigo-300 font-mono text-[11px]">
          {statusMessage}
        </div>
      {/if}

      <!-- Method & URI Row -->
      <div class="flex items-center space-x-2">
        <select
          bind:value={method}
          class="bg-zinc-950 border border-zinc-800 rounded px-2.5 py-2 text-xs font-mono font-bold text-zinc-200 focus:outline-none focus:border-indigo-500 cursor-pointer"
        >
          <option value="GET">GET</option>
          <option value="POST">POST</option>
          <option value="PUT">PUT</option>
          <option value="DELETE">DELETE</option>
          <option value="PATCH">PATCH</option>
          <option value="HEAD">HEAD</option>
          <option value="OPTIONS">OPTIONS</option>
        </select>

        <input
          type="text"
          bind:value={uri}
          placeholder="/customers ou /auth/login"
          class="flex-1 bg-zinc-950 border border-zinc-800 rounded px-3 py-2 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
        />
      </div>

      <!-- Headers Editor -->
      <div class="space-y-2">
        <div class="flex items-center justify-between select-none">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">Headers HTTP ({headers.length})</span>
          <button
            onclick={addHeader}
            class="text-[11px] px-2 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer border border-zinc-700"
          >
            + Adicionar Header
          </button>
        </div>

        <div class="space-y-1.5 max-h-40 overflow-y-auto">
          {#each headers as h, idx}
            <div class="flex items-center space-x-2">
              <input
                type="text"
                placeholder="Header (ex: Content-Type)"
                bind:value={h.key}
                class="w-1/3 bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
              />
              <input
                type="text"
                placeholder="Valor (ex: application/json)"
                bind:value={h.value}
                class="flex-1 bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
              />
              <button
                onclick={() => removeHeader(idx)}
                class="text-zinc-500 hover:text-rose-400 p-1 text-xs cursor-pointer"
                title="Remover Header"
              >
                ✕
              </button>
            </div>
          {/each}
        </div>
      </div>

      <!-- Body Editor -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between select-none">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">Payload Body (JSON)</span>
          <button
            onclick={formatJsonBody}
            class="text-[11px] px-2 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer border border-zinc-700 flex items-center space-x-1"
            title="Formatar e identar JSON automaticamente"
          >
            <IconWand2 size={12} class="text-indigo-400" />
            <span>Formatar JSON</span>
          </button>
        </div>

        <textarea
          id="replay-textarea"
          bind:value={body}
          onkeydown={handleBodyKeyDown}
          rows="7"
          placeholder={`{\n  "email": "test@teste.com",\n  "password": "senha_segura_123"\n}`}
          class="w-full bg-zinc-950 border border-zinc-800 rounded-lg p-3 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500 leading-relaxed resize-y"
          spellcheck="false"
        ></textarea>
      </div>
    </div>

    <!-- Footer Buttons -->
    <div class="flex items-center justify-end space-x-2 pt-3 border-t border-zinc-800 select-none">
      <button
        onclick={closeModal}
        class="text-xs px-3.5 py-2 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer"
      >
        Cancelar
      </button>
      <button
        onclick={sendReplay}
        disabled={isSending}
        class="text-xs px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-all shadow-xs flex items-center space-x-2 disabled:opacity-50 cursor-pointer"
      >
        <IconPlay size={12} class="fill-current" />
        <span>{isSending ? "Disparando..." : "Executar Replay"}</span>
      </button>
    </div>
  </div>
</div>
