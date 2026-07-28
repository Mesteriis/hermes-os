import { describe, expect, it, vi } from 'vitest'

import { MailAccountSetupWorkflowV1 } from './mailAccountSetupWorkflow'

describe('MailAccountSetupWorkflowV1', () => {
	it('applies non-secret IMAP settings before Vault binding and activation', async () => {
		const order: string[] = []
		const createTarget = vi.fn().mockImplementation(async () => {
			order.push('target')
			return { configurationInstanceId: 'mail-target', desiredRevision: 1n }
		})
		const apply = vi.fn().mockImplementation(async () => {
			order.push('settings')
			return { settings: { desiredRevision: 2n }, application: {} }
		})
		const status = vi.fn().mockImplementation(async () => {
			order.push('status')
			return { binding: [] }
		})
		const provision = vi.fn().mockImplementation(async () => {
			order.push('vault')
			return { secretRevision: 1n }
		})
		const bind = vi.fn().mockImplementation(async () => {
			order.push('binding')
			return {}
		})
		const activate = vi.fn().mockImplementation(async () => {
			order.push('activation')
			return {}
		})
		const workflow = new MailAccountSetupWorkflowV1({
			configuration: { createTarget, apply },
			activation: { applyManagedIntegration: activate },
			vault: { provision },
			mail: { status, bind },
			oauth: {} as never,
		} as never)

		await workflow.setupImap({
			registrationId: 'mail-registration',
			expectedDesiredRevision: 1n,
			connectionId: 'personal',
			imapHost: 'imap.example.com',
			imapPort: 993n,
			username: 'me@example.com',
			imapPassword: new TextEncoder().encode('secret'),
		})

		expect(order).toEqual(['target', 'settings', 'status', 'vault', 'binding', 'activation'])
		expect(provision).toHaveBeenCalledWith(expect.objectContaining({
			capabilityId: 'mail.imap.credential-provisioning.v1',
			configurationInstanceId: 'mail-target',
			purposeId: 'mail_imap_password',
		}))
		expect(activate).toHaveBeenCalledWith(expect.objectContaining({
			configurationInstanceId: 'mail-target',
			expectedDesiredRevision: 2n,
		}))
	})
})
