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
})
