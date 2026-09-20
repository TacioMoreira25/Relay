<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { SecurityFinding, FindingSeverity, StrideReport } from "$lib/types";
  import {
    IconShield,
    IconTrash,
    IconAlertTriangle,
    IconCode,
    IconCheck,
    IconCopy,
    IconPlay,
    IconActivity,
  } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";
  import ActiveProbesModal from "./ActiveProbesModal.svelte";
  import StrideMap from "./StrideMap.svelte";

  let copyFeedback = $state<string | null>(null);
  let isProbesModalOpen = $state<boolean>(false);
  let activeSubView = $state<"dast" | "stride">("dast");
  let strideReport = $state<StrideReport | null>(null);
  let isStrideLoading = $state<boolean>(false);

  async function loadStrideReport(): Promise<void> {
    isStrideLoading = true;
    try {
      const res = await invoke<StrideReport>("get_stride_report");
      strideReport = res;
    } catch (e) {
      console.error("Erro ao carregar relatorio STRIDE:", e);
    } finally {
      isStrideLoading = false;
    }
  }

  $effect(() => {
    if (activeSubView === "stride" && !strideReport && !isStrideLoading) {
      loadStrideReport();
    }
  });

  // Barra lateral redimensionavel
  let sidebarWidth = $state<number>(
    typeof window !== "undefined"
      ? parseInt(localStorage.getItem("relay_security_sidebar_width") || "340", 10)
      : 340
  );
  let isResizingSidebar = $state<boolean>(false);

  function startResize(e: MouseEvent): void {
    e.preventDefault();
    isResizingSidebar = true;

    const onMouseMove = (moveEvent: MouseEvent) => {
      const maxAllowed = Math.min(700, window.innerWidth * 0.6);
      const newWidth = Math.max(250, Math.min(maxAllowed, moveEvent.clientX));
      sidebarWidth = newWidth;
    };

    const onMouseUp = () => {
      isResizingSidebar = false;
      if (typeof window !== "undefined") {
        localStorage.setItem("relay_security_sidebar_width", sidebarWidth.toString());
      }
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }

  async function copyToClipboard(text: string, id: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(text);
      copyFeedback = id;
      setTimeout(() => {
        if (copyFeedback === id) copyFeedback = null;
      }, 2000);
    } catch (e) {
      console.error("Falha ao copiar:", e);
    }
  }

  function getSeverityClass(severity: FindingSeverity): { badge: string; dot: string; text: string } {
    switch (severity) {
      case "critical":
        return {
          badge: "bg-rose-500/15 text-rose-300 border-rose-500/30",
          dot: "bg-rose-500",
          text: "text-rose-400",
        };
      case "high":
        return {
          badge: "bg-orange-500/15 text-orange-300 border-orange-500/30",
          dot: "bg-orange-500",
          text: "text-orange-400",
        };
      case "medium":
        return {
          badge: "bg-amber-500/15 text-amber-300 border-amber-500/30",
          dot: "bg-amber-500",
          text: "text-amber-400",
        };
      case "low":
        return {
          badge: "bg-sky-500/15 text-sky-300 border-sky-500/30",
          dot: "bg-sky-500",
          text: "text-sky-400",
        };
      case "info":
        return {
          badge: "bg-blue-500/15 text-blue-300 border-blue-500/30",
          dot: "bg-blue-400",
          text: "text-blue-400",
        };
      default:
        return {
          badge: "bg-zinc-800 text-zinc-300 border-zinc-700",
          dot: "bg-zinc-400",
          text: "text-zinc-400",
        };
    }
  }

  function getCategoryLabel(category: string): string {
    switch (category) {
      case "cors":
        return "CORS Inseguro";
      case "security_headers":
        return "Security Headers";
      case "data_exposure":
        return "Exposicao de Dados";
      case "tech_leak":
        return "Vazamento Tecnologico";
      case "jwt_vulnerability":
        return "Vulnerabilidade JWT";
      case "missing_auth":
        return "Autenticacao Ausente";
      default:
        return category;
    }
  }

  function getScoreColor(score: number): { text: string; bg: string; border: string; label: string } {
    if (score >= 90) {
      return {
        text: "text-emerald-400",
        bg: "bg-emerald-500",
        border: "border-emerald-500/30",
        label: "Excelente",
      };
    }
    if (score >= 70) {
      return {
        text: "text-amber-400",
        bg: "bg-amber-500",
        border: "border-amber-500/30",
        label: "Moderado",
      };
    }
    return {
      text: "text-rose-400",
      bg: "bg-rose-500",
      border: "border-rose-500/30",
      label: "Critico / Vulneravel",
    };
  }

  function navigateToExchange(exchangeId: string): void {
    const exchange = relayState.exchanges.find((e) => e.id === exchangeId);
    if (exchange) {
      relayState.select(exchange);
    }
    relayState.activeView = "traffic";
  }

  const scoreInfo = $derived(getScoreColor(relayState.securityScore));

  const severityCounts = $derived.by(() => {
    const counts = { critical: 0, high: 0, medium: 0, low: 0, info: 0 };
    for (const f of relayState.securityFindings) {
      if (counts[f.severity] !== undefined) {
        counts[f.severity]++;
      }
    }
    return counts;
  });
