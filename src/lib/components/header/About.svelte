<script lang="ts">
	import { CircleQuestionMark } from 'lucide-svelte';
	import { Tooltip, Portal, Dialog } from '@skeletonlabs/skeleton-svelte';
	import { open } from '@tauri-apps/plugin-shell';

	function openLink(url: string) {
		void open(url);
	}
</script>

<Tooltip positioning={{ placement: 'top' }}>
	<Tooltip.Trigger>
		{#snippet element(attrs)}
			<Dialog>
				<Dialog.Trigger class="bg-primary-500 btn-icon preset-filled" data-add-profile {...attrs}>
					<CircleQuestionMark size={18} />
				</Dialog.Trigger>
				<Portal>
					<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" />
					<Dialog.Positioner class="fixed inset-0 z-50 flex justify-center items-center">
						<Dialog.Content class="card bg-surface-100-900 w-xl p-4 space-y-2 shadow-xl">
							<Dialog.Title class="text-xl font-400">Help & About</Dialog.Title>
							<Dialog.Description class="flex flex-col gap-2">
								<section class="space-y-1">
									<h3 class="text-lg font-400">About</h3>
									<p class="text-sm text-surface-500-400">
										NipoVPN GUI is a cross-platform desktop application for managing the <a
											href="https://github.com/MortezaBashsiz/nipovpn"
											onclick={(event) => {
												event.preventDefault();
												openLink('https://github.com/MortezaBashsiz/nipovpn');
											}}
											class="text-primary-500 hover:underline"
										>
											NipoVPN
										</a>
										core. The original NipoVPN repository is by
										<a
											href="https://github.com/MortezaBashsiz"
											onclick={(event) => {
												event.preventDefault();
												openLink('https://github.com/MortezaBashsiz');
											}}
											class="text-primary-500 hover:underline"
										>
											Morteza Bashsiz
										</a>.
									</p>
									<p class="text-sm text-surface-500-400">
										This interface is actively maintained by
										<a
											href="https://github.com/Ar4mus"
											onclick={(event) => {
												event.preventDefault();
												openLink('https://github.com/Ar4mus');
											}}
											class="text-primary-500 hover:underline"
										>
											Sattar Bayat
										</a>.
									</p>
								</section>
								<section class="space-y-1">
									<h3 class="text-lg font-400">How to Use</h3>
									<ol class="list-decimal list-inside text-sm text-surface-500-400 space-y-3">
										<li>
											<strong class="text-surface-900-100">Create or import a profile</strong>
											<ul class="list-disc list-inside ml-2 mt-1 text-surface-500-400">
												<li>Click the <code>+</code> button to add a new profile</li>
												<li>
													Or use the Import button to paste a <code>nipovpn://</code> profile code
												</li>
											</ul>
										</li>
										<li>
											<strong class="text-surface-900-100">Configure your VPN settings</strong>
											<ul class="list-disc list-inside ml-2 mt-1 text-surface-500-400">
												<li>Protocol: HTTP or SOCKS5</li>
												<li>Listen Port: Local proxy port</li>
												<li>Server details and authentication</li>
											</ul>
										</li>
										<li>
											<strong class="text-surface-900-100">Connect</strong>
											<ul class="list-disc list-inside ml-2 mt-1 text-surface-500-400">
												<li>Select your profile from the sidebar</li>
												<li>Click the lock button to establish connection</li>
											</ul>
										</li>
										<li>
											<strong class="text-surface-900-100">Monitor</strong>
											<ul class="list-disc list-inside ml-2 mt-1 text-surface-500-400">
												<li>View real-time latency in the main panel</li>
												<li>Check logs for connection details</li>
											</ul>
										</li>
									</ol>
								</section>
								<div class="mt-3 flex gap-3">
									<Dialog.CloseTrigger class="btn preset-tonal">Close</Dialog.CloseTrigger>
								</div>
							</Dialog.Description>
						</Dialog.Content>
					</Dialog.Positioner>
				</Portal>
			</Dialog>
		{/snippet}
	</Tooltip.Trigger>
	<Portal>
		<Tooltip.Positioner>
			<Tooltip.Content class="card bg-surface-100-900 p-2 shadow-xl">
				<span>Help & About</span>
			</Tooltip.Content>
		</Tooltip.Positioner>
	</Portal>
</Tooltip>
