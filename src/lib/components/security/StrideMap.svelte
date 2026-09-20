<script lang="ts">
  import type { StrideReport, StrideThreat } from "$lib/types";
  import {
    IconShield,
    IconAlertTriangle,
  } from "$lib/components/icons";

  let {
    report = null,
  }: {
    report: StrideReport | null;
  } = $props();

  let selectedThreat = $state<StrideThreat | null>(null);

  // Mantem a primeira ameaca selecionada caso nenhuma esteja ativa
  $effect(() => {
    if (report && report.threats.length > 0) {
      if (!selectedThreat || !report.threats.some((t) => t.title === selectedThreat?.title)) {
        selectedThreat = report.threats[0];
      }
    }
  });

  // Barra lateral redimensionavel
  let sidebarWidth = $state<number>(
    typeof window !== "undefined"
      ? parseInt(localStorage.getItem("relay_stride_sidebar_width") || "340", 10)
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
        localStorage.setItem("relay_stride_sidebar_width", sidebarWidth.toString());
      }
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }

  function getCategoryBadge(cat: string): { label: string; class: string } {
    switch (cat) {
      case "spoofing":
        return { label: "Spoofing (Identidade)", class: "text-rose-400 bg-rose-500/10 border-rose-500/30" };
      case "tampering":
        return { label: "Tampering (Adulteracao)", class: "text-orange-400 bg-orange-500/10 border-orange-500/30" };
      case "repudiation":
        return { label: "Repudiation (Repudio)", class: "text-amber-400 bg-amber-500/10 border-amber-500/30" };
      case "information_disclosure":
        return { label: "Info Disclosure (Vazamento)", class: "text-sky-400 bg-sky-500/10 border-sky-500/30" };
      case "denial_of_service":
        return { label: "Denial of Service (DoS)", class: "text-purple-400 bg-purple-500/10 border-purple-500/30" };
      case "elevation_of_privilege":
        return { label: "Elevation of Privilege (Privilegios)", class: "text-red-400 bg-red-500/10 border-red-500/30" };
      default:
        return { label: cat, class: "text-zinc-400 bg-zinc-800 border-zinc-700" };
    }
  }

  function getRiskBadge(risk: string): { label: string; class: string } {
    switch (risk) {
      case "critical":
        return { label: "Critico", class: "bg-rose-500/20 text-rose-300 border-rose-500/40" };
      case "high":
        return { label: "Alto", class: "bg-orange-500/20 text-orange-300 border-orange-500/40" };
      case "medium":
        return { label: "Medio", class: "bg-amber-500/20 text-amber-300 border-amber-500/40" };
      case "low":
      default:
        return { label: "Baixo", class: "bg-sky-500/20 text-sky-300 border-sky-500/40" };
    }
  }
</script>

