<script lang="ts">
	import ProfileItem from '$lib/components/profiles/ProfileItem.svelte';
	import AddProfileButton from '$lib/components/profiles/AddProfileButton.svelte';
	import ImportButton from '$lib/components/profiles/ImportButton.svelte';
	import DeleteProfileDialog from '$lib/components/profiles/DeleteProfileDialog.svelte';
	import EditProfileDialog from '$lib/components/profiles/EditProfileDialog.svelte';
	import ShareProfileDialog from '$lib/components/profiles/ShareProfileDialog.svelte';
	import { profilesStore, type Profile } from '$lib/profiles.svelte';

	let profileToDelete = $state<Profile | null>(null);
	let deleteDialogOpen = $state(false);

	let profileToEdit = $state<Profile | null>(null);
	let editDialogOpen = $state(false);

	let profileToShare = $state<Profile | null>(null);
	let shareDialogOpen = $state(false);

	function triggerDelete(id: string) {
		const profile = profilesStore.profiles.find((p) => p.id === id);
		if (profile) {
			profileToDelete = profile;
			deleteDialogOpen = true;
		}
	}

	function triggerEdit(id: string) {
		const profile = profilesStore.profiles.find((p) => p.id === id);
		if (profile) {
			profileToEdit = JSON.parse(JSON.stringify(profile));
			editDialogOpen = true;
		}
	}

	function triggerShare(id: string) {
		const profile = profilesStore.profiles.find((p) => p.id === id);
		if (profile) {
			profileToShare = profile;
			shareDialogOpen = true;
		}
	}

	async function triggerClone(id: string) {
		const profile = profilesStore.profiles.find((p) => p.id === id);
		if (profile) {
			await profilesStore.addProfile(`${profile.name} (Copy)`, JSON.parse(JSON.stringify(profile.config)));
		}
	}
</script>

<div class="card p-4 preset-outlined-surface-500">
	<div class="flex items-center justify-between">
		<p class="text-xl">Profiles</p>
		<div class="flex gap-3">
			<AddProfileButton />
			<ImportButton />
		</div>
	</div>
	<hr class="w-full border-t border-surface-300-700 my-3" />
	
	{#if profilesStore.isLoading}
		<div class="text-surface-500 text-sm py-4 text-center">Loading profiles...</div>
	{:else}
		<div class="flex flex-col gap-1">
			{#each profilesStore.profiles as profile (profile.id)}
				<ProfileItem
					id={profile.id}
					name={profile.name}
					selected={profilesStore.selectedProfileId === profile.id}
					onSelect={() => profilesStore.selectProfile(profile.id)}
					onDelete={triggerDelete}
					onEdit={triggerEdit}
					onClone={triggerClone}
					onShare={triggerShare}
				/>
			{/each}
			{#if profilesStore.profiles.length === 0}
				<div class="text-surface-500 text-sm py-4 text-center italic">No profiles. Add or Import one!</div>
			{/if}
		</div>
	{/if}
</div>

<DeleteProfileDialog
	open={deleteDialogOpen}
	profile={profileToDelete}
	onConfirm={async () => {
		if (profileToDelete) {
			await profilesStore.deleteProfile(profileToDelete.id);
		}
		deleteDialogOpen = false;
		profileToDelete = null;
	}}
	onCancel={() => {
		deleteDialogOpen = false;
		profileToDelete = null;
	}}
/>

{#if editDialogOpen && profileToEdit}
	<EditProfileDialog
		open={editDialogOpen}
		profile={profileToEdit}
		onClose={() => {
			editDialogOpen = false;
			profileToEdit = null;
		}}
	/>
{/if}

<ShareProfileDialog
	open={shareDialogOpen}
	profile={profileToShare}
	onClose={() => {
		shareDialogOpen = false;
		profileToShare = null;
	}}
/>
