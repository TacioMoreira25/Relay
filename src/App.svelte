<script lang="ts">
  import RequestList from "$lib/components/RequestList.svelte";
  import Inspector from "$lib/components/Inspector.svelte";
  import TestTrigger from "$lib/components/TestTrigger.svelte";
  import ProxyConfigModal from "$lib/components/ProxyConfigModal.svelte";
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { HttpExchange, InterceptedResponse } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let isConfigOpen = $state(false);

  onMount(() => {
    // Escuta eventos assíncronos de requisições emitidos pelo motor Rust
    const unlistenReq = listen<HttpExchange>("relay:request", (event) => {
      relayState.addExchange(event.payload);
    });

    const unlistenRes = listen<InterceptedResponse>("relay:response", (event) => {
      relayState.updateResponse(event.payload.requestId, event.payload);
    });

    const unlistenErr = listen<{ requestId: string; error: string }>("relay:error", (event) => {
      relayState.setError(event.payload.requestId, event.payload.error);
    });

    return () => {
      unlistenReq.then((f) => f());
      unlistenRes.then((f) => f());
      unlistenErr.then((f) => f());
    };
  });

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
</script>

<main class="h-screen w-screen flex flex-col bg-zinc-950 text-zinc-100 font-sans select-none">
  <!-- Top Bar -->
  <header class="h-12 border-b border-zinc-800 bg-zinc-900/80 px-4 flex items-center justify-between">
    <div class="flex items-center space-x-3">
      <div class="flex items-center space-x-1.5 font-bold tracking-tight text-white">
        <div class="w-3 h-3 rounded-full bg-indigo-500 shadow-[0_0_8px_rgba(99,102,241,0.8)]"></div>
        <span class="text-sm">RELAY</span>
      </div>
      <span class="text-xs text-zinc-500">| Native HTTP Interceptor & Replay</span>
    </div>

    <!-- Proxy Controls, Port Badge & Test Trigger -->
    <div class="flex items-center space-x-3">
      <button
        onclick={() => (isConfigOpen = true)}
        class="flex items-center space-x-2 text-xs font-mono bg-zinc-800/80 hover:bg-zinc-800 px-2.5 py-1 rounded border border-zinc-700/60 transition-colors"
        title="Clique para configurar portas e host"
      >
        <span class="text-zinc-400">127.0.0.1:{relayState.config.listenPort}</span>
        <span class="text-zinc-600">➔</span>
        <span class="text-indigo-400">{relayState.config.targetHost}:{relayState.config.targetPort}</span>
        {#if relayState.config.latencyMs > 0}
          <span class="text-[10px] px-1 py-0.2 rounded bg-amber-500/20 text-amber-300 font-sans">
            +{relayState.config.latencyMs}ms
          </span>
        {/if}
      </button>

      <TestTrigger />

      <button
        onclick={toggleProxy}
        class="text-xs px-3 py-1.5 rounded font-medium flex items-center space-x-1.5 transition-all {relayState.isProxyRunning ? 'bg-rose-500/20 text-rose-300 border border-rose-500/40 hover:bg-rose-500/30' : 'bg-indigo-600 text-white hover:bg-indigo-500'}"
      >
        <span class="w-2 h-2 rounded-full {relayState.isProxyRunning ? 'bg-rose-400 animate-ping' : 'bg-white'}"></span>
        <span>{relayState.isProxyRunning ? "Parar Proxy" : "Iniciar Proxy"}</span>
      </button>
    </div>
  </header>

  <!-- Main View Split -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Left Column: Request List -->
    <div class="w-80 border-r border-zinc-800 h-full">
      <RequestList />
    </div>

    <!-- Right Column: Inspector -->
    <div class="flex-1 h-full">
      <Inspector />
    </div>
  </div>

  <!-- Config Modal -->
  <ProxyConfigModal bind:isOpen={isConfigOpen} />
</main>
