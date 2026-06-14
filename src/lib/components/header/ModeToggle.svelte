<script lang="ts">
	import { Sun, Moon } from 'lucide-svelte';
	import { Tooltip, Portal } from '@skeletonlabs/skeleton-svelte';
	import { Store } from '@tauri-apps/plugin-store';
	import { onMount } from 'svelte';
	import type { UnlistenFn } from '@tauri-apps/api/event';
	import { showToast } from '$lib/components/ToastGroup.svelte';

	type Mode = 'light' | 'dark';

	const STORE_PATH = 'settings.json';
	const STORAGE_KEY = 'color-mode';

	let mode = $state<Mode>('dark');
	let store: Store | null = null;
	let unlistenStoreChange: UnlistenFn | null = null;
	let reloadTimer: ReturnType<typeof setInterval> | null = null;
	let saveInProgress = false;

	function normalizeMode(value: unknown): Mode | null {
		return value === 'light' || value === 'dark' ? value : null;
	}

	function applyMode(m: Mode) {
		mode = m;
		document.documentElement.dataset.mode = m;
		document.documentElement.style.setProperty('color-scheme', m);
		localStorage.setItem(STORAGE_KEY, m);
	}

	async function ensureStore() {
		if (store) return store;

		store = await Store.load(STORE_PATH);
		unlistenStoreChange = await store.onKeyChange<Mode>(STORAGE_KEY, (value) => {
			const nextMode = normalizeMode(value);
			if (nextMode) applyMode(nextMode);
		});

		return store;
	}

	async function loadModeFromStore() {
		try {
			const loadedStore = await ensureStore();
			const saved = await loadedStore.get<Mode>(STORAGE_KEY);
			const savedMode = normalizeMode(saved);

			if (savedMode) applyMode(savedMode);
		} catch (e) {
			console.error('Failed to load theme from Tauri store:', e);
			const savedMode = normalizeMode(localStorage.getItem(STORAGE_KEY));
			if (savedMode) applyMode(savedMode);

			showToast({
				type: 'warning',
				title: 'Theme preference unavailable',
				description: 'Could not load the saved theme from app settings. Browser fallback was used.'
			});
		}
	}

	async function syncModeFromStore() {
		if (!store || saveInProgress) return;

		try {
			await store.reload({ ignoreDefaults: true });
			const saved = await store.get<Mode>(STORAGE_KEY);
			const savedMode = normalizeMode(saved);

			if (savedMode) applyMode(savedMode);
		} catch (e) {
			console.error('Failed to sync theme from Tauri store:', e);
		}
	}

	onMount(() => {
		void loadModeFromStore();
		reloadTimer = setInterval(() => void syncModeFromStore(), 1000);

		return () => {
			if (reloadTimer) clearInterval(reloadTimer);
			if (unlistenStoreChange) unlistenStoreChange();
		};
	});

	async function toggleMode() {
		const nextMode: Mode = mode === 'dark' ? 'light' : 'dark';
		applyMode(nextMode);

		if (!store) return;

		saveInProgress = true;
		try {
			await store.set(STORAGE_KEY, nextMode);
			await store.save();
		} catch (e) {
			console.error('Failed to save theme to Tauri store:', e);
			showToast({
				type: 'error',
				title: 'Theme save failed',
				description: 'Could not save the selected theme to app settings.'
			});
		} finally {
			setTimeout(() => (saveInProgress = false), 250);
		}
	}
</script>

<svelte:head>
	<script>
		const saved = localStorage.getItem('color-mode');
		const initialMode = saved === 'light' || saved === 'dark' ? saved : 'dark';
		document.documentElement.setAttribute('data-mode', initialMode);
	</script>
</svelte:head>

<Tooltip positioning={{ placement: 'top' }}>
	<Tooltip.Trigger>
		{#snippet element(attrs)}
			<button
				{...attrs}
				type="button"
				onclick={toggleMode}
				class="bg-primary-500 btn-icon preset-filled transition-all"
				aria-label={mode === 'dark' ? 'Switch to Light theme' : 'Switch to Dark theme'}
			>
				{#if mode === 'dark'}
					<Sun size={18} />
				{:else}
					<Moon size={18} />
				{/if}
			</button>
		{/snippet}
	</Tooltip.Trigger>
	<Portal>
		<Tooltip.Positioner>
			<Tooltip.Content class="card bg-surface-100-900 p-2 shadow-xl">
				<span>{mode === 'dark' ? 'Light Theme' : 'Dark Theme'}</span>
			</Tooltip.Content>
		</Tooltip.Positioner>
	</Portal>
</Tooltip>
