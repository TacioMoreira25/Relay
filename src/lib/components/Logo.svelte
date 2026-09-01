<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";
</script>

<div class="flex items-center space-x-2 select-none mr-2">
  <div class="relative flex items-center justify-center">
    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M6 19V5" stroke="#4F46E5" stroke-width="2.5" stroke-linecap="round"/>
      <path d="M6 5H13C16.5 5 16.5 12 13 12H6" stroke="#6366F1" stroke-width="2.5" stroke-linecap="round"/>
      <path d="M12 12L17 19" stroke="#06B6D4" stroke-width="2.5" stroke-linecap="round"/>
      <circle cx="17" cy="19" r="2.2" class={relayState.isProxyRunning ? 'fill-emerald-400 animate-pulse' : 'fill-emerald-500'} />
    </svg>
  </div>
  <span class="font-bold text-xs tracking-wider text-zinc-100 font-mono uppercase">RELAY</span>
</div>
