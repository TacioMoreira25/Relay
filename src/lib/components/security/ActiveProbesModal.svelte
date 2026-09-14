<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { HttpExchange, ProbeType, ActiveProbeResult } from "$lib/types";
  import {
    IconShield,
    IconPlay,
    IconCheck,
    IconAlertTriangle,
    IconKey,
    IconCopy,
  } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";

  let {
    isOpen = $bindable(false),
    exchange = null,
  }: {
    isOpen: boolean;
    exchange: HttpExchange | null;
  } = $props();

  let selectedExchangeId = $state<string>("");
  let tokenB = $state<string>("");
  let isRunning = $state<boolean>(false);
  let activeProbeRunning = $state<string | null>(null);
  let probeResults = $state<Record<string, ActiveProbeResult>>({});
  let copyFeedback = $state<string | null>(null);

  $effect(() => {
    if (exchange) {
      selectedExchangeId = exchange.id;
    } else if (relayState.selectedExchange) {
      selectedExchangeId = relayState.selectedExchange.id;
    }
  });

  const targetExchange = $derived(
    relayState.exchanges.find((e) => e.id === selectedExchangeId) ||
      relayState.selectedExchange ||
      relayState.exchanges[0] ||
      null
  );

  const probeDefinitions: Array<{ type: ProbeType; name: string; desc: string; iconClass: string }> = [
    {
      type: "mass_assignment",
      name: "Mass Assignment",
      desc: "Injeta 'isAdmin: true' e 'role: admin' no payload JSON para testar se o backend aceita parâmetros privilegiados.",
      iconClass: "text-amber-400",
    },
    {
      type: "auth_bypass",
      name: "Auth Bypass",
      desc: "Despacha a requisição removendo cabeçalhos de autorização e cookies para verificar se o endpoint exige autenticação.",
      iconClass: "text-rose-400",
    },
    {
      type: "bola_ab",
      name: "BOLA / IDOR A-B",
      desc: "Substitui o token pelo Token B para checar se outro usuário tem acesso a recursos alheios.",
      iconClass: "text-indigo-400",
    },
    {
      type: "hidden_verbs",
      name: "Verbos Ocultos",
      desc: "Testa verbos alternativos (PUT, DELETE, PATCH, OPTIONS) na rota para identificar métodos não documentados.",
      iconClass: "text-purple-400",
    },
    {
      type: "stack_trace",
      name: "Vazamento de Stack Trace",
      desc: "Injeta caracteres anômalos para verificar se o backend expõe rastreamento de pilha em respostas de erro 500.",
      iconClass: "text-sky-400",
    },
  ];

  async function runSingleProbe(probeType: ProbeType): Promise<void> {
    if (!targetExchange) return;
    isRunning = true;
    activeProbeRunning = probeType;

    try {
      const res = await invoke<ActiveProbeResult>("run_active_probe", {
        exchangeId: targetExchange.id,
        probeType,
        tokenB: tokenB.trim() || undefined,
      });

      probeResults = {
        ...probeResults,
        [probeType]: res,
      };
    } catch (e) {
      console.error(`Erro ao executar sonda ${probeType}:`, e);
    } finally {
      isRunning = false;
      activeProbeRunning = null;
    }
  }

  async function runAllProbes(): Promise<void> {
    if (!targetExchange) return;
    for (const p of probeDefinitions) {
      await runSingleProbe(p.type);
    }
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
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 bg-black/70 backdrop-blur-xs flex items-center justify-center p-4 select-none"
    role="dialog"
  >
    <div class="bg-zinc-950 border border-zinc-800 rounded-2xl w-full max-w-3xl shadow-2xl flex flex-col max-h-[85vh] overflow-hidden">
      <!-- Header do Modal -->
      <div class="p-4 border-b border-zinc-800 bg-zinc-900/40 flex items-center justify-between">
        <div class="flex items-center space-x-2.5">
          <div class="p-2 rounded-lg bg-indigo-500/10 border border-indigo-500/20 text-indigo-400">
            <IconShield size={18} />
          </div>
          <div>
            <h2 class="text-sm font-bold text-zinc-100">Sondas de Teste Ativo (OWASP API Security)</h2>
            <p class="text-[11px] text-zinc-400">Dispare testes automatizados de injeção e autorização contra o endpoint</p>
          </div>
        </div>

        <button
          onclick={() => (isOpen = false)}
          class="text-zinc-400 hover:text-zinc-200 p-1.5 rounded-lg hover:bg-zinc-800 transition-colors cursor-pointer"
        >
          ✕
        </button>
      </div>

      <!-- Endpoint Selector & Target Info -->
      <div class="p-4 border-b border-zinc-800/80 bg-zinc-900/20 flex items-center justify-between gap-4">
        <div class="flex-1 min-w-0">
          <label for="probe-target-select" class="block text-[10px] uppercase font-bold tracking-wider text-zinc-500 mb-1">
            Endpoint Alvo
          </label>
          {#if targetExchange}
            <div class="flex items-center space-x-2 font-mono text-xs text-zinc-200 bg-zinc-900 px-3 py-1.5 rounded-lg border border-zinc-800 truncate">
              <span class="font-bold text-indigo-400">{targetExchange.request.method}</span>
              <span class="truncate">{targetExchange.request.uri}</span>
            </div>
          {:else}
            <div class="text-xs text-zinc-500">Nenhuma requisição selecionada</div>
          {/if}
        </div>

        <!-- Token B para teste BOLA -->
        <div class="w-64">
          <label for="probe-token-b-input" class="block text-[10px] uppercase font-bold tracking-wider text-zinc-500 mb-1 flex items-center space-x-1">
            <IconKey size={11} class="text-amber-400" />
            <span>Token B (Para Teste BOLA / IDOR)</span>
          </label>
          <input
            id="probe-token-b-input"
            type="text"
            placeholder="Cole o JWT do Usuário B..."
            bind:value={tokenB}
            class="w-full bg-zinc-900 border border-zinc-800 rounded-lg px-2.5 py-1.5 text-xs text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
          />
        </div>

        <!-- Executar Todas -->
        <div class="self-end">
          <button
            onclick={runAllProbes}
            disabled={isRunning || !targetExchange}
            class="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-medium text-xs flex items-center space-x-2 transition-all shadow-md cursor-pointer"
          >
            {#if isRunning}
              <div class="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
              <span>Testando...</span>
            {:else}
              <IconPlay size={13} class="fill-current" />
              <span>Executar Todas as Sondas</span>
            {/if}
          </button>
        </div>
      </div>

      <!-- Lista de Sondas e Resultados -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        {#each probeDefinitions as probe}
          {@const result = probeResults[probe.type]}
          {@const isThisRunning = activeProbeRunning === probe.type}

          <div class="p-3.5 rounded-xl border bg-zinc-900/40 {result ? (result.vulnerable ? 'border-rose-500/40 bg-rose-500/5' : 'border-emerald-500/30 bg-emerald-500/5') : 'border-zinc-800'} transition-colors">
            <div class="flex items-start justify-between gap-3">
              <div class="space-y-1 flex-1">
                <div class="flex items-center space-x-2">
                  <span class="text-xs font-bold text-zinc-200">{probe.name}</span>
                  {#if result}
                    <span class="text-[10px] font-bold px-2 py-0.5 rounded-full uppercase tracking-wider {result.vulnerable ? 'bg-rose-500/20 text-rose-300 border border-rose-500/30' : 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30'}">
                      {result.vulnerable ? "Vulnerável" : "Protegido"}
                    </span>
                  {/if}
                </div>
                <p class="text-[11px] text-zinc-400 leading-relaxed">{probe.desc}</p>
              </div>

              <button
                onclick={() => runSingleProbe(probe.type)}
                disabled={isRunning || !targetExchange}
                class="px-3 py-1.5 rounded-lg border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 text-xs font-medium text-zinc-200 transition-colors flex items-center space-x-1.5 cursor-pointer shrink-0"
              >
                {#if isThisRunning}
                  <div class="w-3 h-3 border-2 border-zinc-400 border-t-indigo-400 rounded-full animate-spin"></div>
                  <span>Testando...</span>
                {:else}
                  <IconPlay size={11} class="fill-current" />
                  <span>Testar</span>
                {/if}
              </button>
            </div>

            <!-- Detalhes do Resultado da Sonda -->
            {#if result}
              <div class="mt-3 pt-3 border-t border-zinc-800/80 space-y-2 text-xs">
                <div class="flex items-center space-x-2">
                  <span class="font-semibold {result.vulnerable ? 'text-rose-400' : 'text-emerald-400'}">
                    {result.title}
                  </span>
                  {#if result.statusCode}
                    <span class="font-mono text-[10px] text-zinc-500">(HTTP {result.statusCode})</span>
                  {/if}
                </div>

                <p class="text-[11px] text-zinc-300 leading-relaxed font-sans">{result.details}</p>

                {#if result.evidence}
                  <div class="p-2.5 rounded-lg bg-zinc-950 font-mono text-[11px] text-zinc-400 border border-zinc-800/80 whitespace-pre-wrap max-h-24 overflow-y-auto">
                    {result.evidence}
                  </div>
                {/if}

                <div class="p-2.5 rounded-lg bg-indigo-950/20 border border-indigo-500/20 text-[11px] text-indigo-200 leading-relaxed">
                  <span class="font-bold text-indigo-300">Como Corrigir: </span>{result.remediation}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Footer -->
      <div class="p-3 border-t border-zinc-800 bg-zinc-900/40 flex items-center justify-between text-[11px] text-zinc-500">
        <span>Vulnerabilidades confirmadas são enviadas automaticamente para o Painel de Segurança.</span>
        <button
          onclick={() => (isOpen = false)}
          class="px-4 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 font-medium transition-colors cursor-pointer"
        >
          Concluído
        </button>
      </div>
    </div>
  </div>
{/if}
