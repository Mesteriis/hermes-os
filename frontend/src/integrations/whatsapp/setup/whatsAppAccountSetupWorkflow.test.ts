import { describe, expect, it, vi } from 'vitest'

import { WhatsAppAccountSetupWorkflowV1 } from './whatsAppAccountSetupWorkflow'

describe('WhatsAppAccountSetupWorkflowV1', () => {
	it('requests the host bridge without exposing WhatsApp semantics to Settings', async () => {
		const apply = vi.fn().mockResolvedValue({ application: {} })
		const workflow = new WhatsAppAccountSetupWorkflowV1({ apply })

		await workflow.setup({
			registrationId: 'whatsapp-registration',
			expectedDesiredRevision: 1n,
			accountId: ' personal ',
		})

		expect(apply).toHaveBeenCalledWith(expect.objectContaining({
			storageCapabilityId: 'whatsapp.storage.v1',
			configurationInstanceId: 'personal',
			requestHostBridge: true,
			values: [{
				settingId: 'whatsapp.account_id',
				value: { case: 'stringValue', value: 'personal' },
			}],
		}))
	})
})
