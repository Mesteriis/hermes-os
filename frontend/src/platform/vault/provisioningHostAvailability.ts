export function hasNativeOwnerVaultProvisioningHostV1(): boolean {
	return typeof window !== 'undefined'
		&& '__TAURI_INTERNALS__' in window
}

export function hasDevelopmentOwnerVaultProvisioningHostV1(): boolean {
	return import.meta.env.VITE_HERMES_DEV_OWNER_VAULT_HOST === '1'
}

export function hasOwnerVaultProvisioningHostV1(): boolean {
	return hasNativeOwnerVaultProvisioningHostV1()
		|| hasDevelopmentOwnerVaultProvisioningHostV1()
}
