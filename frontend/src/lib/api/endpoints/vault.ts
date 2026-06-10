import { ApiClient } from '../client';
import type { V1Status, VaultStatus, VaultEntropyEvent, VaultRecoveryExportResponse } from '../types';

export async function fetchV1Status(): Promise<V1Status> {
	return ApiClient.instance.get<V1Status>('/api/v1/status', 'V1 status request failed');
}

export async function fetchVaultStatus(): Promise<VaultStatus> {
	return ApiClient.instance.get<VaultStatus>('/api/v1/vault/status', 'Vault status request failed');
}

export async function collectVaultEntropy(events: VaultEntropyEvent[]): Promise<VaultStatus> {
	return ApiClient.instance.post<VaultStatus>('/api/v1/vault/collect-entropy', { events }, 'Vault entropy request failed');
}

export async function createVault(): Promise<VaultStatus> {
	return ApiClient.instance.post<VaultStatus>('/api/v1/vault/create', {}, 'Vault create request failed');
}

export async function unlockVault(): Promise<VaultStatus> {
	return ApiClient.instance.post<VaultStatus>('/api/v1/vault/unlock', {}, 'Vault unlock request failed');
}

export async function exportVaultRecovery(): Promise<VaultRecoveryExportResponse> {
	return ApiClient.instance.post<VaultRecoveryExportResponse>('/api/v1/vault/recovery/export', {}, 'Vault recovery export failed');
}

export async function importVaultRecovery(recovery_phrase: string): Promise<VaultStatus> {
	return ApiClient.instance.post<VaultStatus>('/api/v1/vault/recovery/import', { recovery_phrase }, 'Vault recovery import failed');
}
