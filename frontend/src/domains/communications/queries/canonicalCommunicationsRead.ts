import type {
	AccountSummaryV1,
	ConversationSummaryV1,
	MessageSummaryV1,
} from '../../../gen/hermes/communications/query/v1/query_pb'
import { getCommunicationsQueryConnectClient } from '../../../platform/connect/communicationsQueryClient'

const MAX_PAGE_LIMIT = 100

export async function listCanonicalCommunicationAccounts(
	limit = 50,
): Promise<readonly AccountSummaryV1[]> {
	assertPageLimit(limit)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: { case: 'listAccounts', value: { limit } },
	})
	if (response.errorCode || response.result.case !== 'listAccounts') {
		throw new Error('Canonical communication accounts are unavailable')
	}
	return response.result.value.accounts
}

export async function listCanonicalConversations(
	accountCursorSha256: Uint8Array,
	limit = 100,
): Promise<readonly ConversationSummaryV1[]> {
	assertDigest('account cursor', accountCursorSha256)
	assertPageLimit(limit)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: {
			case: 'listConversations',
			value: { accountCursorSha256, limit },
		},
	})
	if (response.errorCode || response.result.case !== 'listConversations') {
		throw new Error('Canonical conversations are unavailable')
	}
	return response.result.value.conversations
}

export async function listCanonicalConversationMessages(
	conversationId: Uint8Array,
	limit = 100,
): Promise<readonly MessageSummaryV1[]> {
	assertIdentifier('conversation ID', conversationId)
	assertPageLimit(limit)
	const response = await getCommunicationsQueryConnectClient().query({
		protocolMajor: 1,
		operation: {
			case: 'listConversationMessages',
			value: { conversationId, limit },
		},
	})
	if (response.errorCode || response.result.case !== 'listConversationMessages') {
		throw new Error('Canonical communication messages are unavailable')
	}
	return response.result.value.messages
}

function assertPageLimit(limit: number): void {
	if (!Number.isInteger(limit) || limit < 1 || limit > MAX_PAGE_LIMIT) {
		throw new RangeError(`Canonical Communications page limit must be between 1 and ${MAX_PAGE_LIMIT}`)
	}
}

function assertDigest(label: string, value: Uint8Array): void {
	if (value.byteLength !== 32) {
		throw new RangeError(`Canonical Communications ${label} must be a SHA-256 digest`)
	}
}

function assertIdentifier(label: string, value: Uint8Array): void {
	if (value.byteLength === 0 || value.byteLength > 64) {
		throw new RangeError(`Canonical Communications ${label} is invalid`)
	}
}
