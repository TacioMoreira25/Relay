<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import ReplayModal from "$lib/components/ReplayModal.svelte";

  let activeTab = $state<"request" | "response">("request");
  let exchange = $derived(relayState.selectedExchange);
  let copyFeedback = $state<string | null>(null);
  let isReplayOpen = $state<boolean>(false);

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
        <span class="text-xs font-mono text-zinc-300 truncate max-w-sm sm:max-w-md" title={exchange.request.uri}>
          {exchange.request.uri}
        </span>
      </div>

      <div class="flex items-center space-x-3">
        <!-- Replay Trigger Button -->
        <button
          onclick={() => (isReplayOpen = true)}
          class="text-xs px-3 py-1.5 rounded bg-indigo-600/90 hover:bg-indigo-600 text-white font-medium flex items-center space-x-1.5 transition-all shadow-sm cursor-pointer"
          title="Abrir editor de parâmetros e reenviar chamada"
        >
          <span>🔁</span>
          <span>Replay Chamada</span>
        </button>

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
    <div class="flex-1 overflow-y-auto p-4 space-y-5">
      {#if activeTab === "request"}
        <!-- Request Details Tab -->
        <div class="space-y-4">
          <!-- Request Headers Table -->
          <div class="space-y-2">
            <div class="flex items-center justify-between select-none">
              <span class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
                Headers ({exchange.request.headers.length})
              </span>
              {#if exchange.request.headers.length > 0}
                <button
                  onclick={() => copyToClipboard(JSON.stringify(exchange?.request.headers, null, 2), "req_headers")}
                  class="text-[11px] text-zinc-400 hover:text-zinc-200 transition-colors"
                >
                  {copyFeedback === "req_headers" ? "✓ Copiado!" : "Copiar Headers"}
                </button>
              {/if}
            </div>

            <div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-900/40">
              <table class="w-full text-left text-xs font-mono">
                <thead class="bg-zinc-900 border-b border-zinc-800/80 text-zinc-400 select-none">
                  <tr>
                    <th class="py-2 px-3 w-1/3 font-medium">Header</th>
                    <th class="py-2 px-3 font-medium">Valor</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-zinc-800/40">
                  {#each exchange.request.headers as header}
                    <tr class="hover:bg-zinc-800/30 transition-colors">
                      <td class="py-1.5 px-3 font-semibold text-indigo-300 break-all">{header.key}</td>
                      <td class="py-1.5 px-3 text-zinc-300 break-all select-all">{header.value}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>

          <!-- Request Payload Body -->
          <div class="space-y-2">
            <div class="flex items-center justify-between select-none">
              <span class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
                Payload Body ({exchange.request.sizeBytes} bytes)
              </span>
              {#if exchange.request.body}
                <button
                  onclick={() => copyToClipboard(exchange?.request.body || "", "req_body")}
                  class="text-[11px] text-zinc-400 hover:text-zinc-200 transition-colors"
                >
                  {copyFeedback === "req_body" ? "✓ Copiado!" : "Copiar Body"}
                </button>
              {/if}
            </div>

            {#if exchange.request.body}
              {@const { formatted, isJson } = formatBody(exchange.request.body)}
              <div class="relative group">
                <pre class="bg-zinc-900/80 border border-zinc-800 rounded-lg p-4 text-xs font-mono text-zinc-200 overflow-x-auto leading-relaxed max-h-96">{formatted}</pre>
                {#if isJson}
                  <span class="absolute top-2 right-2 text-[10px] font-mono px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 select-none">
                    JSON Formatted
                  </span>
                {/if}
              </div>
            {:else}
              <div class="p-3 bg-zinc-900/40 border border-zinc-800/60 rounded-lg text-xs text-zinc-500 italic select-none">
                Sem corpo de requisição (Vazio).
              </div>
            {/if}
          </div>
        </div>
      {:else}
        <!-- Response Details Tab -->
        {#if exchange.response}
          <div class="space-y-4">
            <!-- Response Meta (Status & Latency) -->
            <div class="flex items-center space-x-4 text-xs font-mono bg-zinc-900/60 p-2.5 rounded-lg border border-zinc-800/80 select-none">
              <div>
                <span class="text-zinc-500">Status:</span>
                <span class="font-bold ml-1 {exchange.response.statusCode < 400 ? 'text-emerald-400' : 'text-rose-400'}">
                  {exchange.response.statusCode}
                </span>
              </div>
              <div>
                <span class="text-zinc-500">Latência:</span>
                <span class="text-zinc-300 ml-1 font-semibold">{exchange.response.durationMs} ms</span>
              </div>
              <div>
                <span class="text-zinc-500">Tamanho:</span>
                <span class="text-zinc-300 ml-1">{exchange.response.sizeBytes} bytes</span>
              </div>
            </div>

            <!-- Response Headers Table -->
            <div class="space-y-2">
              <div class="flex items-center justify-between select-none">
                <span class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
                  Headers de Resposta ({exchange.response.headers.length})
                </span>
                {#if exchange.response.headers.length > 0}
                  <button
                    onclick={() => copyToClipboard(JSON.stringify(exchange?.response?.headers, null, 2), "res_headers")}
                    class="text-[11px] text-zinc-400 hover:text-zinc-200 transition-colors"
                  >
                    {copyFeedback === "res_headers" ? "✓ Copiado!" : "Copiar Headers"}
                  </button>
                {/if}
              </div>

              <div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-900/40">
                <table class="w-full text-left text-xs font-mono">
                  <thead class="bg-zinc-900 border-b border-zinc-800/80 text-zinc-400 select-none">
                    <tr>
                      <th class="py-2 px-3 w-1/3 font-medium">Header</th>
                      <th class="py-2 px-3 font-medium">Valor</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-zinc-800/40">
                    {#each exchange.response.headers as header}
                      <tr class="hover:bg-zinc-800/30 transition-colors">
                        <td class="py-1.5 px-3 font-semibold text-emerald-400 break-all">{header.key}</td>
                        <td class="py-1.5 px-3 text-zinc-300 break-all select-all">{header.value}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>

            <!-- Response Body Payload -->
            <div class="space-y-2">
              <div class="flex items-center justify-between select-none">
                <span class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
                  Response Body ({exchange.response.sizeBytes} bytes)
                </span>
                {#if exchange.response.body}
                  <button
                    onclick={() => copyToClipboard(exchange?.response?.body || "", "res_body")}
                    class="text-[11px] text-zinc-400 hover:text-zinc-200 transition-colors"
                  >
                    {copyFeedback === "res_body" ? "✓ Copiado!" : "Copiar Body"}
                  </button>
                {/if}
              </div>

              {#if exchange.response.body}
                {@const { formatted, isJson } = formatBody(exchange.response.body)}
                <div class="relative group">
                  <pre class="bg-zinc-900/80 border border-zinc-800 rounded-lg p-4 text-xs font-mono text-zinc-200 overflow-x-auto leading-relaxed max-h-96">{formatted}</pre>
                  {#if isJson}
                    <span class="absolute top-2 right-2 text-[10px] font-mono px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 select-none">
                      JSON Formatted
                    </span>
                  {/if}
                </div>
              {:else}
                <div class="p-3 bg-zinc-900/40 border border-zinc-800/60 rounded-lg text-xs text-zinc-500 italic select-none">
                  Sem corpo de resposta.
                </div>
              {/if}
            </div>
          </div>
        {:else if exchange.status === "failed"}
          <div class="p-4 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-300 space-y-1">
            <div class="font-semibold text-xs uppercase tracking-wider">Falha na Conexão / Resposta</div>
            <div class="text-xs font-mono text-rose-200">{exchange.error || "Erro de Gateway / Conexão recusada"}</div>
          </div>
        {:else}
          <div class="p-4 rounded-lg bg-zinc-900/40 border border-zinc-800 text-zinc-400 text-xs flex items-center space-x-2 select-none">
            <span class="animate-spin text-indigo-400">⏳</span>
            <span>Aguardando resposta do servidor upstream...</span>
          </div>
        {/if}
      {/if}
    </div>
  {/if}

  <!-- Replay Modal Instance -->
  <ReplayModal bind:isOpen={isReplayOpen} {exchange} />
</div>
