import { invoke } from '@tauri-apps/api/core'

export type StartedProvisioningHostSessionV1 = {
	hostSessionId: string
	responseRecipientHpkePublicKeyX25519: Uint8Array
}

export type AuthorizedProvisioningHostInputV1 = {
	vaultRuntimeGeneration: bigint
	vaultHpkePublicKeyX25519: Uint8Array
	audienceRegistrationId: string
	audienceRuntimeInstanceId: string
	audienceRuntimeGeneration: bigint
	audienceGrantEpoch: bigint
	leaseRequestId: Uint8Array
	leaseOperationDigestSha256: Uint8Array
	commandRequestId: Uint8Array
	leaseResponseHpkeEncappedKey: Uint8Array
	leaseResponseCiphertext: Uint8Array
	leaseResponseHpkeAuthenticationTag: Uint8Array
}

export type SealProvisioningHostInputV1 = {
	hostSessionId: string
	operationId: Uint8Array
	action: number
	secretClass: number
	secretPayload: Uint8Array
	authorized: AuthorizedProvisioningHostInputV1
}

export type SealedProvisioningHostCommandV1 = {
	operationDigestSha256: Uint8Array
	hpkeEncappedKey: Uint8Array
	ciphertext: Uint8Array
	hpkeAuthenticationTag: Uint8Array
}

export type CommittedProvisioningHostInputV1 = {
	vaultRuntimeGeneration: bigint
	commandRequestId: Uint8Array
	operationDigestSha256: Uint8Array
	receiptHpkeEncappedKey: Uint8Array
	receiptCiphertext: Uint8Array
	receiptHpkeAuthenticationTag: Uint8Array
}

export type SanitizedProvisioningHostReceiptV1 = {
	operationId: Uint8Array
	action: number
	secretRevision: bigint
	state: number
}

export interface OwnerVaultProvisioningHostV1 {
	start(): Promise<StartedProvisioningHostSessionV1>
	seal(input: SealProvisioningHostInputV1): Promise<SealedProvisioningHostCommandV1>
	openReceipt(
		hostSessionId: string,
		committed: CommittedProvisioningHostInputV1,
	): Promise<SanitizedProvisioningHostReceiptV1>
	cancel(hostSessionId: string): Promise<void>
}

type HostInvoke = (
	command: string,
	args?: Record<string, unknown>,
) => Promise<unknown>

export class NativeOwnerVaultProvisioningHostV1 implements OwnerVaultProvisioningHostV1 {
	constructor(private readonly invokeImpl: HostInvoke = invoke as HostInvoke) {}

	async start(): Promise<StartedProvisioningHostSessionV1> {
		const response = await this.invokeImpl(
			'owner_vault_provisioning_host_start',
		) as NativeStartedSession
		return {
			hostSessionId: response.hostSessionId,
			responseRecipientHpkePublicKeyX25519: bytes(response.responseRecipientHpkePublicKeyX25519),
		}
	}

	async seal(input: SealProvisioningHostInputV1): Promise<SealedProvisioningHostCommandV1> {
		const secretPayload = Array.from(input.secretPayload)
		try {
			const response = await this.invokeImpl(
				'owner_vault_provisioning_host_seal',
				{ request: {
					hostSessionId: input.hostSessionId,
					operationId: Array.from(input.operationId),
					action: input.action,
					secretClass: input.secretClass,
					secretPayload,
					authorized: {
						vaultRuntimeGeneration: input.authorized.vaultRuntimeGeneration.toString(),
						vaultHpkePublicKeyX25519: Array.from(input.authorized.vaultHpkePublicKeyX25519),
						audienceRegistrationId: input.authorized.audienceRegistrationId,
						audienceRuntimeInstanceId: input.authorized.audienceRuntimeInstanceId,
						audienceRuntimeGeneration: input.authorized.audienceRuntimeGeneration.toString(),
						audienceGrantEpoch: input.authorized.audienceGrantEpoch.toString(),
						leaseRequestId: Array.from(input.authorized.leaseRequestId),
						leaseOperationDigestSha256: Array.from(input.authorized.leaseOperationDigestSha256),
						commandRequestId: Array.from(input.authorized.commandRequestId),
						leaseResponseHpkeEncappedKey: Array.from(input.authorized.leaseResponseHpkeEncappedKey),
						leaseResponseCiphertext: Array.from(input.authorized.leaseResponseCiphertext),
						leaseResponseHpkeAuthenticationTag: Array.from(input.authorized.leaseResponseHpkeAuthenticationTag),
					},
				} },
			) as NativeSealedCommand
			return {
				operationDigestSha256: bytes(response.operationDigestSha256),
				hpkeEncappedKey: bytes(response.hpkeEncappedKey),
				ciphertext: bytes(response.ciphertext),
				hpkeAuthenticationTag: bytes(response.hpkeAuthenticationTag),
			}
		} finally {
			secretPayload.fill(0)
		}
	}

	async openReceipt(
		hostSessionId: string,
		committed: CommittedProvisioningHostInputV1,
	): Promise<SanitizedProvisioningHostReceiptV1> {
		const response = await this.invokeImpl(
			'owner_vault_provisioning_host_open_receipt',
			{ request: {
				hostSessionId,
				committed: {
					vaultRuntimeGeneration: committed.vaultRuntimeGeneration.toString(),
					commandRequestId: Array.from(committed.commandRequestId),
					operationDigestSha256: Array.from(committed.operationDigestSha256),
					receiptHpkeEncappedKey: Array.from(committed.receiptHpkeEncappedKey),
					receiptCiphertext: Array.from(committed.receiptCiphertext),
					receiptHpkeAuthenticationTag: Array.from(committed.receiptHpkeAuthenticationTag),
				},
			} },
		) as NativeSanitizedReceipt
		return {
			operationId: bytes(response.operationId),
			action: response.action,
			secretRevision: BigInt(response.secretRevision),
			state: response.state,
		}
	}

	async cancel(hostSessionId: string): Promise<void> {
		await this.invokeImpl('owner_vault_provisioning_host_cancel', { hostSessionId })
	}
}

type NativeStartedSession = {
	hostSessionId: string
	responseRecipientHpkePublicKeyX25519: number[]
}

type NativeSealedCommand = {
	operationDigestSha256: number[]
	hpkeEncappedKey: number[]
	ciphertext: number[]
	hpkeAuthenticationTag: number[]
}

type NativeSanitizedReceipt = {
	operationId: number[]
	action: number
	secretRevision: string
	state: number
}

function bytes(value: number[]): Uint8Array {
	return Uint8Array.from(value)
}
