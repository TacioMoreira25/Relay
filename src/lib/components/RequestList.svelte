<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { HttpExchange } from "$lib/types";

  const methods = ["ALL", "GET", "POST", "PUT", "DELETE", "PATCH"];
  const statusOptions = ["ALL", "2xx", "3xx", "4xx", "5xx", "ERR"];

  function getStatusColor(status?: number): string {
    if (!status) return "text-zinc-400";
    if (status >= 200 && status < 300) return "text-emerald-400 font-semibold";
    if (status >= 300 && status < 400) return "text-cyan-400 font-semibold";
    if (status >= 400 && status < 500) return "text-amber-400 font-semibold";
    return "text-rose-400 font-semibold";
  }

  function getMethodBadge(method: string): string {
    switch (method) {
      case "GET": return "bg-blue-500/10 text-blue-400 border-blue-500/20";
      case "POST": return "bg-emerald-500/10 text-emerald-400 border-emerald-500/20";
      case "PUT": return "bg-amber-500/10 text-amber-400 border-amber-500/20";
      case "DELETE": return "bg-rose-500/10 text-rose-400 border-rose-500/20";
      case "PATCH": return "bg-purple-500/10 text-purple-400 border-purple-500/20";
      default: return "bg-zinc-500/10 text-zinc-400 border-zinc-500/20";
    }
  }
</script>

<div class="flex flex-col h-full bg-zinc-900 border-r border-zinc-800 select-none">
  <!-- Header with Counter & Clear -->
  <div class="p-3 border-b border-zinc-800 flex items-center justify-between">
    <div class="flex items-center space-x-2">
      <span class="text-xs font-bold uppercase tracking-wider text-zinc-400">Tráfego HTTP</span>
      <span class="text-xs px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-300 font-mono">
        {relayState.filteredExchanges.length}{relayState.filteredExchanges.length !== relayState.totalRequests ? `/${relayState.totalRequests}` : ""}
      </span>
    </div>
    <button 
      onclick={() => relayState.clear()}
      class="text-xs text-zinc-400 hover:text-zinc-200 transition-colors px-2 py-1 rounded bg-zinc-800/60 hover:bg-zinc-800"
    >
      Limpar
    </button>
  </div>

  <!-- Search Input -->
  <div class="p-2 border-b border-zinc-800/80 bg-zinc-900/60">
    <div class="relative">
      <input
        type="text"
        placeholder="Filtrar por URL, método ou body..."
        bind:value={relayState.searchQuery}
        class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1 text-xs text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-indigo-500"
      />
      {#if relayState.searchQuery}
        <button
          onclick={() => (relayState.searchQuery = "")}
          class="absolute right-2 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300 text-xs"
        >
          ✕
        </button>
      {/if}
    </div>
  </div>

  <!-- Filter Pills (Method & Status) -->
  <div class="p-2 border-b border-zinc-800/60 flex flex-wrap gap-1 bg-zinc-900/40 text-[10px] font-mono">
    <div class="flex items-center space-x-1 overflow-x-auto w-full pb-1">
      {#each methods as m}
        <button
          onclick={() => (relayState.methodFilter = m)}
          class="px-1.5 py-0.5 rounded border transition-colors {relayState.methodFilter === m ? 'bg-indigo-600 text-white border-indigo-500 font-bold' : 'bg-zinc-800/60 text-zinc-400 border-zinc-700/40 hover:bg-zinc-800'}"
        >
          {m}
        </button>
      {/each}
    </div>
    <div class="flex items-center space-x-1 overflow-x-auto w-full pt-0.5">
      {#each statusOptions as s}
        <button
          onclick={() => (relayState.statusFilter = s)}
          class="px-1.5 py-0.5 rounded border transition-colors {relayState.statusFilter === s ? 'bg-indigo-600 text-white border-indigo-500 font-bold' : 'bg-zinc-800/60 text-zinc-400 border-zinc-700/40 hover:bg-zinc-800'}"
        >
          {s}
        </button>
      {/each}
    </div>
  </div>

  <!-- Virtualized / Reactive Request List -->
  <div class="flex-1 overflow-y-auto divide-y divide-zinc-800/50">
    {#if relayState.filteredExchanges.length === 0}
      <div class="p-8 text-center text-zinc-500 text-sm">
        {#if relayState.exchanges.length === 0}
          Nenhuma requisição interceptada ainda.<br />
          <span class="text-xs text-zinc-600 mt-1 block">Inicie o proxy e envie chamadas HTTP para o alvo.</span>
        {:else}
          Nenhuma requisição corresponde aos filtros ativos.<br />
          <button
            onclick={() => { relayState.searchQuery = ""; relayState.methodFilter = "ALL"; relayState.statusFilter = "ALL"; }}
            class="text-xs text-indigo-400 hover:underline mt-2 inline-block"
          >
            Redefinir filtros
          </button>
        {/if}
      </div>
    {:else}
      {#each relayState.filteredExchanges as exchange (exchange.id)}
        <button
          type="button"
          class="w-full text-left p-3 hover:bg-zinc-800/40 transition-colors flex flex-col space-y-1.5 {relayState.selectedExchange?.id === exchange.id ? 'bg-zinc-800/80 border-l-2 border-indigo-500' : ''}"
          onclick={() => relayState.select(exchange)}
        >
          <div class="flex items-center justify-between w-full">
            <div class="flex items-center space-x-2">
              <span class="text-[11px] px-1.5 py-0.5 rounded border font-mono font-medium {getMethodBadge(exchange.request.method)}">
                {exchange.request.method}
              </span>
              <span class="text-xs font-mono text-zinc-200 truncate max-w-[180px]" title={exchange.request.uri}>
                {exchange.request.uri}
              </span>
            </div>
            <div>
              {#if exchange.status === 'completed' && exchange.response}
                <span class="text-xs font-mono {getStatusColor(exchange.response.statusCode)}">
                  {exchange.response.statusCode}
                </span>
              {:else if exchange.status === 'pending'}
                <span class="text-[11px] font-mono text-amber-400 animate-pulse">...</span>
              {:else}
                <span class="text-[11px] font-mono text-rose-400">ERR</span>
              {/if}
            </div>
          </div>

          <div class="flex items-center justify-between text-[11px] text-zinc-500 font-mono">
            <span>{new Date(exchange.request.timestamp).toLocaleTimeString()}</span>
            <span>
              {#if exchange.response}
                {exchange.response.durationMs}ms
              {:else}
                -
              {/if}
            </span>
          </div>
        </button>
      {/each}
    {/if}
  </div>
</div>
