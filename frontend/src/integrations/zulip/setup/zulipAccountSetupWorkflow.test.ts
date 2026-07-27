import { describe, expect, it, vi } from 'vitest'

import { ZulipAccountSetupWorkflowV1 } from './zulipAccountSetupWorkflow'

describe('ZulipAccountSetupWorkflowV1', () => {
	it('keeps settings, Vault provisioning, binding and activation as distinct receipts', async () => {
		const provision = vi.fn().mockResolvedValue({ secretRevision: 1n })
		const apply = vi.fn().mockResolvedValue({
			settings: { desiredRevision: 2n },
			application: { effectiveRevision: 2n },
		})
		const bind = vi.fn().mockResolvedValue({ bindingRevision: 1n })
		const activate = vi.fn().mockResolvedValue({ effectiveRevision: 2n })
		const workflow = new ZulipAccountSetupWorkflowV1({
			configuration: { apply },
			vault: { provision },
			lifecycle: { bind },
			activation: { applyManagedIntegration: activate },
		} as never)

		await workflow.setup({
			registrationId: 'zulip-registration',
			expectedDesiredRevision: 1n,
			accountId: 'work',
			accountEmail: 'account@example.com',
			realmUrl: 'https://example.zulipchat.com',
			apiKey: new TextEncoder().encode('secret'),
		})

		expect(provision).toHaveBeenCalledWith(expect.objectContaining({
			capabilityId: 'zulip.api-key.credential-provisioning.v1',
			purposeId: 'zulip_api_key',
		}))
		expect(bind).toHaveBeenCalledWith({
			accountId: 'work',
			expectedBindingRevision: 0n,
			credentialRevision: 1n,
		})
		expect(activate).toHaveBeenCalledWith(expect.objectContaining({
			expectedDesiredRevision: 2n,
		}))
	})
})
