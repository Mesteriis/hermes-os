import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
	listCachedTelegramChats,
	listCachedTelegramMessages,
	sendTelegramText,
} from './telegramOperationalGateway'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

vi.mock('./telegramOperationalClient', () => ({
	getTelegramOperationalConnectClient: vi.fn(),
}))

const executeQuery = vi.fn()
const executeCommand = vi.fn()

describe('Telegram operational Gateway adapter', () => {
	beforeEach(() => {
		executeQuery.mockReset()
		executeCommand.mockReset()
		vi.mocked(getTelegramOperationalConnectClient).mockReturnValue({
			executeQuery,
			executeCommand,
		} as never)
	})

	it('reads provider-owned chat and message projections through exact generated queries', async () => {
		executeQuery
			.mockResolvedValueOnce({
				response: { case: 'chats', value: { chat: [{ providerChatId: 'chat-1' }] } },
			})
			.mockResolvedValueOnce({
				response: { case: 'cachedMessages', value: { item: [{ messageId: 'message-1' }] } },
			})

		await expect(listCachedTelegramChats(' account-1 ')).resolves.toHaveLength(1)
		await expect(listCachedTelegramMessages('account-1', 'chat-1')).resolves.toHaveLength(1)

		expect(executeQuery).toHaveBeenNthCalledWith(1, {
			query: {
				case: 'cachedChats',
				value: { accountId: 'account-1', limit: 100 },
			},
		})
		expect(executeQuery).toHaveBeenNthCalledWith(2, {
			query: {
				case: 'cachedMessages',
				value: { accountId: 'account-1', providerChatId: 'chat-1', limit: 100 },
			},
		})
	})

	it('sends text through the provider command contract', async () => {
		executeCommand.mockResolvedValue({ operationId: 'operation-1', state: 'accepted' })

		await expect(sendTelegramText({
			accountId: 'account-1',
			providerChatId: 'chat-1',
			text: ' Hello ',
			operationId: 'operation-1',
		})).resolves.toMatchObject({ state: 'accepted' })

		expect(executeCommand).toHaveBeenCalledWith({
			command: {
				case: 'sendText',
				value: {
					accountId: 'account-1',
					providerChatId: 'chat-1',
					text: 'Hello',
					operationId: 'operation-1',
				},
			},
		})
	})

	it('rejects missing identifiers before transport', async () => {
		await expect(listCachedTelegramChats(' ')).rejects.toThrow('account ID is required')
		await expect(sendTelegramText({
			accountId: 'account-1',
			providerChatId: '',
			text: 'Hello',
			operationId: 'operation-1',
		})).rejects.toThrow('chat ID is required')
		expect(executeQuery).not.toHaveBeenCalled()
		expect(executeCommand).not.toHaveBeenCalled()
	})
})
