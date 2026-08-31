<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { GeneratedCa } from "$lib/types";
  import { IconFileJson, IconShield, IconDownload, IconKey, IconCheck } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";

  let { isOpen = $bindable(false) }: { isOpen: boolean } = $props();

  let activeTab = $state<"export" | "https">("export");
  let statusMessage = $state<string | null>(null);
  let isGeneratingCa = $state<boolean>(false);
  let caCert = $state<GeneratedCa | null>(null);
  let caCommonName = $state<string>("Relay Root CA Local");
  let caInstallFeedback = $state<string | null>(null);

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

  async function copyInstallCommand(): Promise<void> {
    const cmd = "sudo cp relay-root-ca.crt /etc/pki/ca-trust/source/anchors/ && sudo update-ca-trust";
    try {
      await navigator.clipboard.writeText(cmd);
      caInstallFeedback = "Comando copiado!";
      setTimeout(() => (caInstallFeedback = null), 2500);
    } catch (e) {
      console.error(e);
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-xl w-full p-5 shadow-2xl space-y-4 max-h-[90vh] flex flex-col">
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 pb-3 select-none">
        <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-100">
          Exportação & HTTPS MITM
        </h3>
        <button
          onclick={() => (isOpen = false)}
          class="text-zinc-500 hover:text-zinc-300 text-xs p-1 cursor-pointer"
        >
          ✕
        </button>
      </div>

      <!-- Navigation Tabs -->
      <div class="flex space-x-1 bg-zinc-950 p-1 rounded-lg border border-zinc-800 text-xs select-none">
        <button
          onclick={() => (activeTab = "export")}
          class="flex-1 py-1.5 rounded-md transition-colors text-center font-medium flex items-center justify-center space-x-1.5 cursor-pointer {activeTab === 'export' ? 'bg-zinc-800 text-white shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <IconFileJson size={13} />
          <span>Exportar Tráfego (HAR / OpenAPI)</span>
        </button>
        <button
          onclick={() => (activeTab = "https")}
          class="flex-1 py-1.5 rounded-md transition-colors text-center font-medium flex items-center justify-center space-x-1.5 cursor-pointer {activeTab === 'https' ? 'bg-zinc-800 text-white shadow-xs' : 'text-zinc-400 hover:text-zinc-200'}"
        >
          <IconShield size={13} />
          <span>Certificados HTTPS / CA</span>
        </button>
      </div>

      {#if statusMessage}
        <div class="p-2.5 rounded bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 font-mono text-xs">
          {statusMessage}
        </div>
      {/if}

      <!-- Tab 1: Export Content -->
      {#if activeTab === "export"}
        <div class="flex-1 space-y-3 text-xs">
          <!-- Card HAR -->
          <div class="p-4 bg-zinc-900/60 border border-zinc-800 rounded-lg space-y-3">
            <div class="flex items-center justify-between">
              <div class="space-y-0.5">
                <div class="flex items-center space-x-2">
                  <h4 class="font-bold text-zinc-200 text-xs">Arquivo de Tráfego HTTP</h4>
                  <span class="text-[10px] font-mono px-1.5 py-0.2 rounded bg-indigo-500/10 text-indigo-300 border border-indigo-500/20">.har</span>
                </div>
                <p class="text-[11px] text-zinc-400">
                  Compatível com DevTools, Postman, Insomnia, Charles Proxy e ferramentas de QA.
                </p>
              </div>
              <button
                onclick={handleExportHar}
                disabled={relayState.totalRequests === 0}
                class="px-3 py-1.5 rounded-md bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 font-medium transition-colors shadow-xs disabled:opacity-40 cursor-pointer flex items-center space-x-1.5 shrink-0"
              >
                <IconDownload size={12} />
                <span>Exportar HAR</span>
              </button>
            </div>
            <div class="text-[10px] text-zinc-500 font-mono">
              Total de requisições prontas: {relayState.totalRequests}
            </div>
          </div>

          <!-- Card OpenAPI -->
          <div class="p-4 bg-zinc-900/60 border border-zinc-800 rounded-lg space-y-3">
            <div class="flex items-center justify-between">
              <div class="space-y-0.5">
                <div class="flex items-center space-x-2">
                  <h4 class="font-bold text-zinc-200 text-xs">Especificação OpenAPI 3.0</h4>
                  <span class="text-[10px] font-mono px-1.5 py-0.2 rounded bg-amber-500/10 text-amber-300 border border-amber-500/20">.json (Swagger)</span>
                </div>
                <p class="text-[11px] text-zinc-400">
                  Gera documentação viva de endpoints REST observados em tempo de execução.
                </p>
              </div>
              <button
                onclick={handleExportOpenApi}
                disabled={relayState.totalRequests === 0}
                class="px-3 py-1.5 rounded-md bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 font-medium transition-colors shadow-xs disabled:opacity-40 cursor-pointer flex items-center space-x-1.5 shrink-0"
              >
                <IconFileJson size={12} />
                <span>Exportar OpenAPI</span>
              </button>
            </div>
            <div class="text-[10px] text-zinc-500 font-mono">
              Host alvo configurado: {relayState.config.targetHost}:{relayState.config.targetPort}
            </div>
          </div>
        </div>
      {:else}
        <!-- Tab 2: HTTPS CA Generator -->
        <div class="flex-1 space-y-3 text-xs overflow-y-auto pr-1">
          <div class="p-4 bg-zinc-900/60 border border-zinc-800 rounded-lg space-y-3">
            <div class="space-y-1">
              <h4 class="font-bold text-zinc-200 text-xs">Autoridade Certificadora Local (Root CA)</h4>
              <p class="text-[11px] text-zinc-400 leading-relaxed">
                Gere um certificado raiz seguro para confiar no Relay como autoridade local e permitir interceptação de chamadas HTTPS locais.
              </p>
            </div>

            <div class="space-y-1.5 pt-1">
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
                  class="px-3.5 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-xs disabled:opacity-50 cursor-pointer flex items-center space-x-1.5 shrink-0"
                >
                  <IconKey size={12} />
                  <span>{isGeneratingCa ? "Gerando..." : "Gerar CA"}</span>
                </button>
              </div>
            </div>
          </div>

          {#if caCert}
            <div class="space-y-3 p-4 bg-zinc-900/60 border border-zinc-800 rounded-lg">
              <div class="flex items-center justify-between">
                <div class="flex items-center space-x-2">
                  <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
                  <span class="font-bold text-zinc-200 text-xs">Certificado Pronto</span>
                </div>
                <div class="flex space-x-2">
                  <button
                    onclick={() => downloadTextFile(caCert!.certPem, "relay-root-ca.crt")}
                    class="text-[11px] px-2 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors cursor-pointer border border-zinc-700"
                  >
                    Baixar .CRT
                  </button>
                  <button
                    onclick={() => downloadTextFile(caCert!.keyPem, "relay-root-ca.key")}
                    class="text-[11px] px-2 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors cursor-pointer border border-zinc-700"
                  >
                    Baixar .KEY
                  </button>
                </div>
              </div>

              <!-- 1-Click Fedora Trust Helper -->
              <div class="bg-zinc-950 p-2.5 rounded border border-zinc-800 flex items-center justify-between text-xs font-mono">
                <div class="truncate text-zinc-400 text-[11px]">
                  <span>Instalar no Fedora / Linux:</span>
                  <code class="text-sky-300 ml-1">update-ca-trust</code>
                </div>
                <button
                  onclick={copyInstallCommand}
                  class="text-[10px] text-indigo-400 hover:text-indigo-300 px-2 py-0.5 rounded bg-zinc-900 border border-zinc-700 flex items-center space-x-1 cursor-pointer shrink-0 ml-2"
                >
                  {#if caInstallFeedback}
                    <IconCheck size={11} class="text-emerald-400" />
                    <span class="text-emerald-400">Copiado</span>
                  {:else}
                    <IconDownload size={11} />
                    <span>Copiar Comando</span>
                  {/if}
                </button>
              </div>

              <pre class="p-2 bg-zinc-950 border border-zinc-800/80 rounded text-[9px] font-mono text-zinc-400 max-h-24 overflow-y-auto leading-tight select-all">{caCert.certPem}</pre>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Footer -->
      <div class="flex items-center justify-end pt-3 border-t border-zinc-800 select-none">
        <button
          onclick={() => (isOpen = false)}
          class="text-xs px-4 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors cursor-pointer"
        >
          Fechar
        </button>
      </div>
    </div>
  </div>
{/if}
