<script lang="ts" module>
	import { createToaster } from '@skeletonlabs/skeleton-svelte';
	import { BadgeCheck, CircleX, TriangleAlert, Info } from 'lucide-svelte';
	import { Toast } from '@skeletonlabs/skeleton-svelte';

	export type ToastType = 'info' | 'success' | 'warning' | 'error';

	interface ShowToastOptions {
		type?: ToastType;
		title?: string;
		description: string;
		duration?: number;
	}

	const defaultTitles: Record<ToastType, string> = {
		info: 'Info',
		success: 'Success',
		warning: 'Warning',
		error: 'Error'
	};

	export const toastIcons: Record<ToastType, typeof BadgeCheck> = {
		info: Info,
		success: BadgeCheck,
		warning: TriangleAlert,
		error: CircleX
	};

	export const toastIconClasses: Record<ToastType, string> = {
		info: 'text-primary-500',
		success: 'text-success-500',
		warning: 'text-warning-500',
		error: 'text-error-500'
	};

	export const toaster = createToaster({
		placement: 'top',
		overlap: false
	});

	export function showToast({
		type = 'info',
		title = defaultTitles[type],
		description,
		duration = 4000
	}: ShowToastOptions) {
		toaster[type]({
			title,
			description,
			duration
		});
	}

	export function getToastType(toast: { type?: unknown }): ToastType {
		return typeof toast.type === 'string' && toast.type in toastIcons
			? (toast.type as ToastType)
			: 'info';
	}
</script>

<Toast.Group {toaster}>
	{#snippet children(toast)}
		{@const type = getToastType(toast)}
		<Toast {toast}>
			<Toast.Message class="flex items-start gap-2">
				{#if type === 'success'}
					<BadgeCheck class="size-5 shrink-0" style="color: oklch(var(--color-success-500))" />
				{:else if type === 'warning'}
					<TriangleAlert class="size-5 shrink-0" style="color: oklch(var(--color-warning-500))" />
				{:else if type === 'error'}
					<CircleX class="shrink-0" style="color: oklch(var(--color-error-500))" />
				{:else}
					<Info class="shrink-0" style="color: oklch(var(--color-primary-500))" />
				{/if}
				<div class="min-w-0 flex-1">
					<Toast.Title>{toast.title}</Toast.Title>
					<Toast.Description>{toast.description}</Toast.Description>
				</div>
			</Toast.Message>
			<Toast.CloseTrigger />
		</Toast>
	{/snippet}
</Toast.Group>
