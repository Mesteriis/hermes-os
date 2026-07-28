import { describe, expect, it, vi } from 'vitest'

vi.mock('./canonicalCommunicationsRead', () => ({
	getCanonicalMessage: vi.fn(async (messageId: Uint8Array) => ({
		messageId,
		conversationId: new Uint8Array(16).fill(2),
	})),
	getCanonicalConversation: vi.fn(async (conversationId: Uint8Array) => ({ conversationId })),
	listCanonicalConversationParticipants: vi.fn(async () => ({ items: [], nextCursor: new Uint8Array() })),
	listCanonicalMessageAttachmentAnchors: vi.fn(async () => ({ items: [], nextCursor: new Uint8Array() })),
	listCanonicalMessageReferences: vi.fn(async () => ({ items: [], nextCursor: new Uint8Array() })),
	listCanonicalMessageEvidence: vi.fn(async () => ({ items: [], nextCursor: new Uint8Array() })),
}))

import { loadCanonicalCommunicationDetail } from './canonicalCommunicationsDetail'

describe('canonical Communications detail fan-in', () => {
	it('loads detail only through the Communications owner contract', async () => {
		const messageId = new Uint8Array(16).fill(7)
		const detail = await loadCanonicalCommunicationDetail(messageId)

		expect(detail.message.messageId).toEqual(messageId)
		expect(detail.conversation.conversationId).toEqual(detail.message.conversationId)
		expect(detail.participants.items).toEqual([])
		expect(detail.attachments.items).toEqual([])
		expect(detail.references.items).toEqual([])
		expect(detail.evidence.items).toEqual([])
	})
})
