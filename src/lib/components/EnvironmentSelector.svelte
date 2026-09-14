<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { TargetEnvironment, DiscoveredTarget } from "$lib/types";
  import { IconPlus, IconTrash, IconPencil, IconActivity } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let isOpen = $state<boolean>(false);
  let isAddModalOpen = $state<boolean>(false);
  let editingTargetId = $state<string | null>(null);
  let isTargetActive = $state<boolean>(false);

  // Form para adicionar/editar alvo manual
  let newName = $state<string>("");
  let newHost = $state<string>("127.0.0.1");
  let newPort = $state<number>(3000);
  let newIsHttps = $state<boolean>(false);

  async function checkTargetStatus(): Promise<void> {
    try {
      const active = await invoke<boolean>("check_target_active", {
        host: relayState.config.targetHost,
        port: relayState.config.targetPort,
      });
      isTargetActive = active;
    } catch {
      const found = relayState.discoveredTargets.find(
        (t) => t.port === relayState.config.targetPort && (t.host === relayState.config.targetHost || t.host === "127.0.0.1")
      );
      isTargetActive = found ? found.isActive : false;
    }
  }

  async function scanPorts(): Promise<void> {
    relayState.isScanningTargets = true;
    try {
      const results = await invoke<DiscoveredTarget[]>("scan_active_targets");
      relayState.discoveredTargets = results;
      await checkTargetStatus();
    } catch (e) {
      console.warn("Falha ao escanear portas locais:", e);
    } finally {
      relayState.isScanningTargets = false;
    }
  }

  function handleToggleOpen(): void {
    isOpen = !isOpen;
    if (isOpen) {
      scanPorts();
    }
  }

  async function selectTarget(target: TargetEnvironment): Promise<void> {
    relayState.selectTarget(target);
    try {
      await invoke("update_proxy_config", { config: relayState.config });
    } catch (e) {
      console.error("Erro ao sincronizar target:", e);
    }
    await checkTargetStatus();
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
  }

  onMount(() => {
    checkTargetStatus();
    scanPorts();
    const interval = setInterval(checkTargetStatus, 4000);
    return () => clearInterval(interval);
  });
</script>

