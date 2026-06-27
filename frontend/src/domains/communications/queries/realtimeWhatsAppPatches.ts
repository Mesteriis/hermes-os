import type { WhatsappWebMessage } from '../../../shared/communications/types/whatsapp'
import type { CommunicationProviderConversation } from '../types/providerChannels'
import {
	isRecord,
	storedEventEnvelope,
	stringValue,
} from '../../../shared/communications/queries/realtimePatchShared'

type WhatsAppEventPayload = Record<string, unknown>

export type WhatsAppRealtimePatchQueryClient = {
	getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
		[readonly unknown[], TData | undefined]
	>
	setQueryData?: <TData>(
		queryKey: readonly unknown[],
		updater: TData | ((data: TData | undefined) => TData | undefined)
	) => unknown
}

export function applyWhatsAppRealtimePatch(
	eventData: string,
	queryClient: WhatsAppRealtimePatchQueryClient
): boolean {
	const { getQueriesData, setQueryData } = queryClient
	if (!getQueriesData || !setQueryData) return false

	const envelope = storedEventEnvelope(eventData)
	const eventType = stringValue(envelope?.event?.event_type)
	if (!eventType || !eventType.startsWith('whatsapp.')) return false

	const payload = isRecord(envelope?.event?.payload)
		? (envelope.event?.payload as WhatsAppEventPayload)
		: undefined
	const snapshot = whatsappMessageSnapshot(payload?.message)
	const conversationSnapshot = whatsappConversationSnapshot(payload)
	let patched = false

	for (const [queryKey, data] of getQueriesData<CommunicationProviderConversation[]>({
		queryKey: ['communications', 'whatsapp', 'conversations'],
	})) {
		const updated = patchConversationList(queryKey, data, eventType, payload, conversationSnapshot)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<CommunicationProviderConversation | null>({
		queryKey: ['communications', 'whatsapp', 'conversation-detail'],
	})) {
		const updated = patchConversationDetail(queryKey, data, eventType, payload, conversationSnapshot)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	for (const [queryKey, data] of getQueriesData<WhatsappWebMessage[]>({
		queryKey: ['communications', 'whatsapp', 'messages'],
	})) {
		const updated = patchMessageList(queryKey, data, eventType, payload, snapshot)
		if (updated !== data) {
			setQueryData(queryKey, updated)
			patched = true
		}
	}

	return patched
}

function patchConversationList(
	queryKey: readonly unknown[],
	conversations: CommunicationProviderConversation[] | undefined,
	eventType: string,
	payload: WhatsAppEventPayload | undefined,
	snapshot: CommunicationProviderConversation | null
): CommunicationProviderConversation[] | undefined {
	if (!conversations || eventType !== 'whatsapp.dialog.updated' || !payload) return conversations

	const accountId = queryScopeValue(queryKey[3])
	const limit = typeof queryKey[4] === 'number' ? queryKey[4] : null
	if (snapshot && accountId && snapshot.account_id !== accountId) return conversations

	const conversationId =
		stringValue(payload.conversation_id) ??
		snapshot?.conversation_id ??
		snapshot?.provider_chat_id ??
		null
	if (!conversationId) return conversations

	const index = conversations.findIndex((conversation) =>
		conversation.conversation_id === conversationId || conversation.provider_chat_id === conversationId
	)
	if (index < 0) {
		if (!snapshot || (accountId && snapshot.account_id !== accountId)) return conversations
		const next = [snapshot, ...conversations]
		return typeof limit === 'number' ? next.slice(0, limit) : next
	}

	const current = conversations[index]
	const nextConversation = patchExistingConversation(current, payload, snapshot)
	if (nextConversation === current) return conversations

	return conversations.map((conversation, currentIndex) =>
		currentIndex === index ? nextConversation : conversation
	)
}

function patchConversationDetail(
	queryKey: readonly unknown[],
	conversation: CommunicationProviderConversation | null | undefined,
	eventType: string,
	payload: WhatsAppEventPayload | undefined,
	snapshot: CommunicationProviderConversation | null
): CommunicationProviderConversation | null | undefined {
	if (!conversation || eventType !== 'whatsapp.dialog.updated' || !payload) return conversation

	const expectedConversationId =
		typeof queryKey[3] === 'string' && queryKey[3] !== 'none' ? queryKey[3] : null
	const payloadConversationId =
		stringValue(payload.conversation_id) ??
		snapshot?.conversation_id ??
		snapshot?.provider_chat_id ??
		null
	if (!payloadConversationId) return conversation
	if (
		expectedConversationId &&
		expectedConversationId !== payloadConversationId &&
		conversation.provider_chat_id !== payloadConversationId
	) {
		return conversation
	}

	return patchExistingConversation(conversation, payload, snapshot)
}

function patchExistingConversation(
	conversation: CommunicationProviderConversation,
	payload: WhatsAppEventPayload,
	snapshot: CommunicationProviderConversation | null
): CommunicationProviderConversation {
	const metadata = {
		...conversation.metadata,
		...snapshot?.metadata,
	}

	patchConversationMetadataFlag(metadata, 'is_pinned', payload.is_pinned)
	patchConversationMetadataFlag(metadata, 'pinned', payload.is_pinned)
	patchConversationMetadataFlag(metadata, 'is_archived', payload.is_archived)
	patchConversationMetadataFlag(metadata, 'archived', payload.is_archived)
	patchConversationMetadataFlag(metadata, 'is_muted', payload.is_muted)
	patchConversationMetadataFlag(metadata, 'muted', payload.is_muted)
	patchConversationMetadataFlag(metadata, 'is_unread', payload.is_unread)
	if (typeof payload.unread_count === 'number') metadata.unread_count = payload.unread_count
	if (typeof payload.participant_count === 'number') metadata.participant_count = payload.participant_count
	if (typeof payload.chat_kind === 'string') metadata.chat_kind = payload.chat_kind
	if (typeof payload.chat_title === 'string') metadata.provider_label = payload.chat_title

	const nextConversation: CommunicationProviderConversation = {
		...(snapshot ?? conversation),
		conversation_id: snapshot?.conversation_id ?? conversation.conversation_id,
		account_id: snapshot?.account_id ?? conversation.account_id,
		provider_chat_id: snapshot?.provider_chat_id ?? conversation.provider_chat_id,
		chat_kind: snapshot?.chat_kind ?? conversation.chat_kind ?? stringValue(payload.chat_kind) ?? undefined,
		title: snapshot?.title ?? conversation.title,
		last_message_at: snapshot?.last_message_at ?? conversation.last_message_at,
		metadata,
		created_at: snapshot?.created_at ?? conversation.created_at,
		updated_at: snapshot?.updated_at ?? conversation.updated_at,
	}

	return shallowConversationEqual(nextConversation, conversation) ? conversation : nextConversation
}

function patchMessageList(
	queryKey: readonly unknown[],
	messages: WhatsappWebMessage[] | undefined,
	eventType: string,
	payload: WhatsAppEventPayload | undefined,
	snapshot: WhatsappWebMessage | null
): WhatsappWebMessage[] | undefined {
	if (!messages || !payload) return messages

	const accountId = queryScopeValue(queryKey[3])
	const providerChatId = queryScopeValue(queryKey[4])
	const payloadAccountId = stringValue(payload.account_id)
	const payloadProviderChatId =
		stringValue(payload.provider_chat_id) ?? snapshot?.provider_chat_id ?? null

	if (accountId && payloadAccountId && payloadAccountId !== accountId) return messages
	if (providerChatId && payloadProviderChatId && payloadProviderChatId !== providerChatId) {
		return messages
	}

	const messageId =
		stringValue(payload.message_id) ??
		stringValue(payload.raw_message_id) ??
		stringValue(payload.provider_message_id) ??
		snapshot?.message_id ??
		null
	if (!messageId) return messages

	const matchIndex = messages.findIndex((message) => message.message_id === messageId)
	if (matchIndex < 0) {
		if (eventType === 'whatsapp.message.created' && snapshot && matchesQueryScope(snapshot, accountId, providerChatId)) {
			return insertMessage(queryKey, messages, snapshot)
		}
		return messages
	}

	const matched = messages[matchIndex]
	const nextMessage = patchExistingMessage(matched, eventType, payload, snapshot)
	if (nextMessage === matched) return messages

	return messages.map((message, index) => (index === matchIndex ? nextMessage : message))
}

function patchExistingMessage(
	message: WhatsappWebMessage,
	eventType: string,
	payload: WhatsAppEventPayload,
	snapshot: WhatsappWebMessage | null
): WhatsappWebMessage {
	if (eventType === 'whatsapp.message.created' && snapshot) {
		return snapshot
	}

	if (eventType === 'whatsapp.message.updated') {
		const nextMetadata = {
			...(snapshot?.metadata ?? message.metadata),
			lifecycle: {
				...(isRecord(message.metadata.lifecycle) ? message.metadata.lifecycle : {}),
				edited: payload.edited === true,
			},
		}
		return { ...(snapshot ?? message), metadata: nextMetadata }
	}

	if (eventType === 'whatsapp.message.deleted') {
		return {
			...(snapshot ?? message),
			metadata: {
				...(snapshot?.metadata ?? message.metadata),
				tombstone: {
					tombstone_id: stringValue(payload.tombstone_id),
					is_visible: false,
				},
			},
		}
	}

	if (eventType === 'whatsapp.reaction.changed') {
		const reaction = stringValue(payload.reaction)
		if (!reaction) return message
		const reactions = normalizeReactionSummary(message.metadata.reaction_summary)
		const nextReactions = payload.is_active === false
			? reactions.filter((item) => item.reaction !== reaction)
			: upsertReactionSummary(reactions, reaction)
		return {
			...message,
			metadata: {
				...message.metadata,
				reaction_summary: { reactions: nextReactions },
			},
		}
	}

	if (eventType === 'whatsapp.receipt.changed') {
		const deliveryState = stringValue(payload.delivery_state)
		return deliveryState ? { ...message, delivery_state: deliveryState } : message
	}

	return message
}

function insertMessage(
	queryKey: readonly unknown[],
	messages: WhatsappWebMessage[],
	snapshot: WhatsappWebMessage
): WhatsappWebMessage[] {
	const limit = typeof queryKey[5] === 'number' ? queryKey[5] : null
	const nextMessages = [snapshot, ...messages]
	return typeof limit === 'number' ? nextMessages.slice(0, limit) : nextMessages
}

function queryScopeValue(value: unknown): string | null {
	return typeof value === 'string' && value !== 'all' ? value : null
}

function matchesQueryScope(
	message: WhatsappWebMessage,
	accountId: string | null,
	providerChatId: string | null
): boolean {
	if (accountId && message.account_id !== accountId) return false
	if (providerChatId && message.provider_chat_id !== providerChatId) return false
	return true
}

function whatsappMessageSnapshot(value: unknown): WhatsappWebMessage | null {
	if (!isRecord(value)) return null

	const messageId = stringValue(value.message_id)
	const accountId = stringValue(value.account_id)
	const providerMessageId = stringValue(value.provider_message_id)
	if (!messageId || !accountId || !providerMessageId) return null

	return {
		message_id: messageId,
		raw_record_id: stringValue(value.raw_record_id) ?? '',
		account_id: accountId,
		provider_message_id: providerMessageId,
		provider_chat_id: stringValue(value.provider_chat_id),
		chat_title: stringValue(value.chat_title) ?? '',
		sender: stringValue(value.sender) ?? '',
		sender_display_name: stringValue(value.sender_display_name),
		text: stringValue(value.text) ?? '',
		occurred_at: stringValue(value.occurred_at),
		projected_at: stringValue(value.projected_at) ?? new Date().toISOString(),
		channel_kind: 'whatsapp_web',
		delivery_state: stringValue(value.delivery_state) ?? 'received',
		metadata: isRecord(value.metadata) ? value.metadata : {},
	}
}

function whatsappConversationSnapshot(
	value: unknown
): CommunicationProviderConversation | null {
	if (!isRecord(value)) return null

	const conversationId = stringValue(value.conversation_id)
	const accountId = stringValue(value.account_id)
	const providerChatId = stringValue(value.provider_chat_id)
	const title = stringValue(value.chat_title)
	const createdAt = stringValue(value.created_at) ?? new Date().toISOString()
	const updatedAt = stringValue(value.updated_at) ?? new Date().toISOString()
	if (!accountId || !providerChatId || !title) return null

	const metadata: Record<string, unknown> = {}
	patchConversationMetadataFlag(metadata, 'is_pinned', value.is_pinned)
	patchConversationMetadataFlag(metadata, 'pinned', value.is_pinned)
	patchConversationMetadataFlag(metadata, 'is_archived', value.is_archived)
	patchConversationMetadataFlag(metadata, 'archived', value.is_archived)
	patchConversationMetadataFlag(metadata, 'is_muted', value.is_muted)
	patchConversationMetadataFlag(metadata, 'muted', value.is_muted)
	patchConversationMetadataFlag(metadata, 'is_unread', value.is_unread)
	if (typeof value.unread_count === 'number') metadata.unread_count = value.unread_count
	if (typeof value.participant_count === 'number') metadata.participant_count = value.participant_count
	if (typeof value.chat_kind === 'string') metadata.chat_kind = value.chat_kind

	return {
		conversation_id: conversationId ?? providerChatId,
		account_id: accountId,
		provider_chat_id: providerChatId,
		chat_kind: stringValue(value.chat_kind) ?? undefined,
		title,
		last_message_at: stringValue(value.observed_at) ?? null,
		metadata,
		created_at: createdAt,
		updated_at: updatedAt,
	}
}

function patchConversationMetadataFlag(
	metadata: Record<string, unknown>,
	key: string,
	value: unknown
): void {
	if (typeof value === 'boolean') metadata[key] = value
}

function shallowConversationEqual(
	left: CommunicationProviderConversation,
	right: CommunicationProviderConversation
): boolean {
	return (
		left.conversation_id === right.conversation_id &&
		left.account_id === right.account_id &&
		left.provider_chat_id === right.provider_chat_id &&
		left.chat_kind === right.chat_kind &&
		left.title === right.title &&
		left.last_message_at === right.last_message_at &&
		left.created_at === right.created_at &&
		left.updated_at === right.updated_at &&
		JSON.stringify(left.metadata) === JSON.stringify(right.metadata)
	)
}

function normalizeReactionSummary(
	value: unknown
): Array<{ reaction: string; count: number }> {
	if (!isRecord(value) || !Array.isArray(value.reactions)) return []

	return value.reactions
		.filter(
			(item): item is { reaction: string; count?: unknown } =>
				isRecord(item) && typeof item.reaction === 'string'
		)
		.map((item) => ({
			reaction: item.reaction,
			count: typeof item.count === 'number' ? item.count : 1,
		}))
}

function upsertReactionSummary(
	reactions: Array<{ reaction: string; count: number }>,
	reaction: string
): Array<{ reaction: string; count: number }> {
	const index = reactions.findIndex((item) => item.reaction === reaction)
	if (index < 0) {
		return [{ reaction, count: 1 }, ...reactions]
	}

	return reactions.map((item, currentIndex) =>
		currentIndex === index ? { ...item, count: Math.max(1, item.count + 1) } : item
	)
}
