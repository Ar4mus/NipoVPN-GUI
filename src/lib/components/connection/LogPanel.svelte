<script lang="ts">
	import { Terminal, Trash2, ChevronDown } from 'lucide-svelte';
	import { slide } from 'svelte/transition';
	import { Store } from '@tauri-apps/plugin-store';
	import { onMount } from 'svelte';
	import { showToast } from '$lib/components/ToastGroup.svelte';

	type Tab = 'core' | 'system';
	type LogLevel = 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';

	const LEVEL_RANK: Record<LogLevel, number> = {
		DEBUG: 0,
		INFO: 1,
		WARN: 2,
		ERROR: 3
	};

	const MIN_LEVEL_RANK = LEVEL_RANK.WARN;
	const STORE_PATH = 'settings.json';
	const KEY_TAB = 'log-panel-tab';

	const {
		coreLogContent = '',
		systemLogContent = '',
		onClearCore,
		onClearSystem,
		onClose
	}: {
		coreLogContent?: string;
		systemLogContent?: string;
		onClearCore: () => void | Promise<void>;
		onClearSystem: () => void | Promise<void>;
		onClose: () => void;
	} = $props();

	let activeTab = $state<Tab>('core');
	let terminalElement = $state<HTMLDivElement | null>(null);

	let store: Store | null = null;
	let saveInProgress = false;

	function normalizeTab(v: unknown): Tab | null {
		return v === 'core' || v === 'system' ? v : null;
	}

	function normalizeLevel(v: unknown): LogLevel | null {
		return v === 'DEBUG' || v === 'INFO' || v === 'WARN' || v === 'ERROR' ? v : null;
	}

	async function ensureStore(): Promise<Store> {
		if (store) return store;
		store = await Store.load(STORE_PATH);
		return store;
	}

	async function loadPreferences() {
		try {
			const s = await ensureStore();
			const savedTab = normalizeTab(await s.get<Tab>(KEY_TAB));
			if (savedTab) activeTab = savedTab;
		} catch (e) {
			console.error('[LogPanel] Failed to load preferences from store:', e);
			showToast({
				type: 'warning',
				title: 'Log preferences unavailable',
				description: 'Could not load the saved log panel tab preference.'
			});
		}
	}

	async function savePreferences() {
		if (saveInProgress) return;
		saveInProgress = true;
		try {
			const s = await ensureStore();
			await s.set(KEY_TAB, activeTab);
			await s.save();
		} catch (e) {
			console.error('[LogPanel] Failed to save preferences to store:', e);
			showToast({
				type: 'error',
				title: 'Log preference save failed',
				description: 'Could not save the selected log panel tab.'
			});
		} finally {
			setTimeout(() => (saveInProgress = false), 250);
		}
	}

	onMount(() => {
		void loadPreferences();
	});

	function toLines(raw: string): string[] {
		if (!raw || raw.trim().length === 0) return [];
		return raw.split('\n').filter((l) => l.trim().length > 0);
	}

	function parseLineLevel(line: string): LogLevel | null {
		const matches = line.match(/\[([A-Z]+)\]/g);
		if (!matches) return null;
		const level = matches
			.map((m) => m.slice(1, -1))
			.reverse()
			.find((v) => normalizeLevel(v) !== null);
		return level ? normalizeLevel(level) : null;
	}

	function filterLines(raw: string): string[] {
		return toLines(raw).filter((line) => {
			const lvl = parseLineLevel(line);
			if (lvl === null) return false;
			return LEVEL_RANK[lvl] >= MIN_LEVEL_RANK;
		});
	}

	const coreLines = $derived(toLines(coreLogContent));
	const systemLines = $derived(filterLines(systemLogContent));
	const visibleLines = $derived(activeTab === 'core' ? coreLines : systemLines);

	$effect(() => {
		if (!terminalElement) return;
		const len = visibleLines.length;
		requestAnimationFrame(() => {
			if (!terminalElement) return;
			terminalElement.scrollTo({
				top: terminalElement.scrollHeight,
				behavior: len === 0 ? 'auto' : 'smooth'
			});
		});
	});

	function lineColorClass(line: string): string {
		const lvl = parseLineLevel(line);
		if (lvl === 'ERROR') return 'text-error-500';
		if (lvl === 'WARN') return 'text-warning-500';
		return '';
	}

	function handleTabSwitch(tab: Tab) {
		activeTab = tab;
		void savePreferences();
	}

	async function handleClear() {
		const label = activeTab === 'core' ? 'Core' : 'System';

		if (visibleLines.length === 0) {
			showToast({
				type: 'warning',
				title: 'No logs to clear',
				description: `${label} logs are already empty.`
			});
			return;
		}

		try {
			if (activeTab === 'core') {
				await onClearCore();
			} else {
				await onClearSystem();
			}

			showToast({
				type: 'success',
				title: 'Logs cleared',
				description: `${label} logs were cleared.`
			});
		} catch (e: unknown) {
			const message = e instanceof Error ? e.message : String(e);
			showToast({
				type: 'error',
				title: 'Clear logs failed',
				description: message || `Could not clear ${label.toLowerCase()} logs.`
			});
		}
	}
