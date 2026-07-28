import { describe, expect, it, vi } from 'vitest'

import { TelegramAccountSetupWorkflowV1 } from './telegramAccountSetupWorkflow'

describe('TelegramAccountSetupWorkflowV1', () => {
	it('provisions both credential purposes before the first runtime apply', async () => {
		const order: string[] = []
		const createTarget = vi.fn().mockImplementation(async () => {
			order.push('target')
			return { configurationInstanceId: 'telegram-target', desiredRevision: 1n }
		})
		const provision = vi.fn().mockImplementation(async (input) => {
			order.push(input.purposeId)
			return { secretRevision: 1n }
		})
		const apply = vi.fn().mockImplementation(async () => {
			order.push('settings_apply')
			return { settings: { desiredRevision: 2n }, application: {} }
		})
		const lifecycle = vi.fn().mockImplementation(async () => {
			order.push('lifecycle')
			return { accountId: 'personal' }
		})
		const workflow = new TelegramAccountSetupWorkflowV1({
			configuration: { createTarget, apply },
			vault: { provision },
			lifecycle: { provision: lifecycle },
		} as never)

		await workflow.setup({
			registrationId: 'telegram-registration',
			expectedDesiredRevision: 1n,
			accountId: 'personal',
			displayName: 'Personal',
			apiId: 42n,
			apiHash: new TextEncoder().encode('hash'),
		})

		expect(order).toEqual([
			'target',
			'telegram_api_hash',
			'telegram_session_encryption_key',
			'settings_apply',
			'lifecycle',
		])
		expect(lifecycle).toHaveBeenCalledWith(expect.objectContaining({
			credentials: [
				{ purpose: 'telegram_api_hash', revision: 1n },
				{ purpose: 'telegram_session_encryption_key', revision: 1n },
			],
		}))
		expect(provision).toHaveBeenCalledWith(expect.objectContaining({
			configurationInstanceId: 'telegram-target',
		}))
	})
})
