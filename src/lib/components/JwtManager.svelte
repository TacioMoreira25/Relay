<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { ExtractedJwt } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let copyFeedback = $state<string | null>(null);

  async function copyToClipboard(text: string, id: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(text);
      copyFeedback = id;
      setTimeout(() => {
        if (copyFeedback === id) copyFeedback = null;
      }, 2000);
    } catch (e) {
      console.error("Falha ao copiar token:", e);
    }
  }

  async function clearAllJwts(): Promise<void> {
    try {
      await invoke("clear_session_jwts");
      relayState.clearJwts();
    } catch (e) {
      console.error("Erro ao limpar tokens:", e);
    }
  }

  function isExpired(expiresAt?: number): boolean {
    if (!expiresAt) return false;
    // expiresAt geralmente vem em segundos
    const nowSec = Math.floor(Date.now() / 1000);
    return expiresAt < nowSec;
  }

  function formatExpiration(expiresAt?: number): string {
    if (!expiresAt) return "Sem expiração definida";
    const nowSec = Math.floor(Date.now() / 1000);
    const diff = expiresAt - nowSec;

    if (diff < 0) {
      return `Expirado há ${Math.abs(Math.floor(diff / 60))} min`;
    }
    if (diff < 60) {
      return `Expira em ${diff}s`;
    }
    if (diff < 3600) {
      return `Expira em ${Math.floor(diff / 60)} min`;
    }
    return `Expira em ${Math.floor(diff / 3600)}h ${Math.floor((diff % 3600) / 60)}m`;
  }
</script>

