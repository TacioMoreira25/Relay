<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
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

  async function saveConfig(): Promise<void> {
    relayState.config.listenPort = Number(tempListenPort);
    relayState.config.targetHost = tempTargetHost.trim();
    relayState.config.targetPort = Number(tempTargetPort);
    relayState.config.latencyMs = Number(tempLatencyMs);
    relayState.config.jitterMs = Number(tempJitterMs);
    relayState.config.simulateFailureRate = Number(tempFailurePercent) / 100;
    relayState.config.failureStatusCode = Number(tempFailureStatusCode);
    relayState.config.autoExtractJwt = tempAutoExtractJwt;

    // Sincroniza dinamicamente com o backend Rust
    try {
      await invoke("update_proxy_config", { config: relayState.config });
    } catch (e) {
      console.error("Erro ao atualizar config no Rust:", e);
    }

    isOpen = false;
  }

  function cancel(): void {
    tempListenPort = relayState.config.listenPort;
    tempTargetHost = relayState.config.targetHost;
    tempTargetPort = relayState.config.targetPort;
    tempLatencyMs = relayState.config.latencyMs;
    tempJitterMs = relayState.config.jitterMs;
    tempFailurePercent = Math.round(relayState.config.simulateFailureRate * 100);
    tempFailureStatusCode = relayState.config.failureStatusCode;
    tempAutoExtractJwt = relayState.config.autoExtractJwt;
    isOpen = false;
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-lg w-full p-5 shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3">
        <h3 class="text-sm font-semibold text-zinc-100 flex items-center space-x-2">
          <span>⚙️</span>
          <span>Configurações do Proxy & Chaos Simulator</span>
        </h3>
        <button
          onclick={cancel}
          class="text-zinc-500 hover:text-zinc-300 text-xs p-1"
        >
          ✕
        </button>
      </div>

      <div class="space-y-4 text-xs">
        <!-- Seção 1: Roteamento de Rede -->
        <div class="space-y-2">
          <span class="text-[11px] font-bold uppercase tracking-wider text-indigo-400">Roteamento</span>
          
          <div class="grid grid-cols-3 gap-2">
            <div>
              <label class="block text-zinc-400 font-medium mb-1" for="listenPort">
                Porta Proxy
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
                Host Destino
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

        <!-- Seção 2: Chaos Engineering (Latência, Jitter e Falhas) -->
        <div class="space-y-3 pt-2 border-t border-zinc-800/80">
          <span class="text-[11px] font-bold uppercase tracking-wider text-amber-400">Chaos Simulator (Falhas & Latência)</span>

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
                  Taxa de Erros Simulados
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
                Status de Falha Injetado
              </label>
              <select
                id="failureStatus"
                bind:value={tempFailureStatusCode}
                class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
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

        <!-- Seção 3: Auto-Extração JWT -->
        <div class="pt-2 border-t border-zinc-800/80 flex items-center justify-between">
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

      <div class="flex items-center justify-end space-x-2 pt-3 border-t border-zinc-800">
        <button
          onclick={cancel}
          class="text-xs px-3 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors"
        >
          Cancelar
        </button>
        <button
          onclick={saveConfig}
          class="text-xs px-3 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-sm"
        >
          Salvar Alterações
        </button>
      </div>
    </div>
  </div>
{/if}
