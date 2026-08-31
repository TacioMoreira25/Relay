<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { GeneratedCa } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let { isOpen = $bindable(false) }: { isOpen: boolean } = $props();

  let activeTab = $state<"export" | "https">("export");
  let statusMessage = $state<string | null>(null);
  let isGeneratingCa = $state<boolean>(false);
  let caCert = $state<GeneratedCa | null>(null);
  let caCommonName = $state<string>("Relay Root CA Local");

  function downloadJsonFile(content: unknown, filename: string): void {
    const jsonStr = typeof content === "string" ? content : JSON.stringify(content, null, 2);
    const blob = new Blob([jsonStr], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function downloadTextFile(text: string, filename: string): void {
    const blob = new Blob([text], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  async function handleExportHar(): Promise<void> {
    try {
      const harData = await invoke("export_har");
      downloadJsonFile(harData, `relay-traffic-${new Date().toISOString().slice(0, 10)}.har`);
      statusMessage = "Exportação HAR 1.2 concluída com sucesso!";
      setTimeout(() => (statusMessage = null), 3000);
    } catch (e) {
      statusMessage = `Erro ao exportar HAR: ${e}`;
    }
  }

  async function handleExportOpenApi(): Promise<void> {
    try {
      const openApiData = await invoke("export_openapi");
      downloadJsonFile(openApiData, `relay-openapi-${new Date().toISOString().slice(0, 10)}.json`);
      statusMessage = "Especificação OpenAPI 3.0 exportada com sucesso!";
      setTimeout(() => (statusMessage = null), 3000);
    } catch (e) {
      statusMessage = `Erro ao exportar OpenAPI: ${e}`;
    }
  }

  async function handleGenerateCa(): Promise<void> {
    isGeneratingCa = true;
    statusMessage = null;
    try {
      const res = await invoke<GeneratedCa>("create_ca_certificate", {
        commonName: caCommonName.trim() || "Relay Root CA Local",
      });
      caCert = res;
      statusMessage = "Certificado Raiz CA gerado com sucesso!";
      setTimeout(() => (statusMessage = null), 3000);
    } catch (e) {
      statusMessage = `Erro ao gerar certificado CA: ${e}`;
    } finally {
      isGeneratingCa = false;
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-xl w-full p-5 shadow-2xl space-y-4 max-h-[90vh] flex flex-col">
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3 select-none">
        <div class="flex items-center space-x-2">
          <span class="text-sm font-semibold text-zinc-100">Exportação & HTTPS MITM</span>
        </div>
        <button
          onclick={() => (isOpen = false)}
          class="text-zinc-500 hover:text-zinc-300 text-xs p-1"
        >
          ✕
        </button>
      </div>

      <!-- Navigation Tabs -->
      <div class="flex space-x-1 bg-zinc-950 p-1 rounded-lg border border-zinc-800 text-xs select-none">
        <button
          onclick={() => (activeTab = "export")}
          class="flex-1 py-1.5 rounded transition-colors text-center font-medium {activeTab === 'export' ? 'bg-zinc-800 text-white shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          📦 Exportar Tráfego (HAR / OpenAPI)
        </button>
        <button
          onclick={() => (activeTab = "https")}
          class="flex-1 py-1.5 rounded transition-colors text-center font-medium {activeTab === 'https' ? 'bg-zinc-800 text-white shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          🔒 Certificados HTTPS / CA
        </button>
      </div>

      {#if statusMessage}
        <div class="p-2.5 rounded bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 font-mono text-xs">
          {statusMessage}
        </div>
      {/if}

      <!-- Tab 1: Export Content -->
      {#if activeTab === "export"}
        <div class="flex-1 space-y-4 text-xs">
          <div class="p-3.5 bg-zinc-950 border border-zinc-800 rounded-lg space-y-2">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-bold text-zinc-200 text-xs">Arquivo de Tráfego HTTP (HAR 1.2)</h4>
                <p class="text-[11px] text-zinc-400">
                  Compatível com DevTools, Postman, Insomnia, Charles Proxy e ferramentas de QA.
                </p>
              </div>
              <button
                onclick={handleExportHar}
                disabled={relayState.totalRequests === 0}
                class="px-3.5 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-sm disabled:opacity-40 cursor-pointer"
              >
                Exportar HAR
              </button>
            </div>
            <div class="text-[10px] text-zinc-500 font-mono">
              Total de requisições capturadas: {relayState.totalRequests}
            </div>
          </div>

          <div class="p-3.5 bg-zinc-950 border border-zinc-800 rounded-lg space-y-2">
            <div class="flex items-center justify-between">
              <div>
                <h4 class="font-bold text-zinc-200 text-xs">Especificação OpenAPI 3.0 (Swagger)</h4>
                <p class="text-[11px] text-zinc-400">
                  Gera documentação viva de endpoints e métodos REST observados em tempo de execução.
                </p>
              </div>
              <button
                onclick={handleExportOpenApi}
                disabled={relayState.totalRequests === 0}
                class="px-3.5 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 font-medium transition-colors shadow-sm disabled:opacity-40 cursor-pointer border border-zinc-700"
              >
                Exportar OpenAPI
              </button>
            </div>
            <div class="text-[10px] text-zinc-500 font-mono">
              Host alvo: {relayState.config.targetHost}:{relayState.config.targetPort}
            </div>
          </div>
        </div>
      {:else}
        <!-- Tab 2: HTTPS CA Generator -->
        <div class="flex-1 space-y-4 text-xs overflow-y-auto pr-1">
          <div class="p-3 bg-zinc-950 border border-zinc-800 rounded-lg space-y-2">
            <h4 class="font-bold text-zinc-200 text-xs">Autoridade Certificadora Local (Root CA)</h4>
            <p class="text-[11px] text-zinc-400 leading-relaxed">
              Gere um certificado raiz seguro para confiar no Relay como autoridade local e permitir a interceptação transparente de chamadas HTTPS em clientes locais (cURL, browsers, apps).
            </p>

            <div class="space-y-1 pt-1">
              <label class="block text-zinc-400 font-medium text-[11px]" for="caName">
                Common Name (CN) da Autoridade
              </label>
              <div class="flex space-x-2">
                <input
                  id="caName"
                  type="text"
                  bind:value={caCommonName}
                  class="flex-1 bg-zinc-900 border border-zinc-800 rounded px-2.5 py-1.5 text-xs font-mono text-zinc-200 focus:outline-none focus:border-indigo-500"
                />
                <button
                  onclick={handleGenerateCa}
                  disabled={isGeneratingCa}
                  class="px-3.5 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-sm disabled:opacity-50 cursor-pointer"
                >
                  {isGeneratingCa ? "Gerando..." : "Gerar CA"}
                </button>
              </div>
            </div>
          </div>

          {#if caCert}
            <div class="space-y-3 p-3 bg-zinc-950 border border-zinc-800 rounded-lg">
              <div class="flex items-center justify-between">
                <span class="font-bold text-zinc-200 text-xs">Certificado Raiz Gerado</span>
                <div class="flex space-x-2">
                  <button
                    onclick={() => downloadTextFile(caCert!.certPem, "relay-root-ca.crt")}
                    class="text-[11px] px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors"
                  >
                    Baixar .CRT
                  </button>
                  <button
                    onclick={() => downloadTextFile(caCert!.keyPem, "relay-root-ca.key")}
                    class="text-[11px] px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-rose-300 transition-colors"
                  >
                    Baixar .KEY
                  </button>
                </div>
              </div>

              <pre class="p-2.5 bg-zinc-900 border border-zinc-800 rounded text-[10px] font-mono text-zinc-400 max-h-32 overflow-y-auto leading-tight select-all">{caCert.certPem}</pre>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Footer -->
      <div class="flex items-center justify-end pt-3 border-t border-zinc-800 select-none">
        <button
          onclick={() => (isOpen = false)}
          class="text-xs px-4 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors"
        >
          Fechar
        </button>
      </div>
    </div>
  </div>
{/if}
