import { describe, expect, it, vi } from 'vitest'

import { ManagedWorkflowSetupV1 } from './managedWorkflowSetup'

describe('ManagedWorkflowSetupV1', () => {
	it('updates desired settings and uses the distinct workflow apply operation', async () => {
		const settings = {
			createConfigurationTarget: vi.fn(),
			updateDesired: vi.fn().mockResolvedValue({ desiredRevision: 3n }),
			applyManagedWorkflow: vi.fn().mockResolvedValue({
				configurationInstanceId: 'configuration-1',
				effectiveRevision: 3n,
			}),
		}
		const setup = new ManagedWorkflowSetupV1(settings)
		await setup.apply({
			registrationId: 'workflow-registration',
			configurationInstanceId: 'configuration-1',
			expectedDesiredRevision: 2n,
			storageCapabilityId: 'workflow.storage.v1',
			values: [{ settingId: 'workflow.enabled', value: { case: 'booleanValue', value: true } }],
		})

		expect(settings.updateDesired).toHaveBeenCalledOnce()
		expect(settings.applyManagedWorkflow).toHaveBeenCalledWith(expect.objectContaining({
			expectedDesiredRevision: 3n,
		}))
	})
})
