<script lang="ts">
  import RequestList from "$lib/components/RequestList.svelte";
  import Inspector from "$lib/components/Inspector.svelte";
  import ProxyConfigModal from "$lib/components/ProxyConfigModal.svelte";
  import ExportModal from "$lib/components/ExportModal.svelte";
  import TipsModal from "$lib/components/TipsModal.svelte";
  import ReplayModal from "$lib/components/ReplayModal.svelte";
  import ProjectModal from "$lib/components/ProjectModal.svelte";
  import JwtManager from "$lib/components/JwtManager.svelte";
  import EnvironmentSelector from "$lib/components/EnvironmentSelector.svelte";
  import ProjectSelector from "$lib/components/ProjectSelector.svelte";
  import Logo from "$lib/components/Logo.svelte";
  import {
    IconActivity,
    IconShield,
    IconSettings,
    IconDownload,
    IconPlay,
    IconSquare,
    IconHelpCircle,
    IconPlus,
  } from "$lib/components/icons";
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { HttpExchange, InterceptedResponse, ExtractedJwt, SavedRequestTemplate } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let isConfigOpen = $state(false);
  let isExportOpen = $state(false);
  let isTipsOpen = $state(false);
  let isNewRequestOpen = $state(false);
  
  // Project Modal
  let isProjectModalOpen = $state(false);
  let selectedProjectIdToEdit = $state<string | null>(null);

  let activeTestingTemplate = $state<SavedRequestTemplate | null>(null);

  function handleOpenTemplate(tpl: SavedRequestTemplate): void {
    activeTestingTemplate = tpl;
    isNewRequestOpen = true;
  }

  function handleOpenCreateProject(): void {
    selectedProjectIdToEdit = null;
    isProjectModalOpen = true;
  }

  function handleOpenEditProject(id: string): void {
    selectedProjectIdToEdit = id;
    isProjectModalOpen = true;
  }

  async function syncInitialState(): Promise<void> {
    try {
      // Sincroniza a configuração salva do projeto ativo para o backend Rust
      if (relayState.config) {
        await invoke("update_proxy_config", { config: relayState.config });
      }

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
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      const searchInput = document.querySelector<HTMLInputElement>('input[type="text"]');
      searchInput?.focus();
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "l") {
      event.preventDefault();
      invoke("clear_exchanges");
      relayState.clear();
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "p") {
      event.preventDefault();
      toggleProxy();
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "e") {
      event.preventDefault();
      isExportOpen = true;
    } else if ((event.ctrlKey || event.metaKey) && (event.key === "/" || event.key === "?")) {
      event.preventDefault();
      isTipsOpen = true;
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "n") {
      event.preventDefault();
      activeTestingTemplate = null;
      isNewRequestOpen = true;
    }
  }

  onMount(() => {
    let unlistenReq: UnlistenFn | undefined;
    let unlistenRes: UnlistenFn | undefined;
    let unlistenErr: UnlistenFn | undefined;
    let unlistenJwt: UnlistenFn | undefined;

    syncInitialState();
    window.addEventListener("keydown", handleKeyDown);

    listen<HttpExchange>("relay:request", (event) => {
      relayState.addExchange(event.payload);
    }).then((fn) => (unlistenReq = fn));

    listen<InterceptedResponse>("relay:response", (event) => {
      relayState.updateResponse(event.payload.requestId, event.payload);
    }).then((fn) => (unlistenRes = fn));

    listen<{ requestId: string; error: string }>("relay:error", (event) => {
      relayState.setError(event.payload.requestId, event.payload.error);
    }).then((fn) => (unlistenErr = fn));

    listen<ExtractedJwt>("relay:jwt", (event) => {
      relayState.addJwt(event.payload);
    }).then((fn) => (unlistenJwt = fn));

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      unlistenReq?.();
      unlistenRes?.();
      unlistenErr?.();
      unlistenJwt?.();
    };
  });
</script>

