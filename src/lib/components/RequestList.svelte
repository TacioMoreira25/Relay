<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import { IconSearch, IconTrash, IconTerminal } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";

  const methods = ["ALL", "GET", "POST", "PUT", "DELETE", "PATCH"];

  function getMethodBadgeStyle(method: string): string {
    switch (method.toUpperCase()) {
      case "GET":
        return "text-sky-400 bg-sky-500/10 border-sky-500/20";
      case "POST":
        return "text-emerald-400 bg-emerald-500/10 border-emerald-500/20";
      case "PUT":
        return "text-amber-400 bg-amber-500/10 border-amber-500/20";
      case "DELETE":
        return "text-rose-400 bg-rose-500/10 border-rose-500/20";
      case "PATCH":
        return "text-purple-400 bg-purple-500/10 border-purple-500/20";
      default:
        return "text-zinc-400 bg-zinc-800 border-zinc-700";
    }
  }

  function getStatusStyle(code?: number, statusStr?: string): string {
    if (statusStr === "failed" || (code && code >= 400)) {
      return "text-rose-400 font-semibold";
    }
    if (code && code >= 200 && code < 300) {
      return "text-emerald-400 font-semibold";
    }
    if (code && code >= 300 && code < 400) {
      return "text-amber-400";
    }
    return "text-zinc-500";
  }

  async function clearTraffic(): Promise<void> {
    try {
      await invoke("clear_exchanges");
      relayState.clear();
    } catch (e) {
      console.error("Erro ao limpar tráfego:", e);
    }
  }
</script>

<div class="flex flex-col h-full bg-zinc-950 text-zinc-200 select-none">
  <!-- Top Search & Actions -->
  <div class="p-2.5 border-b border-zinc-800/80 space-y-2 bg-zinc-900/30">
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-1.5 text-xs text-zinc-400 font-medium">
        <span>Interceptações</span>
        <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-800 text-zinc-300 font-mono">
          {relayState.filteredExchanges.length}
        </span>
      </div>

      {#if relayState.exchanges.length > 0}
        <button
          onclick={clearTraffic}
          class="text-[11px] text-zinc-500 hover:text-rose-400 transition-colors p-1 rounded hover:bg-zinc-800 flex items-center space-x-1 cursor-pointer"
          title="Limpar histórico de tráfego (Ctrl+L)"
        >
          <IconTrash size={12} />
          <span>Limpar</span>
        </button>
      {/if}
    </div>

    <!-- Clean Search Input (pl-8 para evitar sobreposição do ícone) -->
    <div class="relative flex items-center">
      <div class="absolute left-2.5 flex items-center pointer-events-none text-zinc-500">
        <IconSearch size={14} />
      </div>
      <input
        type="text"
        placeholder="Filtrar tráfego... (Ctrl+K)"
        bind:value={relayState.searchQuery}
        class="w-full bg-zinc-900 border border-zinc-800/90 rounded-md pl-8 pr-2.5 py-1.5 text-xs text-zinc-200 placeholder-zinc-500 font-mono focus:outline-none focus:border-indigo-500/80 transition-colors"
      />
    </div>

    <!-- Compact Method Pills -->
    <div class="flex items-center space-x-1 overflow-x-auto pb-0.5 no-scrollbar text-[10px] font-mono">
      {#each methods as m}
        <button
          onclick={() => (relayState.methodFilter = m)}
          class="px-2 py-0.5 rounded transition-colors {relayState.methodFilter === m ? 'bg-zinc-700 text-white font-bold' : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-900'}"
        >
          {m}
        </button>
      {/each}
    </div>
  </div>

  <!-- Traffic Exchanges List -->
  <div class="flex-1 overflow-y-auto divide-y divide-zinc-900">
    {#if relayState.filteredExchanges.length === 0}
      <!-- Minimalist Educational Empty State -->
      <div class="h-full p-6 flex flex-col items-center justify-center text-center space-y-3 text-zinc-500">
        <div class="p-3 rounded-full bg-zinc-900 border border-zinc-800 text-zinc-400">
          <IconTerminal size={20} />
        </div>
        <div class="space-y-1">
          <div class="text-xs font-medium text-zinc-300">Aguardando Tráfego HTTP</div>
          <p class="text-[11px] text-zinc-500 max-w-[220px] leading-relaxed">
            Envie chamadas para <span class="font-mono text-indigo-400">:{relayState.config.listenPort}</span>
          </p>
        </div>
      </div>
    {:else}
      {#each relayState.filteredExchanges as exchange (exchange.id)}
        <button
          type="button"
          class="w-full text-left p-2.5 hover:bg-zinc-900/50 transition-colors flex items-center justify-between border-l-2 {relayState.selectedExchange?.id === exchange.id ? 'bg-zinc-900/80 border-indigo-500' : 'border-transparent'}"
          onclick={() => relayState.select(exchange)}
        >
          <div class="flex items-center space-x-2.5 overflow-hidden flex-1 pr-2">
            <span class="text-[10px] px-1.5 py-0.5 rounded font-mono font-bold border {getMethodBadgeStyle(exchange.request.method)}">
              {exchange.request.method}
            </span>
            <span class="text-xs font-mono text-zinc-200 truncate" title={exchange.request.uri}>
              {exchange.request.uri}
            </span>
          </div>

          <div class="flex flex-col items-end shrink-0 text-right">
            <span class="text-xs font-mono {getStatusStyle(exchange.response?.statusCode, exchange.status)}">
              {exchange.response ? exchange.response.statusCode : (exchange.status === 'failed' ? 'ERR' : '...')}
            </span>
            <span class="text-[10px] font-mono text-zinc-600">
              {exchange.response ? `${exchange.response.durationMs}ms` : ''}
            </span>
          </div>
        </button>
      {/each}
    {/if}
  </div>
</div>
