<script lang="ts">
  import { fade } from 'svelte/transition';

  let {
    connectionState,
    proxyText,
    showLogs
  }: {
    connectionState: 'disconnected' | 'connecting' | 'connected';
    proxyText: string;
    showLogs: boolean;
  } = $props();
</script>

<div class="flex flex-col items-center text-center transition-all duration-500
  {showLogs ? 'min-h-12' : 'min-h-16'}">

  <h2 class="tracking-wide font-light transition-all duration-500
    {showLogs ? 'text-xl' : 'text-2xl'}
    {connectionState === 'connected'
      ? 'text-success-500'
      : connectionState === 'connecting'
        ? 'text-primary-300-700'
        : 'text-surface-900-100'}">
    {#if connectionState === 'disconnected'}
      Disconnected
    {:else if connectionState === 'connecting'}
      <span class="animate-pulse">Connecting...</span>
    {:else}
      Connected
    {/if}
  </h2>

  <div class="h-6 flex items-center justify-center transition-all duration-500
    {showLogs ? 'mt-0.5' : 'mt-2'}">
    {#if connectionState === 'disconnected'}
      <span transition:fade={{ duration: 200 }}
        class="text-sm tracking-wider text-surface-500-400/70">
        Tap to connect
      </span>
    {:else if connectionState === 'connected' && proxyText}
      <span transition:fade={{ duration: 300 }}
        class="font-mono tracking-widest text-surface-900-100 transition-all duration-500
          {showLogs ? 'text-base' : 'text-lg'}">
        {proxyText}
      </span>
    {/if}
  </div>
</div>