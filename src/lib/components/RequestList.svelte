<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import { IconSearch, IconTrash, IconFileJson, IconHistory, IconBookmark, IconPlay, IconPlus } from "$lib/components/icons";
  import type { SavedRequestTemplate } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let {
    onOpenTemplate = (tpl: SavedRequestTemplate) => {},
    onOpenNewRequest = () => {}
  }: {
    onOpenTemplate?: (tpl: SavedRequestTemplate) => void;
    onOpenNewRequest?: () => void;
  } = $props();

  let fileInputRef = $state<HTMLInputElement | null>(null);
  const methods = ["ALL", "GET", "POST", "PUT", "DELETE", "PATCH"];

  function getMethodBadgeStyle(method: string): string {
    switch (method.toUpperCase()) {
      case "GET":
        return "text-sky-400 bg-sky-500/10 border-sky-500/30";
      case "POST":
        return "text-emerald-400 bg-emerald-500/10 border-emerald-500/30";
      case "PUT":
        return "text-amber-400 bg-amber-500/10 border-amber-500/30";
      case "DELETE":
        return "text-rose-400 bg-rose-500/10 border-rose-500/30";
      case "PATCH":
        return "text-purple-400 bg-purple-500/10 border-purple-500/30";
      default:
        return "text-zinc-400 bg-zinc-800 border-zinc-700";
    }
  }

  function getMethodPillActiveStyle(m: string): string {
    if (relayState.methodFilter !== m) {
      return "text-zinc-500 hover:text-zinc-300 hover:bg-zinc-900 border-transparent";
    }
    switch (m) {
      case "GET":
        return "bg-sky-500/20 text-sky-300 font-bold border-sky-500/40 shadow-xs";
      case "POST":
        return "bg-emerald-500/20 text-emerald-300 font-bold border-emerald-500/40 shadow-xs";
      case "PUT":
        return "bg-amber-500/20 text-amber-300 font-bold border-amber-500/40 shadow-xs";
      case "DELETE":
        return "bg-rose-500/20 text-rose-300 font-bold border-rose-500/40 shadow-xs";
      case "PATCH":
        return "bg-purple-500/20 text-purple-300 font-bold border-purple-500/40 shadow-xs";
      default:
        return "bg-zinc-700 text-white font-bold border-zinc-600 shadow-xs";
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

  function handleImportCollectionFile(event: Event): void {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const content = e.target?.result as string;
        const templates = await invoke<SavedRequestTemplate[]>("parse_collection_json", {
          jsonContent: content
        });

        relayState.setTemplates(templates);
        relayState.sidebarTab = "collection";
      } catch (err) {
        console.error("Erro ao importar coleção:", err);
      } finally {
        if (target) target.value = "";
      }
    };
    reader.readAsText(file);
  }
</script>