<div class="flex h-full w-full bg-zinc-950 text-zinc-200 overflow-hidden">
  <!-- Left Column: Token List -->
  <div class="w-80 border-r border-zinc-800 bg-zinc-900 flex flex-col h-full select-none">
    <div class="p-3 border-b border-zinc-800 flex items-center justify-between">
      <div class="flex items-center space-x-2">
        <span class="text-xs font-bold uppercase tracking-wider text-zinc-400">Tokens JWT Detectados</span>
        <span class="text-xs px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-300 font-mono">
          {relayState.totalJwts}
        </span>
      </div>
      {#if relayState.totalJwts > 0}
        <button
          onclick={clearAllJwts}
          class="text-xs text-zinc-400 hover:text-zinc-200 transition-colors px-2 py-1 rounded bg-zinc-800/60 hover:bg-zinc-800"
        >
          Limpar
        </button>
      {/if}
    </div>

    <div class="flex-1 overflow-y-auto divide-y divide-zinc-800/50">
      {#if relayState.jwts.length === 0}
        <div class="p-8 text-center text-zinc-500 text-sm">
          Nenhum token JWT interceptado ainda.<br />
          <span class="text-xs text-zinc-600 mt-1 block">
            Tokens enviados em cabeçalhos de autenticação ou retornados em respostas de login serão auto-capturados aqui.
          </span>
        </div>
      {:else}
        {#each relayState.jwts as jwt (jwt.token)}
          {@const expired = isExpired(jwt.expiresAt)}
          <button
            type="button"
            class="w-full text-left p-3 hover:bg-zinc-800/40 transition-colors flex flex-col space-y-1.5 {relayState.selectedJwt?.token === jwt.token ? 'bg-zinc-800/80 border-l-2 border-indigo-500' : ''}"
            onclick={() => relayState.selectJwt(jwt)}
          >
            <div class="flex items-center justify-between w-full">
              <div class="flex items-center space-x-2">
                <span class="text-[11px] px-1.5 py-0.5 rounded border font-mono font-medium bg-amber-500/10 text-amber-300 border-amber-500/20">
                  JWT
                </span>
                <span class="text-xs font-mono font-medium text-zinc-200 truncate max-w-[170px]" title={jwt.subject || jwt.source}>
                  {jwt.subject ? `sub: ${jwt.subject}` : jwt.source}
                </span>
              </div>
              <span class="text-[10px] px-1.5 py-0.2 rounded font-mono {expired ? 'bg-rose-500/20 text-rose-300' : 'bg-emerald-500/20 text-emerald-300'}">
                {expired ? "EXP" : "ATIVO"}
              </span>
            </div>

            <div class="text-[11px] text-zinc-400 font-mono truncate">
              {jwt.token.substring(0, 24)}...
            </div>

            <div class="flex items-center justify-between text-[10px] text-zinc-500 font-mono">
              <span>{jwt.source}</span>
              <span>{new Date(jwt.detectedAt).toLocaleTimeString()}</span>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Right Column: JWT Inspector -->
  <div class="flex-1 h-full overflow-y-auto p-5 space-y-6">
    {#if !relayState.selectedJwt}
      <div class="h-full flex flex-col items-center justify-center text-zinc-600 text-sm space-y-2 select-none">
        <div class="text-3xl">🛡️</div>
        <div>Selecione um token JWT capturado na lista para inspecionar claims decodificados.</div>
      </div>
    {:else}
      {@const jwt = relayState.selectedJwt}
      <!-- Token Overview Banner -->
      <div class="bg-zinc-900 border border-zinc-800 rounded-xl p-4 space-y-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-3">
            <span class="text-sm font-bold text-white">Token Interceptado</span>
            <span class="text-xs px-2 py-0.5 rounded-full font-mono {isExpired(jwt.expiresAt) ? 'bg-rose-500/20 text-rose-300 border border-rose-500/30' : 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30'}">
              {formatExpiration(jwt.expiresAt)}
            </span>
          </div>

          <div class="flex items-center space-x-2">
            <button
              onclick={() => copyToClipboard(`Bearer ${jwt.token}`, "bearer")}
              class="text-xs px-3 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors shadow-sm"
            >
              {copyFeedback === "bearer" ? "✓ Bearer Copiado!" : "Copiar como Bearer"}
            </button>
            <button
              onclick={() => copyToClipboard(jwt.token, "raw")}
              class="text-xs px-3 py-1.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors"
            >
              {copyFeedback === "raw" ? "✓ Token Copiado!" : "Copiar Token"}
            </button>
          </div>
        </div>

        <div class="grid grid-cols-3 gap-4 text-xs font-mono pt-2 border-t border-zinc-800/80">
          <div>
            <span class="text-zinc-500 block text-[10px] uppercase tracking-wider">Subject (sub)</span>
            <span class="text-zinc-200 font-semibold">{jwt.subject || "N/A"}</span>
          </div>
          <div>
            <span class="text-zinc-500 block text-[10px] uppercase tracking-wider">Origem</span>
            <span class="text-zinc-200 font-semibold">{jwt.source}</span>
          </div>
          <div>
            <span class="text-zinc-500 block text-[10px] uppercase tracking-wider">Capturado Em</span>
            <span class="text-zinc-200">{new Date(jwt.detectedAt).toLocaleString()}</span>
          </div>
        </div>
      </div>

      <!-- Raw Encoded Token -->
      <div class="space-y-2">
        <h4 class="text-xs font-semibold uppercase tracking-wider text-zinc-400 select-none">
          Token Raw (Base64 URL-Safe)
        </h4>
        <pre class="bg-zinc-900 border border-zinc-800 p-3 rounded-lg text-xs font-mono text-amber-300/90 break-all select-all leading-relaxed">{jwt.token}</pre>
      </div>

      <!-- Decoded Claims Payload -->
      <div class="space-y-2">
        <div class="flex items-center justify-between select-none">
          <h4 class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
            Claims Decodificados (Payload)
          </h4>
          {#if jwt.claims}
            <button
              onclick={() => copyToClipboard(JSON.stringify(jwt.claims, null, 2), "claims")}
              class="text-[11px] text-zinc-400 hover:text-zinc-200 transition-colors"
            >
              {copyFeedback === "claims" ? "✓ Copiado!" : "Copiar Claims"}
            </button>
          {/if}
        </div>

        {#if jwt.claims}
          <pre class="bg-zinc-900 border border-zinc-800 p-4 rounded-lg text-xs font-mono text-emerald-300/90 overflow-x-auto leading-relaxed select-text">{JSON.stringify(jwt.claims, null, 2)}</pre>
        {:else}
          <div class="p-3 bg-zinc-900 border border-zinc-800 rounded-lg text-xs text-zinc-500 italic">
            Sem claims detectados.
          </div>
        {/if}
      </div>

      <!-- Decoded Header -->
      {#if jwt.header}
        <div class="space-y-2">
          <h4 class="text-xs font-semibold uppercase tracking-wider text-zinc-400 select-none">
            Header do Token
          </h4>
          <pre class="bg-zinc-900 border border-zinc-800 p-3 rounded-lg text-xs font-mono text-cyan-300/90 overflow-x-auto select-text">{JSON.stringify(jwt.header, null, 2)}</pre>
        </div>
      {/if}
    {/if}
  </div>
</div>
