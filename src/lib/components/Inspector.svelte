<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";

  let activeTab = $state<"request" | "response">("request");
  let exchange = $derived(relayState.selectedExchange);
</script>

<div class="flex flex-col h-full bg-zinc-950 text-zinc-200 overflow-hidden">
  {#if !exchange}
    <div class="flex-1 flex items-center justify-center text-zinc-600 text-sm">
      Selecione uma requisição na lista lateral para inspecionar headers e payload.
    </div>
  {:else}
    <div class="border-b border-zinc-800 bg-zinc-900/60 p-3 flex items-center justify-between">
      <div class="flex items-center space-x-3">
        <span class="text-sm font-mono font-bold text-zinc-100">{exchange.request.method}</span>
        <span class="text-xs font-mono text-zinc-400 truncate max-w-lg">{exchange.request.uri}</span>
      </div>
      <div class="flex space-x-1 bg-zinc-800/80 p-1 rounded-lg text-xs">
        <button
          class="px-3 py-1 rounded transition-colors {activeTab === 'request' ? 'bg-zinc-700 text-white font-medium shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
          onclick={() => (activeTab = "request")}
        >
          Request
        </button>
        <button
          class="px-3 py-1 rounded transition-colors {activeTab === 'response' ? 'bg-zinc-700 text-white font-medium shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
          onclick={() => (activeTab = "response")}
        >
          Response
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto p-4 space-y-6">
      {#if activeTab === "request"}
        <div>
          <h3 class="text-xs font-semibold uppercase tracking-wider text-zinc-400 mb-2">Headers</h3>
          <div class="bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
            <table class="w-full text-left text-xs font-mono">
              <thead class="bg-zinc-800/40 text-zinc-400 border-b border-zinc-800">
                <tr>
                  <th class="p-2 font-medium">Header</th>
                  <th class="p-2 font-medium">Valor</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-zinc-800/50">
                {#each exchange.request.headers as h}
                  <tr class="hover:bg-zinc-800/20">
                    <td class="p-2 text-zinc-400 w-1/3">{h.key}</td>
                    <td class="p-2 text-zinc-200 break-all">{h.value}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>

        {#if exchange.request.body}
          <div>
            <h3 class="text-xs font-semibold uppercase tracking-wider text-zinc-400 mb-2">Payload Body</h3>
            <pre class="bg-zinc-900 border border-zinc-800 p-3 rounded-lg text-xs font-mono text-zinc-300 overflow-x-auto selection:bg-indigo-500/30">{exchange.request.body}</pre>
          </div>
        {/if}
      {:else}
        {#if exchange.response}
          <div class="flex items-center space-x-4 text-xs font-mono bg-zinc-900 border border-zinc-800 p-3 rounded-lg">
            <div>
              <span class="text-zinc-500">Status:</span>
              <span class="font-bold text-emerald-400 ml-1">{exchange.response.statusCode}</span>
            </div>
            <div>
              <span class="text-zinc-500">Latência:</span>
              <span class="text-zinc-200 ml-1">{exchange.response.durationMs} ms</span>
            </div>
            <div>
              <span class="text-zinc-500">Tamanho:</span>
              <span class="text-zinc-200 ml-1">{exchange.response.sizeBytes} bytes</span>
            </div>
          </div>

          <div>
            <h3 class="text-xs font-semibold uppercase tracking-wider text-zinc-400 mb-2">Headers</h3>
            <div class="bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
              <table class="w-full text-left text-xs font-mono">
                <thead class="bg-zinc-800/40 text-zinc-400 border-b border-zinc-800">
                  <tr>
                    <th class="p-2 font-medium">Header</th>
                    <th class="p-2 font-medium">Valor</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-zinc-800/50">
                  {#each exchange.response.headers as h}
                    <tr class="hover:bg-zinc-800/20">
                      <td class="p-2 text-zinc-400 w-1/3">{h.key}</td>
                      <td class="p-2 text-zinc-200 break-all">{h.value}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>

          {#if exchange.response.body}
            <div>
              <h3 class="text-xs font-semibold uppercase tracking-wider text-zinc-400 mb-2">Response Body</h3>
              <pre class="bg-zinc-900 border border-zinc-800 p-3 rounded-lg text-xs font-mono text-zinc-300 overflow-x-auto selection:bg-indigo-500/30">{exchange.response.body}</pre>
            </div>
          {/if}
        {:else}
          <div class="text-xs text-zinc-500 p-4">
            Aguardando resposta do servidor ou requisição pendente...
          </div>
        {/if}
      {/if}
    </div>
  {/if}
</div>