</script>

<div
	transition:slide={{ duration: 450 }}
	class="w-full h-[35vh] bg-surface-100-900 border-t border-surface-200-800
    flex flex-col rounded-t-xl shadow-2xl overflow-hidden"
>
	<div
		class="flex items-center justify-between px-5 py-3
      bg-surface-200-800/40 border-b border-surface-200-800"
	>
		<div class="flex items-center gap-2">
			<Terminal size={14} class="text-surface-700-200" />
			<span class="text-xs font-semibold tracking-widest text-surface-900-100">LOGS</span>
		</div>

		<div class="flex items-center gap-1">
			<button
				type="button"
				class="btn-icon p-1.5 rounded-base text-surface-700-200 hover:text-error-500
          hover:bg-surface-200-800/40 transition-colors cursor-pointer"
				onclick={handleClear}
				title="Clear {activeTab === 'core' ? 'Core' : 'System'} Logs"
			>
				<Trash2 size={15} />
			</button>
			<button
				type="button"
				class="btn-icon p-1.5 rounded-base text-surface-700-200 hover:text-surface-900-100
          hover:bg-surface-200-800/40 transition-colors cursor-pointer"
				onclick={onClose}
				title="Minimize Panel"
			>
				<ChevronDown size={16} />
			</button>
		</div>
	</div>

	<div
		class="flex items-center justify-between gap-3 px-5 py-2
      bg-surface-100-900 border-b border-surface-200-800/60"
	>
		<div class="flex items-center gap-1 bg-surface-200-800/40 rounded-base p-0.5">
			<button
				type="button"
				onclick={() => handleTabSwitch('core')}
				class="px-3 py-1 text-[10px] font-semibold tracking-wider rounded-base transition-colors cursor-pointer
          {activeTab === 'core'
					? 'bg-surface-900-100 text-surface-50-950 shadow-sm'
					: 'text-surface-700-200 hover:text-surface-900-100'}"
			>
				CORE (NipoVPN)
			</button>
			<button
				type="button"
				onclick={() => handleTabSwitch('system')}
				class="px-3 py-1 text-[10px] font-semibold tracking-wider rounded-base transition-colors cursor-pointer
          {activeTab === 'system'
					? 'bg-surface-900-100 text-surface-50-950 shadow-sm'
					: 'text-surface-700-200 hover:text-surface-900-100'}"
			>
				SYSTEM (APP)
			</button>
		</div>
	</div>

	<div
		bind:this={terminalElement}
		class="flex-1 overflow-y-auto px-5 pb-5 font-mono text-[11px]
      space-y-1.5 bg-surface-100-900 selection:bg-surface-200-800"
	>
		{#if visibleLines.length === 0}
			<div class="text-surface-700-200 italic pt-2">
				{activeTab === 'core'
					? 'No core (VPN) logs available for this run...'
					: 'No system logs available...'}
			</div>
		{:else}
			{#each visibleLines as line, index (index)}
				<div class="leading-relaxed break-all {lineColorClass(line)}">{line}</div>
			{/each}
		{/if}
	</div>
</div>
