<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/icon.png';
	import ModeToggle from '$lib/components/header/ModeToggle.svelte';
	import Profiles from '$lib/components/profiles/Profiles.svelte';
	import Logo from '$lib/components/header/Logo.svelte';
	import About from '$lib/components/header/About.svelte';
	import ToastGroup from '$lib/components/ToastGroup.svelte';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="grid h-screen grid-rows-[auto_1fr_auto]">
	<header
		class="bg-surface-200-900 p-4 border-b border-surface-200-800 flex items-center justify-between"
	>
		<Logo />
		<div class="flex items-center gap-4">
			<ModeToggle />
			<About />
		</div>
	</header>
	<div class="flex flex-row min-h-0 overflow-hidden">
		<aside class="w-1/3 p-4 border-r border-surface-200-800 overflow-y-auto"><Profiles /></aside>
		<main class="w-2/3 p-4 flex flex-col min-h-0 overflow-hidden">
			{@render children()}
		</main>
	</div>
	<ToastGroup />
</div>
