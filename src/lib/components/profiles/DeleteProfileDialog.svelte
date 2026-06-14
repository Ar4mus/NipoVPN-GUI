<script lang="ts">
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { logMessage } from '$lib/logger';
	import { showToast } from '$lib/components/ToastGroup.svelte';

	interface Props {
		open: boolean;
		profile: { id: string; name: string } | null;
		errorMessage?: string | null;
		onConfirm: () => void;
		onCancel: () => void;
		isRunning?: boolean;
		onBeforeDelete?: () => Promise<void>;
	}

	let { open, profile, errorMessage = null, onConfirm, onCancel, isRunning = false, onBeforeDelete }: Props = $props();

	let isDeleting = $state(false);

	async function handleDelete() {
		if (!profile) {
			showToast({
				type: 'warning',
				title: 'Profile unavailable',
				description: 'No profile is selected for deletion.'
			});
			return;
		}

		const profileId = profile.id;
		const profileName = profile.name;

		isDeleting = true;

		try {
			if (onBeforeDelete) {
				await onBeforeDelete();
			}

			await invoke('delete_profile_config', { profileId });
			await onConfirm();

			showToast({
				type: 'success',
				title: 'Profile deleted',
				description: `"${profileName}" has been deleted.`
			});
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			await logMessage('error', 'DeleteProfileDialog', `delete_profile_config failed: ${message}`);
			console.error('Failed to delete profile:', error);
			showToast({
				type: 'error',
				title: 'Delete failed',
				description: message || `Could not delete "${profileName}".`
			});
		} finally {
			isDeleting = false;
		}
	}
</script>

<Dialog
	{open}
	onOpenChange={(e) => {
		if (!e.open) onCancel();
	}}
>
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" onclick={onCancel} />
		<Dialog.Positioner class="fixed inset-0 z-50 flex justify-center items-center">
			<Dialog.Content
				role="dialog"
				aria-modal="true"
				aria-labelledby="delete-dialog-title"
				aria-describedby="delete-dialog-desc"
				class="card bg-surface-100-900 w-sm p-6 space-y-4 shadow-xl"
			>
				<Dialog.Title id="delete-dialog-title" class="text-lg font-semibold">
					Delete "{profile?.name}"
				</Dialog.Title>
				<Dialog.Description id="delete-dialog-desc">
					<p>Are you sure you want to delete the profile <strong>{profile?.name}</strong>?</p>
					{#if isRunning}
						<p class="text-warning-500 text-sm mt-2">This profile is currently running. It will be stopped.</p>
					{/if}
					<p class="text-error-500 text-xs mt-2">This action cannot be undone.</p>
				</Dialog.Description>
				{#if errorMessage}
					<p class="text-error-500 text-sm" role="alert">{errorMessage}</p>
				{/if}
				<div class="flex gap-3">
					<button 
						class="btn preset-tonal" 
						onclick={onCancel} 
						aria-label="Cancel"
						disabled={isDeleting}
					> 
						Cancel 
					</button>
					<button
						class="btn preset-filled-error-500"
						onclick={handleDelete}
						aria-label="Delete profile {profile?.name}"
						disabled={isDeleting}
					>
						{isDeleting ? 'Deleting...' : 'Delete'}
					</button>
				</div>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
