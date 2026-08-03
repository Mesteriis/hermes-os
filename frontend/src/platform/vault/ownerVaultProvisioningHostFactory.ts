import { DevelopmentOwnerVaultProvisioningHostV1 } from './developmentOwnerVaultProvisioningHost'
import { AndroidOwnerVaultProvisioningHostV1 } from './ownerVaultProvisioningAndroidHost'
import {
	NativeOwnerVaultProvisioningHostV1,
	type OwnerVaultProvisioningHostV1,
} from './ownerVaultProvisioningHost'
import {
	hasAndroidOwnerVaultProvisioningHostV1,
	hasDevelopmentOwnerVaultProvisioningHostV1,
	hasNativeOwnerVaultProvisioningHostV1,
} from './provisioningHostAvailability'

export function createOwnerVaultProvisioningHostV1(): OwnerVaultProvisioningHostV1 {
	if (hasAndroidOwnerVaultProvisioningHostV1()) {
		return new AndroidOwnerVaultProvisioningHostV1()
	}
	if (hasNativeOwnerVaultProvisioningHostV1()) {
		return new NativeOwnerVaultProvisioningHostV1()
	}
	if (hasDevelopmentOwnerVaultProvisioningHostV1()) {
		return new DevelopmentOwnerVaultProvisioningHostV1()
	}
	return new UnavailableOwnerVaultProvisioningHostV1()
}

class UnavailableOwnerVaultProvisioningHostV1 implements OwnerVaultProvisioningHostV1 {
	start(): Promise<never> {
		return Promise.reject(new Error('owner Vault provisioning host is unavailable'))
	}

	seal(): Promise<never> {
		return Promise.reject(new Error('owner Vault provisioning host is unavailable'))
	}

	openReceipt(): Promise<never> {
		return Promise.reject(new Error('owner Vault provisioning host is unavailable'))
	}

	cancel(): Promise<void> {
		return Promise.resolve()
	}
}