</script>

<div class="flex flex-col h-full w-full bg-zinc-950 text-zinc-200 overflow-hidden select-none">
  <!-- Secondary Sub-bar: Equilibrada, Clara e com Tipografia Legivel -->
  <div class="h-12 px-3 sm:px-4 border-b border-zinc-800/80 bg-zinc-900/60 backdrop-blur-xs flex items-center justify-between shrink-0 gap-3 relative z-20">
    <!-- Lado Esquerdo: Seletor de Modo (DAST vs STRIDE) + Indicador de Metrica -->
    <div class="flex items-center space-x-2 shrink-0">
      <!-- Seletor de Modo Principal -->
      <div class="h-8 flex items-center space-x-0.5 bg-zinc-950/80 p-0.5 rounded-xl border border-zinc-800/80 text-xs shrink-0 shadow-xs">
        <button
          onclick={() => (activeSubView = "dast")}
          class="h-7 px-3 rounded-lg transition-all flex items-center space-x-1.5 cursor-pointer active:scale-[0.98] {activeSubView === 'dast' ? 'bg-zinc-800 text-zinc-100 font-semibold shadow-xs border border-zinc-700/50' : 'text-zinc-400 hover:text-zinc-200'}"
          title="Auditoria DAST: Inspecao de seguranca em tempo real. Analisa passivamente headers ausentes, tokens JWT, CORS e vazamento de dados, alem de testes ativos de BOLA/IDOR, Mass Assignment e quebra de autenticacao."
        >
          <IconShield size={13} class="text-indigo-400 shrink-0" />
          <span>Auditoria DAST</span>
        </button>
        <button
          onclick={() => (activeSubView = "stride")}
          class="h-7 px-3 rounded-lg transition-all flex items-center space-x-1.5 cursor-pointer active:scale-[0.98] {activeSubView === 'stride' ? 'bg-zinc-800 text-zinc-100 font-semibold shadow-xs border border-zinc-700/50' : 'text-zinc-400 hover:text-zinc-200'}"
          title="Modelo STRIDE: Modelagem arquitetural defensiva nas fronteiras da API. Avalia riscos de Spoofing (Identidade), Tampering (Adulteracao), Repudio, Vazamento de Dados, Negacao de Servico (DoS) e Elevacao de Privilegio."
        >
          <IconAlertTriangle size={13} class="text-amber-400 shrink-0" />
          <span>Modelo STRIDE</span>
        </button>
      </div>

      <!-- Badge de Metrica (Score no DAST, Contagem no STRIDE) -->
      {#if activeSubView === "dast"}
        <div class="h-8 flex items-center space-x-2 px-3 rounded-xl bg-zinc-950/80 border {scoreInfo.border} shrink-0 shadow-xs" title="Pontuacao Geral DAST: {scoreInfo.label}">
          <span class="text-xs uppercase font-bold text-zinc-400 tracking-wider">Score</span>
          <span class="text-sm font-black font-mono {scoreInfo.text}">{relayState.securityScore}</span>
          <span class="w-2 h-2 rounded-full {scoreInfo.bg}"></span>
          <span class="text-xs font-semibold {scoreInfo.text} hidden md:inline">{scoreInfo.label}</span>
        </div>
      {:else}
        <div class="h-8 flex items-center space-x-2 px-3 rounded-xl bg-zinc-950/80 border border-zinc-800/80 text-xs shrink-0 shadow-xs">
          <IconShield size={14} class="text-indigo-400 shrink-0" />
          <span class="text-xs uppercase font-bold text-zinc-400 tracking-wider">STRIDE</span>
          <span class="text-sm font-bold font-mono text-zinc-100">{strideReport?.totalThreats || 0}</span>
          <span class="text-zinc-400 text-xs hidden md:inline">Ameacas Identificadas</span>
        </div>
      {/if}
    </div>

    <!-- Centro: Filtros no DAST / Status no STRIDE -->
    <div class="flex-1 flex items-center justify-center min-w-0 px-2">
      {#if activeSubView === "dast"}
        <!-- Filtros de Severidade Centralizados -->
        <div class="flex items-center space-x-0.5 bg-zinc-950/80 p-0.5 rounded-xl border border-zinc-800/80 text-xs shrink-0 shadow-xs overflow-x-auto">
          <button
            onclick={() => (relayState.findingSeverityFilter = "ALL")}
            class="h-7 px-2.5 rounded-lg transition-all flex items-center space-x-1.5 cursor-pointer active:scale-[0.98] {relayState.findingSeverityFilter === 'ALL' ? 'bg-zinc-800 text-zinc-100 font-semibold shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
            title="Todos os achados ({relayState.totalFindings})"
          >
            <span>Todos</span>
            <span class="text-xs px-1.5 py-0.2 rounded-full bg-zinc-700/80 font-mono text-zinc-200">
              {relayState.totalFindings}
            </span>
          </button>

          <button
            onclick={() => (relayState.findingSeverityFilter = "critical")}
            class="h-7 px-2.5 rounded-lg transition-all flex items-center space-x-1.5 cursor-pointer active:scale-[0.98] {relayState.findingSeverityFilter === 'critical' ? 'bg-rose-500/20 text-rose-300 font-semibold border border-rose-500/40 shadow-xs' : 'text-zinc-400 hover:text-rose-400'}"
            title="Achados Criticos ({severityCounts.critical})"
          >
            <span class="w-2 h-2 rounded-full bg-rose-500"></span>
            <span>Criticos</span>
            {#if severityCounts.critical > 0}
              <span class="text-xs px-1.5 py-0.2 rounded-full bg-rose-500/30 font-mono text-rose-200">
                {severityCounts.critical}
              </span>
            {/if}
          </button>

          <button
            onclick={() => (relayState.findingSeverityFilter = "high")}
            class="h-7 px-2.5 rounded-lg transition-all flex items-center space-x-1.5 cursor-pointer active:scale-[0.98] {relayState.findingSeverityFilter === 'high' ? 'bg-orange-500/20 text-orange-300 font-semibold border border-orange-500/40 shadow-xs' : 'text-zinc-400 hover:text-orange-400'}"
            title="Achados Altos ({severityCounts.high})"
          >
            <span class="w-2 h-2 rounded-full bg-orange-500"></span>
            <span>Altos</span>
            {#if severityCounts.high > 0}
              <span class="text-xs px-1.5 py-0.2 rounded-full bg-orange-500/30 font-mono text-orange-200">
                {severityCounts.high}
              </span>
            {/if}
          </button>

          <button
            onclick={() => (relayState.findingSeverityFilter = "medium")}
            class="h-7 px-2.5 rounded-lg transition-all flex items-center space-x-1.5 cursor-pointer active:scale-[0.98] {relayState.findingSeverityFilter === 'medium' ? 'bg-amber-500/20 text-amber-300 font-semibold border border-amber-500/40 shadow-xs' : 'text-zinc-400 hover:text-amber-400'}"
            title="Achados Medios ({severityCounts.medium})"
          >
            <span class="w-2 h-2 rounded-full bg-amber-500"></span>
            <span>Medios</span>
            {#if severityCounts.medium > 0}
              <span class="text-xs px-1.5 py-0.2 rounded-full bg-amber-500/30 font-mono text-amber-200">
                {severityCounts.medium}
              </span>
            {/if}
          </button>

          <button
            onclick={() => (relayState.findingSeverityFilter = "low")}
            class="h-7 px-2.5 rounded-lg transition-all flex items-center space-x-1.5 cursor-pointer active:scale-[0.98] {relayState.findingSeverityFilter === 'low' ? 'bg-sky-500/20 text-sky-300 font-semibold border border-sky-500/40 shadow-xs' : 'text-zinc-400 hover:text-sky-400'}"
            title="Achados Baixos ({severityCounts.low})"
          >
            <span class="w-2 h-2 rounded-full bg-sky-500"></span>
            <span>Baixos</span>
            {#if severityCounts.low > 0}
              <span class="text-xs px-1.5 py-0.2 rounded-full bg-sky-500/30 font-mono text-sky-200">
                {severityCounts.low}
              </span>
            {/if}
          </button>

          <button
            onclick={() => (relayState.findingSeverityFilter = "info")}
            class="h-7 px-2.5 rounded-lg transition-all flex items-center space-x-1.5 cursor-pointer active:scale-[0.98] {relayState.findingSeverityFilter === 'info' ? 'bg-blue-500/20 text-blue-300 font-semibold border border-blue-500/40 shadow-xs' : 'text-zinc-400 hover:text-blue-400'}"
            title="Achados Informativos ({severityCounts.info})"
          >
            <span class="w-2 h-2 rounded-full bg-blue-400"></span>
            <span>Info</span>
            {#if severityCounts.info > 0}
              <span class="text-xs px-1.5 py-0.2 rounded-full bg-blue-500/30 font-mono text-blue-200">
                {severityCounts.info}
              </span>
            {/if}
          </button>
        </div>
      {:else}
        <!-- Resumo STRIDE Centralizado e Limpo -->
        <div class="flex items-center space-x-2 text-xs text-zinc-400">
          <span class="px-3 py-1.5 rounded-xl bg-zinc-950/80 border border-zinc-800/80 font-medium">
            Fronteiras Mapeadas: <strong class="text-zinc-100 font-mono">{strideReport?.trustBoundaries?.length || 0}</strong> zonas de confianca
          </span>
        </div>
      {/if}
    </div>

    <!-- Lado Direito: Acoes Contextuais -->
    <div class="flex items-center space-x-2 shrink-0">
      {#if activeSubView === "dast"}
        <button
          onclick={() => (isProbesModalOpen = true)}
          class="h-8 px-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold transition-all flex items-center space-x-1.5 cursor-pointer shadow-xs active:scale-[0.98] shrink-0"
          title="Disparar testes ativos de Mass Assignment, BOLA, Auth Bypass, etc."
        >
          <IconPlay size={11} class="fill-current" />
          <span>Testes Ativos</span>
        </button>

        <button
          onclick={() => relayState.clearSecurityFindings()}
          disabled={relayState.totalFindings === 0}
          class="h-8 px-2.5 rounded-xl border transition-all flex items-center space-x-1.5 shrink-0 text-xs font-medium active:scale-[0.98] {relayState.totalFindings === 0 ? 'opacity-40 border-zinc-800/60 bg-zinc-950/40 text-zinc-600 cursor-not-allowed' : 'border-zinc-800 hover:border-rose-500/40 bg-zinc-950/80 hover:bg-rose-500/10 text-zinc-400 hover:text-rose-300 cursor-pointer shadow-xs'}"
          title="Limpar todos os achados de seguranca"
        >
          <IconTrash size={13} />
          <span>Limpar</span>
        </button>
      {:else}
        <button
          onclick={loadStrideReport}
          disabled={isStrideLoading}
          class="h-8 px-3 rounded-xl border border-zinc-800/80 hover:border-zinc-700 bg-zinc-950/80 hover:bg-zinc-800/90 text-xs font-semibold text-zinc-200 transition-all flex items-center space-x-1.5 cursor-pointer shadow-xs active:scale-[0.98] shrink-0"
          title="Recalcular modelo de ameacas com base no trafego atual"
        >
          <IconActivity size={13} class={isStrideLoading ? 'animate-spin text-indigo-400' : 'text-zinc-400'} />
          <span>Atualizar Modelo</span>
        </button>
      {/if}
    </div>
  </div>


  {#if activeSubView === "stride"}
    <div class="flex-1 overflow-hidden">
      <StrideMap report={strideReport} />
    </div>
  {:else}
    <!-- Main Content Area: Split View Redimensionavel -->
    <div class="flex-1 flex overflow-hidden {isResizingSidebar ? 'cursor-col-resize select-none' : ''}">
      <!-- Coluna Esquerda: Lista de Vulnerabilidades -->
      <div
        class="h-full bg-zinc-950 flex flex-col shrink-0 overflow-hidden"
        style="width: {sidebarWidth}px;"
      >
        <!-- Header da Barra Lateral de Seguranca -->
        <div class="h-10 px-3 border-b border-zinc-800/80 bg-zinc-900/30 flex items-center justify-between shrink-0 gap-2">
          <div class="flex items-center space-x-2 text-xs font-semibold text-zinc-300 min-w-0">
            <IconAlertTriangle size={13} class="text-indigo-400 shrink-0" />
            <span class="truncate">Vulnerabilidades Detectadas</span>
            <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-800 text-zinc-300 font-mono shrink-0">
              {relayState.filteredFindings.length}
            </span>
          </div>
        </div>

        <!-- Lista de Cards -->
        <div class="flex-1 overflow-y-auto p-2 space-y-1.5">
          {#if relayState.filteredFindings.length === 0}
            <div class="h-full p-6 flex flex-col items-center justify-center text-center space-y-3 text-zinc-500">
              <div class="p-3.5 rounded-full bg-zinc-900 border border-zinc-800 text-emerald-400">
                <IconShield size={24} />
              </div>
              <div class="text-sm font-semibold text-zinc-300">Nenhum Achado Critico</div>
              <p class="text-xs text-zinc-500 max-w-[240px] leading-relaxed">
                O motor de auditoria passiva inspeciona todas as requisicoes e respostas HTTP do proxy em tempo real.
              </p>
            </div>
          {:else}
            {#each relayState.filteredFindings as finding (finding.id)}
              {@const sev = getSeverityClass(finding.severity)}
              {@const isSelected = relayState.selectedFinding?.id === finding.id}
              <button
                type="button"
                class="w-full text-left p-3 rounded-xl border transition-all cursor-pointer flex flex-col space-y-1.5 active:scale-[0.99] {isSelected ? 'bg-zinc-900 border-indigo-500/60 shadow-sm ring-1 ring-indigo-500/20' : 'bg-zinc-950/60 hover:bg-zinc-900/60 border-zinc-800/60 hover:border-zinc-700/60'}"
                onclick={() => relayState.selectFinding(finding)}
              >
                <div class="flex items-center justify-between w-full">
                  <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-md border {sev.badge}">
                    {finding.severity}
                  </span>
                  <span class="text-[10px] text-zinc-500 font-mono truncate max-w-[130px]">
                    {getCategoryLabel(finding.category)}
                  </span>
                </div>

                <div class="text-xs font-semibold text-zinc-200 line-clamp-1">
                  {finding.title}
                </div>

                <div class="text-[11px] font-mono text-zinc-400 truncate w-full flex items-center space-x-1">
                  <span class="text-zinc-500">Recurso:</span>
                  <span class="text-zinc-300 truncate">{finding.affectedResource}</span>
                </div>
              </button>
            {/each}
          {/if}
        </div>
      </div>

      <!-- Divisor Redimensionavel (Splitter Handle) -->
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

      <!-- Right Column: Finding Inspector & Remediation Guide -->
      <div class="flex-1 h-full bg-zinc-950 overflow-y-auto p-4 sm:p-6 min-w-0">
      {#if !relayState.selectedFinding}
        <div class="h-full flex flex-col items-center justify-center text-center space-y-3 text-zinc-500">
          <div class="p-4 rounded-full bg-zinc-900 border border-zinc-800 text-zinc-400">
            <IconShield size={32} />
          </div>
          <div class="text-sm font-medium text-zinc-300">Selecione uma vulnerabilidade para detalhes</div>
          <p class="text-xs text-zinc-500 max-w-sm leading-relaxed">
            Consulte a descricao tecnica completa, severidade, impacto e guia didatico de remediacao recomendada.
          </p>
        </div>
      {:else}
        {@const current = relayState.selectedFinding}
        {@const sev = getSeverityClass(current.severity)}

        <div class="max-w-5xl mx-auto space-y-5">
          <!-- Hero Header Card -->
          <div class="p-5 rounded-2xl bg-zinc-900/60 border border-zinc-800/80 space-y-3.5 shadow-xs">
            <div class="flex items-start justify-between gap-4">
              <div class="space-y-2 min-w-0">
                <div class="flex items-center space-x-2 flex-wrap gap-1.5">
                  <span class="text-xs font-bold uppercase tracking-wider px-2.5 py-0.5 rounded-lg border {sev.badge}">
                    Severidade: {current.severity}
                  </span>
                  <span class="text-xs font-medium px-2.5 py-0.5 rounded-lg bg-zinc-900 text-zinc-300 border border-zinc-800">
                    {getCategoryLabel(current.category)}
                  </span>
                  <span class="text-[11px] font-mono text-zinc-500 px-2 py-0.5 rounded-lg bg-zinc-950/80 border border-zinc-800/60">
                    Shift-Left DAST
                  </span>
                </div>
                <h1 class="text-xl font-bold text-zinc-100 leading-snug">{current.title}</h1>
              </div>

              <!-- Botao de Acao: Ver no Historico -->
              <button
                onclick={() => navigateToExchange(current.exchangeId)}
                class="h-8 px-3.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold transition-all flex items-center space-x-1.5 cursor-pointer shadow-xs whitespace-nowrap shrink-0 active:scale-[0.98]"
                title="Abrir a requisicao HTTP completa no painel de trafego"
              >
                <IconCode size={13} />
                <span>Inspecionar Requisicao</span>
              </button>
            </div>

            <!-- Rota e Metadados do Recurso -->
            <div class="p-3 rounded-xl bg-zinc-950/80 border border-zinc-800/80 flex items-center justify-between gap-3">
              <div class="flex items-center space-x-2 text-xs font-mono min-w-0">
                <span class="text-zinc-500 shrink-0">Recurso / Rota Afetada:</span>
                <span class="text-zinc-200 font-semibold truncate select-all">{current.affectedResource}</span>
              </div>
              <button
                onclick={() => copyToClipboard(current.affectedResource, "resource")}
                class="h-7 px-2.5 rounded-lg bg-zinc-800/60 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-all flex items-center space-x-1 cursor-pointer shrink-0 active:scale-[0.95]"
                title="Copiar rota afetada"
              >
                {#if copyFeedback === "resource"}
                  <IconCheck size={13} class="text-emerald-400" />
                  <span class="text-[10px] text-emerald-400 font-medium">Copiado</span>
                {:else}
                  <IconCopy size={13} />
                  <span class="text-[10px]">Copiar</span>
                {/if}
              </button>
            </div>
          </div>

          <!-- Grid Principal de Diagnostico e Remediacao -->
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-5">
            <!-- Coluna Principal (2/3 da largura): Diagnostico e Remediacao -->
            <div class="lg:col-span-2 space-y-5">
              <!-- Diagnostico da Vulnerabilidade -->
              <div class="p-5 rounded-2xl bg-zinc-900/40 border border-zinc-800/80 space-y-2.5">
                <h2 class="text-xs font-bold uppercase tracking-wider text-zinc-400 flex items-center space-x-1.5">
                  <IconAlertTriangle size={13} class="text-amber-400" />
                  <span>Diagnostico Tecnico</span>
                </h2>
                <div class="text-xs text-zinc-300 leading-relaxed font-sans">
                  {current.description}
                </div>
              </div>

              <!-- Guia de Remediacao -->
              <div class="p-5 rounded-2xl bg-emerald-950/10 border border-emerald-500/20 space-y-3">
                <div class="flex items-center justify-between">
                  <h2 class="text-xs font-bold uppercase tracking-wider text-emerald-400 flex items-center space-x-1.5">
                    <IconShield size={14} />
                    <span>Como Corrigir (Guia de Remediacao)</span>
                  </h2>
                  <button
                    onclick={() => copyToClipboard(current.remediation, "remediation")}
                    class="h-7 px-2.5 rounded-lg bg-emerald-950/40 hover:bg-emerald-900/50 border border-emerald-500/30 text-emerald-300 text-xs font-medium transition-all flex items-center space-x-1 cursor-pointer active:scale-[0.95]"
                    title="Copiar solucao de remediacao"
                  >
                    {#if copyFeedback === "remediation"}
                      <IconCheck size={12} class="text-emerald-400" />
                      <span class="text-[11px] text-emerald-400 font-medium">Copiado</span>
                    {:else}
                      <IconCopy size={12} />
                      <span class="text-[11px]">Copiar</span>
                    {/if}
                  </button>
                </div>
                <div class="p-3.5 rounded-xl bg-zinc-950/70 border border-emerald-500/20 text-xs font-mono text-zinc-200 leading-relaxed whitespace-pre-wrap">
                  {current.remediation}
                </div>
              </div>
            </div>

            <!-- Coluna Lateral (1/3 da largura): Metadados & Dicas Didaticas -->
            <div class="space-y-5">
              <!-- Resumo do Risco -->
              <div class="p-4 rounded-2xl bg-zinc-900/50 border border-zinc-800/80 space-y-3">
                <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-400">Resumo da Auditoria</h3>
                <div class="space-y-2 text-xs">
                  <div class="flex items-center justify-between py-1 border-b border-zinc-800/60">
                    <span class="text-zinc-500">Classificacao:</span>
                    <span class="font-semibold text-zinc-200">{getCategoryLabel(current.category)}</span>
                  </div>
                  <div class="flex items-center justify-between py-1 border-b border-zinc-800/60">
                    <span class="text-zinc-500">Nivel de Risco:</span>
                    <span class="font-bold uppercase {sev.text}">{current.severity}</span>
                  </div>
                  <div class="flex items-center justify-between py-1 border-b border-zinc-800/60">
                    <span class="text-zinc-500">Auditoria:</span>
                    <span class="text-zinc-300">Shift-Left Passiva</span>
                  </div>
                </div>
              </div>

              <!-- Dica Educativa Shift-Left -->
              <div class="p-4 rounded-2xl bg-zinc-900/30 border border-zinc-800/60 space-y-2">
                <div class="flex items-center space-x-1.5 text-indigo-400 text-xs font-semibold">
                  <IconShield size={13} />
                  <span>Boas Praticas Shift-Left</span>
                </div>
                <p class="text-[11px] text-zinc-400 leading-relaxed">
                  Corrigir essas falhas durante o desenvolvimento elimina retrabalho em testes de invasao (pentest) e protege suas APIs antes de alcancarem producao.
                </p>
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
  {/if}

  <ActiveProbesModal bind:isOpen={isProbesModalOpen} exchange={null} />
</div>
