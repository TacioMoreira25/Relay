<script lang="ts">
  import RequestList from "$lib/components/RequestList.svelte";
  import Inspector from "$lib/components/Inspector.svelte";
  import TestTrigger from "$lib/components/TestTrigger.svelte";
  import ProxyConfigModal from "$lib/components/ProxyConfigModal.svelte";
  import JwtManager from "$lib/components/JwtManager.svelte";
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { HttpExchange, InterceptedResponse, ExtractedJwt } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let isConfigOpen = $state(false);

  async function syncInitialState(): Promise<void> {
    try {
      const items = await invoke<HttpExchange[]>("get_exchanges");
      if (items && items.length > 0) {
        relayState.exchanges = items.slice().reverse();
      }

      const tokens = await invoke<ExtractedJwt[]>("get_session_jwts");
      if (tokens && tokens.length > 0) {
        relayState.jwts = tokens;
      }
    } catch (e) {
      console.warn("Falha ao sincronizar dados iniciais:", e);
    }
  }

  async function toggleProxy(): Promise<void> {
    try {
      if (relayState.isProxyRunning) {
        await invoke("stop_proxy");
        relayState.isProxyRunning = false;
      } else {
        await invoke("start_proxy", { config: relayState.config });
        relayState.isProxyRunning = true;
      }
    } catch (err) {
      console.error("Erro ao alterar estado do proxy:", err);
    }
  }

  function handleKeyDown(event: KeyboardEvent): void {
    // Atalho: Ctrl+K / Cmd+K -> Focar campo de busca
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      const searchInput = document.querySelector<HTMLInputElement>('input[type="text"]');
      searchInput?.focus();
    }
    // Atalho: Ctrl+L / Cmd+L -> Limpar lista de tráfego
    else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "l") {
      event.preventDefault();
      invoke("clear_exchanges");
      relayState.clear();
    }
    // Atalho: Ctrl+P / Cmd+P -> Iniciar / Parar Proxy
    else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "p") {
      event.preventDefault();
      toggleProxy();
    }
  }

  onMount(() => {
    let unlistenReq: UnlistenFn | undefined;
    let unlistenRes: UnlistenFn | undefined;
    let unlistenErr: UnlistenFn | undefined;
    let unlistenJwt: UnlistenFn | undefined;

    // Sincroniza estado inicial do backend
    syncInitialState();

    window.addEventListener("keydown", handleKeyDown);

    // Registra os listeners IPC do Tauri v2
    listen<HttpExchange>("relay:request", (event) => {
      relayState.addExchange(event.payload);
    }).then((fn) => {
      unlistenReq = fn;
    });

    listen<InterceptedResponse>("relay:response", (event) => {
      relayState.updateResponse(event.payload.requestId, event.payload);
    }).then((fn) => {
      unlistenRes = fn;
    });

    listen<{ requestId: string; error: string }>("relay:error", (event) => {
      relayState.setError(event.payload.requestId, event.payload.error);
    }).then((fn) => {
      unlistenErr = fn;
    });

    listen<ExtractedJwt>("relay:jwt", (event) => {
      relayState.addJwt(event.payload);
    }).then((fn) => {
      unlistenJwt = fn;
    });

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      unlistenReq?.();
      unlistenRes?.();
      unlistenErr?.();
      unlistenJwt?.();
    };
  });
</script>

