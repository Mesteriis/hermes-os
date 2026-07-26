import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	listTelegramOperations,
	loadTelegramChatContext,
	loadTelegramHistory,
	searchTelegramChats,
	searchTelegramMessages,
} from './telegramDiscoveryGateway'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

vi.mock('./telegramOperationalClient', () => ({
	getTelegramOperationalConnectClient: vi.fn(),
}))

const executeQuery = vi.fn()

describe('Telegram discovery Gateway adapter', () => {
	beforeEach(() => {
		executeQuery.mockReset()
		vi.mocked(getTelegramOperationalConnectClient).mockReturnValue({ executeQuery } as never)
	})

	it('keeps chat and message search as exact query variants', async () => {
		executeQuery
			.mockResolvedValueOnce({
				response: { case: 'chats', value: { chat: [{ providerChatId: 'chat-1' }] } },
			})
			.mockResolvedValueOnce({
				response: { case: 'cachedMessages', value: { item: [{ messageId: 'message-1' }] } },
			})

		await expect(searchTelegramChats(' account-1 ', ' design ')).resolves.toHaveLength(1)
		await expect(searchTelegramMessages('account-1', 'chat-1', ' decision ')).resolves.toHaveLength(1)

		expect(executeQuery).toHaveBeenNthCalledWith(1, {
			query: {
				case: 'searchChats',
				value: { accountId: 'account-1', query: 'design', limit: 100 },
			},
		})
		expect(executeQuery).toHaveBeenNthCalledWith(2, {
			query: {
				case: 'searchMessages',
				value: {
					accountId: 'account-1',
					providerChatId: 'chat-1',
					query: 'decision',
					limit: 100,
				},
			},
		})
	})

	it('loads provider history and operation receipts independently', async () => {
		executeQuery
			.mockResolvedValueOnce({
				response: {
					case: 'historyPage',
					value: { page: { item: [], hasMore: false } },
				},
			})
			.mockResolvedValueOnce({
				response: {
					case: 'operations',
					value: { operation: [{ operationId: 'operation-1' }] },
				},
			})

		await expect(loadTelegramHistory('account-1', 'chat-1')).resolves.toMatchObject({
			hasMore: false,
		})
		await expect(listTelegramOperations('account-1')).resolves.toHaveLength(1)

		expect(executeQuery).toHaveBeenNthCalledWith(1, {
			query: {
				case: 'loadHistory',
				value: {
					accountId: 'account-1',
					providerChatId: 'chat-1',
					mode: 'older',
					limit: 100,
				},
			},
		})
	})

	it('assembles selected chat context from distinct typed queries', async () => {
		executeQuery
			.mockResolvedValueOnce({
				response: { case: 'chatState', value: { state: { unreadCount: 2n } } },
			})
			.mockResolvedValueOnce({
				response: {
					case: 'chatOperationalState',
					value: { state: { isArchived: false, isPinned: true } },
				},
			})
			.mockResolvedValueOnce({
				response: { case: 'chatPositions', value: { position: [] } },
			})
			.mockResolvedValueOnce({
				response: {
					case: 'participants',
					value: { item: [{ providerMemberId: 'member-1' }] },
				},
			})
			.mockResolvedValueOnce({
				response: { case: 'topics', value: { topic: [{ providerTopicId: 'topic-1' }] } },
			})

		await expect(loadTelegramChatContext('account-1', 'chat-1')).resolves.toMatchObject({
			participants: [{ providerMemberId: 'member-1' }],
			topics: [{ providerTopicId: 'topic-1' }],
			folders: [],
		})
		expect(executeQuery).toHaveBeenCalledTimes(5)
	})

	it('rejects missing search input before transport', async () => {
		await expect(searchTelegramChats('account-1', ' ')).rejects.toThrow(
			'Telegram search query is required',
		)
		await expect(loadTelegramHistory('', 'chat-1')).rejects.toThrow('account ID is required')
		expect(executeQuery).not.toHaveBeenCalled()
	})
})
