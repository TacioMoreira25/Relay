<script lang="ts">
  import type { StrideReport, StrideThreat } from "$lib/types";
  import {
    IconShield,
    IconAlertTriangle,
    IconCheck,
    IconActivity,
    IconKey,
  } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let report = $state<StrideReport | null>(null);
  let isLoading = $state<boolean>(false);
  let selectedThreat = $state<StrideThreat | null>(null);

  async function loadStrideReport(): Promise<void> {
    isLoading = true;
    try {
      const res = await invoke<StrideReport>("get_stride_report");
      report = res;
      if (res.threats.length > 0 && !selectedThreat) {
        selectedThreat = res.threats[0];
      }
    } catch (e) {
      console.error("Erro ao carregar relatório STRIDE:", e);
    } finally {
      isLoading = false;
    }
  }

  onMount(() => {
    loadStrideReport();
  });

  function getCategoryBadge(cat: string): { label: string; class: string } {
    switch (cat) {
      case "spoofing":
        return { label: "Spoofing (Identidade)", class: "text-rose-400 bg-rose-500/10 border-rose-500/30" };
      case "tampering":
        return { label: "Tampering (Adulteração)", class: "text-orange-400 bg-orange-500/10 border-orange-500/30" };
      case "repudiation":
        return { label: "Repudiation (Repúdio)", class: "text-amber-400 bg-amber-500/10 border-amber-500/30" };
      case "information_disclosure":
        return { label: "Info Disclosure (Vazamento)", class: "text-sky-400 bg-sky-500/10 border-sky-500/30" };
      case "denial_of_service":
        return { label: "Denial of Service (DoS)", class: "text-purple-400 bg-purple-500/10 border-purple-500/30" };
      case "elevation_of_privilege":
        return { label: "Elevation of Privilege (Privilégios)", class: "text-red-400 bg-red-500/10 border-red-500/30" };
      default:
        return { label: cat, class: "text-zinc-400 bg-zinc-800 border-zinc-700" };
    }
  }

  function getRiskBadge(risk: string): { label: string; class: string } {
    switch (risk) {
      case "critical":
        return { label: "Crítico", class: "bg-rose-500/20 text-rose-300 border-rose-500/40" };
      case "high":
        return { label: "Alto", class: "bg-orange-500/20 text-orange-300 border-orange-500/40" };
      case "medium":
        return { label: "Médio", class: "bg-amber-500/20 text-amber-300 border-amber-500/40" };
      case "low":
      default:
        return { label: "Baixo", class: "bg-sky-500/20 text-sky-300 border-sky-500/40" };
    }
  }
</script>

