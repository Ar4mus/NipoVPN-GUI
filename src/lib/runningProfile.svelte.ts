import { writable } from 'svelte/store';

export const runningProfileId = writable<string | null>(null);
