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

  let configFeedback = $state<string | null>(null);

  $effect(() => {
    if (isOpen) {
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
          <label class="text-[11px] px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors border border-zinc-700 cursor-pointer flex items-center space-x-1">
            <IconFileJson size={12} class="text-amber-400" />
            <span>Importar JSON</span>
            <input type="file" accept=".json" onchange={handleImportFileInput} class="hidden" />
          </label>
          <button
            onclick={exportConfigFile}
            class="text-[11px] px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors border border-zinc-700 cursor-pointer flex items-center space-x-1"
            title="Salva as portas e rotas em um arquivo relay.config.json"
          >
            <IconDownload size={12} />
            <span>Salvar JSON</span>
          </button>
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

      <!-- Form Content -->
      <div class="flex-1 overflow-y-auto space-y-5 pr-1 text-xs">
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

        <!-- Roteamento por Prefixo de Rota & Mocks -->
        <div class="space-y-2.5 pt-2 border-t border-zinc-800/80">
          <div class="flex items-center justify-between select-none">
            <div>
              <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 block">
                Roteamento Multisserviço & Mocks ({tempRoutes.length})
              </span>
              <span class="text-[10px] text-zinc-500">
                Encaminhe prefixos para portas distintas ou responda diretamente como Mock.
              </span>
            </div>
            <button
              onclick={addRouteRule}
              class="text-[11px] px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors cursor-pointer border border-zinc-700 shadow-xs"
            >
              + Adicionar Rota
            </button>
          </div>

          {#if tempRoutes.length === 0}
            <div class="p-3 bg-zinc-900/40 border border-zinc-800/60 rounded-lg text-[11px] text-zinc-400 italic">
              Nenhuma regra customizada. Todas as requisições irão para o Host Padrão ({tempTargetHost}:{tempTargetPort}).
            </div>
          {:else}
            <div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-900/30">
              <table class="w-full text-left text-xs font-mono">
                <thead class="bg-zinc-900/80 border-b border-zinc-800/80 text-zinc-500 text-[10px] uppercase tracking-wider select-none">
                  <tr>
                    <th class="py-2 px-3">Prefixo da Rota</th>
                    <th class="py-2 px-3 w-28">Ação / Mock</th>
                    <th class="py-2 px-3 w-28">Destino / Status</th>
                    <th class="py-2 px-2 text-right">Ação</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-zinc-800/40">
                  {#each tempRoutes as route, idx}
                    <tr class="hover:bg-zinc-900/50 transition-colors">
                      <td class="py-2 px-3">
                        <input
                          type="text"
                          placeholder="/api/v1/auth"
                          bind:value={route.pathPrefix}
                          class="w-full bg-zinc-900 border border-zinc-800 rounded px-2 py-1 text-zinc-200 focus:outline-none focus:border-indigo-500 text-xs"
                        />
                      </td>
                      <td class="py-2 px-3">
                        <label class="flex items-center space-x-1.5 text-xs text-zinc-300 cursor-pointer">
                          <input type="checkbox" bind:checked={route.isMock} class="accent-indigo-500 w-3.5 h-3.5" />
                          <span class={route.isMock ? 'text-amber-400 font-bold' : ''}>{route.isMock ? 'Mock' : 'Proxy'}</span>
                        </label>
                      </td>
                      <td class="py-2 px-3">
                        {#if route.isMock}
                          <input
                            type="number"
                            placeholder="Status (200)"
                            bind:value={route.mockStatusCode}
                            class="w-full bg-zinc-900 border border-zinc-800 rounded px-2 py-1 text-amber-300 font-bold focus:outline-none focus:border-indigo-500 text-xs"
                          />
                        {:else}
                          <input
                            type="number"
                            placeholder="Porta (3001)"
                            bind:value={route.targetPort}
                            class="w-full bg-zinc-900 border border-zinc-800 rounded px-2 py-1 text-zinc-200 focus:outline-none focus:border-indigo-500 text-xs"
                          />
                        {/if}
                      </td>
                      <td class="py-2 px-2 text-right">
                        <button
                          onclick={() => removeRouteRule(idx)}
                          class="text-zinc-500 hover:text-rose-400 p-1 text-xs cursor-pointer"
                          title="Remover regra"
                        >
                          ✕
                        </button>
                      </td>
                    </tr>
                    {#if route.isMock}
                      <tr class="bg-zinc-950/40">
                        <td colspan="4" class="p-2.5">
                          <textarea
                            bind:value={route.mockBody}
                            rows="2"
                            placeholder={`{\n  "mockData": "exemplo"\n}`}
                            class="w-full bg-zinc-900 border border-zinc-800 rounded p-2 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500 leading-relaxed"
                          ></textarea>
                        </td>
                      </tr>
                    {/if}
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>

        <!-- Chaos Simulator (Falhas & Latência com Tags de Alto Contraste) -->
        <div class="space-y-3 pt-2 border-t border-zinc-800/80">
          <div class="flex items-center justify-between select-none">
            <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
              Chaos Simulator (Falhas & Latência)
            </span>
            <span class="text-[10px] px-2 py-0.5 rounded bg-amber-500/10 text-amber-300 border border-amber-500/20 font-sans">
              Testes de Resiliência
            </span>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div class="bg-zinc-900/60 p-3 rounded-lg border border-zinc-800 space-y-2">
              <div class="flex justify-between items-center text-xs">
                <label for="latencySlider" class="text-zinc-400 font-medium">Latência Base</label>
                <span class="font-mono text-amber-400 font-bold bg-amber-500/10 px-2 py-0.5 rounded border border-amber-500/20">{tempLatencyMs} ms</span>
              </div>
              <input
                id="latencySlider"
                type="range"
                min="0"
                max="3000"
                step="50"
                bind:value={tempLatencyMs}
                class="w-full accent-amber-500 cursor-pointer"
              />
            </div>

            <div class="bg-zinc-900/60 p-3 rounded-lg border border-zinc-800 space-y-2">
              <div class="flex justify-between items-center text-xs">
                <label for="failureSlider" class="text-zinc-400 font-medium">Taxa de Falhas</label>
                <span class="font-mono text-rose-400 font-bold bg-rose-500/10 px-2 py-0.5 rounded border border-rose-500/20">{tempFailurePercent}% ({tempFailureStatusCode})</span>
              </div>
              <input
                id="failureSlider"
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

        <!-- Toggle Switch Moderno para JWT -->
        <div class="pt-3 border-t border-zinc-800/80 flex items-center justify-between select-none">
          <div>
            <span class="text-xs font-semibold text-zinc-200 block">
              Auto-Extração de Sessão & JWT
            </span>
            <span class="text-[11px] text-zinc-500">
              Captura e decodifica automaticamente tokens Bearer e Claims JSON no tráfego.
            </span>
          </div>

          <!-- Toggle Switch Style -->
          <button
            type="button"
            onclick={() => (tempAutoExtractJwt = !tempAutoExtractJwt)}
            aria-label="Ativar auto-extração de JWT" title="Ativar auto-extração de JWT" class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {tempAutoExtractJwt ? 'bg-indigo-600' : 'bg-zinc-800'}"
          >
            <span
              class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out {tempAutoExtractJwt ? 'translate-x-4' : 'translate-x-0'}"
            ></span>
          </button>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-end space-x-2 pt-3 border-t border-zinc-800 select-none">
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
      </div>
    </div>
  </div>
{/if}
