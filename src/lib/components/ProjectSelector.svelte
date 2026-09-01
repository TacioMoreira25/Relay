<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import { IconFolder, IconPlus, IconTrash, IconPencil } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";

  let {
    onOpenCreate = () => {},
    onOpenEdit = (_id: string) => {},
  }: {
    onOpenCreate?: () => void;
    onOpenEdit?: (id: string) => void;
  } = $props();

  let isOpen = $state<boolean>(false);

  async function handleSwitch(id: string): Promise<void> {
    relayState.switchProject(id);
    try {
      await invoke("update_proxy_config", { config: relayState.config });
    } catch (e) {
      console.warn("Falha ao sincronizar config do projeto:", e);
    }
    isOpen = false;
  }

  function handleDelete(id: string, e: MouseEvent): void {
    e.stopPropagation();
    if (confirm("Deseja realmente excluir este projeto e todas as suas rotas e templates?")) {
      relayState.deleteProject(id);
    }
  }
</script>

<div class="relative inline-block text-left">
  <!-- Botão Gatilho na TopBar -->
  <button
    onclick={() => (isOpen = !isOpen)}
    class="flex items-center space-x-2 bg-zinc-950/80 hover:bg-zinc-900 border border-zinc-800/80 px-2.5 py-1.5 rounded-lg text-xs font-mono text-zinc-300 transition-all cursor-pointer shadow-xs"
    title="Alternar ou gerenciar projetos de APIs"
  >
    <IconFolder size={13} class="text-indigo-400 shrink-0" />
    <span class="font-semibold text-zinc-100 max-w-[120px] truncate">{relayState.activeProject.name}</span>
    <span class="text-zinc-500 text-[9px]">▼</span>
  </button>

  <!-- Dropdown de Projetos -->
  {#if isOpen}
    <div
      role="menu"
      tabindex="0"
      class="absolute left-0 top-10 w-72 bg-zinc-900/95 backdrop-blur-md border border-zinc-800 rounded-xl shadow-2xl p-2 z-50 space-y-2 font-mono text-xs text-zinc-200"
      onmouseleave={() => (isOpen = false)}
    >
      <div class="flex items-center justify-between px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-zinc-400 select-none">
        <span>Projetos ({relayState.projects.length})</span>
      </div>

      <div class="space-y-0.5 max-h-48 overflow-y-auto pr-0.5">
        {#each relayState.projects as proj}
          <div
            role="menuitem"
            tabindex="0"
            onclick={() => handleSwitch(proj.id)}
            onkeydown={(e) => { if (e.key === 'Enter') handleSwitch(proj.id); }}
            class="group w-full text-left px-2.5 py-1.5 rounded-lg hover:bg-zinc-800/80 flex items-center justify-between transition-colors cursor-pointer {relayState.activeProjectId === proj.id ? 'bg-zinc-800/90 font-bold border border-zinc-700/60' : ''}"
          >
            <div class="flex items-center space-x-2 truncate flex-1 pr-1">
              <IconFolder size={12} class={relayState.activeProjectId === proj.id ? 'text-indigo-400' : 'text-zinc-400'} />
              <div class="truncate">
                <div class="text-xs text-zinc-200 truncate">{proj.name}</div>
                {#if proj.description}
                  <div class="text-[10px] text-zinc-500 font-sans truncate font-normal">{proj.description}</div>
                {/if}
              </div>
            </div>

            <!-- Ações -->
            <div class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                type="button"
                onclick={(e) => { e.stopPropagation(); isOpen = false; onOpenEdit(proj.id); }}
                class="p-1 text-zinc-400 hover:text-indigo-300 transition-colors"
                title="Editar nome/descrição do projeto"
              >
                <IconPencil size={11} />
              </button>
              {#if relayState.projects.length > 1}
                <button
                  type="button"
                  onclick={(e) => handleDelete(proj.id, e)}
                  class="p-1 text-zinc-400 hover:text-rose-400 transition-colors"
                  title="Excluir projeto"
                >
                  <IconTrash size={11} />
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>

      <!-- Rodapé / Criar Novo Projeto -->
      <div class="pt-1.5 border-t border-zinc-800/80 flex items-center justify-between px-1">
        <button
          onclick={() => { isOpen = false; onOpenCreate(); }}
          class="w-full text-[11px] px-2.5 py-1.5 rounded-lg bg-zinc-800/60 hover:bg-zinc-800 text-zinc-300 hover:text-white transition-colors border border-zinc-700/60 flex items-center justify-center space-x-1.5 cursor-pointer"
        >
          <IconPlus size={12} class="text-indigo-400" />
          <span>Novo Projeto</span>
        </button>
      </div>
    </div>
  {/if}
</div>
