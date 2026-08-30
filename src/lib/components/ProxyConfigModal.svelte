<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";

  let { isOpen = $bindable(false) }: { isOpen: boolean } = $props();

  let tempListenPort = $state(relayState.config.listenPort);
  let tempTargetHost = $state(relayState.config.targetHost);
  let tempTargetPort = $state(relayState.config.targetPort);
  let tempLatencyMs = $state(relayState.config.latencyMs);

  function saveConfig(): void {
    relayState.config.listenPort = Number(tempListenPort);
    relayState.config.targetHost = tempTargetHost.trim();
    relayState.config.targetPort = Number(tempTargetPort);
    relayState.config.latencyMs = Number(tempLatencyMs);
    isOpen = false;
  }

  function cancel(): void {
    tempListenPort = relayState.config.listenPort;
    tempTargetHost = relayState.config.targetHost;
    tempTargetPort = relayState.config.targetPort;
    tempLatencyMs = relayState.config.latencyMs;
    isOpen = false;
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-md w-full p-5 shadow-2xl space-y-4">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3">
        <h3 class="text-sm font-semibold text-zinc-100 flex items-center space-x-2">
          <span>⚙️</span>
          <span>Configurações do Proxy HTTP</span>
        </h3>
        <button
          onclick={cancel}
          class="text-zinc-500 hover:text-zinc-300 text-xs p-1"
        >
          ✕
        </button>
      </div>

      <div class="space-y-3 text-xs">
        <div>
          <label class="block text-zinc-400 font-medium mb-1" for="listenPort">
            Porta Local de Escuta (Relay)
          </label>
          <input
            id="listenPort"
            type="number"
            bind:value={tempListenPort}
            disabled={relayState.isProxyRunning}
            class="w-full bg-zinc-950 border border-zinc-800 rounded px-3 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500 disabled:opacity-50"
          />
          {#if relayState.isProxyRunning}
            <span class="text-[10px] text-amber-400 mt-1 block">Pare o proxy para alterar a porta de escuta.</span>
          {/if}
        </div>

        <div class="grid grid-cols-3 gap-2">
          <div class="col-span-2">
            <label class="block text-zinc-400 font-medium mb-1" for="targetHost">
              Host de Destino (Upstream)
            </label>
            <input
              id="targetHost"
              type="text"
              bind:value={tempTargetHost}
              placeholder="127.0.0.1"
              class="w-full bg-zinc-950 border border-zinc-800 rounded px-3 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
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
              class="w-full bg-zinc-950 border border-zinc-800 rounded px-3 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
            />
          </div>
        </div>

        <div>
          <label class="block text-zinc-400 font-medium mb-1" for="latencyMs">
            Injeção de Latência Artificial (ms)
          </label>
          <input
            id="latencyMs"
            type="number"
            min="0"
            step="50"
            bind:value={tempLatencyMs}
            placeholder="0"
            class="w-full bg-zinc-950 border border-zinc-800 rounded px-3 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
          />
          <span class="text-[10px] text-zinc-500 mt-1 block">0 = repasse instantâneo sem delay.</span>
        </div>
      </div>

      <div class="flex items-center justify-end space-x-2 pt-2 border-t border-zinc-800">
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
          Salvar
        </button>
      </div>
    </div>
  </div>
{/if}
