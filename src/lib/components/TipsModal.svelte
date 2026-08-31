<script lang="ts">
  import { IconCommand, IconTerminal, IconShield, IconActivity, IconFileJson } from "$lib/components/icons";

  let { isOpen = $bindable(false) }: { isOpen: boolean } = $props();

  const shortcuts = [
    { key: "Ctrl + P", desc: "Iniciar / Parar o Proxy HTTP" },
    { key: "Ctrl + K", desc: "Focar e filtrar campo de busca de tráfego" },
    { key: "Ctrl + L", desc: "Limpar histórico de requisições capturadas" },
    { key: "Ctrl + E", desc: "Abrir modal de Exportação (HAR / OpenAPI) & Certificados HTTPS" },
    { key: "Ctrl + /", desc: "Abrir este painel de atalhos e dicas" },
  ];

  const tips = [
    {
      title: "Substituto Leve do Postman (1-Click Replay)",
      desc: "Clique em qualquer chamada na lista lateral e depois em 'Replay' para abrir o editor de parâmetros, alterar JSONs e reenviar requisições instantaneamente sem recriá-las do zero.",
      icon: IconTerminal,
    },
    {
      title: "Auto-Injeção e Decodificação de JWT",
      desc: "Faça login no seu app web ou mobile e o Relay capturará os tokens JWT automaticamente. No modal de Replay, clique em 'Auto-Injetar JWT' para autenticar chamadas sem copiar e colar tokens.",
      icon: IconShield,
    },
    {
      title: "Testes de Resiliência & Loading States (Chaos)",
      desc: "No menu de configurações, defina 'Latência: 500ms ± 100ms' para testar skeletons/spinners de carregamento na UI ou injete 'Taxa de Falhas: 20% (503)' para testar tratamentos de erro no frontend.",
      icon: IconActivity,
    },
    {
      title: "Exportação de Documentação OpenAPI e HAR",
      desc: "Pressione Ctrl+E para exportar sua sessão em formato OpenAPI 3.0 (Swagger) para documentação automática de endpoints ou em HAR 1.2 para compartilhar bugs com o time de QA.",
      icon: IconFileJson,
    },
  ];
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-xl w-full p-5 shadow-2xl space-y-4 max-h-[90vh] flex flex-col">
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3 select-none">
        <div class="flex items-center space-x-2">
          <IconCommand size={14} class="text-indigo-400" />
          <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-100">
            Guia Rápido & Atalhos de Teclado
          </h3>
        </div>
        <button
          onclick={() => (isOpen = false)}
          class="text-zinc-500 hover:text-zinc-300 text-xs p-1 cursor-pointer"
        >
          ✕
        </button>
      </div>

      <!-- Modal Body -->
      <div class="flex-1 overflow-y-auto space-y-5 pr-1 text-xs">
        <!-- Atalhos de Teclado -->
        <div class="space-y-2">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none block">
            Atalhos do Teclado
          </span>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            {#each shortcuts as s}
              <div class="flex items-center justify-between bg-zinc-950 p-2.5 rounded-lg border border-zinc-800/80">
                <span class="text-zinc-400 text-[11px]">{s.desc}</span>
                <kbd class="px-2 py-0.5 rounded bg-zinc-800 border border-zinc-700 text-zinc-200 font-mono text-[10px] font-semibold shrink-0 ml-2">
                  {s.key}
                </kbd>
              </div>
            {/each}
          </div>
        </div>

        <!-- Dicas de Uso Diário -->
        <div class="space-y-2.5 pt-2 border-t border-zinc-800/80">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none block">
            Dicas para o Fluxo Diário Fullstack
          </span>
          <div class="space-y-2">
            {#each tips as t}
              {@const IconComp = t.icon}
              <div class="p-3 bg-zinc-950 border border-zinc-800/80 rounded-lg space-y-1">
                <div class="flex items-center space-x-2 text-zinc-200 font-semibold text-xs">
                  <IconComp size={13} class="text-indigo-400 shrink-0" />
                  <span>{t.title}</span>
                </div>
                <p class="text-[11px] text-zinc-400 leading-relaxed pl-5">
                  {t.desc}
                </p>
              </div>
            {/each}
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end pt-3 border-t border-zinc-800 select-none">
        <button
          onclick={() => (isOpen = false)}
          class="text-xs px-4 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer"
        >
          Entendi
        </button>
      </div>
    </div>
  </div>
{/if}
