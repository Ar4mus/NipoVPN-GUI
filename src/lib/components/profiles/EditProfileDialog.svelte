<script lang="ts">
	import { Copy } from 'lucide-svelte';
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import {
		profilesStore,
		DEFAULT_LOG_LEVEL,
		LOG_LEVELS,
		normalizeLogLevel,
		type Profile
	} from '$lib/profiles.svelte';
	import { logMessage } from '$lib/logger';
	import { showToast } from '$lib/components/ToastGroup.svelte';

	interface Props {
		open: boolean;
		profile: Profile;
		onClose: () => void;
	}

	let { open, profile, onClose }: Props = $props();

	let profileName = $state('');
	let token = $state('');
	let protocol = $state('SOCKS5');
	let timeout = $state<number | undefined>(undefined);
	let pull = $state<number | undefined>(undefined);
	let listenIp = $state('');
	let listenPort = $state<number | undefined>(undefined);
	let serverIp = $state('');
	let serverPort = $state<number | undefined>(undefined);
	let userAgent = $state('');
	let fakeUrls = $state('');
	let endpoints = $state('');
	let selectedMethods = $state<string[]>([]);

	let httpVersion = $state('1.1');
	let logLevel = $state(DEFAULT_LOG_LEVEL);
	let tlsVerifyPeer = $state(false);
	let tlsCaFile = $state('');
	let tlsCertFile = $state('/etc/nipovpn/server.crt');
	let tlsKeyFile = $state('/etc/nipovpn/server.key');

	let tunnelEnable = $state(false);
	let connectionReuse = $state(true);
	let tlsEnable = $state(false);
	let bufferSize = $state<number>(65536);

	$effect(() => {
		if (profile) {
			profileName = profile.name;
			token = profile.config.token || '';
			protocol = profile.config.protocol?.toUpperCase() || 'SOCKS5';
			timeout = profile.config.timeout ? Number(profile.config.timeout) : undefined;
			pull = profile.config.pullTimeout ? Number(profile.config.pullTimeout) : undefined;
			listenIp = profile.config.listenIp || '';
			listenPort = profile.config.listenPort ? Number(profile.config.listenPort) : undefined;
			serverIp = profile.config.serverIp || '';
			serverPort = profile.config.serverPort ? Number(profile.config.serverPort) : undefined;
			userAgent = profile.config.userAgent || '';
			fakeUrls = profile.config.fakeUrls || '';
			endpoints = profile.config.endPoints || '';
			selectedMethods = profile.config.methods
				? profile.config.methods.split('\n').filter(Boolean)
				: [];

			tunnelEnable = !!profile.config.tunnelEnable;
			connectionReuse = !!profile.config.connectionReuse;
			tlsEnable = !!profile.config.tlsEnable;

			httpVersion = profile.config.httpVersion || '1.1';
			logLevel = normalizeLogLevel(profile.config.logLevel);
			tlsVerifyPeer = !!profile.config.tlsVerifyPeer;
			tlsCaFile = profile.config.tlsCaFile || '';
			tlsCertFile = profile.config.tlsCertFile || '/etc/nipovpn/server.crt';
			tlsKeyFile = profile.config.tlsKeyFile || '/etc/nipovpn/server.key';
			bufferSize = profile.config.bufferSize ? Number(profile.config.bufferSize) : 65536;
		}
	});

	function toggleMethod(m: string) {
		if (selectedMethods.includes(m)) {
			selectedMethods = selectedMethods.filter((x) => x !== m);
		} else {
			selectedMethods = [...selectedMethods, m];
		}
	}

	async function copyToken() {
		if (!token.trim()) {
			showToast({
				type: 'warning',
				title: 'Token is empty',
				description: 'There is no token to copy.'
			});
			return;
		}

		try {
			await navigator.clipboard.writeText(token);
			showToast({
				type: 'success',
				title: 'Token copied',
				description: 'Token copied to clipboard.'
			});
		} catch (e) {
			const message = e instanceof Error ? e.message : String(e);
			showToast({
				type: 'error',
				title: 'Copy failed',
				description: message || 'Could not copy token to clipboard.'
			});
		}
	}

	async function handleSave() {
		const name = profileName.trim();
		if (!name) {
			showToast({
				type: 'warning',
				title: 'Profile name is required',
				description: 'Enter a name before saving the profile.'
			});
			return;
		}

		const config = {
			token,
			protocol: protocol.toLowerCase(),
			fakeUrls,
			methods: selectedMethods.join('\n'),
			endPoints: endpoints,
			timeout: timeout?.toString() || '10',
			pullTimeout: pull?.toString() || '50',
			tunnelEnable,
			connectionReuse,
			tlsEnable,
			tlsVerifyPeer,
			tlsCertFile,
			tlsKeyFile,
			tlsCaFile,
			logLevel,
			listenIp: listenIp || '127.0.0.1',
			listenPort: listenPort?.toString() || '1080',
			serverIp: serverIp || '',
			serverPort: serverPort?.toString() || '',
			httpVersion,
			userAgent: userAgent || '',
			bufferSize: bufferSize.toString()
		};

		try {
			await profilesStore.updateProfile(profile.id, name, config);

			onClose();
			showToast({
				type: 'success',
				title: 'Profile saved',
				description: `"${name}" was saved successfully.`
			});
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			await logMessage('error', 'EditProfileDialog', `Failed to save profile: ${message}`);
			console.error('Failed to save profile:', error);
			showToast({
				type: 'error',
				title: 'Profile save failed',
				description: message || `Could not save "${name}".`
			});
		}
	}
