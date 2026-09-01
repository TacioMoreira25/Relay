<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { TargetEnvironment, DiscoveredTarget } from "$lib/types";
  import { IconServer, IconPlus, IconTrash, IconPencil } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let isOpen = $state<boolean>(false);
  let isAddModalOpen = $state<boolean>(false);
  let editingTargetId = $state<string | null>(null);

  // Form para adicionar/editar alvo manual
  let newName = $state<string>("");
  let newHost = $state<string>("127.0.0.1");
  let newPort = $state<number>(3000);
  let newIsHttps = $state<boolean>(false);

  async function scanPorts(): Promise<void> {
    relayState.isScanningTargets = true;
    try {
      const results = await invoke<DiscoveredTarget[]>("scan_active_targets");
      relayState.discoveredTargets = results;
    } catch (e) {
      console.warn("Falha ao escanear portas locais:", e);
    } finally {
      relayState.isScanningTargets = false;
    }
  }

  async function selectTarget(target: TargetEnvironment): Promise<void> {
    relayState.selectTarget(target);
    try {
      await invoke("update_proxy_config", { config: relayState.config });
    } catch (e) {
      console.error("Erro ao sincronizar target:", e);
    }
    isOpen = false;
  }

  function handleSelectDiscovered(disc: DiscoveredTarget): void {
    const target: TargetEnvironment = {
      id: disc.id,
      name: disc.label,
      host: disc.host,
      port: disc.port,
      isHttps: false,
      isActive: disc.isActive,
      type: "auto",
    };
    selectTarget(target);
  }

  function openCreateModal(): void {
    editingTargetId = null;
    newName = "";
    newHost = "127.0.0.1";
    newPort = 3000;
    newIsHttps = false;
    isOpen = false;
    isAddModalOpen = true;
  }

  function openEditModal(env: TargetEnvironment, e: MouseEvent): void {
    e.stopPropagation();
    editingTargetId = env.id;
    newName = env.name;
    newHost = env.host;
    newPort = env.port;
    newIsHttps = env.isHttps;
    isOpen = false;
    isAddModalOpen = true;
  }

  function handleDeleteTarget(id: string, e: MouseEvent): void {
    e.stopPropagation();
    relayState.removeSavedEnvironment(id);
  }

  function handleSaveManualTarget(): void {
    const hostVal = newHost.trim() || "127.0.0.1";
    const portVal = Number(newPort) || 3000;
    const nameVal = newName.trim() || `${hostVal}:${portVal}`;

    if (editingTargetId) {
      const updated: TargetEnvironment = {
        id: editingTargetId,
        name: nameVal,
        host: hostVal,
        port: portVal,
        isHttps: newIsHttps,
        isActive: true,
        type: "saved",
      };
      relayState.updateSavedEnvironment(updated);
    } else {
      const target: TargetEnvironment = {
        id: `manual-${Date.now()}`,
        name: nameVal,
        host: hostVal,
        port: portVal,
        isHttps: newIsHttps,
        isActive: true,
        type: "saved",
      };
      relayState.addSavedEnvironment(target);
      selectTarget(target);
    }

    isAddModalOpen = false;
    editingTargetId = null;
    newName = "";
    newHost = "127.0.0.1";
    newPort = 3000;
    newIsHttps = false;
  }

  function handleToggleOpen(): void {
    isOpen = !isOpen;
    if (isOpen) {
      scanPorts();
    }
  }

  onMount(() => {
    scanPorts();
  });
</script>

