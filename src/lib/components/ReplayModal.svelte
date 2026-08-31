<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { HeaderEntry, HttpExchange, HttpMethod } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let { isOpen = $bindable(false), exchange = null }: { isOpen: boolean; exchange: HttpExchange | null } = $props();

  let method = $state<HttpMethod>("GET");
  let uri = $state<string>("/");
  let headers = $state<HeaderEntry[]>([]);
  let body = $state<string>("");
  let isSending = $state<boolean>(false);
  let statusMessage = $state<string | null>(null);

  $effect(() => {
    if (exchange) {
      method = exchange.request.method;
      uri = exchange.request.uri;
      headers = exchange.request.headers.map(h => ({ ...h }));
      body = exchange.request.body || "";
    }
  });

  function addHeader(): void {
    headers.push({ key: "", value: "" });
  }

  function removeHeader(index: number): void {
    headers.splice(index, 1);
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
      const res = await invoke<HttpExchange>("execute_replay", {
        payload: {
          method,
          uri,
          headers: activeHeaders,
          body: body.trim() ? body : null,
        }
      });

      if (res) {
        relayState.select(res);
      }
      isOpen = false;
    } catch (err) {
      statusMessage = `Erro ao disparar replay: ${err}`;
    } finally {
      isSending = false;
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-2xl w-full p-5 shadow-2xl space-y-4 max-h-[90vh] flex flex-col">
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3 select-none">
        <h3 class="text-sm font-semibold text-zinc-100 flex items-center space-x-2">
          <span>🔁</span>
          <span>HTTP Client & Replay de Chamada</span>
        </h3>
        <button
          onclick={() => (isOpen = false)}
          class="text-zinc-500 hover:text-zinc-300 text-xs p-1"
        >
          ✕
        </button>
      </div>

      <!-- Content Form -->
      <div class="flex-1 overflow-y-auto space-y-4 pr-1 text-xs">
        <!-- Target Info & JWT Quick Action -->
        <div class="flex items-center justify-between bg-zinc-950 p-2.5 rounded-lg border border-zinc-800 font-mono">
          <div class="flex items-center space-x-2 text-[11px]">
            <span class="text-zinc-500">Destino:</span>
            <span class="text-indigo-400 font-semibold">{relayState.config.targetHost}:{relayState.config.targetPort}</span>
          </div>

          <button
            onclick={injectLatestJwt}
            class="text-[11px] px-2.5 py-1 rounded bg-amber-500/10 text-amber-300 border border-amber-500/30 hover:bg-amber-500/20 transition-all flex items-center space-x-1"
            title="Injeta o token JWT mais recente no header Authorization"
          >
            <span>🛡️</span>
            <span>Auto-Injetar JWT da Sessão</span>
          </button>
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
            class="bg-zinc-950 border border-zinc-800 rounded px-2.5 py-2 text-xs font-mono font-bold text-zinc-200 focus:outline-none focus:border-indigo-500"
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
            placeholder="/api/v1/resource"
            class="flex-1 bg-zinc-950 border border-zinc-800 rounded px-3 py-2 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
          />
        </div>

        <!-- Headers Editor -->
        <div class="space-y-2">
          <div class="flex items-center justify-between select-none">
            <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">Headers HTTP ({headers.length})</span>
            <button
              onclick={addHeader}
              class="text-[11px] px-2 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors"
            >
              + Adicionar Header
            </button>
          </div>

          <div class="space-y-1.5 max-h-40 overflow-y-auto">
            {#each headers as h, idx}
              <div class="flex items-center space-x-2">
                <input
                  type="text"
                  placeholder="Header (ex: Authorization)"
                  bind:value={h.key}
                  class="w-1/3 bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
                />
                <input
                  type="text"
                  placeholder="Valor"
                  bind:value={h.value}
                  class="flex-1 bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
                />
                <button
                  onclick={() => removeHeader(idx)}
                  class="text-zinc-500 hover:text-rose-400 p-1 text-xs"
                >
                  ✕
                </button>
              </div>
            {/each}
          </div>
        </div>

        <!-- Body Editor -->
        <div class="space-y-1.5">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none">Payload Body (JSON / Texto)</span>
          <textarea
            bind:value={body}
            rows="6"
            placeholder={`{\n  "example": "data"\n}`}
            class="w-full bg-zinc-950 border border-zinc-800 rounded p-3 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500 leading-relaxed"
          ></textarea>
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="flex items-center justify-end space-x-2 pt-3 border-t border-zinc-800 select-none">
        <button
          onclick={() => (isOpen = false)}
          class="text-xs px-3.5 py-2 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors"
        >
          Cancelar
        </button>
        <button
          onclick={sendReplay}
          disabled={isSending}
          class="text-xs px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-all shadow-sm flex items-center space-x-2 disabled:opacity-50 cursor-pointer"
        >
          <span>⚡</span>
          <span>{isSending ? "Disparando Chamada..." : "Executar Replay"}</span>
        </button>
      </div>
    </div>
  </div>
{/if}