<main class="h-screen w-screen flex flex-col bg-zinc-950 text-zinc-200 font-sans select-none antialiased relative">
  <!-- Minimalist Clean TopBar -->
  <header class="relative z-30 h-12 border-b border-zinc-800/80 bg-zinc-900/70 backdrop-blur-md px-4 flex items-center justify-between">
    <!-- Brand, Projects, Views & Smart Environment Dropdown -->
    <div class="flex items-center space-x-3">
      <!-- Novo Logo SVG Vetorial -->
      <Logo />

      <!-- Seletor de Projetos -->
      <ProjectSelector onOpenCreate={handleOpenCreateProject} onOpenEdit={handleOpenEditProject} />

      <!-- Segmented View Tabs -->
      <nav class="flex items-center space-x-1 bg-zinc-950/80 p-0.5 rounded-lg border border-zinc-800/80 text-xs">
        <button
          onclick={() => (relayState.activeView = "traffic")}
          class="px-3 py-1 rounded-md transition-all flex items-center space-x-1.5 {relayState.activeView === 'traffic' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <IconActivity size={13} />
          <span>Tráfego</span>
          {#if relayState.totalRequests > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-700/80 text-zinc-300 font-mono">
              {relayState.totalRequests}
            </span>
          {/if}
        </button>

        <button
          onclick={() => (relayState.activeView = "jwt")}
          class="px-3 py-1 rounded-md transition-all flex items-center space-x-1.5 {relayState.activeView === 'jwt' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <IconShield size={13} />
          <span>Sessão & JWT</span>
          {#if relayState.totalJwts > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-indigo-500/30 text-indigo-300 font-mono font-medium">
              {relayState.totalJwts}
            </span>
          {/if}
        </button>
      </nav>

      <!-- Dropdown Inteligente de Ambientes e Auto-Descoberta de Portas -->
      <EnvironmentSelector />
    </div>

    <!-- Actions & Controls -->
    <div class="flex items-center space-x-2">
      <!-- Nova Requisição Direta -->
      <button
        onclick={() => { activeTestingTemplate = null; isNewRequestOpen = true; }}
        class="text-xs px-2.5 py-1 rounded-md bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-200 transition-colors flex items-center space-x-1.5 cursor-pointer shadow-xs"
        title="Criar e disparar nova requisição HTTP direta (Ctrl+N)"
      >
        <IconPlus size={13} class="text-indigo-400" />
        <span class="text-[11px] font-medium">Nova Requisição</span>
      </button>

      <!-- Sutil Badge de Roteamento -->
      <button
        onclick={() => (isConfigOpen = true)}
        class="flex items-center space-x-1.5 text-[11px] font-mono text-zinc-400 hover:text-zinc-200 px-2 py-1 rounded-md hover:bg-zinc-800/60 transition-colors cursor-pointer"
        title="Configurações de portas, rotas, mocks e chaos simulator"
      >
        <IconSettings size={13} class="text-zinc-500" />
        <span>:{relayState.config.listenPort} → {relayState.config.targetHost}:{relayState.config.targetPort}</span>
        {#if relayState.config.routes.some(r => r.isMock)}
          <span class="text-[9px] px-1 py-0.2 rounded bg-indigo-500/10 text-indigo-300 border border-indigo-500/20 font-sans">Mock</span>
        {/if}
      </button>

      <!-- Ações Secundárias (Icon Only com Tooltip) -->
      <button
        onclick={() => (isExportOpen = true)}
        class="p-1.5 rounded-md hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
        title="Exportar HAR / OpenAPI ou Certificados HTTPS (Ctrl+E)"
      >
        <IconDownload size={14} />
      </button>

      <button
        onclick={() => (isTipsOpen = true)}
        class="p-1.5 rounded-md hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
        title="Guia Rápido & Atalhos (Ctrl+/)"
      >
        <IconHelpCircle size={14} />
      </button>

      <!-- Botão Iniciar com Destaque Total -->
      <button
        onclick={toggleProxy}
        class="text-xs px-3.5 py-1.5 rounded-md font-medium flex items-center space-x-1.5 transition-all shadow-md cursor-pointer {relayState.isProxyRunning ? 'bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-bold shadow-[0_0_12px_rgba(16,185,129,0.4)]' : 'bg-indigo-600 hover:bg-indigo-500 text-white'}"
        title="Atalho: Ctrl+P"
      >
        {#if relayState.isProxyRunning}
          <IconSquare size={12} class="fill-current" />
          <span>Ativo (:8080)</span>
        {:else}
          <IconPlay size={12} class="fill-current" />
          <span>Iniciar Proxy</span>
        {/if}
      </button>
    </div>
  </header>

  <!-- Main View Content Area -->
  <div class="flex-1 flex overflow-hidden">
    {#if relayState.activeView === "traffic"}
      <!-- Left Column: Request List with History / Collection Segmented Tabs -->
      <div class="w-80 border-r border-zinc-800/80 h-full bg-zinc-950">
        <RequestList onOpenTemplate={handleOpenTemplate} onOpenNewRequest={() => { activeTestingTemplate = null; isNewRequestOpen = true; }} />
      </div>

      <!-- Right Column: Inspector or Educational Empty State -->
      <div class="flex-1 h-full bg-zinc-950">
        <Inspector onOpenNewRequest={() => { activeTestingTemplate = null; isNewRequestOpen = true; }} onToggleProxy={toggleProxy} />
      </div>
    {:else}
      <!-- JWT Manager View -->
      <JwtManager />
    {/if}
  </div>

  <!-- Modals Renderizados na Raiz do App (Sem sobreposição/cortes de header) -->
  <ProjectModal bind:isOpen={isProjectModalOpen} bind:projectId={selectedProjectIdToEdit} />
  <ProxyConfigModal bind:isOpen={isConfigOpen} />
  <ExportModal bind:isOpen={isExportOpen} />
  <TipsModal bind:isOpen={isTipsOpen} />
  {#if isNewRequestOpen}
    <ReplayModal bind:isOpen={isNewRequestOpen} exchange={null} template={activeTestingTemplate} />
  {/if}
</main>
