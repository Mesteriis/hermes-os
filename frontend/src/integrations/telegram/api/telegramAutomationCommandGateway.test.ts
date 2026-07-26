import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	previewTelegramAutomationPolicy,
	upsertTelegramAutomationPolicy,
	upsertTelegramAutomationTemplate,
} from './telegramAutomationCommandGateway'
import { getTelegramAutomationCommandClient } from './telegramAutomationCommandClient'

vi.mock('./telegramAutomationCommandClient', () => ({
	getTelegramAutomationCommandClient: vi.fn(),
}))

const execute = vi.fn()

describe('Telegram automation command gateway', () => {
	beforeEach(() => {
		execute.mockReset()
		vi.mocked(getTelegramAutomationCommandClient).mockReturnValue({ execute } as never)
	})

	it('uses exact generated template, policy and preview command variants', async () => {
		execute
			.mockResolvedValueOnce({
				response: { case: 'template', value: { templateId: 'template-1', revision: 1n } },
			})
			.mockResolvedValueOnce({
				response: { case: 'policy', value: { policyId: 'policy-1', revision: 1n } },
			})
			.mockResolvedValueOnce({
				response: { case: 'preview', value: { previewId: 'preview-1' } },
			})

		await upsertTelegramAutomationTemplate({
			mutationId: 'mutation-template-1',
			expectedRevision: 0n,
			templateId: 'template-1',
			name: 'Greeting',
			bodyTemplate: 'Hello {{name}}',
			requiredVariables: ['name'],
		})
		await upsertTelegramAutomationPolicy({
			mutationId: 'mutation-policy-1',
			expectedRevision: 0n,
			policyId: 'policy-1',
			templateId: 'template-1',
			name: 'Scoped greeting',
			enabled: true,
			accountId: 'account-1',
			providerChatIds: ['chat-1'],
		})
		await previewTelegramAutomationPolicy({
			previewId: 'preview-1',
			policyId: 'policy-1',
			accountId: 'account-1',
			providerChatId: 'chat-1',
			variables: [{ name: 'name', value: 'Ada' }],
		})

		expect(execute.mock.calls.map(([request]) => request.command.case)).toEqual([
			'upsertTemplate',
			'upsertPolicy',
			'previewPolicy',
		])
		expect(execute).toHaveBeenLastCalledWith({
			command: {
				case: 'previewPolicy',
				value: {
					previewId: 'preview-1',
					policyId: 'policy-1',
					accountId: 'account-1',
					providerChatId: 'chat-1',
					variables: [{ name: 'name', value: 'Ada' }],
				},
			},
		})
	})

	it('rejects duplicate scopes and variables before transport', async () => {
		await expect(
			upsertTelegramAutomationPolicy({
				mutationId: 'mutation-policy-1',
				expectedRevision: 0n,
				policyId: 'policy-1',
				templateId: 'template-1',
				name: 'Policy',
				enabled: true,
				accountId: 'account-1',
				providerChatIds: ['chat-1', 'chat-1'],
			}),
		).rejects.toThrow('1-128 unique chats')
		await expect(
			previewTelegramAutomationPolicy({
				previewId: 'preview-1',
				policyId: 'policy-1',
				accountId: 'account-1',
				providerChatId: 'chat-1',
				variables: [
					{ name: 'name', value: 'Ada' },
					{ name: 'name', value: 'Grace' },
				],
			}),
		).rejects.toThrow('variables must be unique')
		expect(execute).not.toHaveBeenCalled()
	})
})
