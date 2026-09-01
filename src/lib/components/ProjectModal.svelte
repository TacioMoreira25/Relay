<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let {
    isOpen = $bindable(false),
    projectId = $bindable<string | null>(null),
  }: {
    isOpen: boolean;
    projectId?: string | null;
  } = $props();

  let formName = $state<string>("");
  let formDesc = $state<string>("");

  $effect(() => {
    if (isOpen) {
      if (projectId) {
        const p = relayState.projects.find(proj => proj.id === projectId);
        formName = p?.name || "";
        formDesc = p?.description || "";
      } else {
        formName = "";
        formDesc = "";
      }
    }
  });

  async function handleSave(): Promise<void> {
    if (!formName.trim()) return;

    if (projectId) {
      relayState.updateProject(projectId, formName, formDesc);
    } else {
      relayState.createProject(formName, formDesc);
      try {
        await invoke("update_proxy_config", { config: relayState.config });
      } catch (e) {
        console.warn("Erro ao sincronizar novo projeto:", e);
      }
    }

    isOpen = false;
    projectId = null;
    formName = "";
    formDesc = "";
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-[100] p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-sm w-full p-5 shadow-2xl space-y-4 flex flex-col font-sans">
      <div class="flex items-center justify-between border-b border-zinc-800 pb-2 select-none">
        <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-100">
          {projectId ? 'Editar Projeto' : 'Criar Novo Projeto'}
        </h3>
        <button onclick={() => (isOpen = false)} class="text-zinc-500 hover:text-zinc-300 text-xs p-1 cursor-pointer">
          ✕
        </button>
      </div>

      <div class="space-y-3 text-xs">
        <div>
          <label class="block text-zinc-400 font-medium mb-1" for="projModalName">Nome do Projeto</label>
          <input
            id="projModalName"
            type="text"
            placeholder="Ex: Banco App, E-commerce, Auth Microservice"
            bind:value={formName}
            class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 focus:outline-none focus:border-indigo-500 text-xs font-mono"
          />
        </div>

        <div>
          <label class="block text-zinc-400 font-medium mb-1" for="projModalDesc">Descrição (Opcional)</label>
          <input
            id="projModalDesc"
            type="text"
            placeholder="Ex: Rotas e mocks do ecossistema de pagamentos"
            bind:value={formDesc}
            class="w-full bg-zinc-950 border border-zinc-800 rounded px-2.5 py-1.5 text-zinc-200 focus:outline-none focus:border-indigo-500 text-xs font-sans"
          />
        </div>
      </div>

      <div class="flex items-center justify-end space-x-2 pt-2 border-t border-zinc-800 select-none">
        <button
          onclick={() => (isOpen = false)}
          class="text-xs px-3 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer"
        >
          Cancelar
        </button>
        <button
          onclick={handleSave}
          class="text-xs px-4 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-xs cursor-pointer"
        >
          {projectId ? 'Salvar Alterações' : 'Criar Projeto'}
        </button>
      </div>
    </div>
  </div>
{/if}
