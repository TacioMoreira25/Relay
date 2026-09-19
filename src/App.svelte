<script lang="ts">
  import RequestList from "$lib/components/RequestList.svelte";
  import Inspector from "$lib/components/Inspector.svelte";
  import ExportModal from "$lib/components/ExportModal.svelte";
  import TipsModal from "$lib/components/TipsModal.svelte";
  import ReplayModal from "$lib/components/ReplayModal.svelte";
  import ProjectModal from "$lib/components/ProjectModal.svelte";
  import JwtManager from "$lib/components/JwtManager.svelte";
  import SecurityDashboard from "$lib/components/security/SecurityDashboard.svelte";
  import EnvironmentSelector from "$lib/components/EnvironmentSelector.svelte";
  import ProjectSelector from "$lib/components/ProjectSelector.svelte";
  import ChaosPopover from "$lib/components/ChaosPopover.svelte";
  import Logo from "$lib/components/Logo.svelte";
  import {
    IconActivity,
    IconShield,
    IconDownload,
    IconPlay,
    IconSquare,
    IconHelpCircle,
    IconPlus,
    IconKey,
    IconAlertTriangle,
  } from "$lib/components/icons";
  import { relayState } from "$lib/stores/traffic.svelte";
  import type {
    HttpExchange,
    InterceptedResponse,
    ExtractedJwt,
    SavedRequestTemplate,
    SecurityFinding,
  } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let isExportOpen = $state(false);
  let isTipsOpen = $state(false);
  let isNewRequestOpen = $state(false);
  
  // Project Modal
  let isProjectModalOpen = $state(false);
  let selectedProjectIdToEdit = $state<string | null>(null);

  let activeTestingTemplate = $state<SavedRequestTemplate | null>(null);

  // Barra lateral redimensionável
  let sidebarWidth = $state<number>(
    typeof window !== "undefined"
      ? parseInt(localStorage.getItem("relay_sidebar_width") || "320", 10)
      : 320
  );
  let isResizingSidebar = $state<boolean>(false);

  function startResize(e: MouseEvent): void {
    e.preventDefault();
    isResizingSidebar = true;

    const onMouseMove = (moveEvent: MouseEvent) => {
      const maxAllowed = Math.min(750, window.innerWidth * 0.65);
      const newWidth = Math.max(260, Math.min(maxAllowed, moveEvent.clientX));
      sidebarWidth = newWidth;
    };

    const onMouseUp = () => {
      isResizingSidebar = false;
      if (typeof window !== "undefined") {
        localStorage.setItem("relay_sidebar_width", sidebarWidth.toString());
      }
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }

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

      const findings = await invoke<SecurityFinding[]>("get_security_findings");
      if (findings && findings.length > 0) {
        for (const f of findings) {
          relayState.addSecurityFinding(f);
        }
      }
    } catch (e) {
      console.error("Erro na inicialização:", e);
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
      console.error("Erro ao alternar proxy:", err);
    }
  }

  onMount(() => {
    syncInitialState();

    let unlistenReq: UnlistenFn;
    let unlistenRes: UnlistenFn;
    let unlistenErr: UnlistenFn;
    let unlistenJwt: UnlistenFn;
    let unlistenSec: UnlistenFn;

    listen<HttpExchange>("relay:request", (event) => {
      relayState.addExchange(event.payload);
    }).then((unlisten) => (unlistenReq = unlisten));

    listen<InterceptedResponse>("relay:response", (event) => {
      relayState.updateResponse(event.payload.requestId, event.payload);
    }).then((unlisten) => (unlistenRes = unlisten));

    listen<{ requestId: string; error: string }>("relay:error", (event) => {
      relayState.setError(event.payload.requestId, event.payload.error);
    }).then((unlisten) => (unlistenErr = unlisten));

    listen<ExtractedJwt>("relay:jwt", (event) => {
      relayState.addJwt(event.payload);
    }).then((unlisten) => (unlistenJwt = unlisten));

    listen<SecurityFinding>("relay:security_finding", (event) => {
      relayState.addSecurityFinding(event.payload);
    }).then((unlisten) => (unlistenSec = unlisten));

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey) {
        if (e.key === "p" || e.key === "P") {
          e.preventDefault();
          toggleProxy();
        } else if (e.key === "l" || e.key === "L") {
          e.preventDefault();
          invoke("clear_exchanges").then(() => relayState.clear());
        } else if (e.key === "e" || e.key === "E") {
          e.preventDefault();
          isExportOpen = true;
        } else if (e.key === "n" || e.key === "N") {
          e.preventDefault();
          activeTestingTemplate = null;
          isNewRequestOpen = true;
        } else if (e.key === "/") {
          e.preventDefault();
          isTipsOpen = true;
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      unlistenReq?.();
      unlistenRes?.();
      unlistenErr?.();
      unlistenJwt?.();
      unlistenSec?.();
      window.removeEventListener("keydown", handleKeyDown);
    };
  });
