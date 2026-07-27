import { describe, expect, it, vi } from 'vitest'

import { NativeOwnerVaultProvisioningHostV1 } from './ownerVaultProvisioningHost'

describe('NativeOwnerVaultProvisioningHostV1', () => {
	it('uses decimal u64 fields and clears the temporary secret transfer array', async () => {
		let transferredSecret: number[] | undefined
		const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
			if (command !== 'owner_vault_provisioning_host_seal') throw new Error('unexpected command')
			const request = args?.request as { secretPayload: number[] }
			transferredSecret = request.secretPayload
			return {
				operationDigestSha256: Array.from({ length: 32 }, () => 1),
				hpkeEncappedKey: Array.from({ length: 32 }, () => 2),
				ciphertext: [3],
				hpkeAuthenticationTag: Array.from({ length: 16 }, () => 4),
			}
		})
		const host = new NativeOwnerVaultProvisioningHostV1(invoke)

		await host.seal({
			hostSessionId: 'host-session',
			operationId: new Uint8Array(16).fill(1),
			action: 1,
			secretClass: 1,
			secretPayload: Uint8Array.from([7, 8]),
			authorized: {
				vaultRuntimeGeneration: 9_007_199_254_740_993n,
				vaultHpkePublicKeyX25519: new Uint8Array(32).fill(2),
				audienceRegistrationId: 'registration',
				audienceRuntimeInstanceId: 'runtime',
				audienceRuntimeGeneration: 2n,
				audienceGrantEpoch: 3n,
				leaseRequestId: new Uint8Array(16).fill(4),
				leaseOperationDigestSha256: new Uint8Array(32).fill(5),
				commandRequestId: new Uint8Array(16).fill(6),
				leaseResponseHpkeEncappedKey: new Uint8Array(32).fill(7),
				leaseResponseCiphertext: new Uint8Array([8]),
				leaseResponseHpkeAuthenticationTag: new Uint8Array(16).fill(9),
			},
		})

		const request = invoke.mock.calls[0]?.[1]?.request as {
			authorized: { vaultRuntimeGeneration: string }
		}
		expect(request.authorized.vaultRuntimeGeneration).toBe('9007199254740993')
		expect(transferredSecret).toEqual([0, 0])
	})
})
