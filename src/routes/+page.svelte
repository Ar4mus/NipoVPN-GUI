<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { fade } from 'svelte/transition';
	import { ChevronUp } from 'lucide-svelte';

	import { get } from 'svelte/store';
	import { profilesStore, type Profile } from '$lib/profiles.svelte';
	import { runningProfileId } from '$lib/runningProfile.svelte';
	import { logMessage } from '$lib/logger';
	import { showToast } from '$lib/components/ToastGroup.svelte';
	import EditProfileDialog from '$lib/components/profiles/EditProfileDialog.svelte';
	import DeleteProfileDialog from '$lib/components/profiles/DeleteProfileDialog.svelte';
	import ConnectionButton from '$lib/components/connection/ConnectionButton.svelte';
	import ConnectionStatus from '$lib/components/connection/ConnectionStatus.svelte';
	import LatencyDisplay from '$lib/components/connection/LatencyDisplay.svelte';
	import LogPanel from '$lib/components/connection/LogPanel.svelte';

	let connectionState = $state<'disconnected' | 'connecting' | 'connected'>('disconnected');
	let ping = $state<number | null>(null);
	let showLogs = $state(true);

	let editDialogOpen = $state(false);
	let deleteDialogOpen = $state(false);
	let editingProfile: Profile | null = $state(null);
	let deletingProfile: Profile | null = $state(null);

	let logFileContent = $state('');
	let logClearedAt = $state<Date | null>(null);
	let systemLogFileContent = $state('');
	let systemLogClearedAt = $state<Date | null>(null);

	type LogSnapshot = { content: string; modifiedMs: number | null };

	let currentProfile = $derived.by(() =>
		profilesStore.profiles.find((p) => p.id === profilesStore.selectedProfileId)
	);
	let canConnect = $derived(Boolean(currentProfile));

	let previousActiveProfileSnapshot = $state<string | null>(null);

	let proxyText = $derived.by(() => {
		if (connectionState !== 'connected' || !currentProfile) return '';
		const protocol = currentProfile.config.protocol?.trim().toLowerCase();
		const port = currentProfile.config.listenPort?.trim();
		if (protocol === 'socks5') return port ? `Use SOCKS5 on port ${port}` : 'Use SOCKS5';
		if (protocol === 'http') return port ? `Use HTTP on port ${port}` : 'Use HTTP';
		return protocol ? `Use ${protocol.toUpperCase()} proxy` : 'Use proxy';
	});

	let isCurrentProfileRunning = $derived(
		connectionState === 'connected' && currentProfile?.id === profilesStore.selectedProfileId
	);

	let pingInterval: ReturnType<typeof setInterval>;
	let logRefreshInterval: ReturnType<typeof setInterval>;
	let connectionOperation: Promise<void> = Promise.resolve();
	let lastToastAt = $state<Record<string, number>>({});

	type ToastOptions = Parameters<typeof showToast>[0];

	function getErrorMessage(error: unknown): string {
		return error instanceof Error ? error.message : String(error);
	}

	function showThrottledToast(key: string, options: ToastOptions, minIntervalMs = 30000) {
		const now = Date.now();
		if ((lastToastAt[key] ?? 0) + minIntervalMs > now) return;
		lastToastAt[key] = now;
		showToast(options);
	}

	function runConnectionOperation(operation: () => Promise<void>) {
		connectionOperation = connectionOperation.then(operation, operation);
		return connectionOperation;
	}

	function handleConnectionToggle() {
		if (!canConnect) return;
		if (connectionState === 'disconnected') {
			void runConnectionOperation(connectVpn);
		} else {
			void runConnectionOperation(disconnectVpn);
		}
	}

	async function connectVpn() {
		const profile = currentProfile;
		if (!profile) return;

		connectionState = 'connecting';
		logFileContent = '';
		logClearedAt = null;
		await logMessage('info', 'ConnectionButton', `connect action for profile ${profile.id}`);
		try {
			await invoke('start_vpn', { profileId: profile.id, config: profile.config });

			if (profile.id !== currentProfile?.id) return;

			runningProfileId.set(profile.id);
			const protocol = profile.config.protocol?.trim().toLowerCase();
			if (protocol !== 'http' && protocol !== 'socks5') {
				connectionState = 'connected';
			}
			startMetrics();
			await loadLogFile();
			startLogRefresh();
		} catch (error) {
			if (profile.id === currentProfile?.id) {
				const message = getErrorMessage(error);
				connectionState = 'disconnected';
				await logMessage('error', 'ConnectionButton', `start_vpn failed: ${message}`);
				showToast({
					type: 'error',
					title: 'VPN start failed',
					description: message || 'Could not start the VPN.'
				});
			}
		}
	}

	async function disconnectVpn() {
		const profileId = get(runningProfileId) ?? currentProfile?.id ?? 'unknown';
		await logMessage('info', 'ConnectionButton', `disconnect action for profile ${profileId}`);
		try {
			await invoke('stop_vpn');
		} catch (error) {
			const message = getErrorMessage(error);
			if (!message.includes('No VPN process is currently running')) {
				await logMessage('error', 'ConnectionButton', `stop_vpn failed: ${message}`);
				showToast({
					type: 'error',
					title: 'VPN stop failed',
					description: message || 'Could not stop the VPN.'
				});
			}
		}

		connectionState = 'disconnected';
		runningProfileId.set(null);
		stopMetrics();
		stopLogRefresh();
	}

	async function reconnectCurrentProfile() {
		const profile = currentProfile;
		if (!profile) return;

		await runConnectionOperation(async () => {
			if (connectionState === 'connected' || connectionState === 'connecting') {
				await disconnectVpn();
				await new Promise((r) => setTimeout(r, 500));
			}
			await connectVpn();
		});
	}

	function getActiveProfileSnapshot(): string | null {
		if (!currentProfile) return null;
		return JSON.stringify({
			id: currentProfile.id,
			name: currentProfile.name,
			config: currentProfile.config
		});
	}

	async function handleConfirmDelete() {
		if (deletingProfile) {
			await profilesStore.deleteProfile(deletingProfile.id);
			deleteDialogOpen = false;
			deletingProfile = null;
		}
	}

	function runPingLoop() {
		const profile = currentProfile;
		if (!profile) {
			ping = null;
			return;
		}
		const profileId = profile.id;
		const protocol = profile.config.protocol.toLowerCase();
		if (protocol !== 'http' && protocol !== 'socks5') {
			ping = null;
			return;
		}
		invoke<number>('measure_ping', {
			proxyPort: Number(profile.config.listenPort),
			protocol
		})
			.then((ms) => {
				if (profileId !== currentProfile?.id || connectionState !== 'connecting') return;
				ping = ms;
				connectionState = 'connected';
			})
			.catch((error) => {
				if (profileId !== currentProfile?.id) return;
				const message = getErrorMessage(error);
				void logMessage('error', 'LatencyDisplay', `measure_ping failed: ${message}`);
				ping = null;
				showThrottledToast('ping', {
					type: 'warning',
					title: 'Latency check failed',
					description: message || 'Could not measure proxy latency.'
				});
			});
	}

	function startMetrics() {
		clearInterval(pingInterval);
		pingInterval = setInterval(runPingLoop, 30000);
		runPingLoop();
	}

	function stopMetrics() {
		clearInterval(pingInterval);
		ping = null;
	}

	function startLogRefresh() {
		clearInterval(logRefreshInterval);
		logRefreshInterval = setInterval(() => {
			void loadLogFile();
			void loadSystemLogFile();
		}, 1000);
	}

	function stopLogRefresh() {
		clearInterval(logRefreshInterval);
	}

	async function loadLogFile() {
		if (!currentProfile) {
			logFileContent = '';
			return;
		}
		try {
			const snapshot = await invoke<LogSnapshot>('get_profile_log', {
				profileId: currentProfile.id
			});
			if (
				logClearedAt &&
				snapshot.modifiedMs !== null &&
				snapshot.modifiedMs <= logClearedAt.getTime()
			) {
				logFileContent = '';
				return;
			}
			logFileContent = snapshot.content;
		} catch (error) {
			const message = getErrorMessage(error);
			await logMessage('error', 'LogPanel', `get_profile_log failed: ${message}`);
			logFileContent = '';
			showThrottledToast('profile-log', {
				type: 'warning',
				title: 'VPN log refresh failed',
				description: message || 'Could not load VPN logs.'
			});
		}
	}

	async function loadSystemLogFile() {
		try {
			const snapshot = await invoke<LogSnapshot>('get_system_log');
			if (
				systemLogClearedAt &&
				snapshot.modifiedMs !== null &&
				snapshot.modifiedMs <= systemLogClearedAt.getTime()
			) {
				systemLogFileContent = '';
				return;
			}
			systemLogFileContent = snapshot.content;
		} catch (error) {
			const message = getErrorMessage(error);
			await logMessage('error', 'LogPanel', `get_system_log failed: ${message}`);
			systemLogFileContent = '';
			showThrottledToast('system-log', {
				type: 'warning',
				title: 'System log refresh failed',
				description: message || 'Could not load system logs.'
			});
		}
	}

	async function clearLogs() {
		logFileContent = '';
		logClearedAt = new Date();
		if (!currentProfile) return;
		try {
			await invoke('clear_profile_log', { profileId: currentProfile.id });
		} catch (error) {
			const message = getErrorMessage(error);
			await logMessage('error', 'LogPanel', `clear_profile_log failed: ${message}`);
			console.error('Failed to clear VPN log:', error);
			showToast({
				type: 'error',
				title: 'Clear VPN log failed',
				description: message || 'Could not clear VPN logs.'
			});
		}
	}

	async function clearSystemLogs() {
		systemLogFileContent = '';
		systemLogClearedAt = new Date();
		try {
			await invoke('clear_system_log');
		} catch (error) {
			const message = getErrorMessage(error);
			await logMessage('error', 'LogPanel', `clear_system_log failed: ${message}`);
			console.error('Failed to clear system log:', error);
			showToast({
				type: 'error',
				title: 'Clear system log failed',
				description: message || 'Could not clear system logs.'
			});
		}
	}

	$effect(() => {
		if (showLogs) {
			void loadLogFile();
			void loadSystemLogFile();
			startLogRefresh();
			return () => stopLogRefresh();
		}
		stopLogRefresh();
	});

	$effect(() => {
		const snapshot = getActiveProfileSnapshot();
		if (
			previousActiveProfileSnapshot !== null &&
			snapshot !== previousActiveProfileSnapshot &&
			currentProfile &&
			(connectionState === 'connected' || connectionState === 'connecting')
		) {
			void reconnectCurrentProfile();
		}
		previousActiveProfileSnapshot = snapshot;
	});
