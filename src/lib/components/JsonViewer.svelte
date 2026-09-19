<script lang="ts">
  import { IconCopy, IconCheck } from "$lib/components/icons";

  let {
    code = ""
  }: {
    code: string;
  } = $props();

  let copyFeedback = $state(false);
  let isMinified = $state(false);
  let isCollapsed = $state(false);

  function handleCopy(): void {
    navigator.clipboard.writeText(code);
    copyFeedback = true;
    setTimeout(() => (copyFeedback = false), 2000);
  }

  function highlightJson(jsonStr: string): string {
    if (!jsonStr || !jsonStr.trim()) return "";
    try {
      let parsed = JSON.parse(jsonStr);
      let formatted = isMinified ? JSON.stringify(parsed) : JSON.stringify(parsed, null, 2);
      
      // Escape HTML
      let html = formatted
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");

      return html.replace(
        /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g,
        (match) => {
          let cls = "text-amber-300"; // número
          if (/^"/.test(match)) {
            if (/:$/.test(match)) {
              cls = "text-indigo-300 font-semibold"; // chave JSON
            } else {
              cls = "text-emerald-300"; // valor string
            }
          } else if (/true|false/.test(match)) {
            cls = "text-rose-400 font-semibold"; // booleano
          } else if (/null/.test(match)) {
            cls = "text-zinc-500 italic"; // null
          }
          return `<span class="${cls}">${match}</span>`;
        }
      );
    } catch {
      return jsonStr;
    }
  }

  let highlightedHtml = $derived(highlightJson(code));
</script>

<div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-950/70 shadow-inner">
  <!-- Toolbar Superior do Visualizador JSON -->
  <div class="px-3 py-1.5 bg-zinc-900/90 border-b border-zinc-800/80 flex items-center justify-between text-xs select-none">
    <div class="flex items-center space-x-2">
      <span class="text-[9px] px-1.5 py-0.2 rounded bg-indigo-500/10 text-indigo-300 border border-indigo-500/20 font-mono font-bold">
        JSON
      </span>
      <span class="text-[10px] text-zinc-500 font-mono">{code.length} bytes</span>
    </div>

    <div class="flex items-center space-x-1 font-mono text-[10px]">
      <button
        onclick={() => (isMinified = !isMinified)}
        class="px-2 py-0.5 rounded border transition-colors cursor-pointer {isMinified ? 'bg-indigo-600/30 text-indigo-300 border-indigo-500/40' : 'bg-zinc-800 hover:bg-zinc-700 text-zinc-400 border-zinc-700'}"
        title={isMinified ? "Expandir JSON formatado" : "Minificar JSON em 1 linha"}
      >
        {isMinified ? "Expandir" : "Minificar"}
      </button>

      <button
        onclick={() => (isCollapsed = !isCollapsed)}
        class="px-2 py-0.5 rounded border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-400 transition-colors cursor-pointer"
        title={isCollapsed ? "Mostrar Payload" : "Recolher Payload"}
      >
        {isCollapsed ? "Mostrar" : "Recolher"}
      </button>

      <button
        onclick={handleCopy}
        class="px-2 py-0.5 rounded border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white transition-colors flex items-center space-x-1 cursor-pointer"
      >
        {#if copyFeedback}
          <IconCheck size={11} class="text-emerald-400" />
          <span class="text-emerald-400">Copiado</span>
        {:else}
          <IconCopy size={11} />
          <span>Copiar</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Conteúdo de Código com Realce -->
  {#if !isCollapsed}
    <div class="p-3 overflow-x-auto font-mono text-xs leading-relaxed select-text max-h-[450px]">
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      <pre class="whitespace-pre-wrap break-all">{@html highlightedHtml}</pre>
    </div>
  {/if}
</div>
