import { writable } from 'svelte/store';

export type AccountWizardKind = 'mail' | 'calendar' | 'telegram' | 'whatsapp';
type Provider = 'gmail' | 'icloud' | 'imap';
export type AccountWizardTarget = AccountWizardKind | Provider;

export const accountWizardTarget = writable<AccountWizardTarget>('mail');
export const isAccountDrawerOpen = writable(false);

export function openAccountDrawer(target: AccountWizardTarget = 'mail'): void {
	accountWizardTarget.set(target);
	isAccountDrawerOpen.set(true);
}

export function closeAccountDrawer(): void {
	isAccountDrawerOpen.set(false);
}