<!-- Content Split com Barra Redimensionavel Direto Sem Header Duplicado -->
<div class="flex-1 flex h-full w-full bg-zinc-950 text-zinc-200 overflow-hidden select-none {isResizingSidebar ? 'cursor-col-resize select-none' : ''}">
  <!-- Threat List (Left) -->
  <div
    class="h-full bg-zinc-950 flex flex-col shrink-0 overflow-hidden"
    style="width: {sidebarWidth}px;"
  >
    <div class="h-11 px-3.5 border-b border-zinc-800/80 bg-zinc-900/40 flex items-center justify-between shrink-0">
      <div class="flex items-center space-x-2 text-sm font-bold text-zinc-200">
        <IconAlertTriangle size={15} class="text-amber-400 shrink-0" />
        <span>Ameacas Modeladas</span>
        <span class="text-xs px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-300 font-mono font-medium">
          {report?.threats.length || 0}
        </span>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto p-2.5 space-y-2">
      {#if !report || report.threats.length === 0}
        <div class="p-8 text-center text-zinc-500 text-sm flex flex-col items-center justify-center space-y-2.5 h-full">
          <IconShield size={28} class="text-zinc-600" />
          <div class="font-semibold text-zinc-300">Nenhuma ameaca STRIDE detectada</div>
          <p class="text-xs text-zinc-500 max-w-[240px] leading-relaxed">Intercepte trafego HTTP para gerar o modelo de ameacas arquiteturais.</p>
        </div>
      {:else}
        {#each report.threats as threat}
          {@const cat = getCategoryBadge(threat.category)}
          {@const rk = getRiskBadge(threat.riskLevel)}
          {@const isSelected = selectedThreat?.title === threat.title}

          <button
            type="button"
            class="w-full text-left p-3.5 rounded-xl border transition-all cursor-pointer flex flex-col space-y-2 active:scale-[0.99] {isSelected ? 'bg-zinc-900 border-indigo-500/70 shadow-sm ring-1 ring-indigo-500/20' : 'bg-zinc-950/70 hover:bg-zinc-900/60 border-zinc-800/70 hover:border-zinc-700/70'}"
            onclick={() => (selectedThreat = threat)}
          >
            <div class="flex items-center justify-between w-full gap-1">
              <span class="text-xs font-semibold px-2 py-0.5 rounded-md border {cat.class}">
                {cat.label}
              </span>
              <span class="text-xs font-bold px-2 py-0.5 rounded-md border {rk.class}">
                {rk.label}
              </span>
            </div>

            <div class="text-sm font-semibold text-zinc-100 line-clamp-2 leading-snug">
              {threat.title}
            </div>

            <div class="text-xs text-zinc-400 font-mono">
              {threat.affectedEndpoints.length} endpoint(s) mapeado(s)
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

  <!-- Threat Details (Right) -->
  <div class="flex-1 h-full bg-zinc-950 overflow-y-auto p-5 sm:p-7 min-w-0">
    {#if !selectedThreat}
      <div class="h-full flex flex-col items-center justify-center text-center space-y-3 text-zinc-500">
        <IconShield size={36} class="text-zinc-600" />
        <div class="text-base font-semibold text-zinc-300">Selecione uma ameaca STRIDE</div>
        <p class="text-sm text-zinc-500 max-w-sm leading-relaxed">Consulte as fronteiras de confianca arquiteturais e medidas recomendadas de mitigacao.</p>
      </div>
    {:else}
      {@const cat = getCategoryBadge(selectedThreat.category)}
      {@const rk = getRiskBadge(selectedThreat.riskLevel)}

      <div class="max-w-5xl mx-auto space-y-6">
        <!-- Hero Header Card -->
        <div class="p-6 rounded-2xl bg-zinc-900/60 border border-zinc-800/80 space-y-3.5 shadow-xs">
          <div class="flex items-center space-x-2 flex-wrap gap-2">
            <span class="text-xs font-bold px-3 py-1 rounded-lg border {cat.class}">
              {cat.label}
            </span>
            <span class="text-xs font-bold px-3 py-1 rounded-lg border {rk.class}">
              Risco: {rk.label}
            </span>
            <span class="text-xs font-mono text-zinc-400 px-2.5 py-1 rounded-lg bg-zinc-950/80 border border-zinc-800/60 font-medium">
              Classificacao Arquitetural STRIDE
            </span>
          </div>
          <h1 class="text-2xl font-bold text-zinc-100 tracking-tight">{selectedThreat.title}</h1>
        </div>

        <!-- Grid de Analise, Mitigacao e Fronteiras -->
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <!-- Coluna Principal (2/3): Analise e Mitigacao com fontes confortaveis -->
          <div class="lg:col-span-2 space-y-6">
            <!-- Descricao Tecnica / Analise de Risco -->
            <div class="p-6 rounded-2xl bg-zinc-900/40 border border-zinc-800/80 space-y-3">
              <h2 class="text-xs font-bold uppercase tracking-wider text-amber-400 flex items-center space-x-1.5">
                <IconAlertTriangle size={15} class="text-amber-400" />
                <span>Analise de Risco Arquitetural</span>
              </h2>
              <div class="text-sm sm:text-base text-zinc-200 leading-relaxed font-sans">
                {selectedThreat.description}
              </div>
            </div>

            <!-- Mitigacao STRIDE -->
            <div class="p-6 rounded-2xl bg-emerald-950/15 border border-emerald-500/25 space-y-3.5">
              <h2 class="text-xs font-bold uppercase tracking-wider text-emerald-400 flex items-center space-x-1.5">
                <IconShield size={16} />
                <span>Medida Recomendada de Mitigacao</span>
              </h2>
              <div class="p-4 rounded-xl bg-zinc-950/80 border border-emerald-500/20 text-sm sm:text-base font-sans text-zinc-100 leading-relaxed">
                {selectedThreat.mitigation}
              </div>
            </div>
          </div>

          <!-- Coluna Lateral (1/3): Fronteiras de Confianca & Endpoints -->
          <div class="space-y-6">
            <!-- Fronteiras de Confianca Identificadas no Escopo -->
            {#if report?.trustBoundaries && report.trustBoundaries.length > 0}
              <div class="p-5 rounded-2xl bg-zinc-900/50 border border-zinc-800/80 space-y-3">
                <h3 class="text-xs font-bold uppercase tracking-wider text-indigo-400 flex items-center justify-between">
                  <div class="flex items-center space-x-1.5">
                    <IconShield size={14} class="text-indigo-400" />
                    <span>Fronteiras de Confianca</span>
                  </div>
                  <span class="text-xs px-2 py-0.5 rounded-full bg-indigo-500/20 text-indigo-300 font-mono font-medium">
                    {report.trustBoundaries.length}
                  </span>
                </h3>
                <div class="space-y-2 font-mono text-xs text-zinc-300">
                  {#each report.trustBoundaries as tb}
                    <div class="p-2.5 rounded-xl bg-zinc-950/80 border border-zinc-800/80 leading-relaxed text-zinc-300">
                      {tb}
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Endpoints no Escopo da Ameaca -->
            <div class="p-5 rounded-2xl bg-zinc-900/50 border border-zinc-800/80 space-y-3">
              <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-300 flex items-center justify-between">
                <span>Endpoints Afetados</span>
                <span class="text-xs px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-300 font-mono font-medium">
                  {selectedThreat.affectedEndpoints.length}
                </span>
              </h3>
              {#if selectedThreat.affectedEndpoints.length === 0}
                <p class="text-xs text-zinc-500 italic">Nenhum endpoint especifico</p>
              {:else}
                <div class="space-y-2 font-mono text-xs text-zinc-300 max-h-64 overflow-y-auto pr-1">
                  {#each selectedThreat.affectedEndpoints as ep}
                    <div class="p-2.5 rounded-xl bg-zinc-950/80 border border-zinc-800/80 flex items-center space-x-2 truncate">
                      <span class="text-indigo-400 shrink-0">•</span>
                      <span class="truncate">{ep}</span>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <!-- Cartao Explicativo de Boas Praticas -->
            <div class="p-5 rounded-2xl bg-zinc-900/30 border border-zinc-800/60 space-y-2">
              <div class="flex items-center space-x-1.5 text-indigo-400 text-xs font-bold uppercase tracking-wider">
                <IconShield size={14} />
                <span>Modelagem Defensiva</span>
              </div>
              <p class="text-xs text-zinc-400 leading-relaxed">
                Identificar fronteiras de confianca durante a arquitetura de software previne vulnerabilidades sistemicas antes da publicacao de rotas em producao.
              </p>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
