<script lang="ts">
	import { Copy, Check } from 'lucide-svelte';
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import { profilesStore, type Profile } from '$lib/profiles.svelte';
	import { logMessage } from '$lib/logger';
	import { showToast } from '$lib/components/ToastGroup.svelte';

	interface Props {
		open: boolean;
		profile: Profile | null;
		onClose: () => void;
	}

	let { open, profile, onClose }: Props = $props();

	let shareString = $state('');
	let copied = $state(false);
	let copyTimeoutId: ReturnType<typeof setTimeout> | null = null;

	$effect(() => {
		if (profile) {
			try {
				shareString = profilesStore.exportProfile(profile);
			} catch (e) {
				const message = e instanceof Error ? e.message : String(e);
				void logMessage('error', 'ShareProfileDialog', `Failed to export profile: ${message}`);
				console.error('Failed to export profile:', e);
				shareString = '';
				showToast({
					type: 'error',
					title: 'Share export failed',
					description: message || 'Could not export this profile for sharing.'
				});
			}
		}
	});

	async function copyToClipboard() {
		if (!shareString.trim()) {
			showToast({
				type: 'warning',
				title: 'Nothing to copy',
				description: 'No share code is available for this profile.'
			});
			return;
		}

		try {
			await navigator.clipboard.writeText(shareString);
			copied = true;
			showToast({
				type: 'success',
				title: 'Copied',
				description: 'Profile share code copied to clipboard.'
			});
			if (copyTimeoutId) clearTimeout(copyTimeoutId);
			copyTimeoutId = setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (e) {
			const message = e instanceof Error ? e.message : String(e);
			showToast({
				type: 'error',
				title: 'Copy failed',
				description: message || 'Could not copy the profile share code.'
			});
		}
	}

	function handleClose() {
		copied = false;
		if (copyTimeoutId) clearTimeout(copyTimeoutId);
		onClose();
	}
</script>

<Dialog
	{open}
	onOpenChange={(e) => {
		if (!e.open) handleClose();
	}}
>
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" onclick={handleClose} />
		<Dialog.Positioner class="fixed inset-0 z-50 flex justify-center items-center">
			<Dialog.Content
				role="dialog"
				aria-modal="true"
				aria-labelledby="share-dialog-title"
				aria-describedby="share-dialog-desc"
				class="card bg-surface-100-900 w-md p-4 space-y-2 shadow-xl"
			>
				<Dialog.Title id="share-dialog-title" class="text-xl font-400">
					Share "{profile?.name}"
				</Dialog.Title>
				<Dialog.Description id="share-dialog-desc" class="flex flex-col gap-2">
					<p class="text-sm text-surface-400">Copy and share this profile code:</p>
					<textarea
						readonly
						value={shareString}
						rows="6"
						class="bg-surface-300-700 p-2 rounded-base w-full focus:outline-surface-800-200 focus:outline-1 focus:outline-offset-0 text-sm font-mono"
					></textarea>
					<div class="mt-3 flex gap-3">
						<Dialog.CloseTrigger class="btn preset-tonal">Close</Dialog.CloseTrigger>
						<button
							type="button"
							onclick={copyToClipboard}
							class="btn bg-primary-500 preset-filled flex items-center gap-2"
						>
							{#if copied}
								<Check size={18} />
								<span>Copied!</span>
							{:else}
								<Copy size={18} />
								<span>Copy</span>
							{/if}
						</button>
					</div>
				</Dialog.Description>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
