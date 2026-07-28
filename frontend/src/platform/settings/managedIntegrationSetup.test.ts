import { describe, expect, it, vi } from 'vitest'

import { ManagedIntegrationSetupV1 } from './managedIntegrationSetup'

describe('ManagedIntegrationSetupV1', () => {
	it('applies the receipt revision without interpreting provider values', async () => {
		const updateDesired = vi.fn().mockResolvedValue({
			registrationId: 'provider-registration',
			desiredRevision: 2n,
		})
		const applyManagedIntegration = vi.fn().mockResolvedValue({
			registrationId: 'provider-registration',
			effectiveRevision: 2n,
		})
		const setup = new ManagedIntegrationSetupV1({
			updateDesired,
			applyManagedIntegration,
		} as never)

		const receipt = await setup.apply({
			registrationId: 'provider-registration',
			expectedDesiredRevision: 1n,
			storageCapabilityId: 'provider.storage.v1',
			configurationInstanceId: 'account-1',
			requestHostBridge: true,
			values: [{
				settingId: 'provider.account_id',
				value: { case: 'stringValue', value: 'account-1' },
			}],
		})

		expect(updateDesired).toHaveBeenCalledWith(expect.objectContaining({
			expectedDesiredRevision: 1n,
		}))
		expect(applyManagedIntegration).toHaveBeenCalledWith(expect.objectContaining({
			expectedDesiredRevision: 2n,
			requestHostBridge: true,
		}))
		expect(receipt.application.effectiveRevision).toBe(2n)
	})

	it('forwards stable operation IDs for recovery retries', async () => {
		const createConfigurationTarget = vi.fn().mockResolvedValue({
			configurationInstanceId: 'target-1',
			desiredRevision: 1n,
		})
		const updateDesired = vi.fn().mockResolvedValue({ desiredRevision: 2n })
		const applyManagedIntegration = vi.fn().mockResolvedValue({ effectiveRevision: 2n })
		const setup = new ManagedIntegrationSetupV1({
			createConfigurationTarget,
			updateDesired,
			applyManagedIntegration,
		} as never)
		const createOperationId = new Uint8Array(16).fill(1)
		const updateOperationId = new Uint8Array(16).fill(2)
		const applyOperationId = new Uint8Array(16).fill(3)

		await setup.createTarget('provider-registration', createOperationId)
		await setup.apply({
			registrationId: 'provider-registration',
			expectedDesiredRevision: 1n,
			storageCapabilityId: 'provider.storage.v1',
			configurationInstanceId: 'target-1',
			requestHostBridge: false,
			values: [{
				settingId: 'provider.account_id',
				value: { case: 'stringValue', value: 'account-1' },
			}],
			updateOperationId,
			applyOperationId,
		})

		expect(createConfigurationTarget).toHaveBeenCalledWith({
			registrationId: 'provider-registration',
			operationId: createOperationId,
		})
		expect(updateDesired).toHaveBeenCalledWith(expect.objectContaining({
			operationId: updateOperationId,
		}))
		expect(applyManagedIntegration).toHaveBeenCalledWith(expect.objectContaining({
			operationId: applyOperationId,
		}))
	})
})
