import { beforeEach, describe, expect, it, vi } from 'vitest'

const query = vi.fn()

vi.mock('../../../platform/connect/communicationsQueryClient', () => ({
	getCommunicationsQueryConnectClient: () => ({ query }),
}))

import {
	getCanonicalConversation,
	getCanonicalMessage,
	listCanonicalCommunicationAccounts,
	listCanonicalConversationMessages,
	listCanonicalConversationParticipants,
	listCanonicalConversations,
	listCanonicalMessageAttachmentAnchors,
	listCanonicalMessageEvidence,
	listCanonicalMessageReferences,
} from './canonicalCommunicationsRead'

describe('canonical Communications read adapter', () => {
	beforeEach(() => {
		query.mockReset()
	})

	it('uses every exact generated owner detail operation and preserves cursors', async () => {
		const nextCursor = new Uint8Array([4, 5])
		const conversation = { conversationId: new Uint8Array(16).fill(2) }
		const message = {
			messageId: new Uint8Array(16).fill(3),
			conversationId: conversation.conversationId,
		}
		for (const result of [
			{ case: 'listAccounts', value: { accounts: [], nextCursor } },
			{ case: 'listConversations', value: { conversations: [], nextCursor } },
			{ case: 'getConversation', value: { conversation } },
			{ case: 'getMessage', value: { message } },
			{ case: 'listConversationMessages', value: { messages: [], nextCursor } },
			{ case: 'listConversationParticipants', value: { participants: [], nextCursor } },
			{ case: 'listMessageAttachmentAnchors', value: { anchors: [], nextCursor } },
			{ case: 'listMessageReferences', value: { references: [], nextCursor } },
			{ case: 'listMessageEvidence', value: { evidence: [], nextCursor } },
		]) {
			query.mockResolvedValueOnce({ errorCode: '', result })
		}

		const accountPage = await listCanonicalCommunicationAccounts(25, new Uint8Array([1]))
		await listCanonicalConversations(new Uint8Array(32).fill(1), 50, new Uint8Array([1]))
		await getCanonicalConversation(conversation.conversationId)
		await getCanonicalMessage(message.messageId)
		await listCanonicalConversationMessages(conversation.conversationId, 75, new Uint8Array([1]))
		await listCanonicalConversationParticipants(conversation.conversationId)
		await listCanonicalMessageAttachmentAnchors(message.messageId)
		await listCanonicalMessageReferences(message.messageId)
		await listCanonicalMessageEvidence(message.messageId)

		expect(accountPage.nextCursor).toEqual(nextCursor)
		expect(query.mock.calls.map(([request]) => request.operation.case)).toEqual([
			'listAccounts',
			'listConversations',
			'getConversation',
			'getMessage',
			'listConversationMessages',
			'listConversationParticipants',
			'listMessageAttachmentAnchors',
			'listMessageReferences',
			'listMessageEvidence',
		])
		expect(query.mock.calls.every(([request]) => request.protocolMajor === 1)).toBe(true)
	})

	it('fails closed for invalid bounds, identifiers, cursors and response cases', async () => {
		await expect(listCanonicalCommunicationAccounts(0)).rejects.toThrow(RangeError)
		await expect(listCanonicalConversations(new Uint8Array(31))).rejects.toThrow(RangeError)
		await expect(listCanonicalConversationMessages(new Uint8Array(15))).rejects.toThrow(RangeError)
		await expect(
			listCanonicalCommunicationAccounts(10, new Uint8Array(65)),
		).rejects.toThrow(RangeError)

		query.mockResolvedValueOnce({
			errorCode: 'not_admitted',
			result: { case: undefined },
		})
		await expect(listCanonicalCommunicationAccounts()).rejects.toThrow(
			'Canonical communication accounts are unavailable',
		)
	})
})
