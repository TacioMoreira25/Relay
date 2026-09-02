<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import { IconCommand, IconTerminal, IconShield, IconActivity, IconFileJson } from "$lib/components/icons";

  let { isOpen = $bindable(false) }: { isOpen: boolean } = $props();

  const shortcuts = [
    { key: "Ctrl + P", desc: `Iniciar / Parar o Proxy HTTP (:${relayState.config.listenPort})` },
    { key: "Ctrl + N", desc: "Abrir disparador de nova requisição direta" },
    { key: "Ctrl + K", desc: "Focar no campo de busca de tráfego" },
    { key: "Ctrl + L", desc: "Limpar histórico de requisições" },
    { key: "Ctrl + E", desc: "Exportar HAR / OpenAPI & Certificados" },
    { key: "Ctrl + /", desc: "Abrir este painel de atalhos e dicas" },
  ];

  const tips = [
    {
      title: "O que é o Relay?",
      desc: "Um proxy reverso inteligente e inspetor de tráfego local. Ele fica entre seu frontend e sua API para capturar, testar e auditar requisições em tempo real sem precisar do Postman.",
      icon: IconTerminal,
      color: "text-sky-400",
    },
    {
      title: "Como usar no Frontend?",
      desc: `Basta mudar a URL base da sua API no frontend de 'http://localhost:3000' para 'http://localhost:${relayState.config.listenPort}'. Todas as chamadas, logins e dados passarão automaticamente pelo Relay.`,
      icon: IconFileJson,
      color: "text-emerald-400",
    },
    {
      title: "Auto-Captura & Injeção de JWT",
      desc: "Ao fazer login no seu app, o Relay detecta o Bearer Token, decodifica suas permissões (claims) e permite auto-injetar esse token em qualquer requisição com 1 clique.",
      icon: IconShield,
      color: "text-amber-400",
    },
    {
      title: "Simulador de Rede & Falhas (Chaos Testing)",
      desc: "Clique em 'Simular Rede' no topo para testar como o seu frontend reage a conexões lentas (3G) ou quando o backend cai com erros 500/503.",
      icon: IconActivity,
      color: "text-rose-400",
    },
  ];
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-lg w-full p-5 shadow-2xl space-y-4 max-h-[90vh] flex flex-col">
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3 select-none">
        <div class="flex items-center space-x-2">
          <IconCommand size={14} class="text-indigo-400" />
          <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-100">
            Guia Rápido & Dicas do Relay
          </h3>
        </div>
        <button
          onclick={() => (isOpen = false)}
          class="text-zinc-500 hover:text-zinc-300 text-xs p-1 cursor-pointer transition-colors"
        >
          ✕
        </button>
      </div>

      <!-- Modal Body -->
      <div class="flex-1 overflow-y-auto space-y-4 pr-1 text-xs">
        <!-- Lista Didática de Recursos -->
        <div class="space-y-2">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none block">
            Como Tirar Proveito do Relay
          </span>
          <div class="space-y-2.5">
            {#each tips as t}
              {@const IconComp = t.icon}
              <div class="flex items-start space-x-3 p-3 bg-zinc-950/80 border border-zinc-800/80 rounded-lg">
                <div class="p-2 rounded-md bg-zinc-900 border border-zinc-700/80 shrink-0 mt-0.5">
                  <IconComp size={14} class={t.color} />
                </div>
                <div class="space-y-1">
                  <div class="text-xs font-bold text-zinc-100">{t.title}</div>
                  <div class="text-[11px] text-zinc-400 leading-relaxed">{t.desc}</div>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Tabela Limpa de Atalhos -->
        <div class="space-y-2 pt-2 border-t border-zinc-800/80">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none block">
            Atalhos do Teclado
          </span>
          <div class="border border-zinc-800/80 rounded-lg overflow-hidden bg-zinc-950/50">
            <table class="w-full text-left text-xs">
              <tbody class="divide-y divide-zinc-800/40">
                {#each shortcuts as s}
                  <tr class="hover:bg-zinc-900/40 transition-colors">
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
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-end pt-2 border-t border-zinc-800 select-none">
        <button
          onclick={() => (isOpen = false)}
          class="text-xs px-4 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 font-medium transition-colors cursor-pointer"
        >
          Entendido
        </button>
      </div>
    </div>
  </div>
{/if}
