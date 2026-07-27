import { describe, expect, it } from 'vitest'

import { buildWhatsAppOperationalReadModel } from './whatsAppOperationalReadModel'
import { buildWhatsAppOperationalReplayModel } from './whatsAppOperationalReplayModel'

describe('WhatsApp operational presentation models', () => {
	it('maps provider-owned records without exposing raw transport payloads', () => {
		const model = buildWhatsAppOperationalReadModel({
			canQuery: true,
			state: 'ready',
			statusMessage: '',
			accounts: [{ accountId: 'account-1', registrationId: 'registration-1' }],
			selectedAccountId: 'account-1',
			selectedChatId: 'chat-1',
			searchQuery: '',
			runtime: {
				runtimeState: 'connected',
				projectionReady: true,
				latestEventSequence: 7n,
			} as never,
			dialogs: [{
				providerChatId: 'chat-1',
				title: 'Project',
				kind: 'group',
				isUnread: true,
				unreadCount: 2n,
				observedAtUnixSeconds: 1_700_000_000n,
			} as never],
			messages: [{
				providerChatId: 'chat-1',
				providerMessageId: 'message-1',
				senderDisplayName: 'Owner',
				text: 'Hello',
				occurredAtUnixSeconds: 1_700_000_000n,
			} as never],
			participants: [],
			events: [{
				event: {
					case: 'runtimeStateChanged',
					value: { state: 'connected' },
				},
			} as never],
			searchResults: [],
			hasMoreDialogs: false,
			hasMoreMessages: false,
			hasMoreParticipants: false,
			hasMoreEvents: false,
			hasMoreSearchResults: false,
		})

		expect(model.runtime).toEqual({
			state: 'connected',
			projectionState: 'Ready',
			latestSequence: '7',
		})
		expect(model.dialogs[0]).toMatchObject({ title: 'Project', selected: true })
		expect(model.messages[0]).toMatchObject({ sender: 'Owner', text: 'Hello' })
		expect(model.events[0]).toMatchObject({ kind: 'Runtime state', summary: 'connected' })
	})

	it('preserves explicit replay reset and cursor semantics', () => {
		const model = buildWhatsAppOperationalReplayModel({
			canReplay: true,
			state: 'error',
			statusMessage: 'reset',
			accounts: [{ accountId: 'account-1', registrationId: 'registration-1' }],
			selectedAccountId: 'account-1',
			earliestSequence: 5n,
			latestSequence: 8n,
			nextSequence: 0n,
			resetRequired: true,
			frames: [],
		})

		expect(model).toMatchObject({
			earliestSequence: '5',
			latestSequence: '8',
			nextSequence: '0',
			resetRequired: true,
			hasMore: false,
		})
	})
})
