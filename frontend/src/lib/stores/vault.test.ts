import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';
import {
	continueVaultOnboarding,
	shouldShowVaultOnboarding,
	vaultOnboardingDismissed,
	vaultStatus,
	vaultWizardStep
} from './vault';
import type { VaultStatus } from '$lib/api';

function status(state: VaultStatus['state'], entropyProgress = 0): VaultStatus {
	return {
		state,
		needs_entropy: state === 'uninitialized',
		needs_biometric: false,
		needs_recovery: state === 'uninitialized',
		version: 1,
		recoverable: false,
		entropy_progress: entropyProgress
	};
}

describe('vault store', () => {
	beforeEach(() => {
		vaultStatus.set(null);
		vaultWizardStep.set('intro');
		vaultOnboardingDismissed.set(false);
	});

	it('keeps onboarding visible through recovery even after vault creation unlocks the vault', () => {
		vaultStatus.set(status('uninitialized'));
		expect(get(shouldShowVaultOnboarding)).toBe(true);

		vaultStatus.set(status('unlocked', 100));
		vaultWizardStep.set('recovery');
		expect(get(shouldShowVaultOnboarding)).toBe(true);

		vaultWizardStep.set('done');
		expect(get(shouldShowVaultOnboarding)).toBe(true);

		continueVaultOnboarding();
		expect(get(shouldShowVaultOnboarding)).toBe(false);
	});

	it('shows onboarding again when an initialized vault is locked after backend restart', () => {
		vaultStatus.set(status('unlocked', 100));
		continueVaultOnboarding();
		expect(get(shouldShowVaultOnboarding)).toBe(false);

		vaultStatus.set(status('locked', 100));
		expect(get(shouldShowVaultOnboarding)).toBe(true);
	});
});
