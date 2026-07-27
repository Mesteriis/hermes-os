export function hasNativeOwnerVaultProvisioningHostV1(): boolean {
	return typeof window !== 'undefined'
		&& '__TAURI_INTERNALS__' in window
}
