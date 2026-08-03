import { describe, expect, it, vi } from 'vitest'

import { AndroidOwnerVaultProvisioningHostV1 } from './ownerVaultProvisioningAndroidHost'

describe('AndroidOwnerVaultProvisioningHostV1', () => {
	it('uses host bridge protocol and clears temporary secret payload transfer', async () => {
		const startResponse = {
			host_session_id: 'android-host-session',
			response_recipient_hpke_public_key_x25519: Array.from({ length: 32 }, (_, i) => i),
		}
		const sealResponse = {
			operation_digest_sha256: Array.from({ length: 32 }, (_, i) => i + 1),
			hpke_encapped_key: [2],
			ciphertext: [3],
			hpke_authentication_tag: Array.from({ length: 16 }, (_, i) => i + 4),
		}
		const openReceiptResponse = {
			operation_id: Array.from({ length: 16 }, (_, i) => i + 5),
			action: 1,
			secret_revision: '7',
			state: 2,
		}

		let sealedSecret: number[] | undefined
		const host = new AndroidOwnerVaultProvisioningHostV1({
			vaultProvisioningHost: {
				start: vi.fn(async () => startResponse),
				seal: vi.fn(async ({ request }) => {
					sealedSecret = (request as { secret_payload: number[] }).secret_payload
					return sealResponse
				}),
				open_receipt: vi.fn(async () => openReceiptResponse),
				cancel: vi.fn(async () => undefined),
			},
		})

		const started = await host.start()
		expect(started.hostSessionId).toBe(startResponse.host_session_id)
		expect(started.responseRecipientHpkePublicKeyX25519).toHaveLength(32)

		const authorized = {
			vaultRuntimeGeneration: 9n,
			vaultHpkePublicKeyX25519: new Uint8Array(32),
			audienceRegistrationId: 'registration',
			audienceRuntimeInstanceId: 'runtime',
			audienceRuntimeGeneration: 3n,
			audienceGrantEpoch: 4n,
			leaseRequestId: new Uint8Array(16).fill(8),
			leaseOperationDigestSha256: new Uint8Array(32).fill(9),
			commandRequestId: new Uint8Array(16).fill(10),
			leaseResponseHpkeEncappedKey: new Uint8Array(8).fill(11),
			leaseResponseCiphertext: new Uint8Array([12]),
			leaseResponseHpkeAuthenticationTag: new Uint8Array(16).fill(13),
		}
		const secretPayload = new Uint8Array([7, 8])
		const sealed = await host.seal({
			hostSessionId: started.hostSessionId,
			operationId: new Uint8Array(16).fill(1),
			action: 1,
			secretClass: 1,
			secretPayload,
			authorized,
		})
		expect(sealed.operationDigestSha256).toEqual(new Uint8Array(sealResponse.operation_digest_sha256))
		expect(sealedSecret).toEqual([0, 0])

		const receipt = await host.openReceipt(started.hostSessionId, {
			vaultRuntimeGeneration: 9n,
			commandRequestId: new Uint8Array(16).fill(10),
			operationDigestSha256: new Uint8Array(32).fill(1),
			receiptHpkeEncappedKey: new Uint8Array([1]),
			receiptCiphertext: new Uint8Array([2]),
			receiptHpkeAuthenticationTag: new Uint8Array([3]),
		})
		expect(receipt.secretRevision).toBe(7n)
		expect(receipt.state).toBe(2)
		await expect(host.cancel(started.hostSessionId)).resolves.toBeUndefined()
	})

	it('fails when android bridge is unavailable', () => {
		expect(() => new AndroidOwnerVaultProvisioningHostV1({})).toThrow(
			'android host provisioning bridge is unavailable',
		)
	})
})
