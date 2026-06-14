<script lang="ts">
	import { LockKeyhole, LockKeyholeOpen } from 'lucide-svelte';
	import { fade } from 'svelte/transition';
	let {
		connectionState,
		showLogs,
		disabled,
		onclick
	}: {
		connectionState: 'disconnected' | 'connecting' | 'connected';
		showLogs: boolean;
		disabled: boolean;
		onclick: () => void;
	} = $props();
</script>

<div class="relative flex items-center justify-center">
	{#if connectionState === 'connecting'}
		<div
			transition:fade={{ duration: 300 }}
			class="absolute inset-0 size-full rounded-full bg-primary-300-700/20 animate-ping"
			style="animation-duration: 2s;"
		></div>
		<div
			transition:fade={{ duration: 300 }}
			class="absolute inset-[-10%] size-[120%] rounded-full border border-primary-300-700/30 animate-pulse"
		></div>
	{/if}

	{#if connectionState === 'connected'}
		<div
			transition:fade={{ duration: 500 }}
			class="absolute inset-[-20%] size-[140%] rounded-full bg-success-500/10 blur-2xl"
		></div>
	{/if}

	<button
		type="button"
		{disabled}
		class="relative z-10 rounded-full flex items-center justify-center transition-all duration-700
      ease-[cubic-bezier(0.23,1,0.32,1)] shadow-xl
      {disabled ? 'cursor-not-allowed opacity-40' : 'cursor-pointer'}
      {showLogs ? 'size-32' : 'size-40'}
      {connectionState === 'connected'
			? 'bg-success-500 shadow-[0_0_50px_-12px_rgba(34,197,94,0.6)] text-surface-950-50'
			: connectionState === 'connecting'
				? 'bg-warning-500 shadow-[0_0_50px_-12px_rgba(234,179,8,0.6)] text-surface-950-50'
				: 'bg-surface-300-700 hover:bg-surface-400-600 text-surface-500-400 hover:text-surface-600-300'}"
		{onclick}
		aria-label="Toggle Proxy Connection"
	>
		<div
			class="transition-transform duration-500
      {connectionState === 'connecting' ? 'scale-90 opacity-70' : 'scale-100 opacity-100'}"
		>
			{#if connectionState === 'connected'}
				<LockKeyholeOpen size={showLogs ? 40 : 52} strokeWidth={1.5} />
			{:else}
				<LockKeyhole size={showLogs ? 40 : 52} strokeWidth={1.5} />
			{/if}
		</div>
	</button>
</div>