</script>

<Dialog
	{open}
	onOpenChange={(e) => {
		if (!e.open) onClose();
	}}
>
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" />
		<Dialog.Positioner class="fixed inset-0 z-50 flex justify-center items-center">
			<Dialog.Content class="card bg-surface-100-900 w-md p-4 space-y-2 shadow-xl">
				<Dialog.Title class="text-xl font-400">Edit Profile</Dialog.Title>
				<Dialog.Description class="flex flex-col gap-2">
					<div class="mt-2 flex flex-col gap-3 max-h-[70vh] overflow-y-auto px-1">
						<p class="text-[10px] tracking-widest text-surface-500-400">PROFILE</p>
						<div class="flex flex-col gap-1">
							<label for="edit-profile-name" class="text-[10px] tracking-wider text-surface-500-400"
								>PROFILE NAME</label
							>
							<input
								id="edit-profile-name"
								type="text"
								bind:value={profileName}
								placeholder="e.g., Direct proxy"
								class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
							/>
						</div>
						<div class="flex flex-col gap-1">
							<label for="edit-token" class="text-[10px] tracking-wider text-surface-500-400"
								>TOKEN</label
							>
							<div class="relative">
								<input
									id="edit-token"
									type="text"
									bind:value={token}
									placeholder="8b064f13-13..."
									class="bg-surface-300-700 p-2 pr-9 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
								/>
								<button
									type="button"
									class="absolute right-2 top-1/2 -translate-y-1/2 text-surface-500-400 hover:text-surface-800-200"
									onclick={copyToken}
									aria-label="Copy token"
								>
									<Copy size={14} />
								</button>
							</div>
						</div>

						<hr class="border-surface-300-700" />

						<p class="text-[10px] tracking-widest text-surface-500-400">CONNECTION</p>
						<div class="flex gap-2">
							<div class="flex flex-col gap-1 flex-1 justify-center">
								<label class="text-[10px] tracking-wider text-surface-500-400"
									>PROTOCOL
									<div class="flex gap-1">
										{#each ['HTTP', 'SOCKS5'] as proto (proto)}
											<button
												type="button"
												class="mt-1 flex-1 text-xs py-1.5 rounded-base border transition-colors
													{protocol === proto
													? 'bg-surface-300-700 border-surface-500-400 text-surface-950-50'
													: 'bg-transparent border-surface-300-700 text-surface-500-400 hover:border-surface-500-400'}"
												onclick={() => (protocol = proto)}>{proto}</button
											>
										{/each}
									</div>
								</label>
							</div>
							<div class="flex flex-col gap-1 w-20">
								<label for="edit-timeout" class="text-[10px] tracking-wider text-surface-500-400"
									>TIMEOUT · s</label
								>
								<input
									id="edit-timeout"
									type="number"
									bind:value={timeout}
									placeholder="10"
									class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
								/>
							</div>
							<div class="flex flex-col gap-1 w-20">
								<label for="edit-pull" class="text-[10px] tracking-wider text-surface-500-400"
									>PULL</label
								>
								<input
									id="edit-pull"
									type="number"
									bind:value={pull}
									placeholder="50"
									class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
								/>
							</div>
						</div>
						<div class="flex flex-col gap-1">
							<label for="buffer-size" class="text-[10px] tracking-wider text-surface-500-400"
								>BUFFER SIZE · bytes <span class="text-[8px] text-surface-500-400">
									(1–65536)
								</span>
								<br />
								<span class="text-[8px] text-surface-500-400"
									>Recommended values : 16384, 32768, 65536</span
								>
							</label>
							<input
								id="edit-buffer-size"
								type="number"
								bind:value={bufferSize}
								min="1"
								max="65536"
								placeholder="65536"
								class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
							/>
						</div>
						<div class="flex gap-2 flex-wrap">
							<button
								type="button"
								class="flex items-center gap-2 text-xs px-3 py-1.5 rounded-base border transition-colors
									bg-surface-300-700 border-surface-300-700 text-surface-500-400 hover:border-surface-500-400"
								onclick={() => (tunnelEnable = !tunnelEnable)}
							>
								<span
									class="size-2 rounded-sm {tunnelEnable ? 'bg-success-500' : 'bg-surface-500-400'}"
								></span>
								TUNNEL
							</button>
							<button
								type="button"
								class="flex items-center gap-2 text-xs px-3 py-1.5 rounded-base border transition-colors
									bg-surface-300-700 border-surface-300-700 text-surface-500-400 hover:border-surface-500-400"
								onclick={() => (connectionReuse = !connectionReuse)}
							>
								<span
									class="size-2 rounded-sm {connectionReuse
										? 'bg-success-500'
										: 'bg-surface-500-400'}"
								></span>
								REUSE
							</button>
							<button
								type="button"
								class="flex items-center gap-2 text-xs px-3 py-1.5 rounded-base border transition-colors
									bg-surface-300-700 border-surface-300-700 text-surface-500-400 hover:border-surface-500-400"
								onclick={() => (tlsEnable = !tlsEnable)}
							>
								<span
									class="size-2 rounded-sm {tlsEnable ? 'bg-success-500' : 'bg-surface-500-400'}"
								></span>
								TLS
							</button>
						</div>

						{#if tlsEnable}
							<hr class="border-surface-300-700" />
							<p class="text-[10px] tracking-widest text-surface-500-400">TLS CONFIGURATION</p>

							<div class="flex items-center gap-2">
								<button
									type="button"
									class="flex items-center gap-2 text-xs px-3 py-1.5 rounded-base border transition-colors
										bg-surface-300-700 border-surface-300-700 text-surface-500-400 hover:border-surface-500-400"
									onclick={() => (tlsVerifyPeer = !tlsVerifyPeer)}
								>
									<span
										class="size-2 rounded-sm {tlsVerifyPeer
											? 'bg-success-500'
											: 'bg-surface-500-400'}"
									></span>
									VERIFY PEER
								</button>
							</div>

							<div class="flex flex-col gap-1">
								<label
									for="edit-tls-ca-file"
									class="text-[10px] tracking-wider text-surface-500-400">TLS CA FILE PATH</label
								>
								<input
									id="edit-tls-ca-file"
									type="text"
									bind:value={tlsCaFile}
									placeholder="e.g., /etc/nipovpn/ca.crt"
									class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
								/>
							</div>

							<div class="flex flex-col gap-1">
								<label
									for="edit-tls-cert-file"
									class="text-[10px] tracking-wider text-surface-500-400">TLS CERT FILE PATH</label
								>
								<input
									id="edit-tls-cert-file"
									type="text"
									bind:value={tlsCertFile}
									placeholder="e.g., /etc/nipovpn/server.crt"
									class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
								/>
							</div>

							<div class="flex flex-col gap-1">
								<label
									for="edit-tls-key-file"
									class="text-[10px] tracking-wider text-surface-500-400">TLS KEY FILE PATH</label
								>
								<input
									id="edit-tls-key-file"
									type="text"
									bind:value={tlsKeyFile}
									placeholder="e.g., /etc/nipovpn/server.key"
									class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
								/>
							</div>
						{/if}

						<hr class="border-surface-300-700" />

						<p class="text-[10px] tracking-widest text-surface-500-400">ENDPOINTS</p>
						<div class="flex gap-2">
							<div class="flex flex-col gap-1 flex-2">
								<label for="edit-listen-ip" class="text-[10px] tracking-wider text-surface-500-400"
									>LISTEN IP</label
								>
								<input
									id="edit-listen-ip"
									type="text"
									bind:value={listenIp}
									placeholder="e.g., 192.168.1.100"
									class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
								/>
							</div>
							<div class="flex flex-col gap-1 flex-1">
								<label
									for="edit-listen-port"
									class="text-[10px] tracking-wider text-surface-500-400">PORT</label
								>
								<input
									id="edit-listen-port"
									type="number"
									bind:value={listenPort}
									placeholder="e.g., 8080"
									class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
								/>
							</div>
						</div>
						<div class="flex gap-2">
							<div class="flex flex-col gap-1 flex-2">
								<label for="edit-server-ip" class="text-[10px] tracking-wider text-surface-500-400"
									>SERVER IP</label
								>
								<input
									id="edit-server-ip"
									type="text"
									bind:value={serverIp}
									placeholder="e.g., proxy.example.com or 1.1.1.1"
									class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
								/>
							</div>
							<div class="flex flex-col gap-1 flex-1">
								<label
									for="edit-server-port"
									class="text-[10px] tracking-wider text-surface-500-400">PORT</label
								>
								<input
									id="edit-server-port"
									type="number"
									bind:value={serverPort}
									placeholder="e.g., 8080"
									class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
								/>
							</div>
						</div>

						<hr class="border-surface-300-700" />

						<div class="flex flex-col gap-1">
							<label for="edit-http-version" class="text-[10px] tracking-wider text-surface-500-400"
								>HTTP VERSION</label
							>
							<input
								id="edit-http-version"
								type="text"
								bind:value={httpVersion}
								placeholder="e.g., 1.1"
								class="bg-surface-300-700 p-2 rounded-base w-full text-sm focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
							/>
						</div>

						<fieldset class="flex flex-col gap-1 border-0 p-0 m-0">
							<legend class="text-[10px] tracking-wider text-surface-500-400">LOG LEVEL</legend>
							<div class="flex gap-1">
								{#each LOG_LEVELS as level (level)}
									<button
										type="button"
										class="mt-1 flex-1 text-xs py-1.5 rounded-base border transition-colors
											{logLevel === level
											? 'bg-surface-300-700 border-surface-500-400 text-surface-950-50'
											: 'bg-transparent border-surface-300-700 text-surface-500-400 hover:border-surface-500-400'}"
										onclick={() => (logLevel = level)}>{level}</button
									>
								{/each}
							</div>
						</fieldset>

						<p class="text-[10px] tracking-widest text-surface-500-400">ADVANCED</p>
						<div class="flex flex-col gap-1">
							<label for="edit-user-agent" class="text-[10px] tracking-wider text-surface-500-400"
								>USER AGENT</label
							>
							<textarea
								id="edit-user-agent"
								bind:value={userAgent}
								rows="2"
								placeholder="e.g., Mozilla/5.0 (Windows NT 10.0; Win64; x64)..."
								class="bg-surface-300-700 p-2 rounded-base w-full text-sm resize-y focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
							></textarea>
						</div>

						<div class="flex flex-col gap-1">
							<label for="edit-fake-urls" class="text-[10px] tracking-wider text-surface-500-400"
								>FAKE URLS <span class="text-[8px] tracking-wider text-surface-500-400"
									>(One per line)</span
								></label
							>
							<textarea
								id="edit-fake-urls"
								bind:value={fakeUrls}
								rows="4"
								placeholder="e.g., example.com"
								class="bg-surface-300-700 p-2 rounded-base w-full text-sm resize-y focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
							></textarea>
						</div>
						<div class="flex flex-col gap-1">
							<label for="edit-endpoints" class="text-[10px] tracking-wider text-surface-500-400"
								>ENDPOINTS <span class="text-[8px] tracking-wider text-surface-500-400"
									>(One per line)</span
								></label
							>
							<textarea
								id="edit-endpoints"
								bind:value={endpoints}
								rows="4"
								class="bg-surface-300-700 p-2 rounded-base w-full text-sm resize-y focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0"
							></textarea>
						</div>

						<p class="text-[10px] tracking-widest text-surface-500-400">HTTP METHODS</p>
						<div class="flex gap-2 flex-wrap">
							{#each ['GET', 'POST', 'PUT', 'DELETE'] as method (method)}
								<button
									type="button"
									class="text-xs px-3 py-1.5 rounded-base border transition-colors
										{selectedMethods.includes(method)
										? 'bg-surface-300-700 border-surface-500-400 text-surface-950-50'
										: 'bg-transparent border-surface-300-700 text-surface-500-400 hover:border-surface-500-400'}"
									onclick={() => toggleMethod(method)}>{method}</button
								>
							{/each}
						</div>
					</div>

					<div class="mt-3 flex gap-3">
						<button type="button" onclick={onClose} class="btn preset-tonal">Cancel</button>
						<button type="button" onclick={handleSave} class="btn bg-primary-500 preset-filled"
							>Save</button
						>
					</div>
				</Dialog.Description>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
