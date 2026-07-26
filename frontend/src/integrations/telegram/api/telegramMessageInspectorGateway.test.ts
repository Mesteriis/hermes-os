import { beforeEach, describe, expect, it, vi } from 'vitest'

import { inspectTelegramMessage } from './telegramMessageInspectorGateway'
import { getTelegramOperationalConnectClient } from './telegramOperationalClient'

vi.mock('./telegramOperationalClient', () => ({
	getTelegramOperationalConnectClient: vi.fn(),
}))

const executeQuery = vi.fn()

describe('Telegram message inspector Gateway adapter', () => {
	beforeEach(() => {
		executeQuery.mockReset()
		vi.mocked(getTelegramOperationalConnectClient).mockReturnValue({ executeQuery } as never)
	})

	it('joins exact message audit queries without a generic provider payload', async () => {
		for (const response of [
			{ case: 'cachedMessages', value: { item: [{ messageId: 'message-1' }] } },
			{ case: 'messageVersions', value: { item: [{ versionId: 'version-1' }] } },
			{ case: 'messageTombstones', value: { item: [] } },
			{ case: 'messageMutations', value: { item: [] } },
			{ case: 'messageReferences', value: {} },
			{ case: 'replyChain', value: { item: [] } },
			{ case: 'forwardChain', value: { item: [] } },
			{ case: 'attachment', value: {} },
			{ case: 'reactions', value: { reaction: [] } },
			{ case: 'reactionSummary', value: { summary: [{ emoji: '👍', count: 1 }] } },
			{ case: 'commands', value: { record: [{ operation: { operationId: 'operation-1' } }] } },
			{
				case: 'cachedMessages',
				value: { item: [{ providerMessageId: 'provider-message-1' }] },
			},
		]) {
			executeQuery.mockResolvedValueOnce({ response })
		}

		await expect(inspectTelegramMessage({
			accountId: 'account-1',
			providerChatId: 'chat-1',
			messageId: 'message-1',
			providerMessageId: 'provider-message-1',
		})).resolves.toMatchObject({
			message: { messageId: 'message-1' },
			versions: [{ versionId: 'version-1' }],
			reactionSummary: [{ emoji: '👍', count: 1 }],
			commands: [{ operation: { operationId: 'operation-1' } }],
			pinned: true,
		})

		expect(executeQuery.mock.calls.map(([request]) => request.query.case)).toEqual([
			'messageById',
			'messageVersions',
			'messageTombstones',
			'messageMutations',
			'messageReferences',
			'replyChain',
			'forwardChain',
			'attachmentForMessage',
			'reactions',
			'reactionSummary',
			'commands',
			'pinnedMessages',
		])
	})

	it('fails closed when a required response case is missing', async () => {
		executeQuery.mockResolvedValue({ response: { case: 'chats', value: { chat: [] } } })

		await expect(inspectTelegramMessage({
			accountId: 'account-1',
			providerChatId: 'chat-1',
			messageId: 'message-1',
			providerMessageId: 'provider-message-1',
		})).rejects.toThrow('Telegram message inspection is incomplete')
	})
})
