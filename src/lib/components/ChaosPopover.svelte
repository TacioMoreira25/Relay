<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import { IconActivity } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";

  let isOpen = $state(false);

  let latency = $state(relayState.config.latencyMs);
  let failureRate = $state(Math.round(relayState.config.simulateFailureRate * 100));
  let failureCode = $state(relayState.config.failureStatusCode);

  $effect(() => {
    latency = relayState.config.latencyMs;
    failureRate = Math.round(relayState.config.simulateFailureRate * 100);
    failureCode = relayState.config.failureStatusCode;
  });

  async function updateChaos(): Promise<void> {
    relayState.config.latencyMs = Number(latency);
    relayState.config.simulateFailureRate = Number(failureRate) / 100;
    relayState.config.failureStatusCode = Number(failureCode);
    relayState.saveCurrentProject();

    try {
      await invoke("update_proxy_config", { config: relayState.config });
    } catch (e) {
      console.error("Falha ao sincronizar chaos:", e);
    }
  }

  function resetChaos(): void {
    latency = 0;
    failureRate = 0;
    failureCode = 500;
    updateChaos();
  }

  let hasChaosActive = $derived(relayState.config.latencyMs > 0 || relayState.config.simulateFailureRate > 0);
</script>

<div class="relative inline-block">
  <!-- Botão Discreto no TopBar -->
  <button
    onclick={() => (isOpen = !isOpen)}
    class="text-xs px-2.5 py-1 rounded-md border transition-all flex items-center space-x-1.5 cursor-pointer shadow-xs {hasChaosActive ? 'bg-amber-500/10 border-amber-500/40 text-amber-300 font-bold' : 'bg-zinc-900 hover:bg-zinc-800 border-zinc-800 text-zinc-400 hover:text-zinc-200'}"
    title="Simular Internet Lenta e Falhas de Backend (Chaos Testing)"
  >
    <IconActivity size={13} class={hasChaosActive ? 'text-amber-400 animate-pulse' : 'text-zinc-500'} />
    <span class="text-[11px]">{hasChaosActive ? `${relayState.config.latencyMs}ms | ${Math.round(relayState.config.simulateFailureRate * 100)}% Err` : 'Simular Rede'}</span>
  </button>

  <!-- Popover Rápido (1 Clique) -->
  {#if isOpen}
    <!-- Backdrop Invisível para fechar ao clicar fora -->
    <div class="fixed inset-0 z-40" onclick={() => (isOpen = false)} role="presentation"></div>

    <div class="absolute right-0 mt-2 w-72 bg-zinc-900 border border-zinc-800 rounded-xl p-3.5 shadow-2xl z-50 space-y-3.5 text-xs">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-2">
        <span class="font-bold text-[11px] uppercase tracking-wider text-zinc-200">Simulador de Rede & Falhas</span>
        {#if hasChaosActive}
          <button
            onclick={resetChaos}
            class="text-[10px] text-amber-400 hover:text-amber-300 underline cursor-pointer"
          >
            Resetar
          </button>
        {/if}
      </div>

      <!-- Latência Artificial -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between text-zinc-400 text-[11px]">
          <span>Latência Artificial:</span>
          <span class="font-mono font-bold text-amber-400 bg-zinc-950 px-1.5 py-0.5 rounded border border-zinc-800">
            {latency} ms
          </span>
        </div>
        <input
          type="range"
          min="0"
          max="3000"
          step="50"
          bind:value={latency}
          onchange={updateChaos}
          class="w-full accent-amber-500 cursor-pointer h-1.5 bg-zinc-950 rounded-lg"
        />
        <div class="flex justify-between text-[9px] text-zinc-600 font-mono">
          <span>0ms (Normal)</span>
          <span>500ms (3G)</span>
          <span>3000ms (Lenta)</span>
        </div>
      </div>

      <!-- Injeção de Falhas -->
      <div class="space-y-1.5 pt-2 border-t border-zinc-800/80">
        <div class="flex items-center justify-between text-zinc-400 text-[11px]">
          <span>Injeção de Falhas:</span>
          <span class="font-mono font-bold text-rose-400 bg-zinc-950 px-1.5 py-0.5 rounded border border-zinc-800">
            {failureRate}% (Status {failureCode})
          </span>
        </div>
        <input
          type="range"
          min="0"
          max="100"
          step="5"
          bind:value={failureRate}
          onchange={updateChaos}
          class="w-full accent-rose-500 cursor-pointer h-1.5 bg-zinc-950 rounded-lg"
        />
        <div class="flex items-center justify-between pt-1">
          <span class="text-[10px] text-zinc-500">Status do Erro:</span>
          <select
            bind:value={failureCode}
            onchange={updateChaos}
            class="bg-zinc-950 border border-zinc-800 rounded px-1.5 py-0.5 text-zinc-300 font-mono text-[10px] focus:outline-none"
          >
            <option value={500}>500 Internal Error</option>
            <option value={502}>502 Bad Gateway</option>
            <option value={503}>503 Service Unavailable</option>
            <option value={504}>504 Gateway Timeout</option>
          </select>
        </div>
      </div>
    </div>
  {/if}
</div>