</script>

<div class="flex flex-col h-full w-full font-sans overflow-hidden select-none">
	<div
		class="flex-1 flex flex-col justify-center py-4 overflow-y-auto transition-all duration-500 ease-out"
	>
		<div
			class="flex flex-col items-center justify-center transition-all duration-500 ease-out
      {showLogs ? 'gap-3' : 'gap-8'}"
		>
			<ConnectionButton
				{connectionState}
				{showLogs}
				disabled={!canConnect}
				onclick={handleConnectionToggle}
			/>
			<ConnectionStatus {connectionState} {proxyText} {showLogs} />
		</div>

		<hr
			class="self-center w-xl border-surface-300-700 opacity-40 transition-all duration-500
      {showLogs ? 'my-2' : 'my-5'}"
		/>

		<div
			class="flex flex-col justify-end items-center transition-all duration-500
      {showLogs ? 'pb-2 gap-1' : 'pb-6 gap-2'}"
		>
			<LatencyDisplay {ping} {showLogs} />
		</div>
	</div>

	{#if !showLogs}
		<button
			type="button"
			class="w-full rounded-base bg-surface-200-800 border-t border-surface-300-700 py-3
        text-center text-xs tracking-widest text-surface-500-400 hover:text-surface-900-100
        font-medium transition-colors flex items-center justify-center gap-2 cursor-pointer"
			onclick={() => (showLogs = true)}
			transition:fade={{ duration: 150 }}
		>
			<ChevronUp size={14} />
			SHOW LOGS
		</button>
	{:else}
		<LogPanel
			coreLogContent={logFileContent}
			systemLogContent={systemLogFileContent}
			onClearCore={clearLogs}
			onClearSystem={clearSystemLogs}
			onClose={() => (showLogs = false)}
		/>
	{/if}
</div>

{#if editingProfile}
	<EditProfileDialog
		open={editDialogOpen}
		profile={editingProfile}
		onClose={() => {
			editDialogOpen = false;
			editingProfile = null;
		}}
	/>
{/if}

{#if deletingProfile}
	<DeleteProfileDialog
		open={deleteDialogOpen}
		profile={deletingProfile}
		isRunning={isCurrentProfileRunning && deletingProfile.id === currentProfile?.id}
		onBeforeDelete={async () => {
			if (isCurrentProfileRunning && deletingProfile?.id === currentProfile?.id) {
				await disconnectVpn();
			}
		}}
		onConfirm={handleConfirmDelete}
		onCancel={() => {
			deleteDialogOpen = false;
			deletingProfile = null;
		}}
	/>
{/if}
