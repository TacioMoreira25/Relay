<script lang="ts">
  let { size = 16, class: className = "" }: { size?: number; class?: string } = $props();
</script>
<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="currentColor" class={className}><rect width="18" height="18" x="3" y="3" rx="2"/></svg>