</script>

<main class="h-screen w-screen flex flex-col bg-zinc-950 text-zinc-100 font-sans antialiased overflow-hidden select-none">
  <!-- TopBar Minimalista e Focada -->
  <header class="h-12 border-b border-zinc-800 bg-zinc-950 px-3 flex items-center justify-between shrink-0 gap-2 relative z-30">
    <!-- Brand & Project Switcher -->
    <div class="flex items-center space-x-2 shrink-0 min-w-0">
      <Logo />

      <div class="h-4 w-[1px] bg-zinc-800 shrink-0"></div>

      <!-- Seletor de Projetos -->
      <ProjectSelector
        onOpenCreate={handleOpenCreateProject}
        onOpenEdit={handleOpenEditProject}
      />
    </div>

    <!-- Navigation Tabs & Target Selector -->
    <div class="flex items-center space-x-2 shrink-0">
      <nav class="flex items-center space-x-0.5 bg-zinc-900/90 p-0.5 rounded-lg border border-zinc-800 text-xs shrink-0 h-8">
        <button
          onclick={() => (relayState.activeView = "traffic")}
          class="h-7 px-2.5 rounded-md transition-all flex items-center space-x-1.5 whitespace-nowrap shrink-0 cursor-pointer {relayState.activeView === 'traffic' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/40'}"
        >
          <IconActivity size={13} class="text-indigo-400 shrink-0" />
          <span class="hidden sm:inline">Tráfego</span>
          {#if relayState.totalRequests > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-700/80 text-zinc-300 font-mono">
              {relayState.totalRequests}
            </span>
          {/if}
        </button>

        <button
          onclick={() => (relayState.activeView = "jwt")}
          class="h-7 px-2.5 rounded-md transition-all flex items-center space-x-1.5 whitespace-nowrap shrink-0 cursor-pointer {relayState.activeView === 'jwt' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/40'}"
          title="Sessão & Tokens JWT"
        >
          <IconKey size={13} class="text-amber-400 shrink-0" />
          <span>JWT</span>
          {#if relayState.totalJwts > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-amber-500/20 text-amber-300 font-mono font-medium">
              {relayState.totalJwts}
            </span>
          {/if}
        </button>

        <button
          onclick={() => (relayState.activeView = "security")}
          class="h-7 px-2.5 rounded-md transition-all flex items-center space-x-1.5 whitespace-nowrap shrink-0 cursor-pointer {relayState.activeView === 'security' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/40'}"
          title="Auditoria Shift-Left DAST"
        >
          <IconShield size={13} class="shrink-0 {relayState.criticalFindingsCount > 0 ? 'text-rose-400' : 'text-emerald-400'}" />
          <span class="hidden md:inline">Segurança</span>
          {#if relayState.totalFindings > 0}
            <span class="text-[10px] px-1.5 py-0.2 rounded-full font-mono font-medium {relayState.criticalFindingsCount > 0 ? 'bg-rose-500/30 text-rose-300' : 'bg-emerald-500/30 text-emerald-300'}">
              {relayState.totalFindings}
            </span>
          {/if}
        </button>
      </nav>

      <!-- Seletor de Porta Alvo (Backend Target) -->
      <EnvironmentSelector />
    </div>

    <!-- Actions & Controls -->
    <div class="flex items-center space-x-1.5 shrink-0">
      <!-- Simulador de Caos & Falhas -->
      <ChaosPopover />

      <!-- Nova Requisição Direta -->
      <button
        onclick={() => { activeTestingTemplate = null; isNewRequestOpen = true; }}
        class="h-8 text-xs px-2.5 rounded-lg bg-zinc-900 hover:bg-zinc-800/90 border border-zinc-800 hover:border-zinc-700 text-zinc-200 transition-all flex items-center space-x-1.5 cursor-pointer shadow-xs whitespace-nowrap shrink-0 active:scale-[0.98]"
        title="Criar e disparar nova requisição HTTP direta (Ctrl+N)"
      >
        <IconPlus size={13} class="text-indigo-400 shrink-0" />
        <span class="font-medium hidden lg:inline">Nova Requisição</span>
      </button>

      <!-- Ações Secundárias em formato de ícone com tooltip -->
      <button
        onclick={() => (isExportOpen = true)}
        class="h-8 w-8 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors flex items-center justify-center cursor-pointer shrink-0 active:scale-[0.98]"
        title="Exportar HAR / OpenAPI ou Certificados HTTPS (Ctrl+E)"
      >
        <IconDownload size={14} />
      </button>

      <button
        onclick={() => (isTipsOpen = true)}
        class="h-8 w-8 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors flex items-center justify-center cursor-pointer shrink-0 active:scale-[0.98]"
        title="Guia Rápido & Atalhos (Ctrl+/)"
      >
        <IconHelpCircle size={14} />
      </button>

      <!-- Botão Iniciar Proxy -->
      <button
        onclick={toggleProxy}
        class="h-8 text-xs px-3 rounded-lg font-semibold flex items-center space-x-1.5 transition-all shadow-xs cursor-pointer whitespace-nowrap shrink-0 active:scale-[0.98] {relayState.isProxyRunning ? 'bg-emerald-500 hover:bg-emerald-400 text-zinc-950 shadow-[0_0_10px_rgba(16,185,129,0.3)]' : 'bg-indigo-600 hover:bg-indigo-500 text-white'}"
        title="Atalho: Ctrl+P"
      >
        {#if relayState.isProxyRunning}
          <IconSquare size={12} class="fill-current shrink-0" />
          <span class="hidden sm:inline">Ativo (:{relayState.config.listenPort})</span>
          <span class="sm:hidden">:{relayState.config.listenPort}</span>
        {:else}
          <IconPlay size={12} class="fill-current shrink-0" />
          <span>Iniciar Proxy</span>
        {/if}
      </button>
    </div>
  </header>

  <!-- Main View Content Area -->
  <div class="flex-1 flex overflow-hidden {isResizingSidebar ? 'cursor-col-resize select-none' : ''}">
    {#if relayState.activeView === "traffic"}
      <!-- Left Column: Request List with Resizable Width -->
      <div
        class="h-full bg-zinc-950 shrink-0 overflow-hidden flex flex-col"
        style="width: {sidebarWidth}px;"
      >
        <RequestList onOpenTemplate={handleOpenTemplate} onOpenNewRequest={() => { activeTestingTemplate = null; isNewRequestOpen = true; }} />
      </div>

      <!-- Divisor Redimensionável (Splitter Handle) -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        role="separator"
        aria-orientation="vertical"
        tabindex="-1"
        onmousedown={startResize}
        class="w-1 h-full hover:w-1 bg-zinc-800/80 hover:bg-indigo-500/80 transition-colors cursor-col-resize shrink-0 relative group select-none {isResizingSidebar ? 'bg-indigo-500' : ''}"
        title="Clique e arraste para redimensionar a barra lateral"
      >
        <div class="absolute inset-y-0 -left-1 -right-1 cursor-col-resize"></div>
      </div>

      <!-- Right Column: Inspector or Educational Empty State -->
      <div class="flex-1 h-full bg-zinc-950 min-w-0 overflow-hidden">
        <Inspector onOpenNewRequest={() => { activeTestingTemplate = null; isNewRequestOpen = true; }} onToggleProxy={toggleProxy} />
      </div>
    {:else if relayState.activeView === "jwt"}
      <!-- JWT Manager View -->
      <JwtManager />
    {:else if relayState.activeView === "security"}
      <!-- Shift-Left Security DAST Dashboard -->
      <SecurityDashboard />
    {/if}
  </div>

  <!-- Modals Renderizados na Raiz do App -->
  <ProjectModal bind:isOpen={isProjectModalOpen} bind:projectId={selectedProjectIdToEdit} />
  <ExportModal bind:isOpen={isExportOpen} />
  <TipsModal bind:isOpen={isTipsOpen} />
  {#if isNewRequestOpen}
    <ReplayModal bind:isOpen={isNewRequestOpen} exchange={null} template={activeTestingTemplate} />
  {/if}
</main>