<div class="flex flex-col h-full bg-zinc-950 text-zinc-200 select-none">
  <!-- Top Segmented Tabs: Histórico vs Coleção Salva -->
  <div class="p-3 border-b border-zinc-800/80 bg-zinc-900/40 space-y-3">
    <div class="flex items-center space-x-1 bg-zinc-950 p-0.5 rounded-lg border border-zinc-800/80 text-xs">
      <button
        onclick={() => (relayState.sidebarTab = "history")}
        class="flex-1 py-1 rounded-md transition-all flex items-center justify-center space-x-1.5 {relayState.sidebarTab === 'history' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <IconHistory size={13} />
        <span>Histórico</span>
        {#if relayState.totalRequests > 0}
          <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-700/80 text-zinc-300 font-mono">
            {relayState.totalRequests}
          </span>
        {/if}
      </button>

      <button
        onclick={() => (relayState.sidebarTab = "collection")}
        class="flex-1 py-1 rounded-md transition-all flex items-center justify-center space-x-1.5 {relayState.sidebarTab === 'collection' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <IconBookmark size={13} class={relayState.totalTemplates > 0 ? "text-amber-400" : ""} />
        <span>Coleção</span>
        {#if relayState.totalTemplates > 0}
          <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-amber-500/20 text-amber-300 font-mono font-medium">
            {relayState.totalTemplates}
          </span>
        {/if}
      </button>
    </div>

    <!-- Barra Contextual de Ações (Apenas na aba Coleção ou Limpar no Histórico) -->
    <div class="flex items-center justify-between px-0.5 min-h-5">
      <span class="text-[11px] text-zinc-400 font-medium">
        {relayState.sidebarTab === 'history' ? 'Tráfego Interceptado' : 'Rotas para Teste Rápido'}
      </span>

      <div class="flex items-center space-x-1.5">
        {#if relayState.sidebarTab === 'collection'}
          <label
            class="text-[10px] text-amber-300 hover:text-white transition-colors px-2 py-0.5 rounded bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/30 flex items-center space-x-1 cursor-pointer shadow-xs"
            title="Importar coleção de endpoints (.json)"
          >
            <IconFileJson size={11} class="text-amber-400" />
            <span>Importar JSON</span>
            <input
              bind:this={fileInputRef}
              type="file"
              accept=".json"
              onchange={handleImportCollectionFile}
              class="hidden"
            />
          </label>
        {:else if relayState.exchanges.length > 0}
          <button
            onclick={clearTraffic}
            class="text-[11px] text-zinc-500 hover:text-rose-400 transition-colors p-1 rounded hover:bg-zinc-800 flex items-center space-x-1 cursor-pointer"
            title="Limpar histórico de requisições (Ctrl+L)"
          >
            <IconTrash size={12} />
          </button>
        {/if}
      </div>
    </div>

    <!-- Search Input -->
    <div class="relative flex items-center">
      <div class="absolute left-2.5 flex items-center pointer-events-none text-zinc-500">
        <IconSearch size={13} />
      </div>
      <input
        type="text"
        placeholder={relayState.sidebarTab === 'history' ? "Buscar no tráfego... (Ctrl+K)" : "Buscar rotas..."}
        bind:value={relayState.searchQuery}
        class="w-full bg-zinc-900 border border-zinc-800/90 rounded-md pl-8 pr-2.5 py-1.5 text-xs text-zinc-200 placeholder-zinc-500 font-mono focus:outline-none focus:border-indigo-500 transition-colors"
      />
    </div>

    <!-- Method Pills com espaçamento e cores semânticas -->
    <div class="flex items-center space-x-1.5 overflow-x-auto pt-1 pb-0.5 no-scrollbar text-[10px] font-mono">
      {#each methods as m}
        <button
          onclick={() => (relayState.methodFilter = m)}
          class="px-2 py-0.5 rounded-full border transition-all {getMethodPillActiveStyle(m)}"
        >
          {m}
        </button>
      {/each}
    </div>
  </div>

  <!-- Content List: History vs Collection -->
  <div class="flex-1 overflow-y-auto divide-y divide-zinc-900/60">
    {#if relayState.sidebarTab === "history"}
      <!-- 1. Aba Histórico de Tráfego Real -->
      {#if relayState.filteredExchanges.length === 0}
        <div class="h-full p-6 flex flex-col items-center justify-center text-center space-y-3.5">
          <div class="p-3.5 rounded-full bg-zinc-900 border border-zinc-800 text-indigo-400 shadow-inner">
            <IconSearch size={22} />
          </div>
          <div class="space-y-1.5">
            <div class="text-xs font-semibold text-zinc-200">Nenhuma requisição interceptada</div>
            <p class="text-[11px] text-zinc-400 max-w-[210px] leading-relaxed">
              Envie chamadas para a porta <span class="font-mono text-indigo-400 font-bold">:{relayState.config.listenPort}</span> ou faça um teste manual.
            </p>
          </div>
          <button
            onclick={onOpenNewRequest}
            class="text-[11px] px-3 py-1.5 rounded-md bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-all shadow-xs flex items-center space-x-1.5 cursor-pointer"
          >
            <IconPlus size={12} />
            <span>Enviar Requisição de Teste</span>
          </button>
        </div>
      {:else}
        {#each relayState.filteredExchanges as exchange (exchange.id)}
          <button
            type="button"
            class="w-full text-left p-2.5 hover:bg-zinc-900/50 transition-colors flex items-center justify-between border-l-2 {relayState.selectedExchange?.id === exchange.id ? 'bg-zinc-900/80 border-indigo-500' : 'border-transparent'}"
            onclick={() => relayState.select(exchange)}
          >
            <div class="flex items-center space-x-2.5 overflow-hidden flex-1 pr-2">
              <span class="text-[9px] px-1.5 py-0.5 rounded font-mono font-bold border {getMethodBadgeStyle(exchange.request.method)}">
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
              <span class="text-[10px] font-mono text-zinc-500">
                {exchange.response ? `${exchange.response.durationMs}ms` : ''}
              </span>
            </div>
          </button>
        {/each}
      {/if}
    {:else}
      <!-- 2. Aba Coleção de Requisições Salvas (Prontas para Testar) -->
      {#if relayState.filteredTemplates.length === 0}
        <div class="h-full p-6 flex flex-col items-center justify-center text-center space-y-3">
          <div class="p-3.5 rounded-full bg-zinc-900 border border-zinc-800 text-amber-400">
            <IconBookmark size={22} />
          </div>
          <div class="space-y-1">
            <div class="text-xs font-semibold text-zinc-200">Nenhuma rota na coleção</div>
            <p class="text-[11px] text-zinc-400 max-w-[210px] leading-relaxed">
              Importe um arquivo de coleção JSON para rodar rotas com 1 clique e auto-JWT.
            </p>
          </div>
        </div>
      {:else}
        {#each relayState.filteredTemplates as template (template.id)}
          <div
            class="w-full text-left p-3 hover:bg-zinc-900/50 transition-colors flex flex-col space-y-1.5 border-l-2 {relayState.selectedTemplate?.id === template.id ? 'bg-zinc-900/80 border-amber-500' : 'border-transparent'}"
          >
            <div class="flex items-center justify-between">
              <span class="text-xs font-semibold text-zinc-200 truncate flex items-center space-x-1.5">
                <span>{template.name}</span>
                {#if template.requiresAuth}
                  <span class="text-[9px] px-1 py-0.2 rounded bg-amber-500/10 text-amber-300 border border-amber-500/20 font-mono">
                    JWT
                  </span>
                {/if}
              </span>

              <button
                onclick={() => onOpenTemplate(template)}
                class="text-[11px] px-2 py-0.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors flex items-center space-x-1 shadow-xs cursor-pointer"
                title="Abrir no editor e rodar chamada"
              >
                <IconPlay size={10} class="fill-current" />
                <span>Testar</span>
              </button>
            </div>

            <div class="flex items-center space-x-2">
              <span class="text-[9px] px-1.5 py-0.2 rounded font-mono font-bold border {getMethodBadgeStyle(template.method)}">
                {template.method}
              </span>
              <span class="text-xs font-mono text-zinc-400 truncate">
                {template.uri}
              </span>
            </div>

            {#if template.description}
              <div class="text-[10px] text-zinc-500 leading-tight">
                {template.description}
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    {/if}
  </div>
</div>
