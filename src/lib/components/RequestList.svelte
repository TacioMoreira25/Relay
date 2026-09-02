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
  let showCollectionExampleModal = $state(false);
  let copyFeedback = $state<string | null>(null);

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
  <div class="p-2.5 border-b border-zinc-800/80 bg-zinc-900/30 space-y-2">
    <!-- Linha 1: Abas Principais + Botões de Ação -->
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

      <!-- Botões de Ação Contextual -->
      {#if relayState.sidebarTab === 'collection'}
        <input
          type="file"
          accept=".json"
          class="hidden"
          bind:this={fileInputRef}
          onchange={handleImportCollectionFile}
        />
        
        <div class="flex items-center space-x-1">
          <!-- Botão para Ver Modelo JSON de Exemplo -->
          <button
            onclick={() => (showCollectionExampleModal = true)}
            class="p-1.5 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-amber-400/80 hover:text-amber-300 transition-colors cursor-pointer shrink-0"
            title="Ver e Salvar Modelo JSON de Coleção para a IA"
          >
            <IconFileJson size={14} />
          </button>

          <!-- Botão de Importar Arquivo -->
          <button
            onclick={() => fileInputRef?.click()}
            class="p-1.5 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-300 hover:text-white transition-colors cursor-pointer shrink-0"
            title="Importar Arquivo JSON (OpenAPI / Swagger / Postman)"
          >
            <IconDownload size={14} class="rotate-180" />
          </button>
        </div>
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
                <span class="text-amber-400 text-[10px]">...</span>
              {/if}
            </div>
          </div>

          <div class="text-[11px] font-mono text-zinc-300 truncate" title={exchange.request.uri}>
            {exchange.request.uri}
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
              <span class="text-[10px] text-zinc-500 transition-transform {collapsedFolders[folderName] ? '' : 'rotate-90'}">
                ▶
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
