<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { HeaderEntry, HttpExchange, HttpMethod, SavedRequestTemplate } from "$lib/types";
  import { IconPlay, IconKey, IconSparkles, IconTerminal } from "$lib/components/icons";
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

      // Aplica substituição de variáveis pré-existentes
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
    }
  }

  function injectLatestJwt(): void {
    if (relayState.jwts.length === 0) {
      statusMessage = "Nenhum token JWT capturado no tráfego até o momento.";
      setTimeout(() => (statusMessage = null), 3000);
      return;
    }

    const latestJwt = relayState.jwts[0];
    const bearerVal = `Bearer ${latestJwt.token}`;

    const existingAuthIndex = headers.findIndex(
      (h) => h.key.toLowerCase() === "authorization"
    );

    if (existingAuthIndex >= 0) {
      headers[existingAuthIndex].value = bearerVal;
    } else {
      headers.push({ key: "Authorization", value: bearerVal });
    }

    statusMessage = `JWT injetado com sucesso (${latestJwt.subject ? 'sub: ' + latestJwt.subject : 'token ativo'})!`;
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

  function getMethodColorClass(m: string): string {
    switch (m) {
      case "GET": return "text-sky-400 bg-sky-950/40 border-sky-500/40";
      case "POST": return "text-emerald-400 bg-emerald-950/40 border-emerald-500/40";
      case "PUT": return "text-amber-400 bg-amber-950/40 border-amber-500/40";
      case "DELETE": return "text-rose-400 bg-rose-950/40 border-rose-500/40";
      case "PATCH": return "text-purple-400 bg-purple-950/40 border-purple-500/40";
      default: return "text-zinc-300 bg-zinc-900 border-zinc-700";
    }
  }
</script>

<!-- Backdrop com clique para fechar -->
<div
  role="presentation"
  class="fixed inset-0 bg-black/75 backdrop-blur-xs flex items-center justify-center z-50 p-4"
  onclick={(e) => { if (e.target === e.currentTarget) closeModal(); }}
  onkeydown={(e) => { if (e.key === "Escape") closeModal(); }}
>
  <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-2xl w-full p-5 shadow-2xl space-y-4 max-h-[90vh] flex flex-col">
    <!-- Header Didático -->
    <div class="flex items-center justify-between border-b border-zinc-800 pb-3 select-none">
      <div class="flex items-center space-x-2">
        <IconTerminal size={14} class="text-indigo-400" />
        <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-100">
          {template ? `Testar Rota: ${template.name}` : 'Disparador & Replay de Requisição'}
        </h3>
      </div>
      <button
        onclick={closeModal}
        class="text-zinc-500 hover:text-zinc-300 text-xs p-1 cursor-pointer transition-colors"
      >
        ✕
      </button>
    </div>

    <!-- Content Form -->
    <div class="flex-1 overflow-y-auto space-y-4 pr-1 text-xs">
      <!-- Target Info & Quick Actions Bar Simplificada -->
      <div class="flex items-center justify-between bg-zinc-950/90 px-3 py-2 rounded-lg border border-zinc-800/80">
        <div class="flex items-center space-x-2 text-[11px]">
          <span class="text-zinc-500 font-medium">Destino:</span>
          <span class="text-zinc-300 font-mono font-medium bg-zinc-900/50 px-2 py-0.5 rounded border border-zinc-700/50">
            http://{relayState.config.targetHost}:{relayState.config.targetPort}
          </span>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Seletor de Variáveis Dinâmicas -->
          {#if availableVars.length > 0}
            <div class="relative">
              <button
                onclick={() => (showVariableDropdown = !showVariableDropdown)}
                class="text-[11px] px-2.5 py-1 rounded bg-zinc-800/80 hover:bg-zinc-700 text-zinc-300 border border-zinc-700/50 transition-all flex items-center space-x-1 cursor-pointer shadow-sm"
                title="Insere variáveis salvas dinamicamente (ex: tokens, IDs de resposta)"
              >
                <IconSparkles size={11} class="text-zinc-400" />
                <span>Variáveis ({availableVars.length})</span>
              </button>

              {#if showVariableDropdown}
                <div class="absolute right-0 top-8 w-60 bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl p-1.5 z-50 space-y-1 font-mono text-[11px] max-h-48 overflow-y-auto">
                  {#each availableVars as vName}
                    <button
                      onclick={() => insertVariable(vName)}
                      class="w-full text-left px-2 py-1.5 rounded hover:bg-zinc-800 flex items-center justify-between text-zinc-300 transition-colors cursor-pointer"
                    >
                      <span class="text-indigo-400 font-bold">{`{{${vName}}}`}</span>
                      <span class="text-zinc-500 truncate max-w-[90px] text-[10px]">{relayState.activeVariables[vName]}</span>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}

          <!-- Botão Auto-Injetar JWT Direto -->
          <button
            onclick={injectLatestJwt}
            class="text-[11px] px-2.5 py-1 rounded bg-zinc-800/80 hover:bg-zinc-700 text-zinc-300 border border-zinc-700/50 transition-all flex items-center space-x-1.5 cursor-pointer shadow-sm"
            title="Injeta automaticamente o último token Bearer no cabeçalho Authorization"
          >
            <IconKey size={11} class="text-zinc-400" />
            <span>Injetar JWT</span>
          </button>
        </div>
      </div>

      {#if statusMessage}
        <div class="p-2.5 rounded bg-indigo-500/10 border border-indigo-500/30 text-indigo-300 font-mono text-[11px]">
          {statusMessage}
        </div>
      {/if}

      <!-- URL Builder Moderno com Cores Vibrantes no Método -->
      <div class="flex items-center space-x-1 bg-zinc-950 border border-zinc-800 rounded-lg p-1 focus-within:border-indigo-500 transition-colors">
        <select
          bind:value={method}
          class="rounded px-2.5 py-1.5 text-xs font-mono font-bold focus:outline-none cursor-pointer border {getMethodColorClass(method)}"
        >
          <option value="GET">GET</option>
          <option value="POST">POST</option>
          <option value="PUT">PUT</option>
          <option value="DELETE">DELETE</option>
          <option value="PATCH">PATCH</option>
          <option value="HEAD">HEAD</option>
          <option value="OPTIONS">OPTIONS</option>
        </select>

        <span class="text-zinc-500 font-mono text-xs pl-2 select-none">/</span>
        <input
          type="text"
          value={uri.startsWith('/') ? uri.slice(1) : uri}
          oninput={(e) => {
            const val = (e.target as HTMLInputElement).value;
            uri = val.startsWith('/') ? val : `/${val}`;
          }}
          placeholder="auth/login ou usuarios"
          class="flex-1 bg-transparent px-1 py-1.5 text-xs font-mono text-zinc-100 focus:outline-none"
        />
      </div>

      <!-- Headers Editor -->
      <div class="space-y-2">
        <div class="flex items-center justify-between select-none">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">Headers HTTP ({headers.length})</span>
          <button
            onclick={addHeader}
            class="text-[11px] px-2 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 border border-zinc-700 transition-colors cursor-pointer"
          >
            + Adicionar Header
          </button>
        </div>

        {#if headers.length === 0}
          <div class="p-2.5 text-center text-zinc-500 bg-zinc-950/40 border border-zinc-800/60 rounded font-mono text-[11px]">
            Nenhum header configurado.
          </div>
        {:else}
          <div class="space-y-1.5 max-h-36 overflow-y-auto pr-1">
            {#each headers as h, idx}
              <div class="flex items-center space-x-2">
                <input
                  type="text"
                  bind:value={h.key}
                  placeholder="Header (ex: Authorization)"
                  class="w-1/3 bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
                />
                <input
                  type="text"
                  bind:value={h.value}
                  placeholder="Valor (ex: Bearer token...)"
                  class="flex-1 bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
                />
                <button
                  onclick={() => removeHeader(idx)}
                  class="text-zinc-500 hover:text-rose-400 p-1 cursor-pointer transition-colors text-xs"
                  title="Remover Header"
                >
                  ✕
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Request Body Editor -->
      {#if method !== "GET" && method !== "HEAD"}
        <div class="space-y-1.5">
          <div class="flex items-center justify-between select-none">
            <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">Corpo da Requisição (JSON / Payload)</span>
            <button
              onclick={formatJsonBody}
              class="text-[11px] px-2 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 border border-zinc-700 transition-colors cursor-pointer"
            >
              Formatar JSON
            </button>
          </div>

          <textarea
            id="replay-textarea"
            bind:value={body}
            onkeydown={handleBodyKeyDown}
            rows="7"
            placeholder="JSON do payload..."
            class="w-full bg-zinc-950 border border-zinc-800 rounded-lg p-2.5 text-zinc-200 font-mono text-xs focus:outline-none focus:border-indigo-500 resize-y leading-relaxed"
          ></textarea>
        </div>
      {/if}
    </div>

    <!-- Modal Footer -->
    <div class="flex items-center justify-between pt-3 border-t border-zinc-800 select-none">
      <span class="text-[10px] text-zinc-500">
        Dica: Use <span class="font-mono text-indigo-400 font-bold">{`{{token}}`}</span> ou <span class="font-mono text-indigo-400 font-bold">{`{{id}}`}</span> para dados dinâmicos.
      </span>

      <div class="flex items-center space-x-2">
        <button
          onclick={closeModal}
          class="text-xs px-3 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer"
        >
          Cancelar
        </button>

        <button
          onclick={sendReplay}
          disabled={isSending}
          class="text-xs px-4 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-all shadow-md flex items-center space-x-1.5 cursor-pointer disabled:opacity-50"
        >
          <IconPlay size={11} class="fill-current" />
          <span>{isSending ? 'Enviando...' : 'Executar Replay'}</span>
        </button>
      </div>
    </div>
  </div>
</div>
