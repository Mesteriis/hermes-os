import { beforeEach, describe, expect, it, vi } from 'vitest'

import { getZulipOperationalReadConnectClient } from './zulipOperationalReadClient'
import {
	getZulipOperationalAccountStatus,
	listZulipOperationalConversations,
	listZulipOperationalEvents,
	listZulipOperationalMessages,
	searchZulipOperationalMessages,
} from './zulipOperationalReadGateway'

vi.mock('./zulipOperationalReadClient', () => ({
	getZulipOperationalReadConnectClient: vi.fn(),
}))

const query = vi.fn()

describe('Zulip operational read Gateway adapter', () => {
	beforeEach(() => {
		query.mockReset()
		vi.mocked(getZulipOperationalReadConnectClient).mockReturnValue({ query } as never)
	})

	it('uses every exact generated query with normalized bounded input', async () => {
		query
			.mockResolvedValueOnce({ response: { case: 'messages', value: { item: [] } } })
			.mockResolvedValueOnce({ response: { case: 'messages', value: { item: [] } } })
			.mockResolvedValueOnce({ response: { case: 'conversations', value: { item: [] } } })
			.mockResolvedValueOnce({ response: { case: 'events', value: { item: [] } } })
			.mockResolvedValueOnce({
				response: {
					case: 'accountStatus',
					value: { accountId: 'account-1', latestEventSequence: 0n },
				},
			})

		await listZulipOperationalMessages({
			accountId: ' account-1 ',
			providerConversationId: ' stream:1:topic ',
			limit: 25,
		})
		await searchZulipOperationalMessages({
			accountId: 'account-1',
			searchQuery: ' clean room ',
			cursor: ' cursor-1 ',
		})
		await listZulipOperationalConversations({ accountId: 'account-1' })
		await listZulipOperationalEvents({ accountId: 'account-1' })
		await getZulipOperationalAccountStatus(' account-1 ')

		expect(query.mock.calls.map(([request]) => request.query.case)).toEqual([
			'listMessages',
			'searchMessages',
			'listConversations',
			'listEvents',
			'getAccountStatus',
		])
		expect(query.mock.calls[0]![0].query.value).toMatchObject({
			accountId: 'account-1',
			providerConversationId: 'stream:1:topic',
			limit: 25,
		})
		expect(query.mock.calls[1]![0].query.value).toMatchObject({
			query: 'clean room',
			cursor: 'cursor-1',
			limit: 50,
		})
	})

	it('fails closed before transport for invalid input and mismatched responses', async () => {
		await expect(listZulipOperationalConversations({
			accountId: 'account-1',
			limit: 201,
		})).rejects.toThrow('page limit')
		await expect(listZulipOperationalMessages({
			accountId: 'account-1',
			providerConversationId: 'bad\nconversation',
		})).rejects.toThrow('provider conversation ID is invalid')
		await expect(searchZulipOperationalMessages({
			accountId: 'account-1',
			searchQuery: ' ',
		})).rejects.toThrow('search query is invalid')
		expect(query).not.toHaveBeenCalled()

		query.mockResolvedValueOnce({
			response: { case: 'messages', value: { item: [] } },
		})
		await expect(listZulipOperationalConversations({
			accountId: 'account-1',
		})).rejects.toThrow('conversations response is unavailable')
	})
})
