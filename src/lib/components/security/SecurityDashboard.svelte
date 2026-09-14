<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { SecurityFinding, FindingSeverity } from "$lib/types";
  import {
    IconShield,
    IconTrash,
    IconAlertTriangle,
    IconSearch,
    IconCode,
    IconCheck,
    IconCopy,
    IconPlay,
  } from "$lib/components/icons";
  import ActiveProbesModal from "./ActiveProbesModal.svelte";
  import StrideMap from "./StrideMap.svelte";

  let copyFeedback = $state<string | null>(null);
  let isProbesModalOpen = $state<boolean>(false);
  let activeSubView = $state<"dast" | "stride">("dast");

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
        return "Exposição de Dados";
      case "tech_leak":
        return "Vazamento Tecnológico";
      case "jwt_vulnerability":
        return "Vulnerabilidade JWT";
      case "missing_auth":
        return "Autenticação Ausente";
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
      label: "Crítico / Vulnerável",
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

<div class="flex flex-col h-full w-full bg-zinc-950 text-zinc-200 overflow-hidden">
  <!-- Top Summary Banner / Security Score -->
  <div class="p-4 border-b border-zinc-800 bg-zinc-900/40 flex items-center justify-between shrink-0">
    <div class="flex items-center space-x-6">
      <!-- Security Score Meter -->
      <div class="flex items-center space-x-3.5 bg-zinc-950/80 px-4 py-2.5 rounded-xl border {scoreInfo.border}">
        <div class="flex flex-col">
          <span class="text-[10px] text-zinc-400 font-semibold uppercase tracking-wider">Security Score</span>
          <div class="flex items-baseline space-x-1.5">
            <span class="text-2xl font-black font-mono {scoreInfo.text}">
              {relayState.securityScore}
            </span>
            <span class="text-xs text-zinc-500 font-mono">/ 100</span>
          </div>
        </div>
        <div class="h-8 w-[1px] bg-zinc-800"></div>
        <div class="flex flex-col">
          <span class="text-[11px] font-bold {scoreInfo.text}">{scoreInfo.label}</span>
          <span class="text-[10px] text-zinc-500">Auditoria Shift-Left DAST</span>
        </div>
      </div>

      <!-- Severity Badges Breakdown -->
      <div class="flex items-center space-x-2">
        <button
          onclick={() => (relayState.findingSeverityFilter = "ALL")}
          class="px-2.5 py-1 rounded-lg border text-xs transition-all flex items-center space-x-1.5 {relayState.findingSeverityFilter === 'ALL' ? 'bg-zinc-800 text-zinc-100 border-zinc-600 font-medium' : 'bg-zinc-900/60 border-zinc-800 text-zinc-400 hover:text-zinc-200'}"
        >
          <span>Todos</span>
          <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-700/80 font-mono text-zinc-300">
            {relayState.totalFindings}
          </span>
        </button>

        <button
          onclick={() => (relayState.findingSeverityFilter = "critical")}
          class="px-2.5 py-1 rounded-lg border text-xs transition-all flex items-center space-x-1.5 {relayState.findingSeverityFilter === 'critical' ? 'bg-rose-500/20 text-rose-300 border-rose-500/40 font-medium' : 'bg-zinc-900/60 border-zinc-800 text-zinc-400 hover:text-rose-400'}"
        >
          <span class="w-2 h-2 rounded-full bg-rose-500"></span>
          <span>Críticos</span>
          <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-rose-500/20 font-mono text-rose-300">
            {severityCounts.critical}
          </span>
        </button>

        <button
          onclick={() => (relayState.findingSeverityFilter = "high")}
          class="px-2.5 py-1 rounded-lg border text-xs transition-all flex items-center space-x-1.5 {relayState.findingSeverityFilter === 'high' ? 'bg-orange-500/20 text-orange-300 border-orange-500/40 font-medium' : 'bg-zinc-900/60 border-zinc-800 text-zinc-400 hover:text-orange-400'}"
        >
          <span class="w-2 h-2 rounded-full bg-orange-500"></span>
          <span>Altos</span>
          <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-orange-500/20 font-mono text-orange-300">
            {severityCounts.high}
          </span>
        </button>

        <button
          onclick={() => (relayState.findingSeverityFilter = "medium")}
          class="px-2.5 py-1 rounded-lg border text-xs transition-all flex items-center space-x-1.5 {relayState.findingSeverityFilter === 'medium' ? 'bg-amber-500/20 text-amber-300 border-amber-500/40 font-medium' : 'bg-zinc-900/60 border-zinc-800 text-zinc-400 hover:text-amber-400'}"
        >
          <span class="w-2 h-2 rounded-full bg-amber-500"></span>
          <span>Médios</span>
          <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-amber-500/20 font-mono text-amber-300">
            {severityCounts.medium}
          </span>
        </button>

        <button
          onclick={() => (relayState.findingSeverityFilter = "low")}
          class="px-2.5 py-1 rounded-lg border text-xs transition-all flex items-center space-x-1.5 {relayState.findingSeverityFilter === 'low' ? 'bg-sky-500/20 text-sky-300 border-sky-500/40 font-medium' : 'bg-zinc-900/60 border-zinc-800 text-zinc-400 hover:text-sky-400'}"
        >
          <span class="w-2 h-2 rounded-full bg-sky-500"></span>
          <span>Baixos</span>
          <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-sky-500/20 font-mono text-sky-300">
            {severityCounts.low}
          </span>
        </button>
      </div>
    </div>

    <!-- Sub-navigation & Actions -->
    <div class="flex items-center space-x-2.5 shrink-0">
      <div class="flex items-center space-x-0.5 bg-zinc-950 p-0.5 rounded-lg border border-zinc-800 text-xs">
        <button
          onclick={() => (activeSubView = "dast")}
          class="px-2.5 py-1 rounded-md transition-all flex items-center space-x-1.5 {activeSubView === 'dast' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <IconShield size={12} class="text-indigo-400" />
          <span>Auditoria DAST</span>
        </button>
        <button
          onclick={() => (activeSubView = "stride")}
          class="px-2.5 py-1 rounded-md transition-all flex items-center space-x-1.5 {activeSubView === 'stride' ? 'bg-zinc-800 text-zinc-100 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <IconAlertTriangle size={12} class="text-amber-400" />
          <span>Modelo STRIDE</span>
        </button>
      </div>

      <!-- Botão para Disparar Sondas Ativas -->
      <button
        onclick={() => (isProbesModalOpen = true)}
        class="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors flex items-center space-x-1.5 cursor-pointer shadow-xs"
        title="Disparar testes ativos de Mass Assignment, BOLA, Auth Bypass, etc."
      >
        <IconPlay size={11} class="fill-current" />
        <span>Testes Ativos (Probes)</span>
      </button>

      {#if relayState.totalFindings > 0 && activeSubView === 'dast'}
        <button
          onclick={() => relayState.clearSecurityFindings()}
          class="text-xs text-zinc-400 hover:text-rose-400 transition-colors px-2.5 py-1.5 rounded-lg border border-zinc-800 hover:border-zinc-700 bg-zinc-900 flex items-center space-x-1.5 cursor-pointer shadow-xs"
          title="Limpar todos os achados de segurança"
        >
          <IconTrash size={13} />
          <span>Limpar</span>
        </button>
      {/if}
    </div>
  </div>

  {#if activeSubView === "stride"}
    <div class="flex-1 overflow-hidden">
      <StrideMap />
    </div>
  {:else}
    <!-- Main Content Area: Split View -->
    <div class="flex-1 flex overflow-hidden">
    <!-- Left Column: Findings List -->
    <div class="w-96 border-r border-zinc-800/80 bg-zinc-950 flex flex-col h-full shrink-0 select-none">
      <div class="p-2.5 border-b border-zinc-800/80 flex items-center justify-between bg-zinc-900/20">
        <div class="flex items-center space-x-2 text-xs font-semibold text-zinc-400">
          <IconAlertTriangle size={14} class="text-indigo-400" />
          <span>Vulnerabilidades Detectadas</span>
          <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-800 text-zinc-300 font-mono">
            {relayState.filteredFindings.length}
          </span>
        </div>
      </div>

      <!-- List Items -->
      <div class="flex-1 overflow-y-auto divide-y divide-zinc-900">
        {#if relayState.filteredFindings.length === 0}
          <div class="h-full p-6 flex flex-col items-center justify-center text-center space-y-3 text-zinc-500">
            <div class="p-3.5 rounded-full bg-zinc-900 border border-zinc-800 text-emerald-400">
              <IconShield size={24} />
            </div>
            <div class="text-sm font-semibold text-zinc-300">Nenhum Achado Crítico</div>
            <p class="text-xs text-zinc-500 max-w-[240px] leading-relaxed">
              O motor de auditoria passiva inspeciona todas as requisições e respostas HTTP do proxy em tempo real.
            </p>
          </div>
        {:else}
          {#each relayState.filteredFindings as finding (finding.id)}
            {@const sev = getSeverityClass(finding.severity)}
            {@const isSelected = relayState.selectedFinding?.id === finding.id}
            <button
              type="button"
              class="w-full text-left p-3.5 hover:bg-zinc-900/50 transition-colors flex flex-col space-y-2 border-l-2 cursor-pointer {isSelected ? 'bg-zinc-900/80 border-indigo-500' : 'border-transparent'}"
              onclick={() => relayState.selectFinding(finding)}
            >
              <div class="flex items-center justify-between w-full">
                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-md border {sev.badge}">
                  {finding.severity}
                </span>
                <span class="text-[10px] text-zinc-500 font-mono">
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

    <!-- Right Column: Finding Inspector & Remediation Guide -->
    <div class="flex-1 h-full bg-zinc-950 overflow-y-auto p-6">
      {#if !relayState.selectedFinding}
        <div class="h-full flex flex-col items-center justify-center text-center space-y-3 text-zinc-500">
          <div class="p-4 rounded-full bg-zinc-900 border border-zinc-800 text-zinc-400">
            <IconShield size={32} />
          </div>
          <div class="text-sm font-medium text-zinc-300">Selecione uma vulnerabilidade para detalhes</div>
          <p class="text-xs text-zinc-500 max-w-sm leading-relaxed">
            Consulte a descrição técnica completa, severidade, impacto e guia didático de remediação recomendada.
          </p>
        </div>
      {:else}
        {@const current = relayState.selectedFinding}
        {@const sev = getSeverityClass(current.severity)}

        <div class="max-w-4xl space-y-6">
          <!-- Header of Finding -->
          <div class="flex items-start justify-between border-b border-zinc-800 pb-5">
            <div class="space-y-2">
              <div class="flex items-center space-x-2.5">
                <span class="text-xs font-bold uppercase tracking-wider px-2.5 py-0.5 rounded-md border {sev.badge}">
                  Severidade: {current.severity}
                </span>
                <span class="text-xs font-medium px-2 py-0.5 rounded-md bg-zinc-900 text-zinc-300 border border-zinc-800">
                  {getCategoryLabel(current.category)}
                </span>
              </div>
              <h1 class="text-xl font-bold text-zinc-100">{current.title}</h1>
            </div>

            <!-- Botão de Ação: Ver no Histórico -->
            <button
              onclick={() => navigateToExchange(current.exchangeId)}
              class="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors flex items-center space-x-1.5 cursor-pointer shadow-sm"
              title="Abrir a requisição HTTP completa no painel de tráfego"
            >
              <IconCode size={13} />
              <span>Inspecionar Requisição</span>
            </button>
          </div>

          <!-- Recurso Afetado -->
          <div class="p-3.5 rounded-lg bg-zinc-900/60 border border-zinc-800/80 flex items-center justify-between">
            <div class="flex items-center space-x-2 text-xs font-mono">
              <span class="text-zinc-500">Recurso / Rota:</span>
              <span class="text-zinc-200 font-semibold">{current.affectedResource}</span>
            </div>
            <button
              onclick={() => copyToClipboard(current.affectedResource, "resource")}
              class="text-zinc-400 hover:text-zinc-200 transition-colors p-1 rounded hover:bg-zinc-800"
              title="Copiar recurso afetado"
            >
              {#if copyFeedback === "resource"}
                <IconCheck size={13} class="text-emerald-400" />
              {:else}
                <IconCopy size={13} />
              {/if}
            </button>
          </div>

          <!-- Explicação & Detalhes -->
          <div class="space-y-2">
            <h2 class="text-xs font-bold uppercase tracking-wider text-zinc-400">Diagnóstico da Vulnerabilidade</h2>
            <div class="p-4 rounded-lg bg-zinc-900/40 border border-zinc-800/80 text-xs text-zinc-300 leading-relaxed font-sans">
              {current.description}
            </div>
          </div>

          <!-- Guia de Remediação / Boas Práticas -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <h2 class="text-xs font-bold uppercase tracking-wider text-emerald-400 flex items-center space-x-1.5">
                <IconShield size={14} />
                <span>Como Corrigir (Guia de Remediação)</span>
              </h2>
              <button
                onclick={() => copyToClipboard(current.remediation, "remediation")}
                class="text-xs text-zinc-400 hover:text-zinc-200 transition-colors flex items-center space-x-1 p-1 rounded hover:bg-zinc-800 cursor-pointer"
                title="Copiar texto de remediação"
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
            <div class="p-4 rounded-lg bg-emerald-950/10 border border-emerald-500/20 text-xs text-zinc-200 leading-relaxed font-sans whitespace-pre-wrap">
              {current.remediation}
            </div>
          </div>

          <!-- Dica Educativa Adicional -->
          <div class="p-3.5 rounded-lg bg-zinc-900/30 border border-zinc-800/60 text-[11px] text-zinc-400 leading-relaxed">
            <span class="font-semibold text-zinc-300">Dica Shift-Left:</span> Corrigir essas configurações de segurança durante a fase de desenvolvimento evita retrabalho em auditorias de pentest e previne brechas críticas antes do deploy em homologação e produção.
          </div>
        </div>
      {/if}
    </div>
  </div>
  {/if}

  <ActiveProbesModal bind:isOpen={isProbesModalOpen} exchange={null} />
</div>
