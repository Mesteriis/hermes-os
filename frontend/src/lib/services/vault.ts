import {
	collectVaultEntropy,
	createVault,
	exportVaultRecovery,
	unlockVault,
	fetchV1Status,
	type V1Status,
	type VaultEntropyEvent
} from '$lib/api';
import {
	vaultStatus as vaultStatusStore,
	vaultWizardStep,
	vaultWizardError,
	vaultWizardMessage,
	vaultEntropyEventsCount,
	vaultRecovery,
	isVaultActionSubmitting,
	vaultOnboardingDismissed,
	vaultStatusError
} from '$lib/stores/vault';
import { get } from 'svelte/store';

export async function loadV1Status(): Promise<{ status: V1Status | null; error: string }> {
	try {
			const status = await fetchV1Status();
			vaultStatusStore.set(status.vault_status);
			vaultStatusError.set('');
			if (status.vault_status.state === 'uninitialized') {
				vaultWizardStep.set(status.vault_status.entropy_progress >= 100 ? 'biometric' : 'intro');
				vaultOnboardingDismissed.set(false);
			}
			return { status, error: '' };
	} catch (error) {
		return {
			status: null,
			error: error instanceof Error ? error.message : 'Unknown status error'
		};
	}
}

export function startVaultWizard() {
	vaultOnboardingDismissed.set(false);
	vaultWizardStep.set('entropy');
	vaultWizardError.set('');
	vaultWizardMessage.set('');
}

export type VaultEntropyState = {
	lastEntropyEvent: VaultEntropyEvent | null;
	entropyEvents: VaultEntropyEvent[];
	entropyBuffer: VaultEntropyEvent[];
	status: V1Status | null;
};

export async function handleVaultEntropyMove(
	event: MouseEvent,
	state: VaultEntropyState
): Promise<{ state: VaultEntropyState; shouldFlush: boolean }> {
	if (get(vaultWizardStep) !== 'entropy' || get(isVaultActionSubmitting)) {
		return { state, shouldFlush: false };
	}
	const previous = state.lastEntropyEvent;
	const interval = previous ? Math.max(1, event.timeStamp - previous.timestamp_ms) : 1;
	const dx = previous ? event.clientX - previous.x : 0;
	const dy = previous ? event.clientY - previous.y : 0;
	const velocity = Math.hypot(dx, dy) / interval;
	const acceleration = previous ? Math.abs(velocity - previous.velocity) / interval : 0;
	const entropyEvent: VaultEntropyEvent = {
		x: event.clientX,
		y: event.clientY,
		dx,
		dy,
		timestamp_ms: event.timeStamp,
		velocity,
		acceleration,
		interval_ms: interval
	};
	const entropyEvents = [...state.entropyEvents, entropyEvent].slice(-2000);
	const entropyBuffer = [...state.entropyBuffer, entropyEvent];
	const shouldFlush = entropyBuffer.length >= 100;
	return {
		state: {
			...state,
			lastEntropyEvent: entropyEvent,
			entropyEvents,
			entropyBuffer
		},
		shouldFlush
	};
}

export async function flushVaultEntropy(
	state: VaultEntropyState
): Promise<{ state: VaultEntropyState; error: string }> {
	if (state.entropyBuffer.length === 0) {
		return { state, error: '' };
	}
	const events = state.entropyBuffer;
	try {
		const vault_status = await collectVaultEntropy(events);
		const nextStatus = state.status ? { ...state.status, vault_status } : null;
			vaultStatusStore.set(vault_status);
			vaultStatusError.set('');
			vaultEntropyEventsCount.update((n) => n + events.length);
		if (vault_status.entropy_progress >= 100) {
			vaultWizardStep.set('biometric');
		}
		return {
			state: {
				...state,
				status: nextStatus,
				entropyBuffer: []
			},
			error: ''
		};
	} catch (error) {
			vaultWizardError.set(error instanceof Error ? error.message : 'Vault entropy failed');
			vaultStatusError.set(error instanceof Error ? error.message : 'Vault entropy failed');
		return {
			state: { ...state, entropyBuffer: [] },
			error: error instanceof Error ? error.message : 'Vault entropy failed'
		};
	}
}

export async function createSecureVault(
	state: VaultEntropyState
): Promise<{ state: VaultEntropyState; error: string }> {
	if (get(isVaultActionSubmitting)) {
		return { state, error: '' };
	}
	isVaultActionSubmitting.set(true);
	vaultWizardError.set('');
	try {
		const flushed = await flushVaultEntropy(state);
		if (flushed.error) return { state: flushed.state, error: flushed.error };
		const vault_status = await createVault();
		const nextStatus = flushed.state.status ? { ...flushed.state.status, vault_status } : null;
			vaultStatusStore.set(vault_status);
			vaultOnboardingDismissed.set(false);
			vaultStatusError.set('');
			vaultWizardStep.set('recovery');
		vaultWizardMessage.set('Vault created. Export recovery material before continuing.');
		return { state: { ...flushed.state, status: nextStatus }, error: '' };
	} catch (error) {
			vaultWizardError.set(error instanceof Error ? error.message : 'Vault create failed');
			vaultStatusError.set(error instanceof Error ? error.message : 'Vault create failed');
		return { state, error: error instanceof Error ? error.message : 'Vault create failed' };
	} finally {
		isVaultActionSubmitting.set(false);
	}
}

export async function unlockSecureVault(
	state: VaultEntropyState
): Promise<{ state: VaultEntropyState; error: string }> {
	if (get(isVaultActionSubmitting)) {
		return { state, error: '' };
	}
	isVaultActionSubmitting.set(true);
	vaultWizardError.set('');
	try {
		const vault_status = await unlockVault();
		const nextStatus = state.status ? { ...state.status, vault_status } : null;
			vaultStatusStore.set(vault_status);
			vaultStatusError.set('');
			vaultWizardMessage.set('Vault unlocked for this Hermes session.');
		return { state: { ...state, status: nextStatus }, error: '' };
	} catch (error) {
			vaultWizardError.set(error instanceof Error ? error.message : 'Vault unlock failed');
			vaultStatusError.set(error instanceof Error ? error.message : 'Vault unlock failed');
		return { state, error: error instanceof Error ? error.message : 'Vault unlock failed' };
	} finally {
		isVaultActionSubmitting.set(false);
	}
}

export async function exportRecoveryMaterial(): Promise<{ error: string }> {
	if (get(isVaultActionSubmitting)) {
		return { error: '' };
	}
	isVaultActionSubmitting.set(true);
	vaultWizardError.set('');
	try {
			vaultRecovery.set(await exportVaultRecovery());
			vaultOnboardingDismissed.set(false);
			vaultStatusError.set('');
			vaultWizardStep.set('done');
		vaultWizardMessage.set('Recovery material exported. Store it outside the app.');
		return { error: '' };
	} catch (error) {
			vaultWizardError.set(error instanceof Error ? error.message : 'Vault recovery export failed');
			vaultStatusError.set(error instanceof Error ? error.message : 'Vault recovery export failed');
		return { error: error instanceof Error ? error.message : 'Vault recovery export failed' };
	} finally {
		isVaultActionSubmitting.set(false);
	}
}