<main class="h-screen w-screen flex flex-col bg-zinc-950 text-zinc-100 font-sans select-none">
  <!-- Top Bar -->
  <header class="h-12 border-b border-zinc-800 bg-zinc-900/80 px-4 flex items-center justify-between">
    <div class="flex items-center space-x-6">
      <div class="flex items-center space-x-3">
        <div class="flex items-center space-x-1.5 font-bold tracking-tight text-white">
          <div class="w-3 h-3 rounded-full bg-indigo-500 shadow-[0_0_8px_rgba(99,102,241,0.8)]"></div>
          <span class="text-sm">RELAY</span>
        </div>
        <span class="text-xs text-zinc-500 hidden sm:inline">| Native HTTP Interceptor & Replay</span>
      </div>

      <!-- Navigation Tabs (Traffic vs JWT Session) -->
      <div class="flex items-center space-x-1 bg-zinc-950 p-1 rounded-lg border border-zinc-800 text-xs">
        <button
          onclick={() => (relayState.activeView = "traffic")}
          class="px-3 py-1 rounded transition-colors flex items-center space-x-1.5 {relayState.activeView === 'traffic' ? 'bg-zinc-800 text-white font-medium shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <span>🌐</span>
          <span>Tráfego HTTP</span>
          {#if relayState.totalRequests > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-700 text-zinc-300 font-mono">
              {relayState.totalRequests}
            </span>
          {/if}
        </button>

        <button
          onclick={() => (relayState.activeView = "jwt")}
          class="px-3 py-1 rounded transition-colors flex items-center space-x-1.5 {relayState.activeView === 'jwt' ? 'bg-zinc-800 text-white font-medium shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <span>🛡️</span>
          <span>Sessão & JWT</span>
          {#if relayState.totalJwts > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-indigo-600/60 text-indigo-200 font-mono">
              {relayState.totalJwts}
            </span>
          {/if}
        </button>
      </div>
    </div>

    <!-- Proxy Controls, Port Badge & Test Trigger -->
    <div class="flex items-center space-x-3">
      <button
        onclick={() => (isConfigOpen = true)}
        class="flex items-center space-x-2 text-xs font-mono bg-zinc-800/80 hover:bg-zinc-800 px-2.5 py-1 rounded border border-zinc-700/60 transition-colors cursor-pointer"
        title="Clique para configurar portas, latência, jitter e taxa de falhas"
      >
        <span class="text-zinc-400">127.0.0.1:{relayState.config.listenPort}</span>
        <span class="text-zinc-600">➔</span>
        <span class="text-indigo-400">{relayState.config.targetHost}:{relayState.config.targetPort}</span>
        {#if relayState.config.latencyMs > 0 || relayState.config.jitterMs > 0}
          <span class="text-[10px] px-1.5 py-0.2 rounded bg-amber-500/20 text-amber-300 font-sans">
            +{relayState.config.latencyMs}{relayState.config.jitterMs > 0 ? `±${relayState.config.jitterMs}` : ""}ms
          </span>
        {/if}
        {#if relayState.config.simulateFailureRate > 0}
          <span class="text-[10px] px-1.5 py-0.2 rounded bg-rose-500/20 text-rose-300 font-sans font-bold">
            {Math.round(relayState.config.simulateFailureRate * 100)}% {relayState.config.failureStatusCode}
          </span>
        {/if}
      </button>

      <TestTrigger />

      <button
        onclick={toggleProxy}
        class="text-xs px-3 py-1.5 rounded font-medium flex items-center space-x-1.5 transition-all {relayState.isProxyRunning ? 'bg-rose-500/20 text-rose-300 border border-rose-500/40 hover:bg-rose-500/30' : 'bg-indigo-600 text-white hover:bg-indigo-500'}"
        title="Atalho: Ctrl+P"
      >
        <span class="w-2 h-2 rounded-full {relayState.isProxyRunning ? 'bg-rose-400 animate-ping' : 'bg-white'}"></span>
        <span>{relayState.isProxyRunning ? "Parar Proxy" : "Iniciar Proxy"}</span>
      </button>
    </div>
  </header>

  <!-- Main View Content Switcher -->
  <div class="flex-1 flex overflow-hidden">
    {#if relayState.activeView === "traffic"}
      <!-- Left Column: Request List -->
      <div class="w-80 border-r border-zinc-800 h-full">
        <RequestList />
      </div>

      <!-- Right Column: Inspector -->
      <div class="flex-1 h-full">
        <Inspector />
      </div>
    {:else}
      <!-- JWT Manager View -->
      <JwtManager />
    {/if}
  </div>

  <!-- Config & Chaos Modal -->
  <ProxyConfigModal bind:isOpen={isConfigOpen} />
</main>
