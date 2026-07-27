import { beforeEach, describe, expect, it, vi } from 'vitest'

import { getWhatsAppOperationalReadConnectClient } from './whatsAppOperationalReadClient'
import {
	getWhatsAppOperationalRuntimeStatus,
	listWhatsAppOperationalDialogs,
	listWhatsAppOperationalEvents,
	listWhatsAppOperationalMessages,
	listWhatsAppOperationalParticipants,
	searchWhatsAppOperationalMessages,
} from './whatsAppOperationalReadGateway'

vi.mock('./whatsAppOperationalReadClient', () => ({
	getWhatsAppOperationalReadConnectClient: vi.fn(),
}))

const query = vi.fn()

describe('WhatsApp operational read Gateway adapter', () => {
	beforeEach(() => {
		query.mockReset()
		vi.mocked(getWhatsAppOperationalReadConnectClient).mockReturnValue({ query } as never)
	})

	it('uses every exact generated query with bounded normalized input', async () => {
		query
			.mockResolvedValueOnce({ response: { case: 'messages', value: { item: [] } } })
			.mockResolvedValueOnce({ response: { case: 'messages', value: { item: [] } } })
			.mockResolvedValueOnce({ response: { case: 'dialogs', value: { item: [] } } })
			.mockResolvedValueOnce({ response: { case: 'participants', value: { item: [] } } })
			.mockResolvedValueOnce({ response: { case: 'events', value: { item: [] } } })
			.mockResolvedValueOnce({
				response: {
					case: 'runtimeStatus',
					value: { accountId: 'account-1', latestEventSequence: 0n },
				},
			})

		await listWhatsAppOperationalMessages({
			accountId: ' account-1 ',
			providerChatId: ' chat-1 ',
			limit: 25,
		})
		await searchWhatsAppOperationalMessages({
			accountId: 'account-1',
			searchQuery: ' clean room ',
			cursor: ' cursor-1 ',
		})
		await listWhatsAppOperationalDialogs({ accountId: 'account-1' })
		await listWhatsAppOperationalParticipants({
			accountId: 'account-1',
			providerChatId: 'chat-1',
		})
		await listWhatsAppOperationalEvents({ accountId: 'account-1' })
		await getWhatsAppOperationalRuntimeStatus(' account-1 ')

		expect(query.mock.calls.map(([request]) => request.query.case)).toEqual([
			'listMessages',
			'searchMessages',
			'listDialogs',
			'listParticipants',
			'listEvents',
			'getRuntimeStatus',
		])
		expect(query.mock.calls[0]![0].query.value).toMatchObject({
			accountId: 'account-1',
			providerChatId: 'chat-1',
			limit: 25,
		})
		expect(query.mock.calls[1]![0].query.value).toMatchObject({
			query: 'clean room',
			cursor: 'cursor-1',
			limit: 50,
		})
	})

	it('fails closed before transport for invalid input and mismatched responses', async () => {
		await expect(listWhatsAppOperationalDialogs({
			accountId: 'account-1',
			limit: 201,
		})).rejects.toThrow('page limit')
		await expect(listWhatsAppOperationalParticipants({
			accountId: 'account-1',
			providerChatId: 'bad\nchat',
		})).rejects.toThrow('provider chat ID is invalid')
		await expect(searchWhatsAppOperationalMessages({
			accountId: 'account-1',
			searchQuery: ' ',
		})).rejects.toThrow('search query is invalid')
		expect(query).not.toHaveBeenCalled()

		query.mockResolvedValueOnce({
			response: { case: 'messages', value: { item: [] } },
		})
		await expect(listWhatsAppOperationalDialogs({
			accountId: 'account-1',
		})).rejects.toThrow('dialogs response is unavailable')
	})
})
