<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";

  let activeTab = $state<"request" | "response">("request");
  let exchange = $derived(relayState.selectedExchange);
  let copyFeedback = $state<string | null>(null);

  function formatBody(bodyStr?: string): { formatted: string; isJson: boolean } {
    if (!bodyStr || !bodyStr.trim()) {
      return { formatted: "", isJson: false };
    }
    try {
      const parsed = JSON.parse(bodyStr);
      return { formatted: JSON.stringify(parsed, null, 2), isJson: true };
    } catch {
      return { formatted: bodyStr, isJson: false };
    }
  }

  async function copyToClipboard(text: string, label: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(text);
      copyFeedback = label;
      setTimeout(() => {
        if (copyFeedback === label) copyFeedback = null;
      }, 2000);
    } catch (err) {
      console.error("Falha ao copiar:", err);
    }
  }
</script>

<div class="flex flex-col h-full bg-zinc-950 text-zinc-200 overflow-hidden select-text">
  {#if !exchange}
    <div class="flex-1 flex flex-col items-center justify-center text-zinc-600 text-sm space-y-2 select-none">
      <div class="text-3xl">🔍</div>
      <div>Selecione uma requisição na lista lateral para inspecionar headers e payload.</div>
    </div>
  {:else}
    <!-- Inspector Header Bar -->
    <div class="border-b border-zinc-800 bg-zinc-900/70 p-3 flex items-center justify-between select-none">
      <div class="flex items-center space-x-3 overflow-hidden">
        <span class="text-sm font-mono font-bold text-white px-2 py-0.5 rounded bg-zinc-800 border border-zinc-700">
          {exchange.request.method}
        </span>
        <span class="text-xs font-mono text-zinc-300 truncate max-w-lg" title={exchange.request.uri}>
          {exchange.request.uri}
        </span>
      </div>

      <div class="flex items-center space-x-2">
        <div class="flex space-x-1 bg-zinc-800/90 p-1 rounded-lg text-xs">
          <button
            class="px-3 py-1 rounded transition-colors {activeTab === 'request' ? 'bg-indigo-600 text-white font-medium shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
            onclick={() => (activeTab = "request")}
          >
            Request
          </button>
          <button
            class="px-3 py-1 rounded transition-colors {activeTab === 'response' ? 'bg-indigo-600 text-white font-medium shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
            onclick={() => (activeTab = "response")}
          >
            Response
          </button>
        </div>
      </div>
    </div>

    <!-- Inspector Content Area -->
    <div class="flex-1 overflow-y-auto p-4 space-y-6">
      {#if activeTab === "request"}
        <!-- Request Headers -->
        <div>
          <div class="flex items-center justify-between mb-2 select-none">
            <h3 class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
              Headers ({exchange.request.headers.length})
            </h3>
            <button
              onclick={() => copyToClipboard(JSON.stringify(exchange?.request.headers, null, 2), "req_headers")}
              class="text-[11px] text-zinc-400 hover:text-zinc-200 transition-colors"
            >
              {copyFeedback === "req_headers" ? "✓ Copiado!" : "Copiar Headers"}
            </button>
          </div>

          {#if exchange.request.headers.length === 0}
            <div class="text-xs text-zinc-500 italic p-3 bg-zinc-900 border border-zinc-800 rounded-lg">
              Nenhum header enviado.
            </div>
          {:else}
            <div class="bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
              <table class="w-full text-left text-xs font-mono">
                <thead class="bg-zinc-800/40 text-zinc-400 border-b border-zinc-800">
                  <tr>
                    <th class="p-2.5 font-medium w-1/3">Header</th>
                    <th class="p-2.5 font-medium">Valor</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-zinc-800/50">
                  {#each exchange.request.headers as h}
                    <tr class="hover:bg-zinc-800/30">
                      <td class="p-2.5 text-indigo-400 font-semibold">{h.key}</td>
                      <td class="p-2.5 text-zinc-200 break-all">{h.value}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>

        <!-- Request Body -->
        <div>
          <div class="flex items-center justify-between mb-2 select-none">
            <h3 class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
              Payload Body ({exchange.request.sizeBytes} bytes)
            </h3>
            {#if exchange.request.body}
              <button
                onclick={() => copyToClipboard(exchange?.request.body ?? "", "req_body")}
                class="text-[11px] text-zinc-400 hover:text-zinc-200 transition-colors"
              >
                {copyFeedback === "req_body" ? "✓ Copiado!" : "Copiar Body"}
              </button>
            {/if}
          </div>

          {#if exchange.request.body}
            {@const bodyInfo = formatBody(exchange.request.body)}
            <div class="relative">
              <pre class="bg-zinc-900 border border-zinc-800 p-3 rounded-lg text-xs font-mono text-zinc-200 overflow-x-auto selection:bg-indigo-500/40 leading-relaxed">{bodyInfo.formatted}</pre>
            </div>
          {:else}
            <div class="text-xs text-zinc-500 italic p-3 bg-zinc-900 border border-zinc-800 rounded-lg">
              Sem corpo de requisição (Vazio).
            </div>
          {/if}
        </div>
      {:else}
        <!-- Response Content -->
        {#if exchange.response}
          <!-- Response Overview Bar -->
          <div class="flex items-center space-x-6 text-xs font-mono bg-zinc-900 border border-zinc-800 p-3 rounded-lg select-none">
            <div>
              <span class="text-zinc-500">Status:</span>
              <span class="font-bold {exchange.response.statusCode < 400 ? 'text-emerald-400' : 'text-rose-400'} ml-1.5">
                {exchange.response.statusCode}
              </span>
            </div>
            <div>
              <span class="text-zinc-500">Latência:</span>
              <span class="text-zinc-200 ml-1.5 font-semibold">{exchange.response.durationMs} ms</span>
            </div>
            <div>
              <span class="text-zinc-500">Tamanho:</span>
              <span class="text-zinc-200 ml-1.5">{exchange.response.sizeBytes} bytes</span>
            </div>
          </div>

          <!-- Response Headers -->
          <div>
            <div class="flex items-center justify-between mb-2 select-none">
              <h3 class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
                Headers de Resposta ({exchange.response.headers.length})
              </h3>
              <button
                onclick={() => copyToClipboard(JSON.stringify(exchange?.response?.headers, null, 2), "res_headers")}
                class="text-[11px] text-zinc-400 hover:text-zinc-200 transition-colors"
              >
                {copyFeedback === "res_headers" ? "✓ Copiado!" : "Copiar Headers"}
              </button>
            </div>

            <div class="bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
              <table class="w-full text-left text-xs font-mono">
                <thead class="bg-zinc-800/40 text-zinc-400 border-b border-zinc-800">
                  <tr>
                    <th class="p-2.5 font-medium w-1/3">Header</th>
                    <th class="p-2.5 font-medium">Valor</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-zinc-800/50">
                  {#each exchange.response.headers as h}
                    <tr class="hover:bg-zinc-800/30">
                      <td class="p-2.5 text-emerald-400 font-semibold">{h.key}</td>
                      <td class="p-2.5 text-zinc-200 break-all">{h.value}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>

          <!-- Response Body -->
          <div>
            <div class="flex items-center justify-between mb-2 select-none">
              <h3 class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
                Response Body ({exchange.response.sizeBytes} bytes)
              </h3>
              {#if exchange.response.body}
                <button
                  onclick={() => copyToClipboard(exchange?.response?.body ?? "", "res_body")}
                  class="text-[11px] text-zinc-400 hover:text-zinc-200 transition-colors"
                >
                  {copyFeedback === "res_body" ? "✓ Copiado!" : "Copiar Body"}
                </button>
              {/if}
            </div>

            {#if exchange.response.body}
              {@const bodyInfo = formatBody(exchange.response.body)}
              <div class="relative">
                <pre class="bg-zinc-900 border border-zinc-800 p-3 rounded-lg text-xs font-mono text-zinc-200 overflow-x-auto selection:bg-indigo-500/40 leading-relaxed">{bodyInfo.formatted}</pre>
              </div>
            {:else}
              <div class="text-xs text-zinc-500 italic p-3 bg-zinc-900 border border-zinc-800 rounded-lg">
                Sem corpo de resposta (Vazio).
              </div>
            {/if}
          </div>
        {:else if exchange.status === "failed"}
          <div class="p-4 bg-rose-500/10 border border-rose-500/30 rounded-lg text-rose-300 text-xs space-y-1 font-mono">
            <div class="font-bold">❌ Falha na Conexão com o Upstream</div>
            <div class="text-zinc-400">{exchange.error || "O servidor de destino não respondeu ou rejeitou a conexão."}</div>
          </div>
        {:else}
          <div class="text-xs text-zinc-500 p-4 flex items-center space-x-2 animate-pulse font-mono">
            <span class="w-2 h-2 rounded-full bg-amber-400"></span>
            <span>Aguardando resposta do servidor upstream...</span>
          </div>
        {/if}
      {/if}
    </div>
  {/if}
</div>
