<script lang="ts">
	import { Import } from 'lucide-svelte';
	import { Tooltip, Portal, Dialog } from '@skeletonlabs/skeleton-svelte';
	import { profilesStore } from '$lib/profiles.svelte';
	import { logMessage } from '$lib/logger';
	import { showToast } from '$lib/components/ToastGroup.svelte';

	let open = $state(false);
	let importText = $state('');

	async function handleImport() {
		try {
			if (!importText.trim()) {
				showToast({
					type: 'error',
					title: 'Import failed',
					description: 'Please enter a profile string.',
				});
				return;
			}

			const result = await profilesStore.importProfile(importText);

			importText = '';

			if (result.warnings.length > 0) {
				showToast({
					type: 'warning',
					title: 'Missing profile fields',
					description: `The imported profile is missing fields: ${result.warnings.join(', ')}. If connection fails, manually enter correct values.`,
				});
				return;
			}

			showToast({
				type: 'success',
				title: 'Profile imported',
				description: 'The profile was imported successfully.',
			});
			open = false;
		} catch (e: unknown) {
			const message = e instanceof Error ? e.message : String(e);
			await logMessage('error', 'ImportButton', `profile import failed: ${message}`);
			showToast({
				type: 'error',
				title: 'Import failed',
				description: message || 'Invalid profile string.',
			});
		}
	}

	function resetState() {
		importText = '';
	}
</script>

<Tooltip positioning={{ placement: 'top' }}>
	<Tooltip.Trigger>
		{#snippet element(attrs)}
			<Dialog open={open} onOpenChange={(e) => {
				open = e.open;
				if (!e.open) resetState();
			}}>
				<Dialog.Trigger
					class="bg-primary-500 btn-icon preset-filled"
					{...attrs}
					onclick={() => (open = true)}
				>
					<Import size={18} />
				</Dialog.Trigger>
				<Portal>
					<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" />
					<Dialog.Positioner class="fixed inset-0 z-50 flex justify-center items-center">
						<Dialog.Content class="card bg-surface-100-900 w-md p-4 space-y-2 shadow-xl">
							<Dialog.Title class="text-xl font-400">Import nipovpn profile</Dialog.Title>
							<Dialog.Description class="flex flex-col gap-2">
								<textarea
									placeholder="nipovpn://"
									rows="6"
									bind:value={importText}
									class="bg-surface-300-700 p-2 rounded-base w-full focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0 text-sm font-mono"
								></textarea>
								<div class="mt-3 flex gap-3">
									<Dialog.CloseTrigger class="btn preset-tonal">Close</Dialog.CloseTrigger>
									<button type="button" onclick={handleImport} class="btn bg-primary-500 preset-filled">Add</button>
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
				<span>Import Profile</span>
			</Tooltip.Content>
		</Tooltip.Positioner>
	</Portal>
</Tooltip>