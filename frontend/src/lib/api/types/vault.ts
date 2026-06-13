export type V1Status = {
	version: string;
	surfaces: {
		messages: boolean;
		persons: boolean;
		search: boolean;
		documents: boolean;
		account_setup: boolean;
	};
	vault_status: VaultStatus;
};

export type VaultMode = 'uninitialized' | 'locked' | 'unlocked';

export type VaultStatus = {
	state: VaultMode;
	needs_entropy: boolean;
	needs_biometric: boolean;
	needs_recovery: boolean;
	version: number;
	recoverable: boolean;
	entropy_progress: number;
};

export type VaultEntropyEvent = {
	x: number;
	y: number;
	dx: number;
	dy: number;
	timestamp_ms: number;
	velocity: number;
	acceleration: number;
	interval_ms: number;
};

export type VaultRecoveryExportResponse = {
	path: string;
	recovery_phrase: string;
};