<div class="relative inline-block text-left">
  <!-- Botão Discreto e Objetivo no Header -->
  <button
    onclick={handleToggleOpen}
    class="flex items-center space-x-2 text-xs px-2.5 py-1 rounded-lg bg-zinc-900 hover:bg-zinc-800/90 border border-zinc-800 hover:border-zinc-700 text-zinc-200 transition-all cursor-pointer shadow-xs whitespace-nowrap shrink-0"
    title={isTargetActive ? `Alvo ativo e respondendo em ${relayState.config.targetHost}:${relayState.config.targetPort}` : `Alvo configurado: ${relayState.config.targetHost}:${relayState.config.targetPort}`}
  >
    <div class="flex items-center space-x-1.5">
      <span
        class="w-2 h-2 rounded-full shrink-0 transition-colors {isTargetActive ? 'bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.7)]' : 'bg-zinc-500'}"
      ></span>
      <span class="font-normal text-xs text-zinc-400">Alvo:</span>
    </div>
    <span class="font-mono text-xs font-semibold text-zinc-100">
      {relayState.config.targetHost}:{relayState.config.targetPort}
    </span>
    {#if isTargetActive}
      <span class="text-[9px] px-1.5 py-0.2 rounded font-mono uppercase bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 font-medium">
        online
      </span>
    {/if}
    <span class="text-zinc-500 text-[10px] ml-0.5">▾</span>
  </button>

  <!-- Dropdown de Portas & Ambientes -->
  {#if isOpen}
    <div
      class="fixed inset-0 z-30"
      onclick={() => (isOpen = false)}
      role="presentation"
    ></div>

    <div class="absolute left-0 mt-2 w-80 bg-zinc-900/95 backdrop-blur-md border border-zinc-800 rounded-xl shadow-2xl z-40 p-3 space-y-3 text-xs">
      <!-- Seção: Portas Locais Detectadas -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between text-zinc-400 border-b border-zinc-800 pb-1.5">
          <span class="text-[10px] font-bold uppercase tracking-wider text-zinc-400">APIs Detectadas no Sistema</span>
          <button
            onclick={scanPorts}
            disabled={relayState.isScanningTargets}
            class="text-[11px] text-indigo-400 hover:text-indigo-300 transition-colors flex items-center space-x-1 cursor-pointer disabled:opacity-50"
          >
            <IconActivity size={11} class={relayState.isScanningTargets ? 'animate-spin' : ''} />
            <span>{relayState.isScanningTargets ? 'Escaneando...' : 'Atualizar'}</span>
          </button>
        </div>

        {#if relayState.discoveredTargets.length === 0}
          <div class="px-3 py-2 rounded-lg bg-zinc-950/40 border border-zinc-800/60 text-center space-y-0.5">
            <div class="text-[11px] font-medium text-zinc-300">
              {relayState.isScanningTargets ? 'Buscando portas ativas...' : 'Nenhuma outra API detectada'}
            </div>
            <div class="text-[10px] text-zinc-500">
              Inicie seu serviço (ex: :3000, :4000, :8000) e clique em Atualizar.
            </div>
          </div>
        {:else}
          <div class="space-y-1 max-h-36 overflow-y-auto pr-1">
            {#each relayState.discoveredTargets as disc}
              {@const isSelected = relayState.config.targetPort === disc.port && relayState.config.targetHost === disc.host}
              <button
                onclick={() => handleSelectDiscovered(disc)}
                class="w-full text-left px-2.5 py-1.5 rounded-lg flex items-center justify-between transition-colors cursor-pointer {isSelected ? 'bg-zinc-800 text-zinc-100 font-medium border border-zinc-700' : 'hover:bg-zinc-800/60 text-zinc-300'}"
              >
                <div class="flex items-center space-x-2 truncate">
                  <span class="w-2 h-2 rounded-full shrink-0 {disc.isActive ? 'bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.7)]' : 'bg-zinc-600'}"></span>
                  <span class="text-xs truncate">{disc.label}</span>
                </div>
                <div class="flex items-center space-x-1.5 shrink-0">
                  <span class="font-mono text-[11px] text-zinc-400">:{disc.port}</span>
                  {#if isSelected}
                    <span class="text-emerald-400 text-xs font-bold ml-1">✓</span>
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Seção: Ambientes Salvos (ex: Staging / Docker) -->
      <div class="space-y-1.5 pt-1 border-t border-zinc-800/80">
        <div class="flex items-center justify-between text-zinc-400 pb-1">
          <span class="text-[10px] font-bold uppercase tracking-wider text-zinc-400">Ambientes Salvos</span>
        </div>

        {#if relayState.savedEnvironments.length === 0}
          <div class="p-2 text-zinc-500 text-[10px] italic text-center">
            Nenhum ambiente salvo.
          </div>
        {:else}
          <div class="space-y-1 max-h-28 overflow-y-auto pr-1">
            {#each relayState.savedEnvironments as env}
              {@const isSelected = relayState.config.targetPort === env.port && relayState.config.targetHost === env.host}
              <div
                onclick={() => selectTarget(env)}
                role="button"
                tabindex="0"
                onkeydown={(e) => { if (e.key === "Enter") selectTarget(env); }}
                class="group w-full text-left px-2.5 py-1.5 rounded-lg flex items-center justify-between transition-colors cursor-pointer {isSelected ? 'bg-zinc-800 text-zinc-100 font-medium border border-zinc-700' : 'hover:bg-zinc-800/60 text-zinc-300'}"
              >
                <div class="flex items-center space-x-2 truncate">
                  <span class="w-1.5 h-1.5 rounded-full shrink-0 {isSelected ? 'bg-indigo-400' : 'bg-zinc-600'}"></span>
                  <span class="text-xs truncate">{env.name}</span>
                  {#if env.name !== `${env.host}:${env.port}` && env.name !== `Localhost :${env.port}`}
                    <span class="font-mono text-[10px] text-zinc-500 truncate">({env.host}:{env.port})</span>
                  {/if}
                </div>
                <div class="flex items-center space-x-1.5 font-mono text-[10px] text-zinc-400 shrink-0">
                  {#if isSelected}
                    <span class="text-emerald-400 text-xs font-bold mr-1">✓</span>
                  {/if}
                  <button
                    onclick={(e) => openEditModal(env, e)}
                    class="opacity-0 group-hover:opacity-100 hover:text-zinc-200 p-0.5 cursor-pointer"
                    title="Editar"
                  >
                    <IconPencil size={11} />
                  </button>
                  <button
                    onclick={(e) => handleDeleteTarget(env.id, e)}
                    class="opacity-0 group-hover:opacity-100 hover:text-rose-400 p-0.5 cursor-pointer"
                    title="Excluir"
                  >
                    <IconTrash size={11} />
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        <button
          onclick={openCreateModal}
          class="w-full mt-2 text-xs py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700/80 text-zinc-200 hover:text-white border border-zinc-700 transition-colors flex items-center justify-center space-x-1.5 cursor-pointer font-medium"
        >
          <IconPlus size={13} class="text-indigo-400 shrink-0" />
          <span>Adicionar Alvo Manual</span>
        </button>
      </div>
    </div>
  {/if}
</div>

<!-- Modal Adicionar / Editar Alvo Manual -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-sm w-full p-4 shadow-2xl space-y-3.5 text-xs">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-2 select-none">
        <h4 class="font-bold text-xs uppercase tracking-wider text-zinc-100">
          {editingTargetId ? 'Editar Alvo da API' : 'Novo Alvo da API'}
        </h4>
        <button onclick={() => (isAddModalOpen = false)} class="text-zinc-500 hover:text-zinc-300 text-xs p-1 cursor-pointer">
          ✕
        </button>
      </div>

      <div class="space-y-2.5">
        <div>
          <label class="block text-[11px] text-zinc-400 mb-1" for="env-name">Nome do Ambiente</label>
          <input
            id="env-name"
            type="text"
            bind:value={newName}
            placeholder="Ex: Backend Docker, Staging"
            class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 focus:outline-none focus:border-indigo-500"
          />
        </div>

        <div class="grid grid-cols-3 gap-2">
          <div class="col-span-2">
            <label class="block text-[11px] text-zinc-400 mb-1" for="env-host">Host / IP</label>
            <input
              id="env-host"
              type="text"
              bind:value={newHost}
              placeholder="127.0.0.1"
              class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label class="block text-[11px] text-zinc-400 mb-1" for="env-port">Porta</label>
            <input
              id="env-port"
              type="number"
              bind:value={newPort}
              placeholder="3000"
              class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 font-mono focus:outline-none focus:border-indigo-500"
            />
          </div>
        </div>
      </div>

      <div class="flex items-center justify-end space-x-2 pt-2 border-t border-zinc-800 select-none">
        <button
          onclick={() => (isAddModalOpen = false)}
          class="px-3 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 cursor-pointer"
        >
          Cancelar
        </button>
        <button
          onclick={handleSaveManualTarget}
          class="px-3.5 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium cursor-pointer"
        >
          Salvar
        </button>
      </div>
    </div>
  </div>
{/if}