<div class="relative inline-block text-left">
  <!-- Botão Gatilho (TopBar) -->
  <button
    onclick={handleToggleOpen}
    class="flex items-center space-x-2 bg-zinc-900/90 hover:bg-zinc-800 border border-zinc-800 px-3 py-1.5 rounded-lg text-xs font-mono text-zinc-200 transition-all cursor-pointer shadow-xs"
  >
    {#if relayState.activeTarget}
      <div class="w-2 h-2 rounded-full {relayState.activeTarget.isActive ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)] animate-pulse' : 'bg-zinc-500'}"></div>
      <span class="font-semibold text-zinc-100">{relayState.activeTarget.name}</span>
      <span class="text-indigo-400 text-[11px]">(:{relayState.activeTarget.port})</span>
    {:else}
      <div class="w-2 h-2 rounded-full bg-zinc-600"></div>
      <span class="text-zinc-400">Nenhum Alvo Conectado</span>
    {/if}
    <span class="text-zinc-500 text-[9px] ml-1">▼</span>
  </button>

  <!-- Dropdown Popover Moderno -->
  {#if isOpen}
    <div
      role="menu"
      tabindex="0"
      class="absolute left-0 top-10 w-72 bg-zinc-900/95 backdrop-blur-md border border-zinc-800 rounded-xl shadow-2xl p-2 z-50 space-y-3 font-mono text-xs text-zinc-200"
      onmouseleave={() => (isOpen = false)}
    >
      <!-- Seção 1: Detectados Agora (Portas Ativas no Linux) -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-zinc-400 select-none">
          <span>Detectados Agora ({relayState.discoveredTargets.filter(t => t.isActive).length})</span>
          <button
            onclick={scanPorts}
            class="text-indigo-400 hover:text-indigo-300 transition-colors cursor-pointer text-[10px] lowercase flex items-center space-x-1"
          >
            <span>{relayState.isScanningTargets ? 'escaneando...' : '↻ escanear'}</span>
          </button>
        </div>

        <div class="space-y-0.5 max-h-36 overflow-y-auto pr-0.5">
          {#each relayState.discoveredTargets.filter(t => t.isActive) as target}
            <button
              type="button"
              role="menuitem"
              onclick={() => handleSelectDiscovered(target)}
              class="w-full text-left px-2.5 py-1.5 rounded-lg hover:bg-zinc-800/80 flex items-center justify-between transition-colors cursor-pointer {relayState.activeTarget?.port === target.port && relayState.activeTarget?.host === target.host ? 'bg-zinc-800/90 font-bold border border-zinc-700/60' : ''}"
            >
              <div class="flex items-center space-x-2 truncate">
                <div class="w-1.5 h-1.5 rounded-full bg-emerald-400 shrink-0"></div>
                <span class="truncate text-xs text-zinc-200">{target.label}</span>
              </div>
              <span class="text-[10px] px-1.5 py-0.2 rounded bg-indigo-500/10 text-indigo-300 border border-indigo-500/20 shrink-0 ml-1">
                :{target.port}
              </span>
            </button>
          {/each}

          {#if relayState.discoveredTargets.filter(t => t.isActive).length === 0}
            <div class="px-2 py-1.5 text-[11px] text-zinc-500 italic">
              Nenhuma porta de dev ativa detectada no momento.
            </div>
          {/if}
        </div>
      </div>

      <!-- Seção 2: Ambientes Salvos / Remotos -->
      <div class="space-y-1.5 pt-1.5 border-t border-zinc-800/80">
        <div class="flex items-center justify-between px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-zinc-400 select-none">
          <span>Ambientes Salvos ({relayState.savedEnvironments.length})</span>
        </div>

        <div class="space-y-0.5 max-h-32 overflow-y-auto pr-0.5">
          {#each relayState.savedEnvironments as env}
            <div
              role="menuitem"
              tabindex="0"
              onclick={() => selectTarget(env)}
              onkeydown={(e) => { if (e.key === 'Enter') selectTarget(env); }}
              class="group w-full text-left px-2.5 py-1.5 rounded-lg hover:bg-zinc-800/80 flex items-center justify-between transition-colors cursor-pointer {relayState.activeTarget?.id === env.id ? 'bg-zinc-800/90 font-bold border border-zinc-700/60' : ''}"
            >
              <div class="flex items-center space-x-2 truncate flex-1 pr-1">
                <IconServer size={12} class="text-zinc-400 shrink-0" />
                <span class="truncate text-xs text-zinc-200">{env.name}</span>
                <span class="text-[10px] text-zinc-500 truncate">({env.host}:{env.port})</span>
              </div>

              <!-- Ações ao passar o mouse -->
              <div class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  type="button"
                  onclick={(e) => openEditModal(env, e)}
                  class="p-1 text-zinc-400 hover:text-indigo-300 transition-colors"
                  title="Editar ambiente"
                >
                  <IconPencil size={11} />
                </button>
                <button
                  type="button"
                  onclick={(e) => handleDeleteTarget(env.id, e)}
                  class="p-1 text-zinc-400 hover:text-rose-400 transition-colors"
                  title="Excluir ambiente"
                >
                  <IconTrash size={11} />
                </button>
              </div>
            </div>
          {/each}

          {#if relayState.savedEnvironments.length === 0}
            <div class="px-2 py-1.5 text-[11px] text-zinc-500 italic">
              Nenhum ambiente salvo ainda.
            </div>
          {/if}
        </div>
      </div>

      <!-- Rodapé / Ações Rápidas -->
      <div class="pt-1.5 border-t border-zinc-800/80 flex items-center justify-between px-1">
        <button
          onclick={openCreateModal}
          class="w-full text-[11px] px-2.5 py-1.5 rounded-lg bg-zinc-800/60 hover:bg-zinc-800 text-zinc-300 hover:text-white transition-colors border border-zinc-700/60 flex items-center justify-center space-x-1.5 cursor-pointer"
        >
          <IconPlus size={12} class="text-indigo-400" />
          <span>Adicionar Alvo Manual</span>
        </button>
      </div>
    </div>
  {/if}
</div>

<!-- Modal Adicionar / Editar Alvo Manual -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-sm w-full p-5 shadow-2xl space-y-4 flex flex-col font-sans">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-2 select-none">
        <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-100">
          {editingTargetId ? 'Editar Alvo / Ambiente' : 'Adicionar Alvo / Ambiente'}
        </h3>
        <button onclick={() => (isAddModalOpen = false)} class="text-zinc-500 hover:text-zinc-300 text-xs p-1 cursor-pointer">
          ✕
        </button>
      </div>

      <div class="space-y-3 text-xs">
        <div>
          <label class="block text-zinc-400 font-medium mb-1" for="envName">Nome / Identificador (Opcional)</label>
          <input
            id="envName"
            type="text"
            placeholder="Ex: Minha API Local (ou vazio)"
            bind:value={newName}
            class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 focus:outline-none focus:border-indigo-500 text-xs font-mono"
          />
        </div>

        <div class="grid grid-cols-3 gap-2">
          <div class="col-span-2">
            <label class="block text-zinc-400 font-medium mb-1" for="envHost">Host / Domínio</label>
            <input
              id="envHost"
              type="text"
              placeholder="127.0.0.1"
              bind:value={newHost}
              class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 focus:outline-none focus:border-indigo-500 text-xs font-mono"
            />
          </div>
          <div>
            <label class="block text-zinc-400 font-medium mb-1" for="envPort">Porta</label>
            <input
              id="envPort"
              type="number"
              placeholder="3000"
              bind:value={newPort}
              class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 focus:outline-none focus:border-indigo-500 text-xs font-mono"
            />
          </div>
        </div>

        <label class="flex items-center space-x-2 text-xs text-zinc-300 cursor-pointer pt-1">
          <input type="checkbox" bind:checked={newIsHttps} class="accent-indigo-500 w-3.5 h-3.5" />
          <span>Utiliza HTTPS (SSL/TLS)</span>
        </label>
      </div>

      <div class="flex items-center justify-end space-x-2 pt-2 border-t border-zinc-800 select-none">
        <button
          onclick={() => (isAddModalOpen = false)}
          class="text-xs px-3 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer"
        >
          Cancelar
        </button>
        <button
          onclick={handleSaveManualTarget}
          class="text-xs px-4 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-xs cursor-pointer"
        >
          {editingTargetId ? 'Salvar Alterações' : 'Salvar & Conectar'}
        </button>
      </div>
    </div>
  </div>
{/if}
