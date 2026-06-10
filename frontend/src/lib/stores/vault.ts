import { writable, derived } from 'svelte/store';
import type { VaultStatus as ApiVaultStatus } from '$lib/api';

export type VaultStatus = ApiVaultStatus | null;

export const vaultStatus = writable<VaultStatus>(null);
export const isVaultReady = derived(vaultStatus, ($vs) => $vs?.state === 'unlocked');
export const vaultOnboardingDismissed = writable(false);

export const vaultWizardStep = writable<'intro' | 'entropy' | 'biometric' | 'recovery' | 'done'>('intro');
export const vaultWizardError = writable('');
export const vaultWizardMessage = writable('');
export const vaultEntropyEventsCount = writable(0);
export const vaultRecovery = writable<{ path?: string; recovery_phrase?: string } | null>(null);
export const isVaultActionSubmitting = writable(false);
export const vaultStatusError = writable('');

export const shouldShowVaultOnboarding = derived(
	[vaultStatus, vaultWizardStep, vaultOnboardingDismissed],
	([$vaultStatus, $vaultWizardStep, $vaultOnboardingDismissed]) => {
		if ($vaultStatus?.state === 'locked') {
			return true;
		}
		if ($vaultOnboardingDismissed) {
			return false;
		}
		if ($vaultWizardStep === 'recovery' || $vaultWizardStep === 'done') {
			return true;
		}
		return $vaultStatus?.state === 'uninitialized';
	}
);

export function continueVaultOnboarding(): void {
	vaultOnboardingDismissed.set(true);
	vaultWizardError.set('');
	vaultWizardMessage.set('');
}
