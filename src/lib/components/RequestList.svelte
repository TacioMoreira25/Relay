<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import {
    IconSearch,
    IconTrash,
    IconFileJson,
    IconHistory,
    IconBookmark,
    IconFolder,
    IconDownload,
    IconCheck,
    IconSquare,
    IconFilter,
  } from "$lib/components/icons";
  import type { SavedRequestTemplate, HttpExchange } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  import TrafficDiagnosticModal from "$lib/components/TrafficDiagnosticModal.svelte";

  let {
    onOpenTemplate = (_tpl: SavedRequestTemplate) => {},
    onOpenNewRequest = () => {}
  }: {
    onOpenTemplate?: (tpl: SavedRequestTemplate) => void;
    onOpenNewRequest?: () => void;
  } = $props();

  let fileInputRef = $state<HTMLInputElement | null>(null);
  let showCollectionExampleModal = $state(false);
  let copyFeedback = $state<string | null>(null);
  let isDiagnosticModalOpen = $state(false);
  let isFilterPopoverOpen = $state(false);

  let hasLoopingTraffic = $derived(
    relayState.exchanges.slice(0, 20).filter(
      e => e.response?.statusCode === 404 || relayState.isPollingExchange(e)
    ).length >= 3
  );

  // Modo de seleção múltipla do histórico
  let isSelectMode = $state<boolean>(false);
  let selectedIds = $state<string[]>([]);

  function toggleSelect(id: string, e?: Event): void {
    e?.stopPropagation();
    if (selectedIds.includes(id)) {
      selectedIds = selectedIds.filter(i => i !== id);
    } else {
      selectedIds = [...selectedIds, id];
    }
  }

  function toggleSelectAll(): void {
    const currentFilteredIds = relayState.filteredExchanges.map(e => e.id);
    if (selectedIds.length === currentFilteredIds.length && currentFilteredIds.length > 0) {
      selectedIds = [];
    } else {
      selectedIds = [...currentFilteredIds];
    }
  }

  function deleteSelected(): void {
    if (selectedIds.length === 0) return;
    relayState.removeExchanges(selectedIds);
    selectedIds = [];
    isSelectMode = false;
  }

  let savedIdFeedback = $state<string | null>(null);

  function saveSingleToCollection(exchange: HttpExchange, e: Event): void {
    e.stopPropagation();
    relayState.saveExchangeAsTemplate(exchange);
    savedIdFeedback = exchange.id;
    setTimeout(() => {
      if (savedIdFeedback === exchange.id) savedIdFeedback = null;
    }, 2000);
  }

  function deleteSingle(id: string, e: Event): void {
    e.stopPropagation();
    relayState.removeExchange(id);
    selectedIds = selectedIds.filter(i => i !== id);
  }

  const methods = ["ALL", "GET", "POST", "PUT", "DELETE", "PATCH"];

  // Estado de expansão das pastas (por padrão abertas)
  let collapsedFolders = $state<Record<string, boolean>>({});

  function toggleFolder(folder: string): void {
    collapsedFolders[folder] = !collapsedFolders[folder];
  }

  // Modelo JSON Exemplo para Coleção (Formato OpenAPI/Relay Universal)
  const COLLECTION_EXAMPLE_JSON = `[
  {
    "name": "Login do Usuário",
    "description": "Autenticação via email e senha com retorno de JWT",
    "tag": "Autenticação",
    "method": "POST",
    "uri": "/auth/login",
    "headers": [
      { "key": "Content-Type", "value": "application/json" }
    ],
    "body": "{\\n  \\"email\\": \\"admin@exemplo.com\\",\\n  \\"password\\": \\"123456\\"\\n}",
    "requiresAuth": false
  },
  {
    "name": "Listar Usuários",
    "description": "Retorna todos os usuários cadastrados",
    "tag": "Usuários",
    "method": "GET",
    "uri": "/api/usuarios",
    "headers": [
      { "key": "Authorization", "value": "Bearer {{jwt_token}}" }
    ],
    "requiresAuth": true
  },
  {
    "name": "Criar Usuário",
    "description": "Cadastra um novo usuário no sistema",
    "tag": "Usuários",
    "method": "POST",
    "uri": "/api/usuarios",
    "headers": [
      { "key": "Content-Type", "value": "application/json" },
      { "key": "Authorization", "value": "Bearer {{jwt_token}}" }
    ],
    "body": "{\\n  \\"name\\": \\"Novo Usuário\\",\\n  \\"email\\": \\"novo@exemplo.com\\",\\n  \\"role\\": \\"USER\\"\\n}",
    "requiresAuth": true
  },
  {
    "name": "Emitir Pedido",
    "description": "Gera um novo pedido de compra",
    "tag": "Pedidos",
    "method": "POST",
    "uri": "/api/pedidos/emitir",
    "headers": [
      { "key": "Content-Type", "value": "application/json" },
      { "key": "Authorization", "value": "Bearer {{jwt_token}}" }
    ],
    "body": "{\\n  \\"clienteId\\": \\"{{user_id}}\\",\\n  \\"total\\": 150.00\\n}",
    "requiresAuth": true
  }
]`;

  function copyCollectionExample(): void {
    navigator.clipboard.writeText(COLLECTION_EXAMPLE_JSON);
    copyFeedback = "Modelo copiado para a área de transferência!";
    setTimeout(() => (copyFeedback = null), 2500);
  }

  function downloadCollectionExample(): void {
    const blob = new Blob([COLLECTION_EXAMPLE_JSON], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "relay.collection.example.json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    copyFeedback = "Arquivo relay.collection.example.json baixado!";
    setTimeout(() => (copyFeedback = null), 2500);
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
      return "bg-zinc-900 text-zinc-500 border-zinc-800 hover:text-zinc-300";
    }
    switch (m) {
      case "GET":
        return "bg-sky-950 text-sky-300 font-bold border-sky-500/50 shadow-xs";
      case "POST":
        return "bg-emerald-950 text-emerald-300 font-bold border-emerald-500/50 shadow-xs";
      case "PUT":
        return "bg-amber-950 text-amber-300 font-bold border-amber-500/50 shadow-xs";
      case "DELETE":
        return "bg-rose-950 text-rose-300 font-bold border-rose-500/50 shadow-xs";
      case "PATCH":
        return "bg-purple-950 text-purple-300 font-bold border-purple-500/50 shadow-xs";
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
  <div class="p-2.5 border-b border-zinc-800/70 bg-zinc-900/60 backdrop-blur-xs space-y-2">
    <!-- Linha 1: Abas Principais + Botões de Ação -->
    <div class="flex items-center justify-between gap-1.5 min-w-0">
      <div class="flex items-center space-x-0.5 bg-zinc-950 p-0.5 rounded-lg border border-zinc-800/80 text-xs flex-1 min-w-0">
        <button
          onclick={() => (relayState.sidebarTab = "collection")}
          class="flex-1 min-w-0 py-1 px-1.5 rounded-md transition-all flex items-center justify-center space-x-1 whitespace-nowrap {relayState.sidebarTab === 'collection' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
          title="Coleção de rotas e templates"
        >
          <IconBookmark size={12} class="shrink-0 {relayState.totalTemplates > 0 ? 'text-amber-400' : ''}" />
          <span class="truncate">Coleção</span>
          {#if relayState.totalTemplates > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-amber-500/20 text-amber-300 font-mono font-medium shrink-0">
              {relayState.totalTemplates}
            </span>
          {/if}
        </button>

        <button
          onclick={() => (relayState.sidebarTab = "history")}
          class="flex-1 min-w-0 py-1 px-1.5 rounded-md transition-all flex items-center justify-center space-x-1 whitespace-nowrap {relayState.sidebarTab === 'history' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
          title="Histórico de tráfego interceptado"
        >
          <IconHistory size={12} class="shrink-0" />
          <span class="truncate">Histórico</span>
          {#if relayState.totalRequests > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-700/80 text-zinc-300 font-mono shrink-0">
              {relayState.totalRequests}
            </span>
          {/if}
        </button>
      </div>

      <!-- Botões de Ação Contextual -->
      {#if relayState.sidebarTab === 'collection'}
        <input
          type="file"
          accept=".json"
          class="hidden"
          bind:this={fileInputRef}
          onchange={handleImportCollectionFile}
        />
        
        <div class="flex items-center space-x-1 shrink-0">
          <!-- Botão para Ver Modelo JSON de Exemplo -->
          <button
            onclick={() => (showCollectionExampleModal = true)}
            class="p-1.5 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-amber-400/80 hover:text-amber-300 transition-colors cursor-pointer shrink-0"
            title="Ver e Salvar Modelo JSON de Coleção para a IA"
          >
            <IconFileJson size={13} />
          </button>

          <!-- Botão de Importar Arquivo -->
          <button
            onclick={() => fileInputRef?.click()}
            class="p-1.5 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-300 hover:text-white transition-colors cursor-pointer shrink-0"
            title="Importar Arquivo JSON (OpenAPI / Swagger / Postman)"
          >
            <IconDownload size={13} class="rotate-180" />
          </button>
        </div>
      {:else if relayState.totalRequests > 0}
        <div class="flex items-center space-x-1 shrink-0">
          <button
            onclick={() => { isSelectMode = !isSelectMode; selectedIds = []; }}
            class="p-1.5 rounded-lg border transition-colors cursor-pointer shrink-0 {isSelectMode ? 'bg-indigo-600/30 border-indigo-500/60 text-indigo-300' : 'bg-zinc-900 hover:bg-zinc-800 border-zinc-800 text-zinc-400 hover:text-zinc-200'}"
            title={isSelectMode ? "Sair do modo de seleção" : "Selecionar requisições para apagar"}
          >
            <IconCheck size={13} />
          </button>

          <button
            onclick={clearTraffic}
            class="p-1.5 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-400 hover:text-rose-400 transition-colors cursor-pointer shrink-0"
            title="Limpar Todo o Histórico (Ctrl+L)"
          >
            <IconTrash size={13} />
          </button>
        </div>
      {/if}
    </div>

    <!-- Linha 2: Busca Rápida + Popover de Filtro Compacto Integrado -->
    <div class="relative flex items-center gap-1.5">
      <div class="relative flex-1">
        <IconSearch size={13} class="absolute left-2.5 top-2.5 text-zinc-500 pointer-events-none" />
        <input
          type="text"
          placeholder={relayState.sidebarTab === 'history' ? "Filtrar tráfego... (Ctrl+K)" : "Filtrar rotas ou pastas..."}
          bind:value={relayState.searchQuery}
          class="w-full bg-zinc-950 border border-zinc-800 rounded-lg pl-8 pr-8 py-1.5 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-zinc-700 font-mono transition-colors"
        />

        {#if relayState.methodFilter !== 'ALL'}
          <span class="absolute right-2 top-2 text-[9px] px-1 py-0.2 rounded font-mono font-bold border {getMethodBadgeStyle(relayState.methodFilter)}">
            {relayState.methodFilter}
          </span>
        {/if}
      </div>

      <!-- Botão Único de Filtro Popover -->
      <div class="relative shrink-0">
        <button
          onclick={() => (isFilterPopoverOpen = !isFilterPopoverOpen)}
          class="h-8 px-2.5 rounded-lg border transition-all flex items-center space-x-1.5 cursor-pointer text-xs shrink-0 active:scale-[0.98] {relayState.methodFilter !== 'ALL' || relayState.hidePolling || relayState.historySourceFilter !== 'ALL' ? 'bg-indigo-600/20 border-indigo-500/50 text-indigo-300 font-medium' : 'bg-zinc-950 hover:bg-zinc-900 border-zinc-800 text-zinc-400 hover:text-zinc-200'}"
          title="Filtros por Método HTTP, Origem e Polling"
        >
          <IconFilter size={13} class="shrink-0" />
          <span class="hidden sm:inline text-[11px]">Filtros</span>
          <span class="text-[9px] text-zinc-500">▾</span>
        </button>

        {#if isFilterPopoverOpen}
          <!-- Backdrop para fechar ao clicar fora -->
          <div
            class="fixed inset-0 z-40"
            onclick={() => (isFilterPopoverOpen = false)}
            role="presentation"
          ></div>

          <!-- Popover Compacto -->
          <div class="absolute right-0 mt-2 w-64 bg-zinc-900 border border-zinc-800 rounded-xl p-3 shadow-2xl z-50 space-y-3 text-xs select-none">
            <div class="flex items-center justify-between border-b border-zinc-800 pb-1.5 text-[10px] uppercase font-bold text-zinc-400 tracking-wider">
              <span>Método HTTP</span>
              {#if relayState.methodFilter !== 'ALL'}
                <button
                  onclick={() => (relayState.methodFilter = 'ALL')}
                  class="text-[10px] text-indigo-400 hover:text-indigo-300 transition-colors cursor-pointer lowercase"
                >
                  limpar
                </button>
              {/if}
            </div>

            <!-- Grade de Métodos HTTP Compacta -->
            <div class="grid grid-cols-3 gap-1 font-mono text-[10px]">
              {#each methods as m}
                <button
                  onclick={() => { relayState.methodFilter = m; }}
                  class="py-1 px-1.5 rounded border transition-all cursor-pointer text-center truncate {getMethodPillActiveStyle(m)}"
                >
                  {m}
                </button>
              {/each}
            </div>

            {#if relayState.sidebarTab === 'history'}
              <!-- Seção Origem do Tráfego -->
              <div class="pt-2 border-t border-zinc-800/80 space-y-1.5">
                <span class="text-[10px] uppercase font-bold text-zinc-400 tracking-wider block">Origem do Tráfego</span>
                <div class="flex items-center space-x-1 bg-zinc-950 p-1 rounded-lg border border-zinc-800/80 font-mono text-[10px]">
                  <button
                    onclick={() => (relayState.historySourceFilter = "ALL")}
                    class="flex-1 py-1 rounded text-center transition-colors cursor-pointer {relayState.historySourceFilter === 'ALL' ? 'bg-zinc-800 text-zinc-100 font-bold shadow-xs' : 'text-zinc-500 hover:text-zinc-300'}"
                  >
                    Todas
                  </button>
                  <button
                    onclick={() => (relayState.historySourceFilter = "MANUAL")}
                    class="flex-1 py-1 rounded text-center transition-colors cursor-pointer {relayState.historySourceFilter === 'MANUAL' ? 'bg-indigo-600/30 text-indigo-300 font-bold shadow-xs' : 'text-zinc-500 hover:text-zinc-300'}"
                  >
                    Manual
                  </button>
                  <button
                    onclick={() => (relayState.historySourceFilter = "AUTO")}
                    class="flex-1 py-1 rounded text-center transition-colors cursor-pointer {relayState.historySourceFilter === 'AUTO' ? 'bg-zinc-800 text-zinc-100 font-bold shadow-xs' : 'text-zinc-500 hover:text-zinc-300'}"
                  >
                    Auto
                  </button>
                </div>
              </div>

              <!-- Seção Ocultar Polling -->
              <div class="pt-2 border-t border-zinc-800/80">
                <button
                  onclick={() => (relayState.hidePolling = !relayState.hidePolling)}
                  class="w-full py-1.5 px-2 rounded-lg text-[10px] font-mono transition-all cursor-pointer flex items-center justify-between border {relayState.hidePolling ? 'bg-amber-500/15 border-amber-500/40 text-amber-300 font-medium' : 'bg-zinc-950 border-zinc-800 text-zinc-400 hover:text-zinc-200'}"
                >
                  <span>Ocultar Polling Repetido</span>
                  <span>{relayState.hidePolling ? 'SIM' : 'NÃO'}</span>
                </button>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>

    <!-- Barra de Ação de Seleção Múltipla -->
    {#if relayState.sidebarTab === 'history' && isSelectMode}
      <div class="flex items-center justify-between bg-zinc-900/90 border border-indigo-500/40 rounded-lg px-2 py-1 text-xs">
        <div class="flex items-center space-x-1.5 text-[11px]">
          <button
            onclick={toggleSelectAll}
            class="text-indigo-300 hover:text-indigo-200 transition-colors cursor-pointer underline text-[10px]"
          >
            {selectedIds.length === relayState.filteredExchanges.length && relayState.filteredExchanges.length > 0 ? "Desmarcar" : "Marcar todas"}
          </button>
          <span class="text-zinc-600">•</span>
          <span class="text-zinc-400 font-mono text-[10px]">{selectedIds.length}</span>
        </div>

        <div class="flex items-center space-x-1.5">
          <button
            onclick={() => { isSelectMode = false; selectedIds = []; }}
            class="text-[10px] px-2 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer"
          >
            Cancelar
          </button>
          <button
            onclick={deleteSelected}
            disabled={selectedIds.length === 0}
            class="text-[10px] px-2 py-0.5 rounded bg-rose-600 hover:bg-rose-500 text-white font-medium transition-colors cursor-pointer disabled:opacity-40"
          >
            Apagar ({selectedIds.length})
          </button>
        </div>
      </div>
    {/if}

    <!-- Indicador Didático de Tráfego em Loop (Se detectado) -->
    {#if hasLoopingTraffic && relayState.sidebarTab === 'history'}
      <button
        onclick={() => (isDiagnosticModalOpen = true)}
        class="w-full text-left px-2 py-1 rounded bg-amber-500/10 hover:bg-amber-500/15 border border-amber-500/30 text-amber-300 text-[10px] flex items-center justify-between transition-colors cursor-pointer"
        title="Clique para entender por que tantas requisições estão chegando em loop"
      >
        <div class="flex items-center space-x-1.5 truncate">
          <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse shrink-0"></span>
          <span class="truncate">Tráfego repetido / 404 detectado</span>
        </div>
        <span class="underline shrink-0 ml-1 font-mono font-medium">Diagnóstico</span>
      </button>
    {/if}
  </div>

  <!-- Content List Area -->
  <div class="flex-1 overflow-y-auto divide-y divide-zinc-800/40">
    {#if relayState.sidebarTab === "history"}
      <!-- 1. HISTÓRICO DE TRÁFEGO -->
      {#each relayState.groupedExchanges as { exchange, count } (exchange.id)}
        {@const isManual = exchange.id.startsWith("replay-")}
        {@const isPoll = relayState.isPollingExchange(exchange)}
        {@const isSelected = selectedIds.includes(exchange.id)}
        <div
          role="button"
          tabindex="0"
          onclick={(e) => {
            if (isSelectMode) {
              toggleSelect(exchange.id, e);
            } else {
              relayState.select(exchange);
            }
          }}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              if (isSelectMode) toggleSelect(exchange.id);
              else relayState.select(exchange);
            }
          }}
          class="group p-2.5 text-left w-full hover:bg-zinc-900/60 transition-colors cursor-pointer flex flex-col space-y-1.5 relative {relayState.selectedExchange?.id === exchange.id && !isSelectMode ? 'bg-zinc-900/90 border-l-2 border-indigo-500' : ''} {isSelected ? 'bg-indigo-950/25 border-l-2 border-indigo-400' : ''}"
        >
          <div class="flex items-center justify-between font-mono text-xs">
            <div class="flex items-center space-x-1.5">
              {#if isSelectMode}
                <input
                  type="checkbox"
                  checked={isSelected}
                  onchange={(e) => toggleSelect(exchange.id, e)}
                  onclick={(e) => e.stopPropagation()}
                  class="rounded bg-zinc-900 border-zinc-700 text-indigo-600 focus:ring-0 cursor-pointer h-3.5 w-3.5"
                />
              {/if}

              <span class="px-1.5 py-0.2 rounded text-[10px] font-bold border {getMethodBadgeStyle(exchange.request.method)}">
                {exchange.request.method}
              </span>

              {#if isManual}
                <span class="px-1 py-0.2 rounded text-[9px] font-bold font-mono bg-indigo-500/10 text-indigo-300 border border-indigo-500/20" title="Disparo manual via Disparador/Replay">
                  MANUAL
                </span>
              {:else if count > 1}
                <span class="px-1 py-0.2 rounded text-[9px] font-mono font-medium bg-amber-500/15 text-amber-300 border border-amber-500/30" title="{count} requisições idênticas em curto intervalo agrupadas">
                  {count}x
                </span>
              {:else if isPoll}
                <span class="px-1 py-0.2 rounded text-[9px] font-mono bg-zinc-800 text-zinc-400 border border-zinc-700" title="Requisição repetida rápida em sequência (polling)">
                  POLL
                </span>
              {/if}
            </div>

            <div class="flex items-center space-x-2 text-[11px]">
              {#if exchange.response}
                <span class={getStatusStyle(exchange.response.statusCode)}>
                  {exchange.response.statusCode}
                </span>
                <span class="text-zinc-500 text-[10px]">{exchange.response.durationMs}ms</span>
              {:else if exchange.status === "failed"}
                <span class="text-rose-400 font-bold text-[10px]">ERR</span>
              {:else}
                <span class="text-amber-400 text-[10px]">...</span>
              {/if}

              {#if !isSelectMode}
                <button
                  onclick={(e) => saveSingleToCollection(exchange, e)}
                  class="p-0.5 rounded transition-colors cursor-pointer ml-1 {savedIdFeedback === exchange.id ? 'text-emerald-400' : 'text-zinc-500 hover:text-amber-400'}"
                  title={savedIdFeedback === exchange.id ? "Salvo na Coleção!" : "Salvar na Coleção de Rotas"}
                >
                  {#if savedIdFeedback === exchange.id}
                    <IconCheck size={13} class="text-emerald-400" />
                  {:else}
                    <IconBookmark size={13} />
                  {/if}
                </button>
                <button
                  onclick={(e) => deleteSingle(exchange.id, e)}
                  class="opacity-0 group-hover:opacity-100 hover:text-rose-400 text-zinc-500 p-0.5 rounded transition-opacity cursor-pointer ml-0.5"
                  title="Apagar esta requisição"
                >
                  <IconTrash size={12} />
                </button>
              {/if}
            </div>
          </div>

          <div class="flex items-center text-[11px] truncate min-w-0" title={exchange.request.uri}>
            {#if exchange.request.uri.includes("?")}
              {@const parts = exchange.request.uri.split("?")}
              <span class="font-sans font-medium text-zinc-200 truncate">{parts[0]}</span>
              <span class="font-mono text-[10px] text-zinc-500 truncate ml-0.5">?{parts.slice(1).join("?")}</span>
            {:else}
              <span class="font-sans font-medium text-zinc-200 truncate">{exchange.request.uri}</span>
            {/if}
          </div>
        </div>
      {/each}

      {#if relayState.filteredExchanges.length === 0}
        <div class="p-8 text-center text-zinc-500 text-xs flex flex-col items-center justify-center space-y-3 h-full select-none">
          <div class="w-10 h-10 rounded-full bg-zinc-900 border border-zinc-800 flex items-center justify-center text-zinc-600">
            <IconHistory size={16} />
          </div>
          <div>
            <div class="font-medium text-zinc-400">Nenhum tráfego capturado</div>
            <div class="text-[11px] text-zinc-600 mt-0.5">Envie requisições através do proxy na porta :{relayState.config.listenPort}.</div>
          </div>
        </div>
      {/if}

    {:else}
      <!-- 2. COLEÇÃO DE ROTAS (AGRUPAMENTO EM PASTAS DINÂMICAS) -->
      {#if Object.keys(groupedTemplates).length > 1}
        <!-- Múltiplas Pastas (Accordion) -->
        {#each Object.entries(groupedTemplates) as [folderName, items]}
          <div class="border-b border-zinc-800/40">
            <!-- Cabeçalho da Pasta Retrátil -->
            <button
              onclick={() => toggleFolder(folderName)}
              class="w-full px-3 py-2 bg-zinc-900/40 hover:bg-zinc-900/80 flex items-center justify-between text-xs text-zinc-300 font-semibold transition-colors cursor-pointer"
            >
              <div class="flex items-center space-x-2 truncate">
                <IconFolder size={13} class="text-amber-400/80 shrink-0" />
                <span class="truncate">{folderName}</span>
                <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-800 text-zinc-400 font-mono font-normal">
                  {items.length}
                </span>
              </div>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="text-zinc-500 transition-transform shrink-0 ml-1.5 {collapsedFolders[folderName] ? '' : 'rotate-90'}"
              >
                <polyline points="9 18 15 12 9 6"></polyline>
              </svg>
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
                    <div class="flex items-center justify-between font-mono text-xs">
                      <div class="flex items-center space-x-2 truncate">
                        <span class="px-1.5 py-0.2 rounded text-[9px] font-bold border {getMethodBadgeStyle(tpl.method)}">
                          {tpl.method}
                        </span>
                        <span class="text-xs font-medium text-zinc-200 truncate group-hover:text-amber-200 transition-colors">
                          {tpl.name}
                        </span>
                      </div>
                      <button
                        onclick={(e) => { e.stopPropagation(); relayState.deleteTemplate(tpl.id); }}
                        class="opacity-0 group-hover:opacity-100 text-zinc-500 hover:text-rose-400 p-0.5 rounded transition-opacity cursor-pointer ml-1"
                        title="Excluir rota da coleção"
                      >
                        <IconTrash size={12} />
                      </button>
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

              <div class="flex items-center space-x-1.5">
                {#if tpl.tag}
                  <span class="text-[9px] px-1.5 py-0.2 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 font-sans truncate max-w-[120px]">
                    {tpl.tag}
                  </span>
                {/if}
                <button
                  onclick={(e) => { e.stopPropagation(); relayState.deleteTemplate(tpl.id); }}
                  class="opacity-0 group-hover:opacity-100 text-zinc-500 hover:text-rose-400 p-0.5 rounded transition-opacity cursor-pointer ml-1"
                  title="Excluir rota da coleção"
                >
                  <IconTrash size={12} />
                </button>
              </div>
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
            <div class="font-medium text-zinc-400">Nenhuma rota na coleção</div>
            <div class="text-[11px] text-zinc-600 mt-0.5">Importe rotas da sua API ou veja um modelo JSON pronto.</div>
          </div>
          <div class="flex items-center space-x-2">
            <button
              onclick={() => (showCollectionExampleModal = true)}
              class="text-xs px-2.5 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-amber-300 border border-amber-500/30 transition-colors font-medium cursor-pointer flex items-center space-x-1"
            >
              <IconFileJson size={13} />
              <span>Ver Modelo</span>
            </button>
            <button
              onclick={() => fileInputRef?.click()}
              class="text-xs px-3 py-1.5 rounded-lg bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 border border-amber-500/30 transition-colors font-medium cursor-pointer flex items-center space-x-1"
            >
              <IconDownload size={13} class="rotate-180" />
              <span>Importar</span>
            </button>
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>

<!-- Modal: Visualizar e Copiar/Salvar Modelo de Coleção para a IA -->
{#if showCollectionExampleModal}
  <div class="fixed inset-0 bg-black/75 backdrop-blur-md flex items-center justify-center z-60 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-xl w-full p-5 shadow-2xl space-y-3 flex flex-col">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-2 select-none">
        <h4 class="text-xs font-bold uppercase tracking-wider text-amber-300 flex items-center space-x-1.5">
          <IconFileJson size={14} />
          <span>Modelo JSON de Coleção de Rotas</span>
        </h4>
        <button
          onclick={() => (showCollectionExampleModal = false)}
          class="text-zinc-500 hover:text-zinc-300 text-xs cursor-pointer"
        >
          ✕
        </button>
      </div>

      {#if copyFeedback}
        <div class="p-2 rounded bg-indigo-500/10 border border-indigo-500/30 text-indigo-300 font-mono text-xs select-none">
          {copyFeedback}
        </div>
      {/if}

      <p class="text-xs text-zinc-400">
        Copie este modelo padrão e passe para a IA analisar o contrato da sua API e gerar o arquivo de rotas com pastas organizadas:
      </p>

      <textarea
        readonly
        value={COLLECTION_EXAMPLE_JSON}
        rows="12"
        class="w-full bg-zinc-950 border border-zinc-800 rounded-lg p-3 text-zinc-200 font-mono text-[11px] focus:outline-none select-all resize-none leading-relaxed"
      ></textarea>

      <div class="flex items-center justify-between pt-2 border-t border-zinc-800">
        <span class="text-[10px] text-zinc-500 font-mono">Compatível com OpenAPI 3.0, Swagger e Postman v2.1</span>
        <div class="flex items-center space-x-2">
          <button
            onclick={copyCollectionExample}
            class="text-xs px-3 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors cursor-pointer"
          >
            Copiar JSON
          </button>
          <button
            onclick={downloadCollectionExample}
            class="text-xs px-3.5 py-1.5 rounded bg-amber-600 hover:bg-amber-500 text-white font-medium transition-colors cursor-pointer flex items-center space-x-1"
          >
            <IconDownload size={12} />
            <span>Salvar Arquivo .json</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<TrafficDiagnosticModal bind:isOpen={isDiagnosticModalOpen} />
