<script lang="ts">
	import { Trash2, Pencil, Ellipsis, Copy, Check, Share2 } from 'lucide-svelte';
	import { Menu, Portal } from '@skeletonlabs/skeleton-svelte';
	import { runningProfileId } from '$lib/runningProfile.svelte';

	const {
		id,
		name,
		selected,
		onSelect,
		onDelete,
		onEdit,
		onClone,
		onShare
	}: {
		id: string;
		name: string;
		selected: boolean;
		onSelect: () => void;
		onDelete: (id: string) => void;
		onEdit: (id: string) => void;
		onClone: (id: string) => void;
		onShare: (id: string) => void;
	} = $props();

	let open = $state(false);

	let isRunning = $derived($runningProfileId === id);

	function onContextMenu(e: MouseEvent) {
		e.preventDefault();
		open = true;
	}
</script>

<div class="w-full flex items-center justify-between gap-3 rounded">
	<button
		class="flex items-center gap-2 flex-1 min-w-0 text-left hover:bg-surface-100-900 rounded p-1 pl-2.5 cursor-pointer {selected
			? 'bg-surface-100-900'
			: ''}"
		onclick={onSelect}
		oncontextmenu={onContextMenu}
	>
		{#if selected}
			<Check size={16} class="text-success-500 shrink-0" />
		{/if}
		<p class="text-md truncate">{name}</p>
	</button>

	<Menu {open} onOpenChange={(e) => (open = e.open)}>
		<Menu.Trigger
			class="btn-icon"
			data-profile-trigger
			onclick={(e) => {
				e.stopPropagation();
				open = !open;
			}}
		>
			<Ellipsis size={18} />
		</Menu.Trigger>
		<Portal>
			<Menu.Positioner>
				<Menu.Content>
					<Menu.Item
						value="edit"
						disabled={isRunning}
						onclick={() => {
							open = false;
							onEdit(id);
						}}
					>
						<Menu.ItemText class="flex items-center gap-3 {isRunning ? 'opacity-50 cursor-not-allowed' : ''}">
							<Pencil size={18} />
							<span>{isRunning ? 'Edit (Disconnected to edit)' : 'Edit'}</span>
						</Menu.ItemText>
					</Menu.Item>
					<Menu.Item
						value="share"
						onclick={() => {
							open = false;
							onShare(id);
						}}
					>
						<Menu.ItemText class="flex items-center gap-3">
							<Share2 size={18} />
							<span>Share</span>
						</Menu.ItemText>
					</Menu.Item>
					<Menu.Item
						value="clone"
						onclick={() => {
							open = false;
							onClone(id);
						}}
					>
						<Menu.ItemText class="flex items-center gap-3">
							<Copy size={18} />
							<span>Clone</span>
						</Menu.ItemText>
					</Menu.Item>
					<Menu.Item
						value="delete"
						disabled={isRunning}
						onclick={() => {
							open = false;
							onDelete(id);
						}}
					>
						<Menu.ItemText class="flex items-center gap-3 {isRunning ? 'opacity-50 cursor-not-allowed' : 'text-red-500'}">
							<Trash2 size={18} class={isRunning ? '' : 'text-red-500'} />
							<span class="{isRunning ? '' : 'text-red-500'}">
								{isRunning ? 'Delete (Disconnected to delete)' : 'Delete'}
							</span>
						</Menu.ItemText>
					</Menu.Item>
				</Menu.Content>
			</Menu.Positioner>
		</Portal>
	</Menu>
</div>