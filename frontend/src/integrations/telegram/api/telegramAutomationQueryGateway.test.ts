import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	getTelegramAutomationPolicy,
	listTelegramAutomationPolicies,
	listTelegramAutomationTemplates,
} from './telegramAutomationQueryGateway'
import { getTelegramAutomationQueryClient } from './telegramAutomationQueryClient'

vi.mock('./telegramAutomationQueryClient', () => ({
	getTelegramAutomationQueryClient: vi.fn(),
}))

const query = vi.fn()

describe('Telegram automation query gateway', () => {
	beforeEach(() => {
		query.mockReset()
		vi.mocked(getTelegramAutomationQueryClient).mockReturnValue({ query } as never)
	})

	it('uses distinct generated template and policy query variants', async () => {
		query
			.mockResolvedValueOnce({
				response: {
					case: 'templates',
					value: { items: [], nextAfterTemplateId: '' },
				},
			})
			.mockResolvedValueOnce({
				response: {
					case: 'policies',
					value: { items: [], nextAfterPolicyId: '' },
				},
			})

		await listTelegramAutomationTemplates()
		await listTelegramAutomationPolicies()

		expect(query.mock.calls.map(([request]) => request.request.case)).toEqual([
			'listTemplates',
			'listPolicies',
		])
		expect(query.mock.calls[0]?.[0]).toEqual({
			request: {
				case: 'listTemplates',
				value: { limit: 50, afterTemplateId: '' },
			},
		})
	})

	it('surfaces typed failures without exposing provider content', async () => {
		query.mockResolvedValue({
			response: {
				case: 'failure',
				value: { code: 2, field: 'policy_id' },
			},
		})

		await expect(getTelegramAutomationPolicy('policy-1')).rejects.toThrow('was not found')
	})
})
