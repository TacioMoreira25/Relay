<script lang="ts">
  import type { HeaderEntry } from "$lib/types";

  let { headers = $bindable([]) }: { headers: HeaderEntry[] } = $props();

  function addHeader(): void {
    headers.push({ key: "", value: "" });
  }

  function removeHeader(index: number): void {
    headers.splice(index, 1);
  }
</script>

<div class="space-y-2">
  <div class="flex items-center justify-between">
    <span class="text-xs font-semibold uppercase tracking-wider text-zinc-400">Headers Customizados</span>
    <button
      type="button"
      class="text-xs px-2 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors"
      onclick={addHeader}
    >
      + Adicionar
    </button>
  </div>

  <div class="space-y-1.5">
    {#each headers as header, index}
      <div class="flex items-center space-x-2">
        <input
          type="text"
          placeholder="Chave (ex: Authorization)"
          bind:value={header.key}
          class="flex-1 bg-zinc-900 border border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-indigo-500"
        />
        <input
          type="text"
          placeholder="Valor (ex: Bearer eyJ...)"
          bind:value={header.value}
          class="flex-1 bg-zinc-900 border border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-indigo-500"
        />
        <button
          type="button"
          class="text-xs text-zinc-500 hover:text-rose-400 p-1"
          onclick={() => removeHeader(index)}
        >
          ✕
        </button>
      </div>
    {/each}
  </div>
</div>
