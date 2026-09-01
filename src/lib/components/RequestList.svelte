<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import {
    IconSearch,
    IconTrash,
    IconFileJson,
    IconHistory,
    IconBookmark,
    IconFolder,
  } from "$lib/components/icons";
  import type { SavedRequestTemplate } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let {
    onOpenTemplate = (_tpl: SavedRequestTemplate) => {},
    onOpenNewRequest = () => {}
  }: {
    onOpenTemplate?: (tpl: SavedRequestTemplate) => void;
    onOpenNewRequest?: () => void;
  } = $props();

  let fileInputRef = $state<HTMLInputElement | null>(null);
  const methods = ["ALL", "GET", "POST", "PUT", "DELETE", "PATCH"];

  // Estado de expansão das pastas (por padrão abertas)
  let collapsedFolders = $state<Record<string, boolean>>({});

  function toggleFolder(folder: string): void {
    collapsedFolders[folder] = !collapsedFolders[folder];
  }

  // Extrai todas as tags únicas de forma 100% dinâmica
  function getTemplateFolder(tpl: SavedRequestTemplate): string {
    if (tpl.tag && tpl.tag.trim()) {
      return tpl.tag.trim();
    }
    const parts = tpl.uri.split("/").filter(p => p && !p.startsWith("{") && !p.startsWith(":") && p !== "api" && p !== "v1" && p !== "v2");
    if (parts.length > 0) {
      return parts[0];
    }
    return "Geral";
  }

  // Agrupa os templates filtrados em pastas dinâmicas
  let groupedTemplates = $derived.by(() => {
    const groups: Record<string, SavedRequestTemplate[]> = {};
    for (const tpl of relayState.filteredTemplates) {
      const folder = getTemplateFolder(tpl);
      if (!groups[folder]) {
        groups[folder] = [];
      }
      groups[folder].push(tpl);
    }
    return groups;
  });

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
        collapsedFolders = {};
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
  <!-- Header Compacto e Minimalista -->
  <div class="p-2.5 border-b border-zinc-800/80 bg-zinc-900/30 space-y-2">
    <!-- Linha 1: Abas Principais + Botão de Ação -->
    <div class="flex items-center justify-between space-x-2">
      <div class="flex items-center space-x-1 bg-zinc-950 p-0.5 rounded-lg border border-zinc-800/80 text-xs flex-1">
        <button
          onclick={() => (relayState.sidebarTab = "collection")}
          class="flex-1 py-1 rounded-md transition-all flex items-center justify-center space-x-1.5 {relayState.sidebarTab === 'collection' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <IconBookmark size={12} class={relayState.totalTemplates > 0 ? "text-amber-400" : ""} />
          <span>Coleção</span>
          {#if relayState.totalTemplates > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-amber-500/20 text-amber-300 font-mono font-medium">
              {relayState.totalTemplates}
            </span>
          {/if}
        </button>

        <button
          onclick={() => (relayState.sidebarTab = "history")}
          class="flex-1 py-1 rounded-md transition-all flex items-center justify-center space-x-1.5 {relayState.sidebarTab === 'history' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <IconHistory size={12} />
          <span>Histórico</span>
          {#if relayState.totalRequests > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-700/80 text-zinc-300 font-mono">
              {relayState.totalRequests}
            </span>
          {/if}
        </button>
      </div>

      <!-- Botão Único de Ação Contextual -->
      {#if relayState.sidebarTab === 'collection'}
        <input
          type="file"
          accept=".json"
          class="hidden"
          bind:this={fileInputRef}
          onchange={handleImportCollectionFile}
        />
        <button
          onclick={() => fileInputRef?.click()}
          class="p-1.5 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-amber-400 hover:text-amber-300 transition-colors cursor-pointer shrink-0"
          title="Importar JSON (OpenAPI / Swagger / Postman)"
        >
          <IconFileJson size={14} />
        </button>
      {:else if relayState.totalRequests > 0}
        <button
          onclick={clearTraffic}
          class="p-1.5 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-400 hover:text-rose-400 transition-colors cursor-pointer shrink-0"
          title="Limpar Histórico (Ctrl+L)"
        >
          <IconTrash size={14} />
        </button>
      {/if}
    </div>

    <!-- Linha 2: Busca Rápida -->
    <div class="relative">
      <IconSearch size={13} class="absolute left-2.5 top-2.5 text-zinc-500 pointer-events-none" />
      <input
        type="text"
        placeholder={relayState.sidebarTab === 'history' ? "Filtrar tráfego... (Ctrl+K)" : "Filtrar rotas ou pastas..."}
        bind:value={relayState.searchQuery}
        class="w-full bg-zinc-950 border border-zinc-800 rounded-md pl-8 pr-2.5 py-1.5 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-zinc-600 font-mono transition-colors"
      />
    </div>

    <!-- Linha 3: Filtro por Método HTTP -->
    <div class="flex items-center space-x-1 font-mono text-[10px] select-none">
      {#each methods as m}
        <button
          onclick={() => (relayState.methodFilter = m)}
          class="px-2 py-0.5 rounded border transition-all cursor-pointer {getMethodPillActiveStyle(m)}"
        >
          {m}
        </button>
      {/each}
    </div>
  </div>

  <!-- Content List Area -->
  <div class="flex-1 overflow-y-auto divide-y divide-zinc-800/40">
    {#if relayState.sidebarTab === "history"}
      <!-- 1. HISTÓRICO DE TRÁFEGO -->
      {#each relayState.filteredExchanges as exchange (exchange.id)}
        <div
          role="button"
          tabindex="0"
          onclick={() => relayState.select(exchange)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') relayState.select(exchange); }}
          class="p-2.5 text-left w-full hover:bg-zinc-900/60 transition-colors cursor-pointer flex flex-col space-y-1 {relayState.selectedExchange?.id === exchange.id ? 'bg-zinc-900/90 border-l-2 border-indigo-500' : ''}"
        >
          <div class="flex items-center justify-between font-mono text-xs">
            <span class="px-1.5 py-0.2 rounded text-[10px] font-bold border {getMethodBadgeStyle(exchange.request.method)}">
              {exchange.request.method}
            </span>

            <div class="flex items-center space-x-2 text-[11px]">
              {#if exchange.response}
                <span class={getStatusStyle(exchange.response.statusCode)}>
                  {exchange.response.statusCode}
                </span>
                <span class="text-zinc-500 text-[10px]">{exchange.response.durationMs}ms</span>
              {:else if exchange.status === "failed"}
                <span class="text-rose-400 font-bold text-[10px]">ERR</span>
              {:else}
                <span class="text-indigo-400 animate-pulse text-[10px]">...</span>
              {/if}
            </div>
          </div>

          <div class="text-xs font-mono text-zinc-300 truncate" title={exchange.request.uri}>
            {exchange.request.uri}
          </div>
        </div>
      {/each}

      {#if relayState.filteredExchanges.length === 0}
        <div class="p-8 text-center text-zinc-500 text-xs flex flex-col items-center justify-center space-y-3 h-full select-none">
          <div class="w-10 h-10 rounded-full bg-zinc-900 border border-zinc-800 flex items-center justify-center text-zinc-600">
            <IconSearch size={16} />
          </div>
          <div>
            <div class="font-medium text-zinc-400">Nenhuma requisição interceptada</div>
            <div class="text-[11px] text-zinc-600 mt-0.5">Envie chamadas para a porta :{relayState.config.listenPort} ou faça um teste manual.</div>
          </div>
          <button
            onclick={onOpenNewRequest}
            class="text-xs px-3 py-1.5 rounded-lg bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 border border-indigo-500/30 transition-colors font-medium cursor-pointer"
          >
            + Enviar Requisição de Teste
          </button>
        </div>
      {/if}
    {:else}
      <!-- 2. COLEÇÃO DE ROTAS EM PASTAS NATURAIS -->
      {#if Object.keys(groupedTemplates).length > 1}
        {#each Object.entries(groupedTemplates) as [folderName, items]}
          <div class="border-b border-zinc-800/30 last:border-b-0">
            <!-- Cabeçalho da Pasta Limpo -->
            <button
              onclick={() => toggleFolder(folderName)}
              class="w-full px-3 py-2 bg-zinc-900/30 hover:bg-zinc-800/50 flex items-center justify-between text-left transition-colors cursor-pointer"
            >
              <div class="flex items-center space-x-2 truncate">
                <span class="text-zinc-500 text-[9px] transform transition-transform {collapsedFolders[folderName] ? '' : 'rotate-90'}">
                  ▶
                </span>
                <IconFolder size={13} class="text-amber-400/90 shrink-0" />
                <span class="text-xs font-semibold text-zinc-200 truncate">{folderName}</span>
              </div>
              <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-800/80 text-zinc-400 font-mono">
                {items.length}
              </span>
            </button>

            <!-- Itens dentro da Pasta -->
            {#if !collapsedFolders[folderName]}
              <div class="divide-y divide-zinc-800/20 bg-zinc-950/40">
                {#each items as tpl (tpl.id)}
                  <div
                    role="button"
                    tabindex="0"
                    onclick={() => onOpenTemplate(tpl)}
                    onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onOpenTemplate(tpl); }}
                    class="px-3 py-2 text-left w-full hover:bg-zinc-900/70 transition-colors cursor-pointer flex flex-col space-y-0.5 group border-l-2 border-transparent hover:border-amber-500/60 pl-6"
                  >
                    <div class="flex items-center space-x-2 font-mono text-xs">
                      <span class="px-1.5 py-0.2 rounded text-[9px] font-bold border {getMethodBadgeStyle(tpl.method)}">
                        {tpl.method}
                      </span>
                      <span class="text-xs font-medium text-zinc-200 truncate group-hover:text-amber-200 transition-colors">
                        {tpl.name}
                      </span>
                    </div>

                    <div class="text-[11px] font-mono text-zinc-500 truncate" title={tpl.uri}>
                      {tpl.uri}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      {:else}
        <!-- Lista simples quando tem poucas rotas ou apenas 1 pasta -->
        {#each relayState.filteredTemplates as tpl (tpl.id)}
          <div
            role="button"
            tabindex="0"
            onclick={() => onOpenTemplate(tpl)}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onOpenTemplate(tpl); }}
            class="p-2.5 text-left w-full hover:bg-zinc-900/60 transition-colors cursor-pointer flex flex-col space-y-1 group border-l-2 border-transparent hover:border-amber-500/60"
          >
            <div class="flex items-center justify-between font-mono text-xs">
              <span class="px-1.5 py-0.2 rounded text-[10px] font-bold border {getMethodBadgeStyle(tpl.method)}">
                {tpl.method}
              </span>

              {#if tpl.tag}
                <span class="text-[9px] px-1.5 py-0.2 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 font-sans truncate max-w-[120px]">
                  {tpl.tag}
                </span>
              {/if}
            </div>

            <div class="text-xs font-semibold text-zinc-200 truncate group-hover:text-white transition-colors">
              {tpl.name}
            </div>

            <div class="text-[11px] font-mono text-zinc-500 truncate" title={tpl.uri}>
              {tpl.uri}
            </div>
          </div>
        {/each}
      {/if}

      {#if relayState.filteredTemplates.length === 0}
        <div class="p-8 text-center text-zinc-500 text-xs flex flex-col items-center justify-center space-y-3 h-full select-none">
          <div class="w-10 h-10 rounded-full bg-zinc-900 border border-zinc-800 flex items-center justify-center text-amber-500/60">
            <IconBookmark size={16} />
          </div>
          <div>
            <div class="font-medium text-zinc-400">Nenhuma rota encontrada</div>
            <div class="text-[11px] text-zinc-600 mt-0.5">Tente limpar o filtro de busca ou importe uma nova especificação.</div>
          </div>
          <button
            onclick={() => fileInputRef?.click()}
            class="text-xs px-3 py-1.5 rounded-lg bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 border border-amber-500/30 transition-colors font-medium cursor-pointer flex items-center space-x-1.5"
          >
            <IconFileJson size={13} />
            <span>Importar JSON</span>
          </button>
        </div>
      {/if}
    {/if}
  </div>
</div>
