<script lang="ts">
  let { size = 16, class: className = "" }: { size?: number; class?: string } = $props();
</script>
<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className}><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
