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
  <div class="fixed inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-xl w-full p-5 shadow-2xl space-y-4 max-h-[90vh] flex flex-col">
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3 select-none">
        <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-100">
          Configuração de Rotas & Chaos
        </h3>
        <div class="flex items-center space-x-2">
          <label class="text-[11px] px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors border border-zinc-700 cursor-pointer flex items-center space-x-1">
            <IconFileJson size={12} />
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

      {#if configFeedback}
        <div class="p-2 rounded bg-indigo-500/10 border border-indigo-500/30 text-indigo-300 font-mono text-[11px]">
          {configFeedback}
        </div>
      {/if}

      <!-- Modal Body -->
      <div class="flex-1 overflow-y-auto space-y-4 pr-1 text-xs">
        <!-- Roteamento Global Fallback -->
        <div class="space-y-2">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none">
            Roteamento Global (Default)
          </span>
          
          <div class="grid grid-cols-3 gap-2">
            <div>
              <label class="block text-zinc-400 font-medium mb-1" for="listenPort">
                Porta Proxy Local
              </label>
              <input
                id="listenPort"
                type="number"
                bind:value={tempListenPort}
                disabled={relayState.isProxyRunning}
                class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500 disabled:opacity-50"
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
                class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
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
                class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
              />
            </div>
          </div>
        </div>

        <!-- Roteamento por Prefixo de Rota -->
        <div class="space-y-2 pt-2 border-t border-zinc-800/80">
          <div class="flex items-center justify-between select-none">
            <div>
              <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 block">
                Roteamento por Prefixo de Rota ({tempRoutes.length})
              </span>
              <span class="text-[10px] text-zinc-500">
                Encaminha prefixos específicos para portas distintas (microsserviços/mocks).
              </span>
            </div>
            <button
              onclick={addRouteRule}
              class="text-[11px] px-2 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer border border-zinc-700"
            >
              + Adicionar Rota
            </button>
          </div>

          {#if tempRoutes.length === 0}
            <div class="p-2.5 bg-zinc-950/60 border border-zinc-800/60 rounded text-[11px] text-zinc-500 italic">
              Nenhuma regra de rota customizada. Todas as requisições irão para o Host Padrão ({tempTargetHost}:{tempTargetPort}).
            </div>
          {:else}
            <div class="space-y-2">
              {#each tempRoutes as route, idx}
                <div class="flex items-center space-x-2 bg-zinc-950 p-2 rounded border border-zinc-800 font-mono text-xs">
                  <div class="flex-1">
                    <input
                      type="text"
                      placeholder="Prefixo (ex: /api/v1/auth)"
                      bind:value={route.pathPrefix}
                      class="w-full bg-zinc-900 border border-zinc-800 rounded px-2 py-1 text-zinc-200 focus:outline-none focus:border-indigo-500"
                    />
                  </div>
                  <div class="w-24">
                    <input
                      type="number"
                      placeholder="Porta (4000)"
                      bind:value={route.targetPort}
                      class="w-full bg-zinc-900 border border-zinc-800 rounded px-2 py-1 text-zinc-200 focus:outline-none focus:border-indigo-500"
                    />
                  </div>
                  <button
                    onclick={() => removeRouteRule(idx)}
                    class="text-zinc-500 hover:text-rose-400 p-1 text-xs cursor-pointer"
                    title="Remover regra de rota"
                  >
                    ✕
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Chaos Simulator -->
        <div class="space-y-3 pt-2 border-t border-zinc-800/80">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none">
            Chaos Simulator (Falhas & Latência)
          </span>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-zinc-400 font-medium mb-1" for="latencyMs">
                Latência Base (ms)
              </label>
              <input
                id="latencyMs"
                type="number"
                min="0"
                step="50"
                bind:value={tempLatencyMs}
                placeholder="0"
                class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div>
              <label class="block text-zinc-400 font-medium mb-1" for="jitterMs">
                Jitter Aleatório (ms)
              </label>
              <input
                id="jitterMs"
                type="number"
                min="0"
                step="25"
                bind:value={tempJitterMs}
                placeholder="0"
                class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
              />
            </div>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div>
              <div class="flex items-center justify-between mb-1">
                <label class="text-zinc-400 font-medium" for="failureRate">
                  Taxa de Falhas
                </label>
                <span class="text-amber-400 font-mono font-bold">{tempFailurePercent}%</span>
              </div>
              <input
                id="failureRate"
                type="range"
                min="0"
                max="100"
                step="5"
                bind:value={tempFailurePercent}
                class="w-full accent-amber-500 cursor-pointer"
              />
            </div>

            <div>
              <label class="block text-zinc-400 font-medium mb-1" for="failureStatus">
                Status Injetado
              </label>
              <select
                id="failureStatus"
                bind:value={tempFailureStatusCode}
                class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500 cursor-pointer"
              >
                <option value={500}>500 Internal Server Error</option>
                <option value={502}>502 Bad Gateway</option>
                <option value={503}>503 Service Unavailable</option>
                <option value={504}>504 Gateway Timeout</option>
                <option value={429}>429 Too Many Requests</option>
              </select>
            </div>
          </div>
        </div>

        <!-- JWT -->
        <div class="pt-2 border-t border-zinc-800/80 flex items-center justify-between select-none">
          <div>
            <span class="text-zinc-300 font-medium block">Auto-Captura de Tokens JWT</span>
            <span class="text-[10px] text-zinc-500">Decodifica e grava tokens que transitam em headers e bodies.</span>
          </div>
          <input
            type="checkbox"
            bind:checked={tempAutoExtractJwt}
            class="w-4 h-4 accent-indigo-600 rounded cursor-pointer"
          />
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="flex items-center justify-end space-x-2 pt-3 border-t border-zinc-800 select-none">
        <button
          onclick={cancel}
          class="text-xs px-3.5 py-2 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer"
        >
          Cancelar
        </button>
        <button
          onclick={saveConfig}
          class="text-xs px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-xs cursor-pointer"
        >
          Salvar Alterações
        </button>
      </div>
    </div>
  </div>
{/if}
