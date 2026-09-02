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
      if (!h.key.toLowerCase().startsWith("content-length") && !h.key.toLowerCase().startsWith("host")) {
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

  function getMethodColorClass(m: string): string {
    switch (m.toUpperCase()) {
      case "GET": return "text-sky-400 bg-sky-500/10 border-sky-500/20";
      case "POST": return "text-emerald-400 bg-emerald-500/10 border-emerald-500/20";
      case "PUT": return "text-amber-400 bg-amber-500/10 border-amber-500/20";
      case "DELETE": return "text-rose-400 bg-rose-500/10 border-rose-500/20";
      case "PATCH": return "text-purple-400 bg-purple-500/10 border-purple-500/20";
      default: return "text-zinc-300 bg-zinc-500/10 border-zinc-500/20";
    }
  }
</script>

<div class="h-full flex flex-col bg-zinc-950">
  {#if !exchange}
    <!-- Empty State Didático e Limpo -->
    <div class="flex-1 flex flex-col items-center justify-center p-8 text-center select-none">
      {#if !relayState.isProxyRunning}
        <!-- Proxy Inativo State -->
        <div class="max-w-sm bg-zinc-900/40 border border-zinc-800/60 rounded-2xl p-6 space-y-4 shadow-xl flex flex-col items-center">
          <div class="w-12 h-12 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
            <IconActivity size={24} />
          </div>

          <div class="space-y-1">
            <h3 class="text-sm font-bold text-zinc-100">Proxy Desconectado</h3>
            <p class="text-xs text-zinc-400 leading-relaxed">
              Inicie o proxy na porta <span class="text-indigo-300 font-mono font-semibold">:{relayState.config.listenPort}</span> para interceptar as requisições da sua API.
            </p>
          </div>

          <button
            onclick={onToggleProxy}
            class="text-xs px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-all shadow-md flex items-center space-x-1.5 cursor-pointer"
          >
            <IconPlay size={12} class="fill-current" />
            <span>Iniciar Proxy (:{relayState.config.listenPort})</span>
          </button>
        </div>
      {:else}
        <!-- Proxy Ativo State -->
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
    <div class="h-12 border-b border-zinc-800 bg-zinc-900/50 px-4 flex items-center justify-between shrink-0">
      <div class="flex items-center space-x-2.5 overflow-hidden">
        <span class="text-[11px] font-mono font-bold px-2 py-0.5 rounded border {getMethodColorClass(exchange.request.method)}">
          {exchange.request.method}
        </span>
        <span class="text-xs font-mono text-zinc-200 truncate max-w-sm sm:max-w-md font-medium" title={exchange.request.uri}>
          {exchange.request.uri}
        </span>
      </div>

      <div class="flex items-center space-x-3">
        <!-- Replay Action -->
        <button
          onclick={() => (isReplayOpen = true)}
          class="text-xs px-2.5 py-1 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium flex items-center space-x-1.5 transition-colors cursor-pointer shadow-xs"
          title="Editar parâmetros e reenviar chamada no disparador"
        >
          <IconPlay size={11} class="fill-current" />
          <span>Replay</span>
        </button>

        <!-- Segmented Tabs com Binding Direto na Store Global -->
        <div class="flex space-x-1 bg-zinc-950/80 p-1 rounded-lg border border-zinc-800/80 text-xs shadow-inner">
          <button
            type="button"
            class="px-3 py-1 rounded-md transition-all cursor-pointer font-medium border {relayState.inspectorTab === 'request' ? 'bg-zinc-800 text-zinc-100 shadow-sm border-zinc-700/50' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60'}"
            onclick={() => { relayState.inspectorTab = "request"; }}
          >
            Request
          </button>
          <button
            type="button"
            class="px-3 py-1 rounded-md transition-all cursor-pointer font-medium border {relayState.inspectorTab === 'response' ? 'bg-zinc-800 text-zinc-100 shadow-sm border-zinc-700/50' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60'}"
            onclick={() => { relayState.inspectorTab = "response"; }}
          >
            Response
          </button>
          <button
            type="button"
            class="px-3 py-1 rounded-md transition-all flex items-center space-x-1 cursor-pointer font-medium border {relayState.inspectorTab === 'diff' ? 'bg-zinc-800 text-zinc-100 shadow-sm border-zinc-700/50' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60'}"
            onclick={() => { relayState.inspectorTab = "diff"; }}
            title="Comparar resposta desta chamada com outra do histórico"
          >
            <IconSplit size={12} />
            <span>Diff</span>
          </button>
          <button
            type="button"
            class="px-3 py-1 rounded-md transition-all flex items-center space-x-1 cursor-pointer font-medium border {relayState.inspectorTab === 'curl' ? 'bg-zinc-800 text-zinc-100 shadow-sm border-zinc-700/50' : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/60'}"
            onclick={() => { relayState.inspectorTab = "curl"; }}
            title="Comando cURL pronto para terminal"
          >
            <IconCode size={12} />
            <span>cURL</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Inspector Content Area -->
    <div class="flex-1 min-w-0 overflow-y-auto overflow-x-hidden p-4 space-y-4">
      {#if relayState.inspectorTab === "request"}
        <!-- Request Tab Content -->
        <div class="space-y-4">
          <!-- Request Headers -->
          <div class="space-y-2">
            <div class="flex items-center justify-between">
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
                <thead class="bg-zinc-900/80 border-b border-zinc-800/80 text-zinc-500 text-[10px] uppercase tracking-wider">
                  <tr>
                    <th class="py-1.5 px-3 w-1/3 font-medium">Header</th>
                    <th class="py-1.5 px-3 font-medium">Valor</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-zinc-800/40">
                  {#each exchange.request.headers as header}
                      <tr class="hover:bg-zinc-900/50 transition-colors">
                        <td class="py-1.5 px-3 text-indigo-300 font-medium">{header.key}</td>
                        <td class="py-1.5 px-3 text-zinc-300 break-all select-text">
                          {header.value}
                          {#if header.key.toLowerCase() === 'date'}
                            <span class="text-[10px] text-zinc-500 ml-2 select-none" title="Horário Local">({new Date(header.value).toLocaleString()})</span>
                          {/if}
                        </td>
                      </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>

          <!-- Request Body -->
          {#if exchange.request.body}
            {@const parsed = formatBody(exchange.request.body)}
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
                  Payload Body ({exchange.request.sizeBytes} bytes)
                </span>
                <div class="flex items-center space-x-2">
                  {#if parsed.isJson}
                    <span class="text-[9px] px-1.5 py-0.2 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 font-mono">
                      JSON
                    </span>
                  {/if}
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
                </div>
              </div>

              <div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-900/30 p-3">
                <pre class="text-xs font-mono text-zinc-200 overflow-x-auto whitespace-pre-wrap leading-relaxed break-all w-full select-text">{parsed.formatted}</pre>
              </div>
            </div>
          {/if}
        </div>
      {:else if relayState.inspectorTab === "response"}
        <!-- Response Tab Content -->
        {#if exchange.response}
          {@const res = exchange.response}
          {@const parsedRes = formatBody(res.body)}
          <div class="space-y-4">
            <!-- Status & Meta Info -->
            <div class="flex items-center space-x-3 text-xs font-mono bg-zinc-900/40 p-2.5 rounded-lg border border-zinc-800/60">
              <div class="flex items-center space-x-1.5">
                <span class="text-zinc-500">Status:</span>
                <span class="font-bold px-1.5 py-0.2 rounded {res.statusCode >= 200 && res.statusCode < 300 ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-rose-500/10 text-rose-400 border border-rose-500/20'}">
                  {res.statusCode}
                </span>
              </div>

              <div class="flex items-center space-x-1.5">
                <span class="text-zinc-500">Latência:</span>
                <span class="text-zinc-300 font-medium">{res.durationMs} ms</span>
              </div>

              <div class="flex items-center space-x-1.5">
                <span class="text-zinc-500">Tamanho:</span>
                <span class="text-zinc-300 font-medium">{res.sizeBytes} bytes</span>
              </div>
            </div>

            <!-- Response Headers -->
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
                  Headers de Resposta ({res.headers.length})
                </span>
                <button
                  onclick={() => copyToClipboard(JSON.stringify(res.headers, null, 2), "res_headers")}
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
              </div>

              <div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-900/30">
                <table class="w-full text-left text-xs font-mono">
                  <thead class="bg-zinc-900/80 border-b border-zinc-800/80 text-zinc-500 text-[10px] uppercase tracking-wider">
                    <tr>
                      <th class="py-1.5 px-3 w-1/3 font-medium">Header</th>
                      <th class="py-1.5 px-3 font-medium">Valor</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-zinc-800/40">
                    {#each res.headers as header}
                      <tr class="hover:bg-zinc-900/50 transition-colors">
                        <td class="py-1.5 px-3 text-emerald-300 font-medium">{header.key}</td>
                        <td class="py-1.5 px-3 text-zinc-300 break-all select-text">
                          {header.value}
                          {#if header.key.toLowerCase() === 'date'}
                            <span class="text-[10px] text-zinc-500 ml-2 select-none" title="Horário Local">({new Date(header.value).toLocaleString()})</span>
                          {/if}
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>

            <!-- Response Body -->
            {#if res.body}
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
                    Response Body ({res.sizeBytes} bytes)
                  </span>
                  <div class="flex items-center space-x-2">
                    {#if parsedRes.isJson}
                      <span class="text-[9px] px-1.5 py-0.2 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 font-mono">
                        JSON
                      </span>
                    {/if}
                    <button
                      onclick={() => copyToClipboard(res.body || "", "res_body")}
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
                  </div>
                </div>

                <div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-900/30 p-3">
                  <pre class="text-xs font-mono text-zinc-200 overflow-x-auto whitespace-pre-wrap leading-relaxed break-all w-full select-text">{parsedRes.formatted}</pre>
                </div>
              </div>
            {/if}
          </div>
        {:else if exchange.status === "failed"}
          <div class="p-4 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-300 text-xs font-mono space-y-1">
            <div class="font-bold">Falha na Requisição:</div>
            <p>{exchange.error || "Erro de conexão com o servidor de destino."}</p>
          </div>
        {:else}
          <div class="p-8 text-center text-zinc-500 font-mono text-xs">
            Aguardando resposta do servidor...
          </div>
        {/if}
      {:else if relayState.inspectorTab === "diff"}
        <!-- Diff Tab Content -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
              Comparador de Respostas Lado a Lado (Diff)
            </span>
            <span class="text-[10px] text-zinc-500">
              Selecione o alvo de comparação no dropdown abaixo
            </span>
          </div>

          <!-- Seletor do Alvo de Comparação -->
          <div class="flex items-center space-x-2 bg-zinc-900 p-2 rounded-lg border border-zinc-800">
            <span class="text-xs text-zinc-400 font-medium">Comparar com:</span>
            <select
              value={compareTarget?.id || ""}
              onchange={(e) => {
                const targetId = (e.target as HTMLSelectElement).value;
                const found = relayState.exchanges.find(ex => ex.id === targetId);
                relayState.diffCompareExchange = found || null;
              }}
              class="flex-1 bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500 cursor-pointer"
            >
              <option value="">-- Selecione uma requisição do histórico --</option>
              {#each relayState.exchanges.filter(ex => ex.id !== exchange?.id) as ex}
                <option value={ex.id}>
                  {ex.request.method} {ex.request.uri} ({ex.response?.statusCode ?? 'pending'})
                </option>
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
              <pre class="bg-zinc-900/40 border border-zinc-800 rounded p-3 text-[11px] leading-relaxed max-h-[500px] overflow-y-auto select-text">{formatBody(exchange.response?.body).formatted || '(Sem resposta)'}</pre>
            </div>

            <!-- Requisição Alvo de Comparação -->
            <div class="space-y-2">
              <div class="flex items-center justify-between text-zinc-400 border-b border-zinc-800 pb-1">
                <span class="font-bold text-amber-400">Comparação {compareTarget ? `(${compareTarget.request.method} ${compareTarget.request.uri})` : ''}</span>
                <span>{compareTarget?.response?.statusCode ?? 'N/A'}</span>
              </div>
              <pre class="bg-zinc-900/40 border border-zinc-800 rounded p-3 text-[11px] leading-relaxed max-h-[500px] overflow-y-auto select-text">{compareTarget ? formatBody(compareTarget.response?.body).formatted || '(Sem resposta)' : 'Selecione uma requisição no topo para ver o diff lado a lado.'}</pre>
            </div>
          </div>
        </div>
      {:else if relayState.inspectorTab === "curl"}
        <!-- cURL Export Tab Content -->
        <div class="space-y-3 min-w-0 w-full">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
              Comando cURL Pronto para Execução
            </span>
            <div class="flex items-center space-x-2">
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
          </div>

          <pre class="bg-zinc-900/80 border border-zinc-800 rounded-lg p-3 text-xs font-mono text-sky-300 overflow-x-auto leading-relaxed whitespace-pre-wrap break-all select-text w-full">{generateCurl(exchange)}</pre>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Replay Modal condicional -->
  {#if isReplayOpen}
    <ReplayModal bind:isOpen={isReplayOpen} {exchange} />
  {/if}
</div>
