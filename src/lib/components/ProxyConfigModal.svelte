<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { RouteRule, ProxyConfig } from "$lib/types";
  import { IconDownload, IconFileJson } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";

  let { isOpen = $bindable(false) }: { isOpen: boolean } = $props();

  let tempListenPort = $state(relayState.config.listenPort);
  let tempTargetHost = $state(relayState.config.targetHost);
  let tempTargetPort = $state(relayState.config.targetPort);
  let tempLatencyMs = $state(relayState.config.latencyMs);
  let tempJitterMs = $state(relayState.config.jitterMs);
  let tempFailurePercent = $state(Math.round(relayState.config.simulateFailureRate * 100));
  let tempFailureStatusCode = $state(relayState.config.failureStatusCode);
  let tempAutoExtractJwt = $state(relayState.config.autoExtractJwt);
  let tempRoutes = $state<RouteRule[]>([]);

  let showExampleView = $state(false);
  let configFeedback = $state<string | null>(null);

  const EXAMPLE_JSON = `{
  "listenPort": 8080,
  "targetHost": "127.0.0.1",
  "targetPort": 3000,
  "latencyMs": 0,
  "jitterMs": 0,
  "simulateFailureRate": 0.0,
  "failureStatusCode": 500,
  "autoExtractJwt": true,
  "routes": [
    {
      "pathPrefix": "/api/v1/auth",
      "targetPort": 3000,
      "isMock": false
    },
    {
      "pathPrefix": "/api/v1/pagamentos",
      "targetPort": 3002,
      "isMock": false
    },
    {
      "pathPrefix": "/api/v1/mock-exemplo",
      "targetPort": 3000,
      "isMock": true,
      "mockStatusCode": 200,
      "mockBody": "{\\n  \\"status\\": \\"sucesso\\",\\n  \\"mock\\": true\\n}"
    }
  ]
}`;

  $effect(() => {
    if (isOpen) {
      showExampleView = false;
      tempListenPort = relayState.config.listenPort;
      tempTargetHost = relayState.config.targetHost;
      tempTargetPort = relayState.config.targetPort;
      tempLatencyMs = relayState.config.latencyMs;
      tempJitterMs = relayState.config.jitterMs;
      tempFailurePercent = Math.round(relayState.config.simulateFailureRate * 100);
      tempFailureStatusCode = relayState.config.failureStatusCode;
      tempAutoExtractJwt = relayState.config.autoExtractJwt;
      tempRoutes = relayState.config.routes ? relayState.config.routes.map(r => ({ ...r })) : [];
    }
  });

  function downloadExampleJson(): void {
    const blob = new Blob([EXAMPLE_JSON], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "relay.config.example.json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    
    configFeedback = "relay.config.example.json baixado!";
    setTimeout(() => (configFeedback = null), 2500);
  }

  function copyExampleJson(): void {
    navigator.clipboard.writeText(EXAMPLE_JSON);
    configFeedback = "JSON copiado para a área de transferência!";
    setTimeout(() => (configFeedback = null), 2500);
  }

  function addRouteRule(): void {
    tempRoutes.push({
      pathPrefix: "/api/v1/service",
      targetPort: 4000,
      targetHost: undefined,
      latencyMs: undefined,
      isMock: false,
      mockStatusCode: 200,
      mockBody: '{\n  "mock": true\n}',
    });
  }

  function removeRouteRule(idx: number): void {
    tempRoutes.splice(idx, 1);
  }

  async function saveConfig(): Promise<void> {
    relayState.config.listenPort = Number(tempListenPort);
    relayState.config.targetHost = tempTargetHost.trim();
    relayState.config.targetPort = Number(tempTargetPort);
    relayState.config.latencyMs = Number(tempLatencyMs);
    relayState.config.jitterMs = Number(tempJitterMs);
    relayState.config.simulateFailureRate = Number(tempFailurePercent) / 100;
    relayState.config.failureStatusCode = Number(tempFailureStatusCode);
    relayState.config.autoExtractJwt = tempAutoExtractJwt;
    relayState.config.routes = tempRoutes.filter(r => r.pathPrefix.trim() !== "");

    try {
      await invoke("update_proxy_config", { config: relayState.config });
    } catch (e) {
      console.error("Erro ao atualizar config no Rust:", e);
    }

    isOpen = false;
  }

  function exportConfigFile(): void {
    const configToExport = {
      listenPort: Number(tempListenPort),
      targetHost: tempTargetHost.trim(),
      targetPort: Number(tempTargetPort),
      latencyMs: Number(tempLatencyMs),
      jitterMs: Number(tempJitterMs),
      simulateFailureRate: Number(tempFailurePercent) / 100,
      failureStatusCode: Number(tempFailureStatusCode),
      autoExtractJwt: tempAutoExtractJwt,
      routes: tempRoutes.filter(r => r.pathPrefix.trim() !== ""),
    };

    const blob = new Blob([JSON.stringify(configToExport, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "relay.config.json";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    configFeedback = "relay.config.json exportado!";
    setTimeout(() => (configFeedback = null), 2500);
  }

  function handleImportFileInput(event: Event): void {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const content = e.target?.result as string;
        const loaded = await invoke<ProxyConfig>("load_config_from_json", { jsonContent: content });
        
        relayState.config = loaded;
        tempListenPort = loaded.listenPort;
        tempTargetHost = loaded.targetHost;
        tempTargetPort = loaded.targetPort;
        tempLatencyMs = loaded.latencyMs;
        tempJitterMs = loaded.jitterMs;
        tempFailurePercent = Math.round(loaded.simulateFailureRate * 100);
        tempFailureStatusCode = loaded.failureStatusCode;
        tempAutoExtractJwt = loaded.autoExtractJwt;
        tempRoutes = loaded.routes ? loaded.routes.map(r => ({ ...r })) : [];

        configFeedback = "relay.config.json carregado com sucesso!";
        setTimeout(() => (configFeedback = null), 2500);
      } catch (err) {
        configFeedback = `Erro ao ler arquivo: ${err}`;
        setTimeout(() => (configFeedback = null), 3500);
      }
    };
    reader.readAsText(file);
  }

  function cancel(): void {
    isOpen = false;
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-2xl w-full p-5 shadow-2xl space-y-4 max-h-[90vh] flex flex-col">
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3 select-none">
        <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-100">
          Configuração de Rotas, Mocks & Chaos
        </h3>
        <div class="flex items-center space-x-2">
          <!-- Botão para Alternar e Ver o JSON Exemplo -->
          <button
            onclick={() => (showExampleView = !showExampleView)}
            class="text-[11px] px-2.5 py-1 rounded transition-colors border cursor-pointer flex items-center space-x-1 {showExampleView ? 'bg-amber-500/20 text-amber-300 border-amber-500/40 font-bold' : 'bg-zinc-800 hover:bg-zinc-700 text-amber-400 border-zinc-700'}"
            title="Visualizar e salvar modelo JSON de exemplo para passar para IA"
          >
            <IconFileJson size={12} class="text-amber-400" />
            <span>{showExampleView ? 'Voltar para Form' : 'Ver Exemplo JSON'}</span>
          </button>

          {#if !showExampleView}
            <label class="text-[11px] px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors border border-zinc-700 cursor-pointer flex items-center space-x-1">
              <IconFileJson size={12} class="text-zinc-400" />
              <span>Importar</span>
              <input type="file" accept=".json" onchange={handleImportFileInput} class="hidden" />
            </label>
            <button
              onclick={exportConfigFile}
              class="text-[11px] px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors border border-zinc-700 cursor-pointer flex items-center space-x-1"
              title="Salva as portas e rotas em um arquivo relay.config.json"
            >
              <IconDownload size={12} />
              <span>Salvar</span>
            </button>
          {/if}

          <button
            onclick={cancel}
            class="text-zinc-500 hover:text-zinc-300 text-xs p-1 cursor-pointer"
          >
            ✕
          </button>
        </div>
      </div>

      <!-- Feedback Alert -->
      {#if configFeedback}
        <div class="p-2.5 rounded bg-indigo-500/10 border border-indigo-500/30 text-indigo-300 font-mono text-xs select-none">
          {configFeedback}
        </div>
      {/if}

      <!-- Form Content or Example JSON View -->
      <div class="flex-1 overflow-y-auto pr-1 text-xs">
        {#if showExampleView}
          <!-- Tela Integrada do JSON Exemplo -->
          <div class="space-y-3 py-1">
            <div class="flex items-center justify-between text-zinc-300">
              <span class="text-xs font-medium">Modelo JSON de Configuração (<span class="font-mono text-amber-400">relay.config.json</span>):</span>
              <div class="flex items-center space-x-2">
                <button
                  onclick={copyExampleJson}
                  class="text-[11px] px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors border border-zinc-700 cursor-pointer"
                >
                  Copiar JSON
                </button>
                <button
                  onclick={downloadExampleJson}
                  class="text-[11px] px-2.5 py-1 rounded bg-amber-600 hover:bg-amber-500 text-white font-medium transition-colors cursor-pointer flex items-center space-x-1"
                >
                  <IconDownload size={11} />
                  <span>Salvar Arquivo .json</span>
                </button>
              </div>
            </div>

            <textarea
              readonly
              value={EXAMPLE_JSON}
              rows="13"
              class="w-full bg-zinc-950 border border-zinc-800 rounded-lg p-3 text-zinc-200 font-mono text-[11px] focus:outline-none select-all resize-none leading-relaxed"
            ></textarea>

            <p class="text-[11px] text-zinc-500">
              Dica: Você pode copiar esse JSON ou salvar o arquivo e pedir para qualquer IA mapear as portas dos seus microsserviços.
            </p>
          </div>
        {:else}
          <!-- Form Normal -->
          <div class="space-y-5">
            <!-- Roteamento Global (Default) -->
            <div class="space-y-2.5">
              <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none block">
                Roteamento Global (Padrão)
              </span>

              <div class="grid grid-cols-3 gap-3">
                <div>
                  <label class="block text-zinc-400 font-medium mb-1" for="listenPort">
                    Porta Proxy Local
                  </label>
                  <input
                    id="listenPort"
                    type="number"
                    bind:value={tempListenPort}
                    placeholder="8080"
                    class="w-full bg-zinc-900 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
                  />
                </div>

                <div>
                  <label class="block text-zinc-400 font-medium mb-1" for="targetHost">
                    Host Alvo
                  </label>
                  <input
                    id="targetHost"
                    type="text"
                    bind:value={tempTargetHost}
                    placeholder="127.0.0.1"
                    class="w-full bg-zinc-900 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
                  />
                </div>

                <div>
                  <label class="block text-zinc-400 font-medium mb-1" for="targetPort">
                    Porta Alvo
                  </label>
                  <input
                    id="targetPort"
                    type="number"
                    bind:value={tempTargetPort}
                    placeholder="3000"
                    class="w-full bg-zinc-900 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
                  />
                </div>
              </div>
            </div>

            <!-- Roteamento Multisserviço & Mocks -->
            <div class="space-y-3 pt-3 border-t border-zinc-800">
              <div class="flex items-center justify-between select-none">
                <div>
                  <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 block">
                    Roteamento Multisserviço & Mocks ({tempRoutes.length})
                  </span>
                  <span class="text-[11px] text-zinc-500">
                    Encaminhe prefixos para portas distintas ou responda diretamente como Mock.
                  </span>
                </div>
                <button
                  onclick={addRouteRule}
                  class="text-xs px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors border border-zinc-700 cursor-pointer"
                >
                  + Adicionar Rota
                </button>
              </div>

              {#if tempRoutes.length === 0}
                <div class="p-3 text-center text-zinc-500 rounded bg-zinc-950/40 border border-zinc-800/60 font-mono text-[11px]">
                  Nenhuma regra customizada. Todas as requisições irão para o Host Padrão ({tempTargetHost}:{tempTargetPort}).
                </div>
              {:else}
                <div class="space-y-3">
                  {#each tempRoutes as route, idx}
                    <div class="p-3 rounded-lg bg-zinc-950 border border-zinc-800 space-y-3">
                      <div class="grid grid-cols-12 gap-2 items-center">
                        <div class="col-span-5">
                          <label class="block text-[10px] text-zinc-500 mb-0.5" for="prefix-{idx}">Prefixo da URL</label>
                          <input
                            id="prefix-{idx}"
                            type="text"
                            bind:value={route.pathPrefix}
                            placeholder="/api/v1/auth"
                            class="w-full bg-zinc-900 border border-zinc-800 rounded px-2 py-1 text-zinc-200 font-mono text-xs focus:outline-none focus:border-indigo-500"
                          />
                        </div>

                        <div class="col-span-3">
                          <label class="block text-[10px] text-zinc-500 mb-0.5" for="port-{idx}">Porta Destino</label>
                          <input
                            id="port-{idx}"
                            type="number"
                            bind:value={route.targetPort}
                            disabled={route.isMock}
                            placeholder="3000"
                            class="w-full bg-zinc-900 border border-zinc-800 rounded px-2 py-1 text-zinc-200 font-mono text-xs focus:outline-none focus:border-indigo-500 disabled:opacity-40"
                          />
                        </div>

                        <div class="col-span-3 flex items-center space-x-2 pt-3">
                          <label class="flex items-center space-x-1.5 cursor-pointer text-zinc-300 select-none">
                            <input
                              type="checkbox"
                              bind:checked={route.isMock}
                              class="rounded bg-zinc-900 border-zinc-700 text-indigo-500 focus:ring-0"
                            />
                            <span class="text-xs">Mock</span>
                          </label>
                        </div>

                        <div class="col-span-1 text-right pt-3">
                          <button
                            onclick={() => removeRouteRule(idx)}
                            class="text-zinc-500 hover:text-rose-400 p-1 cursor-pointer transition-colors"
                            title="Remover Rota"
                          >
                            ✕
                          </button>
                        </div>
                      </div>

                      <!-- Se for Mock, exibe campos de StatusCode e Body -->
                      {#if route.isMock}
                        <div class="p-2.5 rounded bg-zinc-900/60 border border-zinc-800/80 space-y-2">
                          <div class="flex items-center space-x-3">
                            <div class="w-24">
                              <label class="block text-[10px] text-zinc-500 mb-0.5" for="mockStatus-{idx}">Status HTTP</label>
                              <input
                                id="mockStatus-{idx}"
                                type="number"
                                bind:value={route.mockStatusCode}
                                placeholder="200"
                                class="w-full bg-zinc-950 border border-zinc-800 rounded px-2 py-1 text-zinc-200 font-mono text-xs focus:outline-none focus:border-indigo-500"
                              />
                            </div>
                            <div class="flex-1">
                              <label class="block text-[10px] text-zinc-500 mb-0.5" for="mockBody-{idx}">Corpo Mockado (JSON)</label>
                              <textarea
                                id="mockBody-{idx}"
                                bind:value={route.mockBody}
                                rows="2"
                                placeholder="JSON de resposta simulada..."
                                class="w-full bg-zinc-950 border border-zinc-800 rounded p-1.5 text-zinc-300 font-mono text-[11px] focus:outline-none focus:border-indigo-500 resize-y"
                              ></textarea>
                            </div>
                          </div>
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <!-- Chaos Simulator -->
            <div class="space-y-3 pt-3 border-t border-zinc-800">
              <div class="flex items-center justify-between select-none">
                <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 block">
                  Chaos Simulator (Falhas & Latência)
                </span>
                <span class="text-[10px] px-2 py-0.5 rounded bg-amber-500/10 text-amber-300 border border-amber-500/20 font-mono">
                  Testes de Resiliência
                </span>
              </div>

              <div class="grid grid-cols-2 gap-4">
                <!-- Latência Artificial -->
                <div class="p-3 rounded-lg bg-zinc-950 border border-zinc-800 space-y-2">
                  <div class="flex items-center justify-between text-zinc-400">
                    <span>Latência Base</span>
                    <span class="font-mono text-amber-400 font-bold px-1.5 py-0.2 bg-zinc-900 rounded border border-zinc-800">
                      {tempLatencyMs} ms
                    </span>
                  </div>
                  <input
                    type="range"
                    min="0"
                    max="3000"
                    step="50"
                    bind:value={tempLatencyMs}
                    class="w-full accent-amber-500 cursor-pointer"
                  />
                </div>

                <!-- Taxa de Falhas -->
                <div class="p-3 rounded-lg bg-zinc-950 border border-zinc-800 space-y-2">
                  <div class="flex items-center justify-between text-zinc-400">
                    <span>Taxa de Falhas</span>
                    <span class="font-mono text-rose-400 font-bold px-1.5 py-0.2 bg-zinc-900 rounded border border-zinc-800">
                      {tempFailurePercent}% ({tempFailureStatusCode})
                    </span>
                  </div>
                  <input
                    type="range"
                    min="0"
                    max="100"
                    step="5"
                    bind:value={tempFailurePercent}
                    class="w-full accent-rose-500 cursor-pointer"
                  />
                </div>
              </div>
            </div>

            <!-- Extração JWT -->
            <div class="pt-3 border-t border-zinc-800 flex items-center justify-between select-none">
              <div>
                <span class="font-medium text-zinc-200 block">Auto-Extração de Sessão & JWT</span>
                <span class="text-[11px] text-zinc-500">Captura e decodifica automaticamente tokens Bearer e Claims JSON no tráfego.</span>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" bind:checked={tempAutoExtractJwt} class="sr-only peer" />
                <div class="w-9 h-5 bg-zinc-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-zinc-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-indigo-600"></div>
              </label>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-end space-x-2 pt-3 border-t border-zinc-800 select-none">
        {#if !showExampleView}
          <button
            onclick={cancel}
            class="text-xs px-3 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer"
          >
            Cancelar
          </button>
          <button
            onclick={saveConfig}
            class="text-xs px-4 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-xs cursor-pointer"
          >
            Salvar Alterações
          </button>
        {:else}
          <button
            onclick={() => (showExampleView = false)}
            class="text-xs px-4 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors cursor-pointer"
          >
            Fechar Exemplo
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
