import { describe, expect, it, vi } from 'vitest'

const query = vi.fn()

vi.mock('../../../platform/connect/communicationsQueryClient', () => ({
	getCommunicationsQueryConnectClient: () => ({ query }),
}))

import {
	listCanonicalCommunicationAccounts,
	listCanonicalConversationMessages,
	listCanonicalConversations,
} from './canonicalCommunicationsRead'

describe('canonical Communications read adapter', () => {
	it('uses exact generated owner operations', async () => {
		query
			.mockResolvedValueOnce({
				errorCode: '',
				result: { case: 'listAccounts', value: { accounts: [] } },
			})
			.mockResolvedValueOnce({
				errorCode: '',
				result: { case: 'listConversations', value: { conversations: [] } },
			})
			.mockResolvedValueOnce({
				errorCode: '',
				result: { case: 'listConversationMessages', value: { messages: [] } },
			})

		await listCanonicalCommunicationAccounts(25)
		await listCanonicalConversations(new Uint8Array(32).fill(1), 50)
		await listCanonicalConversationMessages(new Uint8Array([1]), 75)

		expect(query.mock.calls.map(([request]) => request.operation.case)).toEqual([
			'listAccounts',
			'listConversations',
			'listConversationMessages',
		])
		expect(query.mock.calls.every(([request]) => request.protocolMajor === 1)).toBe(true)
	})

	it('fails closed for invalid bounds and response cases', async () => {
		expect(() => listCanonicalCommunicationAccounts(0)).rejects.toThrow(RangeError)
		expect(() => listCanonicalConversations(new Uint8Array(31))).rejects.toThrow(RangeError)
		expect(() => listCanonicalConversationMessages(new Uint8Array())).rejects.toThrow(RangeError)

		query.mockResolvedValueOnce({
			errorCode: 'not_admitted',
			result: { case: undefined },
		})
		await expect(listCanonicalCommunicationAccounts()).rejects.toThrow(
			'Canonical communication accounts are unavailable',
		)
	})
})