<div class="flex flex-col h-full w-full bg-zinc-950 text-zinc-200 overflow-hidden select-none">
  <!-- Top Summary & Refresh -->
  <div class="p-4 border-b border-zinc-800 bg-zinc-900/40 flex items-center justify-between shrink-0">
    <div class="flex items-center space-x-6">
      <div class="flex items-center space-x-3 bg-zinc-950/80 px-4 py-2 rounded-xl border border-zinc-800">
        <IconShield size={18} class="text-indigo-400" />
        <div class="flex flex-col">
          <span class="text-[10px] text-zinc-500 uppercase tracking-wider font-semibold">Ameaças STRIDE</span>
          <span class="text-lg font-bold font-mono text-zinc-100">
            {report?.totalThreats || 0} identificadas
          </span>
        </div>
      </div>

      <!-- Trust Boundaries -->
      <div class="hidden lg:flex items-center space-x-2">
        <span class="text-xs font-semibold text-zinc-400">Fronteiras de Confiança:</span>
        <div class="flex items-center space-x-1.5">
          {#each report?.trustBoundaries || [] as tb}
            <span class="text-[10px] px-2 py-0.5 rounded-md bg-zinc-900 border border-zinc-800 text-zinc-300 font-mono">
              {tb}
            </span>
          {/each}
        </div>
      </div>
    </div>

    <button
      onclick={loadStrideReport}
      disabled={isLoading}
      class="px-3 py-1.5 rounded-lg border border-zinc-800 bg-zinc-900 hover:bg-zinc-800 text-xs text-zinc-200 transition-colors flex items-center space-x-1.5 cursor-pointer"
    >
      <IconActivity size={12} class={isLoading ? 'animate-spin text-indigo-400' : 'text-zinc-400'} />
      <span>Atualizar Modelo</span>
    </button>
  </div>

  <!-- Content Split -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Threat List (Left) -->
    <div class="w-96 border-r border-zinc-800/80 bg-zinc-950 flex flex-col h-full shrink-0 overflow-y-auto divide-y divide-zinc-900">
      {#if !report || report.threats.length === 0}
        <div class="p-8 text-center text-zinc-500 text-xs flex flex-col items-center justify-center space-y-2 h-full">
          <IconShield size={24} class="text-zinc-600" />
          <div class="font-medium text-zinc-400">Nenhuma ameaça STRIDE detectada</div>
          <p class="text-[11px] text-zinc-600">Intercepte tráfego HTTP para gerar o diagrama de ameaças.</p>
        </div>
      {:else}
        {#each report.threats as threat}
          {@const cat = getCategoryBadge(threat.category)}
          {@const rk = getRiskBadge(threat.riskLevel)}
          {@const isSelected = selectedThreat?.title === threat.title}

          <button
            type="button"
            class="w-full text-left p-3.5 hover:bg-zinc-900/50 transition-colors flex flex-col space-y-2 border-l-2 cursor-pointer {isSelected ? 'bg-zinc-900/80 border-indigo-500' : 'border-transparent'}"
            onclick={() => (selectedThreat = threat)}
          >
            <div class="flex items-center justify-between w-full">
              <span class="text-[10px] font-bold px-2 py-0.5 rounded-md border {cat.class}">
                {cat.label}
              </span>
              <span class="text-[10px] font-bold px-1.5 py-0.2 rounded border {rk.class}">
                {rk.label}
              </span>
            </div>

            <div class="text-xs font-semibold text-zinc-200">
              {threat.title}
            </div>

            <div class="text-[11px] text-zinc-500 font-mono">
              {threat.affectedEndpoints.length} endpoint(s) mapeado(s)
            </div>
          </button>
        {/each}
      {/if}
    </div>

    <!-- Threat Details (Right) -->
    <div class="flex-1 h-full bg-zinc-950 overflow-y-auto p-6">
      {#if !selectedThreat}
        <div class="h-full flex flex-col items-center justify-center text-center space-y-2 text-zinc-500">
          <IconShield size={32} class="text-zinc-600" />
          <div class="text-sm font-medium text-zinc-300">Selecione uma ameaça STRIDE</div>
          <p class="text-xs text-zinc-500 max-w-sm">Consulte as fronteiras de risco e guias de mitigação recomendada.</p>
        </div>
      {:else}
        {@const cat = getCategoryBadge(selectedThreat.category)}
        {@const rk = getRiskBadge(selectedThreat.riskLevel)}

        <div class="max-w-3xl space-y-6">
          <div class="border-b border-zinc-800 pb-4 space-y-2">
            <div class="flex items-center space-x-2">
              <span class="text-xs font-bold px-2 py-0.5 rounded-md border {cat.class}">
                {cat.label}
              </span>
              <span class="text-xs font-bold px-2 py-0.5 rounded-md border {rk.class}">
                Risco: {rk.label}
              </span>
            </div>
            <h1 class="text-xl font-bold text-zinc-100">{selectedThreat.title}</h1>
          </div>

          <!-- Descrição Técnica -->
          <div class="space-y-2">
            <h2 class="text-xs font-bold uppercase tracking-wider text-zinc-400">Análise de Risco</h2>
            <div class="p-4 rounded-xl bg-zinc-900/40 border border-zinc-800/80 text-xs text-zinc-300 leading-relaxed">
              {selectedThreat.description}
            </div>
          </div>

          <!-- Endpoints Afetados -->
          {#if selectedThreat.affectedEndpoints.length > 0}
            <div class="space-y-2">
              <h2 class="text-xs font-bold uppercase tracking-wider text-zinc-400">Endpoints na Fronteira de Risco</h2>
              <div class="p-3 rounded-xl bg-zinc-900/60 border border-zinc-800/80 space-y-1.5 font-mono text-xs text-zinc-300">
                {#each selectedThreat.affectedEndpoints as ep}
                  <div class="flex items-center space-x-2">
                    <span class="text-indigo-400">•</span>
                    <span>{ep}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Mitigação STRIDE -->
          <div class="space-y-2">
            <h2 class="text-xs font-bold uppercase tracking-wider text-emerald-400 flex items-center space-x-1.5">
              <IconShield size={14} />
              <span>Medida de Mitigação Arquitetural</span>
            </h2>
            <div class="p-4 rounded-xl bg-emerald-950/15 border border-emerald-500/25 text-xs text-zinc-200 leading-relaxed font-sans">
              {selectedThreat.mitigation}
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
