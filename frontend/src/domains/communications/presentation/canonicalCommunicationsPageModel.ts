import type {
	AccountSummaryV1,
	CommunicationSearchHitV1,
	ConversationSummaryV1,
	MessageSummaryV1,
} from '../../../gen/hermes/communications/query/v1/query_pb'

export type CanonicalCommunicationsPageStatus = 'empty' | 'error' | 'loading' | 'ready'
export type CanonicalCommunicationsSearchStatus = 'idle' | 'loading' | 'ready' | 'error'

export type CanonicalCommunicationAccountRow = {
	key: string
	sourceLabel: string
	identityLabel: string
	observedRangeLabel: string
	selected: boolean
}

export type CanonicalConversationRow = {
	key: string
	identityLabel: string
	sourceLabel: string
	observedRangeLabel: string
	selected: boolean
}

export type CanonicalMessageRow = {
	key: string
	identityLabel: string
	stateLabel: string
	directionLabel: string
	observedRangeLabel: string
}

export type CanonicalSearchResultRow = {
	key: string
	evidenceLabel: string
	messageLabel: string
	conversationLabel: string
	observedAtLabel: string
	matchLabel: string
}

export type CanonicalCommunicationsPageModel = {
	status: CanonicalCommunicationsPageStatus
	statusMessage: string
	accounts: readonly CanonicalCommunicationAccountRow[]
	conversations: readonly CanonicalConversationRow[]
	messages: readonly CanonicalMessageRow[]
	searchText: string
	searchStatus: CanonicalCommunicationsSearchStatus
	searchMessage: string
	searchResults: readonly CanonicalSearchResultRow[]
}

export function buildCanonicalAccountRows(
	accounts: readonly AccountSummaryV1[],
	selectedKey: string,
): readonly CanonicalCommunicationAccountRow[] {
	return accounts.map((account) => ({
		key: bytesKey(account.accountId),
		sourceLabel: sourceLabel(account.provider),
		identityLabel: identifierLabel('Account', account.accountId),
		observedRangeLabel: observedRange(account.firstObservedAtUnixSeconds, account.lastObservedAtUnixSeconds),
		selected: bytesKey(account.accountId) === selectedKey,
	}))
}

export function buildCanonicalConversationRows(
	conversations: readonly ConversationSummaryV1[],
	selectedKey: string,
): readonly CanonicalConversationRow[] {
	return conversations.map((conversation) => ({
		key: bytesKey(conversation.conversationId),
		identityLabel: identifierLabel('Conversation', conversation.conversationId),
		sourceLabel: sourceLabel(conversation.provider),
		observedRangeLabel: observedRange(
			conversation.firstObservedAtUnixSeconds,
			conversation.lastObservedAtUnixSeconds,
		),
		selected: bytesKey(conversation.conversationId) === selectedKey,
	}))
}

export function buildCanonicalMessageRows(
	messages: readonly MessageSummaryV1[],
): readonly CanonicalMessageRow[] {
	return messages.map((message) => ({
		key: bytesKey(message.messageId),
		identityLabel: identifierLabel('Message', message.messageId),
		stateLabel: `Body ${message.bodyState} · lifecycle ${message.lifecycleState}`,
		directionLabel: `Direction ${message.direction}`,
		observedRangeLabel: observedRange(
			message.firstObservedAtUnixSeconds,
			message.lastObservedAtUnixSeconds,
		),
	}))
}

export function buildCanonicalSearchRows(
	results: readonly CommunicationSearchHitV1[],
): readonly CanonicalSearchResultRow[] {
	return results.map((result) => ({
		key: bytesKey(result.evidenceId),
		evidenceLabel: identifierLabel('Evidence', result.evidenceId),
		messageLabel: identifierLabel('Message', result.messageId),
		conversationLabel: identifierLabel('Conversation', result.conversationId),
		observedAtLabel: formatUnixSeconds(result.observedAtUnixSeconds),
		matchLabel: `${result.matchedTokenCount} exact token${result.matchedTokenCount === 1 ? '' : 's'}`,
	}))
}

export function bytesKey(value: Uint8Array): string {
	return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

function identifierLabel(kind: string, value: Uint8Array): string {
	const key = bytesKey(value)
	return `${kind} ${key ? `#${key.slice(0, 12)}` : 'unavailable'}`
}

function sourceLabel(source: number): string {
	return `Source ${source}`
}

function observedRange(first: bigint, last: bigint): string {
	return first === last
		? formatUnixSeconds(last)
		: `${formatUnixSeconds(first)} — ${formatUnixSeconds(last)}`
}

function formatUnixSeconds(value: bigint): string {
	const milliseconds = Number(value) * 1_000
	if (!Number.isSafeInteger(milliseconds)) return 'Time unavailable'
	const date = new Date(milliseconds)
	if (Number.isNaN(date.getTime())) return 'Time unavailable'
	return new Intl.DateTimeFormat(undefined, {
		dateStyle: 'medium',
		timeStyle: 'short',
	}).format(date)
}
