<script lang="ts">
  import { IconCommand, IconTerminal, IconShield, IconActivity, IconFileJson } from "$lib/components/icons";

  let { isOpen = $bindable(false) }: { isOpen: boolean } = $props();

  const shortcuts = [
    { key: "Ctrl + P", desc: "Iniciar / Parar o Proxy HTTP" },
    { key: "Ctrl + N", desc: "Criar e testar nova requisição direta" },
    { key: "Ctrl + K", desc: "Focar no campo de busca de tráfego" },
    { key: "Ctrl + L", desc: "Limpar histórico de requisições" },
    { key: "Ctrl + E", desc: "Exportar HAR / OpenAPI & Certificados" },
    { key: "Ctrl + /", desc: "Abrir este painel de atalhos e dicas" },
  ];

  const tips = [
    {
      title: "Substituto Leve do Postman (1-Click Replay)",
      desc: "Clique em 'Replay' em qualquer chamada para alterar JSONs e testar rotas sem recriá-las.",
      icon: IconTerminal,
      color: "text-sky-400",
    },
    {
      title: "Auto-Injeção e Extração Dinâmica de JWT / IDs",
      desc: "IDs e Tokens gerados em logins ou cadastros são encadeados automaticamente nas próximas chamadas.",
      icon: IconShield,
      color: "text-amber-400",
    },
    {
      title: "Mocks Locais & Chaos Simulator",
      desc: "Simule rotas antes do backend estar pronto ou injete latência e taxas de falha (5xx) na UI.",
      icon: IconActivity,
      color: "text-rose-400",
    },
    {
      title: "Exportação de OpenAPI 3.0 & HAR",
      desc: "Gere especificações Swagger e arquivos HAR 1.2 completos para compartilhar com seu time.",
      icon: IconFileJson,
      color: "text-indigo-400",
    },
  ];
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-lg w-full p-5 shadow-2xl space-y-4 max-h-[90vh] flex flex-col">
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3 select-none">
        <div class="flex items-center space-x-2">
          <IconCommand size={14} class="text-indigo-400" />
          <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-100">
            Guia Rápido & Atalhos
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
      <div class="flex-1 overflow-y-auto space-y-4 pr-1 text-xs">
        <!-- Tabela Limpa de Atalhos -->
        <div class="space-y-2">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none block">
            Atalhos do Teclado
          </span>
          <div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-900/40">
            <table class="w-full text-left text-xs">
              <tbody class="divide-y divide-zinc-800/40">
                {#each shortcuts as s}
                  <tr class="hover:bg-zinc-900/60 transition-colors">
                    <td class="py-2 px-3 text-zinc-300 text-[11px]">{s.desc}</td>
                    <td class="py-2 px-3 text-right">
                      <kbd class="px-2 py-0.5 rounded bg-zinc-800 border border-zinc-700 text-zinc-200 font-mono text-[10px] font-semibold">
                        {s.key}
                      </kbd>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>

        <!-- Lista Escaneável de Dicas -->
        <div class="space-y-2 pt-2 border-t border-zinc-800/80">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none block">
            Recursos Principais
          </span>
          <div class="space-y-2">
            {#each tips as t}
              {@const IconComp = t.icon}
              <div class="flex items-start space-x-3 p-2.5 bg-zinc-900/40 border border-zinc-800/60 rounded-lg">
                <div class="p-1.5 rounded-md bg-zinc-800 border border-zinc-700/80 shrink-0 mt-0.5">
                  <IconComp size={13} class={t.color} />
                </div>
                <div class="space-y-0.5">
                  <div class="text-xs font-semibold text-zinc-200">{t.title}</div>
                  <p class="text-[11px] text-zinc-400 leading-relaxed">
                    {t.desc}
                  </p>
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
