<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import ReplayModal from "$lib/components/ReplayModal.svelte";
  import { IconCopy, IconCheck, IconPlay, IconActivity, IconCode, IconSplit, IconPlus } from "$lib/components/icons";

  let {
    onOpenNewRequest = () => {},
    onToggleProxy = () => {}
  }: {
    onOpenNewRequest?: () => void;
    onToggleProxy?: () => void;
  } = $props();

  let activeTab = $state<"request" | "response" | "diff" | "curl">("request");
  let exchange = $derived(relayState.selectedExchange);
  let compareTarget = $derived(relayState.diffCompareExchange);
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

  function generateCurl(ex: NonNullable<typeof exchange>): string {
    const method = ex.request.method;
    const url = `http://${relayState.config.targetHost}:${relayState.config.targetPort}${ex.request.uri}`;
    let curl = `curl -i -X ${method} "${url}"`;

    for (const h of ex.request.headers) {
      if (!h.key.toLowerCase().startsWith("content-length")) {
        curl += ` \\\n  -H "${h.key}: ${h.value}"`;
      }
    }

    if (ex.request.body && ex.request.body.trim()) {
      const escapedBody = ex.request.body.replace(/"/g, '\\"');
      curl += ` \\\n  -d "${escapedBody}"`;
    }

    return curl;
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
    <!-- Minimalist Clean Inspector Empty State -->
    <div class="flex-1 flex flex-col items-center justify-center p-8 select-none text-center">
      {#if !relayState.isProxyRunning}
        <!-- Proxy Desconectado State -->
        <div class="max-w-sm bg-zinc-900/40 border border-zinc-800/60 rounded-2xl p-6 space-y-3 shadow-xl flex flex-col items-center">
          <div class="w-10 h-10 rounded-full bg-zinc-800/80 border border-zinc-700 text-zinc-400 flex items-center justify-center">
            <IconActivity size={18} class="stroke-1" />
          </div>
          <div class="space-y-1">
            <h3 class="text-xs font-bold text-zinc-200 uppercase tracking-wider">Proxy Desconectado</h3>
            <p class="text-[11px] text-zinc-400 leading-relaxed">
              Clique em <span class="text-indigo-400 font-semibold">Iniciar Proxy</span> ou pressione <kbd class="px-1.5 py-0.5 rounded bg-zinc-800 border border-zinc-700 font-mono text-[10px] text-zinc-300">Ctrl+P</kbd> para começar a escutar.
            </p>
          </div>
        </div>
      {:else}
        <!-- Proxy Ativo State - Minimalista e Direto -->
        <div class="max-w-sm bg-zinc-900/40 border border-zinc-800/60 rounded-2xl p-6 space-y-4 shadow-xl flex flex-col items-center">
          <div class="flex items-center space-x-2 text-xs font-bold font-mono text-emerald-400 bg-emerald-500/10 px-3 py-1 rounded-full border border-emerald-500/20">
            <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
            <span>Escutando na porta {relayState.config.listenPort}</span>
          </div>

          <div class="space-y-1">
            <p class="text-xs text-zinc-300 font-medium">Aguardando tráfego HTTP...</p>
            <p class="text-[11px] text-zinc-500 leading-relaxed">
              Envie requisições através do seu frontend/app ou crie uma chamada manual.
            </p>
          </div>

          <button
            onclick={onOpenNewRequest}
            class="text-xs px-3.5 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-all shadow-xs flex items-center space-x-1.5 cursor-pointer"
          >
            <IconPlus size={12} />
            <span>Nova Requisição</span>
          </button>
        </div>
      {/if}
    </div>
  {:else}
    <!-- Clean Inspector Header Bar -->
    <div class="h-12 border-b border-zinc-800/80 bg-zinc-900/40 px-4 flex items-center justify-between select-none">
      <div class="flex items-center space-x-2.5 overflow-hidden">
        <span class="text-[11px] font-mono font-bold text-zinc-100 px-2 py-0.5 rounded bg-zinc-800 border border-zinc-700/80">
          {exchange.request.method}
        </span>
        <span class="text-xs font-mono text-zinc-300 truncate max-w-sm sm:max-w-md" title={exchange.request.uri}>
          {exchange.request.uri}
        </span>
      </div>

      <div class="flex items-center space-x-3">
        <!-- Replay Action -->
        <button
          onclick={() => (isReplayOpen = true)}
          class="text-xs px-2.5 py-1 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium flex items-center space-x-1.5 transition-colors cursor-pointer shadow-xs"
          title="Editar parâmetros e reenviar chamada"
        >
          <IconPlay size={11} class="fill-current" />
          <span>Replay</span>
        </button>

        <!-- Segmented Tabs -->
        <div class="flex space-x-1 bg-zinc-900 p-0.5 rounded-md border border-zinc-800 text-xs">
          <button
            class="px-2.5 py-0.5 rounded transition-all {activeTab === 'request' ? 'bg-zinc-800 text-white font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
            onclick={() => (activeTab = "request")}
          >
            Request
          </button>
          <button
            class="px-2.5 py-0.5 rounded transition-all {activeTab === 'response' ? 'bg-zinc-800 text-white font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
            onclick={() => (activeTab = "response")}
          >
            Response
          </button>
          <button
            class="px-2 py-0.5 rounded transition-all flex items-center space-x-1 {activeTab === 'diff' ? 'bg-zinc-800 text-amber-300 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
            onclick={() => (activeTab = "diff")}
            title="Comparar resposta desta chamada com outra do histórico"
          >
            <IconSplit size={12} />
            <span>Diff</span>
          </button>
          <button
            class="px-2 py-0.5 rounded transition-all flex items-center space-x-1 {activeTab === 'curl' ? 'bg-zinc-800 text-sky-300 font-medium shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
            onclick={() => (activeTab = "curl")}
            title="Comando cURL pronto para terminal"
          >
            <IconCode size={12} />
            <span>cURL</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Inspector Content Area -->
    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      {#if activeTab === "request"}
        <!-- Request Tab Content -->
        <div class="space-y-4">
          <!-- Request Headers -->
          <div class="space-y-2">
            <div class="flex items-center justify-between select-none">
              <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
                Headers ({exchange.request.headers.length})
              </span>
              {#if exchange.request.headers.length > 0}
                <button
                  onclick={() => copyToClipboard(JSON.stringify(exchange?.request.headers, null, 2), "req_headers")}
                  class="text-[11px] text-zinc-500 hover:text-zinc-300 transition-colors flex items-center space-x-1 cursor-pointer"
                >
                  {#if copyFeedback === "req_headers"}
                    <IconCheck size={12} class="text-emerald-400" />
                    <span class="text-emerald-400">Copiado</span>
                  {:else}
                    <IconCopy size={12} />
                    <span>Copiar</span>
                  {/if}
                </button>
              {/if}
            </div>

            <div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-900/30">
              <table class="w-full text-left text-xs font-mono">
                <thead class="bg-zinc-900/80 border-b border-zinc-800/80 text-zinc-500 text-[10px] uppercase tracking-wider select-none">
                  <tr>
                    <th class="py-1.5 px-3 w-1/3 font-medium">Header</th>
                    <th class="py-1.5 px-3 font-medium">Valor</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-zinc-800/40">
                  {#each exchange.request.headers as header}
                    <tr class="hover:bg-zinc-900/50 transition-colors">
                      <td class="py-1.5 px-3 font-medium text-indigo-300 break-all">{header.key}</td>
                      <td class="py-1.5 px-3 text-zinc-300 break-all select-all">{header.value}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>

          <!-- Request Body -->
          <div class="space-y-2">
            <div class="flex items-center justify-between select-none">
              <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
                Payload Body ({exchange.request.sizeBytes} bytes)
              </span>
              {#if exchange.request.body}
                <button
                  onclick={() => copyToClipboard(exchange?.request.body || "", "req_body")}
                  class="text-[11px] text-zinc-500 hover:text-zinc-300 transition-colors flex items-center space-x-1 cursor-pointer"
                >
                  {#if copyFeedback === "req_body"}
                    <IconCheck size={12} class="text-emerald-400" />
                    <span class="text-emerald-400">Copiado</span>
                  {:else}
                    <IconCopy size={12} />
                    <span>Copiar</span>
                  {/if}
                </button>
              {/if}
            </div>

            {#if exchange.request.body}
              {@const { formatted, isJson } = formatBody(exchange.request.body)}
              <div class="relative group">
                <pre class="bg-zinc-900/60 border border-zinc-800 rounded-lg p-3 text-xs font-mono text-zinc-200 overflow-x-auto leading-relaxed max-h-96">{formatted}</pre>
                {#if isJson}
                  <span class="absolute top-2 right-2 text-[10px] font-mono px-1.5 py-0.2 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 select-none">
                    JSON
                  </span>
                {/if}
              </div>
            {:else}
              <div class="p-3 bg-zinc-900/20 border border-zinc-800/60 rounded-lg text-xs text-zinc-500 italic select-none">
                Sem corpo de requisição (Vazio).
              </div>
            {/if}
          </div>
        </div>
      {:else if activeTab === "response"}
        <!-- Response Tab Content -->
        {#if exchange.response}
          <div class="space-y-4">
            <!-- Meta Bar -->
            <div class="flex items-center space-x-4 text-xs font-mono bg-zinc-900/40 p-2.5 rounded-lg border border-zinc-800/80 select-none">
              <div>
                <span class="text-zinc-500">Status:</span>
                <span class="font-bold ml-1 {exchange.response.statusCode < 400 ? 'text-emerald-400' : 'text-rose-400'}">
                  {exchange.response.statusCode}
                </span>
              </div>
              <div>
                <span class="text-zinc-500">Latência:</span>
                <span class="text-zinc-300 ml-1">{exchange.response.durationMs} ms</span>
              </div>
              <div>
                <span class="text-zinc-500">Tamanho:</span>
                <span class="text-zinc-300 ml-1">{exchange.response.sizeBytes} bytes</span>
              </div>
            </div>

            <!-- Response Headers -->
            <div class="space-y-2">
              <div class="flex items-center justify-between select-none">
                <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
                  Headers de Resposta ({exchange.response.headers.length})
                </span>
                {#if exchange.response.headers.length > 0}
                  <button
                    onclick={() => copyToClipboard(JSON.stringify(exchange?.response?.headers, null, 2), "res_headers")}
                    class="text-[11px] text-zinc-500 hover:text-zinc-300 transition-colors flex items-center space-x-1 cursor-pointer"
                  >
                    {#if copyFeedback === "res_headers"}
                      <IconCheck size={12} class="text-emerald-400" />
                      <span class="text-emerald-400">Copiado</span>
                    {:else}
                      <IconCopy size={12} />
                      <span>Copiar</span>
                    {/if}
                  </button>
                {/if}
              </div>

              <div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-900/30">
                <table class="w-full text-left text-xs font-mono">
                  <thead class="bg-zinc-900/80 border-b border-zinc-800/80 text-zinc-500 text-[10px] uppercase tracking-wider select-none">
                    <tr>
                      <th class="py-1.5 px-3 w-1/3 font-medium">Header</th>
                      <th class="py-1.5 px-3 font-medium">Valor</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-zinc-800/40">
                    {#each exchange.response.headers as header}
                      <tr class="hover:bg-zinc-900/50 transition-colors">
                        <td class="py-1.5 px-3 font-medium text-emerald-400 break-all">{header.key}</td>
                        <td class="py-1.5 px-3 text-zinc-300 break-all select-all">{header.value}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>

            <!-- Response Body -->
            <div class="space-y-2">
              <div class="flex items-center justify-between select-none">
                <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
                  Response Body ({exchange.response.sizeBytes} bytes)
                </span>
                {#if exchange.response.body}
                  <button
                    onclick={() => copyToClipboard(exchange?.response?.body || "", "res_body")}
                    class="text-[11px] text-zinc-500 hover:text-zinc-300 transition-colors flex items-center space-x-1 cursor-pointer"
                  >
                    {#if copyFeedback === "res_body"}
                      <IconCheck size={12} class="text-emerald-400" />
                      <span class="text-emerald-400">Copiado</span>
                    {:else}
                      <IconCopy size={12} />
                      <span>Copiar</span>
                    {/if}
                  </button>
                {/if}
              </div>

              {#if exchange.response.body}
                {@const { formatted, isJson } = formatBody(exchange.response.body)}
                <div class="relative group">
                  <pre class="bg-zinc-900/60 border border-zinc-800 rounded-lg p-3 text-xs font-mono text-zinc-200 overflow-x-auto leading-relaxed max-h-96">{formatted}</pre>
                  {#if isJson}
                    <span class="absolute top-2 right-2 text-[10px] font-mono px-1.5 py-0.2 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 select-none">
                      JSON
                    </span>
                  {/if}
                </div>
              {:else}
                <div class="p-3 bg-zinc-900/20 border border-zinc-800/60 rounded-lg text-xs text-zinc-500 italic select-none">
                  Sem corpo de resposta.
                </div>
              {/if}
            </div>
          </div>
        {:else if exchange.status === "failed"}
          <div class="p-4 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-300 space-y-1">
            <div class="font-semibold text-xs uppercase tracking-wider">Falha na Conexão / Erro</div>
            <div class="text-xs font-mono text-rose-200">{exchange.error || "Erro de Gateway / Conexão recusada"}</div>
          </div>
        {:else}
          <div class="p-4 rounded-lg bg-zinc-900/20 border border-zinc-800 text-zinc-500 text-xs select-none">
            Aguardando resposta do servidor upstream...
          </div>
        {/if}
      {:else if activeTab === "diff"}
        <!-- Visual Diff Tab Content -->
        <div class="space-y-4">
          <div class="flex items-center justify-between bg-zinc-900/60 p-2.5 rounded-lg border border-zinc-800 select-none">
            <span class="text-xs text-zinc-300 font-medium">Comparar resposta com outra requisição:</span>
            <select
              class="bg-zinc-900 border border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500 cursor-pointer max-w-xs"
              onchange={(e) => {
                const targetId = (e.target as HTMLSelectElement).value;
                relayState.diffCompareExchange = relayState.exchanges.find(ex => ex.id === targetId) || null;
              }}
            >
              <option value="">Selecione para comparar...</option>
              {#each relayState.exchanges.filter(ex => ex.id !== exchange?.id) as ex}
                <option value={ex.id}>{ex.request.method} {ex.request.uri} ({ex.response?.statusCode ?? 'pending'})</option>
              {/each}
            </select>
          </div>

          <div class="grid grid-cols-2 gap-3 font-mono text-xs">
            <!-- Requisição Atual -->
            <div class="space-y-2">
              <div class="flex items-center justify-between text-zinc-400 border-b border-zinc-800 pb-1">
                <span class="font-bold text-indigo-400">Atual ({exchange.request.method} {exchange.request.uri})</span>
                <span>{exchange.response?.statusCode ?? 'N/A'}</span>
              </div>
              <pre class="bg-zinc-900/40 border border-zinc-800 rounded p-3 text-[11px] leading-relaxed max-h-[500px] overflow-y-auto">{formatBody(exchange.response?.body).formatted || '(Sem resposta)'}</pre>
            </div>

            <!-- Requisição Alvo de Comparação -->
            <div class="space-y-2">
              <div class="flex items-center justify-between text-zinc-400 border-b border-zinc-800 pb-1">
                <span class="font-bold text-amber-400">Comparação {compareTarget ? `(${compareTarget.request.method} ${compareTarget.request.uri})` : ''}</span>
                <span>{compareTarget?.response?.statusCode ?? 'N/A'}</span>
              </div>
              <pre class="bg-zinc-900/40 border border-zinc-800 rounded p-3 text-[11px] leading-relaxed max-h-[500px] overflow-y-auto">{compareTarget ? formatBody(compareTarget.response?.body).formatted || '(Sem resposta)' : 'Selecione uma requisição no topo para ver o diff lado a lado.'}</pre>
            </div>
          </div>
        </div>
      {:else if activeTab === "curl"}
        <!-- cURL Export Tab Content -->
        <div class="space-y-3">
          <div class="flex items-center justify-between select-none">
            <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
              Comando cURL Pronto para Execução
            </span>
            <button
              onclick={() => copyToClipboard(generateCurl(exchange!), "curl_cmd")}
              class="text-xs px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition-colors flex items-center space-x-1.5 cursor-pointer shadow-xs"
            >
              {#if copyFeedback === "curl_cmd"}
                <IconCheck size={12} class="text-emerald-400" />
                <span class="text-emerald-400">Copiado</span>
              {:else}
                <IconCopy size={12} />
                <span>Copiar cURL</span>
              {/if}
            </button>
          </div>

          <pre class="bg-zinc-900/80 border border-zinc-800 rounded-lg p-3 text-xs font-mono text-sky-300 overflow-x-auto leading-relaxed whitespace-pre-wrap select-all">{generateCurl(exchange)}</pre>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Replay Modal condicional -->
  {#if isReplayOpen}
    <ReplayModal bind:isOpen={isReplayOpen} {exchange} />
  {/if}
</div>
