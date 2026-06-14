<script lang="ts">
  let { ping, showLogs }: { ping: number | null; showLogs: boolean } = $props();

  function getPingColor(currentPing: number | null): string {
    if (currentPing === null) return 'text-surface-500-400';
    if (currentPing < 500) return 'text-success-500';
    if (currentPing <= 900) return 'text-warning-500';
    return 'text-error-500';
  }
</script>

<div class="flex flex-col items-center gap-1">
  <span class="text-[10px] tracking-[0.2em] text-surface-500-400/60 uppercase">
    Latency
  </span>
  <span class="font-mono transition-all duration-300
    {getPingColor(ping)} {showLogs ? 'text-sm' : 'text-xl'}">
    {ping === null ? '---' : `${ping} ms`}
  </span>
</div>