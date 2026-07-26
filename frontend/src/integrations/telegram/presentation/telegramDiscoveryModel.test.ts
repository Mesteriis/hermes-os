import { describe, expect, it } from 'vitest'

import {
	buildTelegramContextView,
	buildTelegramDiscoveryResults,
	buildTelegramOperationRows,
} from './telegramDiscoveryModel'

describe('Telegram discovery presentation model', () => {
	it('merges typed chat and message results without a provider DTO union', () => {
		expect(buildTelegramDiscoveryResults({
			chats: [{ providerChatId: 'chat-1', title: 'Architecture', kind: 'group' } as never],
			messages: [{ messageId: 'message-1', senderDisplayName: 'Alex', text: 'ADR' } as never],
		})).toEqual([
			{
				id: 'chat-1',
				title: 'Architecture',
				detail: 'group',
				kind: 'chat',
			},
			{
				id: 'message-1',
				title: 'Alex',
				detail: 'ADR',
				kind: 'message',
			},
		])
	})

	it('maps chat context and operation receipts into bounded rows', () => {
		expect(buildTelegramContextView({
			state: { unreadCount: 2n, unreadMentionCount: 1n, isMarkedAsUnread: true } as never,
			operationalState: { isArchived: false, isPinned: true, isMuted: false } as never,
			positions: [],
			folders: [{
				providerFolderId: 4n,
				title: 'Work',
				includedChatId: ['chat-1'],
				pinnedChatId: ['chat-1'],
			} as never],
			participants: [],
			topics: [],
		})).toMatchObject({
			chatState: ['2 unread', '1 mentions', 'marked unread', 'active', 'pinned'],
			folders: [{ id: '4', title: 'Work' }],
		})

		expect(buildTelegramOperationRows([{
			operationId: 'operation-1',
			commandKind: 'send_text',
			state: 'completed',
			retryCount: 0,
			maxRetries: 3,
		} as never])[0]).toMatchObject({
			id: 'operation-1',
			title: 'send_text',
			detail: 'completed · 0/3 retries',
		})
	})
})
