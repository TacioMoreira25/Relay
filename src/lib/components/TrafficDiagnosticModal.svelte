<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import { IconHelpCircle } from "$lib/components/icons";

  let { isOpen = $bindable(false) }: { isOpen: boolean } = $props();

  let count404 = $derived(
    relayState.exchanges.filter(e => e.response?.statusCode === 404).length
  );
  let countPoll = $derived(
    relayState.exchanges.filter(e => relayState.isPollingExchange(e)).length
  );
  let countErrors = $derived(
    relayState.exchanges.filter(e => e.status === "failed" || (e.response && e.response.statusCode >= 400)).length
  );
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/75 backdrop-blur-sm flex items-center justify-center z-[110] p-4 font-sans select-none">
    <div class="bg-zinc-900 border border-zinc-800 rounded-2xl max-w-lg w-full p-5 shadow-2xl space-y-4 flex flex-col max-h-[85vh] overflow-hidden">
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3">
        <div class="flex items-center space-x-2.5">
          <div class="p-2 rounded-lg bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
            <IconHelpCircle size={18} />
          </div>
          <div>
            <h3 class="text-sm font-bold text-zinc-100">Diagnóstico de Tráfego Recorrente</h3>
            <p class="text-[11px] text-zinc-400">Entenda as causas de requisições repetidas em segundo plano</p>
          </div>
        </div>
        <button
          onclick={() => (isOpen = false)}
          class="text-zinc-500 hover:text-zinc-300 text-xs p-1 cursor-pointer rounded hover:bg-zinc-800"
          title="Fechar"
        >
          ✕
        </button>
      </div>

      <!-- Content Area -->
      <div class="space-y-3.5 overflow-y-auto pr-1 text-xs text-zinc-300 leading-relaxed">
        <!-- Métricas Rápidas -->
        <div class="grid grid-cols-3 gap-2 font-mono text-[11px]">
          <div class="bg-zinc-950 p-2.5 rounded-lg border border-zinc-800 flex flex-col">
            <span class="text-zinc-500 text-[10px]">Em Polling / Loop</span>
            <span class="text-indigo-400 font-bold text-sm mt-0.5">{countPoll}</span>
          </div>
          <div class="bg-zinc-950 p-2.5 rounded-lg border border-zinc-800 flex flex-col">
            <span class="text-zinc-500 text-[10px]">Rotas 404 (Not Found)</span>
            <span class="text-rose-400 font-bold text-sm mt-0.5">{count404}</span>
          </div>
          <div class="bg-zinc-950 p-2.5 rounded-lg border border-zinc-800 flex flex-col">
            <span class="text-zinc-500 text-[10px]">Falhas (4xx / 5xx)</span>
            <span class="text-amber-400 font-bold text-sm mt-0.5">{countErrors}</span>
          </div>
        </div>

        <!-- Ponto 1: Polling e Sincronização Automática -->
        <div class="bg-zinc-950/80 p-3.5 rounded-xl border border-indigo-500/20 space-y-1.5">
          <div class="flex items-center space-x-2 text-indigo-300 font-bold text-xs">
            <span>1. Polling Automático e Sincronização em Segundo Plano</span>
          </div>
          <p class="text-[11px] text-zinc-400">
            Aplicações cliente frequentemente configuram rotinas periódicas (temporizadores, watchers ou rotinas de refetch de dados) para manter a interface atualizada.
          </p>
          <div class="bg-zinc-900/60 p-2.5 rounded text-[11px] text-zinc-300 space-y-1 border border-zinc-800">
            <p><strong>O que verificar na sua aplicação:</strong></p>
            <ul class="list-disc list-inside space-y-0.5 text-zinc-400 text-[10px]">
              <li>Se há rotinas de atualização periódica configuradas com intervalos curtos (ex: a cada poucos segundos ou milissegundos).</li>
              <li>Se o cliente está tentando sincronizar dados em segundo plano mesmo sem interação do usuário na tela.</li>
            </ul>
          </div>
        </div>

        <!-- Ponto 2: Erros 404 e Retentativas Contínuas -->
        <div class="bg-zinc-950/80 p-3.5 rounded-xl border border-rose-500/20 space-y-1.5">
          <div class="flex items-center space-x-2 text-rose-300 font-bold text-xs">
            <span>2. Tentativas Repetidas em Rotas Não Encontradas (404)</span>
          </div>
          <p class="text-[11px] text-zinc-400">
            Quando o servidor responde com status 404 (rota ou recurso não encontrado), bibliotecas de consulta ou gerenciadores de estado no cliente costumam interpretar a ausência como instabilidade temporária e refazem a requisição periodicamente.
          </p>
          <div class="bg-zinc-900/60 p-2.5 rounded text-[11px] text-zinc-300 space-y-1 border border-zinc-800">
            <p><strong>O que verificar no backend e rotas:</strong></p>
            <ul class="list-disc list-inside space-y-0.5 text-zinc-400 text-[10px]">
              <li>Se os endpoints declarados no servidor possuem ou omitem prefixos de versão ou escopo (ex: <span class="font-mono text-zinc-300">/api</span> ou <span class="font-mono text-zinc-300">/v1</span>).</li>
              <li>Se o método HTTP utilizado (GET, POST, PUT, etc.) confere exatamente com o método registrado na rota do servidor.</li>
              <li>Você pode utilizar a funcionalidade de regras de rotas do Relay para reescrever ou remover prefixos automaticamente.</li>
            </ul>
          </div>
        </div>

        <!-- Ponto 3: Reconexões de Canais em Tempo Real -->
        <div class="bg-zinc-950/80 p-3.5 rounded-xl border border-amber-500/20 space-y-1.5">
          <div class="flex items-center space-x-2 text-amber-300 font-bold text-xs">
            <span>3. Reconexões Contínuas de Canais em Tempo Real</span>
          </div>
          <p class="text-[11px] text-zinc-400">
            Bibliotecas e clientes que usam WebSockets, SSE ou conexões persistentes possuem retentativa agressiva nativa. Caso o aperto de mão (handshake) ou a negociação retorne erro (como 400 ou 502), o cliente tenta reconectar continuamente em intervalos imediatos.
          </p>
        </div>

        <!-- Ponto 4: Recursos do Relay para Isolamento de Tráfego -->
        <div class="bg-zinc-950/80 p-3.5 rounded-xl border border-zinc-800 space-y-1.5">
          <div class="flex items-center space-x-2 text-zinc-200 font-bold text-xs">
            <span>4. Recursos do Relay para Isolamento de Tráfego</span>
          </div>
          <p class="text-[11px] text-zinc-400">
            Para manter a visualização limpa e focada durante seus testes:
          </p>
          <ul class="list-disc list-inside space-y-1 text-[11px] text-zinc-300 pl-1">
            <li>Ative o botão <span class="font-mono bg-amber-500/10 text-amber-300 px-1 py-0.2 rounded border border-amber-500/20">Ocultar Polling</span> na barra lateral para esconder chamadas idênticas em curto intervalo.</li>
            <li>Alterne para a aba <span class="font-mono bg-zinc-800 text-zinc-200 px-1 py-0.2 rounded">Manuais</span> para enxergar apenas as requisições disparadas deliberadamente por você.</li>
          </ul>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end pt-2 border-t border-zinc-800">
        <button
          onclick={() => (isOpen = false)}
          class="text-xs px-4 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors cursor-pointer"
        >
          Fechar
        </button>
      </div>
    </div>
  </div>
{/if}
