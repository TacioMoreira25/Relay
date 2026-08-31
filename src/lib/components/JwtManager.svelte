<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
  import type { ExtractedJwt } from "$lib/types";
  import { IconKey, IconTrash, IconCopy, IconCheck, IconShield } from "$lib/components/icons";
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
  <div class="w-80 border-r border-zinc-800/80 bg-zinc-950 flex flex-col h-full select-none">
    <div class="p-2.5 border-b border-zinc-800/80 flex items-center justify-between bg-zinc-900/30">
      <div class="flex items-center space-x-1.5 text-xs font-medium text-zinc-400">
        <span>Tokens JWT</span>
        <span class="text-[10px] px-1.5 py-0.2 rounded-full bg-zinc-800 text-zinc-300 font-mono">
          {relayState.totalJwts}
        </span>
      </div>
      {#if relayState.totalJwts > 0}
        <button
          onclick={clearAllJwts}
          class="text-[11px] text-zinc-500 hover:text-rose-400 transition-colors p-1 rounded hover:bg-zinc-800 flex items-center space-x-1 cursor-pointer"
        >
          <IconTrash size={12} />
          <span>Limpar</span>
        </button>
      {/if}
    </div>

    <div class="flex-1 overflow-y-auto divide-y divide-zinc-900">
      {#if relayState.jwts.length === 0}
        <div class="h-full p-6 flex flex-col items-center justify-center text-center space-y-2 text-zinc-500">
          <div class="p-3 rounded-full bg-zinc-900 border border-zinc-800 text-zinc-400">
            <IconKey size={20} />
          </div>
          <div class="text-xs font-medium text-zinc-300">Nenhum JWT Capturado</div>
          <p class="text-[11px] text-zinc-500 max-w-[220px] leading-relaxed">
            Tokens em headers <span class="font-mono text-zinc-400">Authorization</span> ou respostas de login aparecerão aqui.
          </p>
        </div>
      {:else}
        {#each relayState.jwts as jwt (jwt.token)}
          {@const expired = isExpired(jwt.expiresAt)}
          <button
            type="button"
            class="w-full text-left p-3 hover:bg-zinc-900/50 transition-colors flex flex-col space-y-1.5 border-l-2 {relayState.selectedJwt?.token === jwt.token ? 'bg-zinc-900/80 border-indigo-500' : 'border-transparent'}"
            onclick={() => relayState.selectJwt(jwt)}
          >
            <div class="flex items-center justify-between w-full">
              <span class="text-xs font-mono font-medium text-zinc-200 truncate max-w-[180px]" title={jwt.subject || jwt.source}>
                {jwt.subject ? `sub: ${jwt.subject}` : jwt.source}
              </span>
              <span class="text-[9px] px-1.5 py-0.2 rounded font-mono font-bold {expired ? 'bg-rose-500/10 text-rose-300 border border-rose-500/20' : 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20'}">
                {expired ? "EXP" : "ATIVO"}
              </span>
            </div>

            <div class="text-[10px] text-zinc-500 font-mono truncate">
              {jwt.token.substring(0, 28)}...
            </div>

            <div class="flex items-center justify-between text-[10px] text-zinc-600 font-mono">
              <span>{jwt.source}</span>
              <span>{new Date(jwt.detectedAt).toLocaleTimeString()}</span>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Right Column: JWT Inspector -->
  <div class="flex-1 h-full overflow-y-auto p-5 space-y-5 select-text">
    {#if !relayState.selectedJwt}
      <div class="h-full flex flex-col items-center justify-center text-zinc-600 text-xs space-y-2 select-none">
        <IconShield size={24} class="text-zinc-700 stroke-1" />
        <div>Selecione um token JWT na lista para inspecionar claims.</div>
      </div>
    {:else}
      {@const jwt = relayState.selectedJwt}
      <!-- Token Overview Banner -->
      <div class="bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 space-y-3">
        <div class="flex items-center justify-between select-none">
          <div class="flex items-center space-x-2.5">
            <span class="text-xs font-bold text-zinc-100">Token Interceptado</span>
            <span class="text-[10px] px-2 py-0.5 rounded font-mono {isExpired(jwt.expiresAt) ? 'bg-rose-500/10 text-rose-300 border border-rose-500/20' : 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20'}">
              {formatExpiration(jwt.expiresAt)}
            </span>
          </div>

          <div class="flex items-center space-x-2">
            <button
              onclick={() => copyToClipboard(`Bearer ${jwt.token}`, "bearer")}
              class="text-xs px-2.5 py-1 rounded bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition-colors cursor-pointer shadow-xs"
            >
              {copyFeedback === "bearer" ? "Copiado!" : "Copiar como Bearer"}
            </button>
            <button
              onclick={() => copyToClipboard(jwt.token, "raw")}
              class="text-xs px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors cursor-pointer border border-zinc-700/80"
            >
              {copyFeedback === "raw" ? "Copiado!" : "Copiar Token"}
            </button>
          </div>
        </div>

        <div class="grid grid-cols-3 gap-3 text-xs font-mono pt-2 border-t border-zinc-800/80">
          <div>
            <span class="text-zinc-500 block text-[10px] uppercase tracking-wider">Subject (sub)</span>
            <span class="text-zinc-200 font-semibold">{jwt.subject || "N/A"}</span>
          </div>
          <div>
            <span class="text-zinc-500 block text-[10px] uppercase tracking-wider">Origem</span>
            <span class="text-zinc-200">{jwt.source}</span>
          </div>
          <div>
            <span class="text-zinc-500 block text-[10px] uppercase tracking-wider">Detectado Em</span>
            <span class="text-zinc-400">{new Date(jwt.detectedAt).toLocaleTimeString()}</span>
          </div>
        </div>
      </div>

      <!-- Raw Encoded Token -->
      <div class="space-y-1.5">
        <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none block">
          Token Raw (Base64 URL-Safe)
        </span>
        <pre class="bg-zinc-900/40 border border-zinc-800 rounded-lg p-3 text-xs font-mono text-zinc-300 break-all select-all leading-relaxed">{jwt.token}</pre>
      </div>

      <!-- Decoded Claims Payload -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between select-none">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400">
            Claims Decodificados (Payload)
          </span>
          {#if jwt.claims}
            <button
              onclick={() => copyToClipboard(JSON.stringify(jwt.claims, null, 2), "claims")}
              class="text-[11px] text-zinc-500 hover:text-zinc-300 transition-colors flex items-center space-x-1 cursor-pointer"
            >
              {#if copyFeedback === "claims"}
                <IconCheck size={12} class="text-emerald-400" />
                <span class="text-emerald-400">Copiado</span>
              {:else}
                <IconCopy size={12} />
                <span>Copiar Claims</span>
              {/if}
            </button>
          {/if}
        </div>

        {#if jwt.claims}
          <pre class="bg-zinc-900/40 border border-zinc-800 rounded-lg p-3.5 text-xs font-mono text-emerald-300/90 overflow-x-auto leading-relaxed select-text">{JSON.stringify(jwt.claims, null, 2)}</pre>
        {:else}
          <div class="p-3 bg-zinc-900/20 border border-zinc-800 rounded-lg text-xs text-zinc-500 italic">
            Sem claims detectados.
          </div>
        {/if}
      </div>

      <!-- Decoded Header -->
      {#if jwt.header}
        <div class="space-y-1.5">
          <span class="text-[11px] font-bold uppercase tracking-wider text-zinc-400 select-none block">
            Header do Token
          </span>
          <pre class="bg-zinc-900/40 border border-zinc-800 rounded-lg p-3 text-xs font-mono text-cyan-300/90 overflow-x-auto select-text">{JSON.stringify(jwt.header, null, 2)}</pre>
        </div>
      {/if}
    {/if}
  </div>
</div>
