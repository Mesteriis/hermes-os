import type {
  TelegramChat,
  TelegramChatGroupFilter,
  TelegramMediaSearchResponse,
  TelegramMessage,
  TelegramMessageListResponse,
  TelegramMessageSearchResponse,
  TelegramReactionListResponse,
  TelegramRuntimeStatus
} from '../../../shared/communications/types/telegram'
import type { TelegramTopicListResponse } from '../../../shared/communications/types/telegramTopics'
import { isRecord, storedEventEnvelope, stringValue } from '../../../shared/communications/queries/realtimePatchShared'
import { patchTelegramTopicList } from './realtimeTelegramTopicPatches'
import {
  isTelegramMediaDownloadEvent,
  patchTelegramMediaSearch,
  patchTelegramMessageMediaDownloadState,
} from './realtimeTelegramMediaPatches'
import {
  TELEGRAM_TYPING_TTL_MS,
  type TelegramEventPayload,
  type TelegramStoredEventEnvelope,
  chatQueryScope,
  eventSubjectId,
  insertChatByRecency,
  insertMessageByRecency,
  matchesChatScope,
  matchesMessageScope,
  messageQueryScope,
  patchPinMetadata,
  runtimeAccountId,
  telegramChatSnapshot,
  telegramMessageSnapshot
} from './realtimeTelegramPatchShared'

export type TelegramRealtimePatchQueryClient = {
  getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
    [readonly unknown[], TData | undefined]
  >
  setQueryData?: <TData>(
    queryKey: readonly unknown[],
    updater: TData | ((data: TData | undefined) => TData | undefined)
  ) => unknown
}

export function applyTelegramRealtimePatch(
  eventData: string,
  queryClient: TelegramRealtimePatchQueryClient
): boolean {
  const { getQueriesData, setQueryData } = queryClient
  if (!getQueriesData || !setQueryData) return false

  const envelope = storedEventEnvelope(eventData) as TelegramStoredEventEnvelope | null
  const eventType = stringValue(envelope?.event?.event_type)
  if (!eventType || !eventType.startsWith('telegram.')) return false

  const occurredAt = stringValue(envelope?.event?.occurred_at)
  const subjectId = eventSubjectId(envelope?.event?.subject)
  const payload = isRecord(envelope?.event?.payload)
    ? (envelope?.event?.payload as TelegramEventPayload)
    : undefined
  const metadata = isRecord(envelope?.event?.metadata)
    ? (envelope?.event?.metadata as Record<string, unknown>)
    : undefined
  const snapshot = telegramMessageSnapshot(payload?.message)
  const chatSnapshot = telegramChatSnapshot(payload?.chat)

  let patched = false
  for (const [queryKey, data] of getQueriesData<TelegramMessage[]>({
    queryKey: ['communications', 'telegram', 'messages']
  })) {
    const updated = patchMessageList(queryKey, data, eventType, subjectId, payload, snapshot)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramMessageListResponse>({
    queryKey: ['communications', 'telegram', 'chats']
  })) {
    if (!isTelegramPinnedMessagesQueryKey(queryKey)) continue
    const updated = patchPinnedMessages(queryKey, data, eventType, subjectId, payload, snapshot)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramChat[]>({
    queryKey: ['communications', 'telegram', 'chats']
  })) {
    if (isTelegramPinnedMessagesQueryKey(queryKey)) continue
    const updated = patchChatList(queryKey, data, eventType, payload, chatSnapshot, occurredAt)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramChat | null>({
    queryKey: ['communications', 'telegram', 'chat-detail']
  })) {
    const updated = patchChatDetail(queryKey, data, eventType, payload, chatSnapshot, occurredAt)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramChatGroupFilter[]>({
    queryKey: ['communications', 'telegram', 'folders']
  })) {
    const updated = patchFolderFilters(queryKey, data, eventType, payload, metadata)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramMessageSearchResponse>({
    queryKey: ['communications', 'telegram', 'search', 'messages']
  })) {
    const updated = patchMessageSearch(queryKey, data, eventType, subjectId, payload, snapshot)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramMediaSearchResponse>({
    queryKey: ['communications', 'telegram', 'search', 'media']
  })) {
    const updated = patchTelegramMediaSearch(queryKey, data, eventType, payload, snapshot)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramReactionListResponse>({
    queryKey: ['communications', 'telegram', 'message-reactions']
  })) {
    const updated = patchReactionDetail(queryKey, data, eventType, subjectId, payload)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramTopicListResponse>({
    queryKey: ['communications', 'telegram']
  })) {
    const updated = patchTelegramTopicList(queryKey, data, eventType, payload)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  for (const [queryKey, data] of getQueriesData<TelegramRuntimeStatus | null>({
    queryKey: ['integrations', 'telegram', 'runtime']
  })) {
    const updated = patchRuntimeStatus(queryKey, data, eventType, payload, metadata)
    if (updated !== data) {
      setQueryData(queryKey, updated)
      patched = true
    }
  }

  return patched
}

function patchMessageList(
  queryKey: readonly unknown[],
  messages: TelegramMessage[] | undefined,
  eventType: string,
  subjectId: string | null,
  payload: TelegramEventPayload | undefined,
  snapshot: TelegramMessage | null
): TelegramMessage[] | undefined {
  if (!messages) return messages

  if (isTelegramMediaDownloadEvent(eventType)) {
    const nextMessages = messages.map((message) =>
      patchTelegramMessageMediaDownloadState(message, eventType, payload, snapshot)
    )
    return nextMessages.some((message, index) => message !== messages[index])
      ? nextMessages
      : messages
  }

  const targetMessageId = subjectId ?? snapshot?.message_id ?? null
  if (!targetMessageId) return messages
  const [accountId, providerChatId, limit] = messageQueryScope(queryKey)
  if (snapshot && !matchesMessageScope(snapshot, accountId, providerChatId)) {
    return messages
  }

  const patched = messages.map((message) => {
    if (message.message_id !== targetMessageId) return message

    if (eventType === 'telegram.message.created') {
      if (snapshot) return snapshot
      return {
        ...message,
        delivery_state: stringValue(payload?.delivery_state) ?? message.delivery_state,
      }
    }

    if (eventType === 'telegram.message.updated' || eventType === 'telegram.message.edited') {
      const meta = {
        ...(snapshot?.metadata ?? message.metadata),
        lifecycle: {
          ...(snapshot && isRecord(snapshot.metadata.lifecycle) ? snapshot.metadata.lifecycle : {}),
          ...(isRecord(message.metadata.lifecycle) ? message.metadata.lifecycle : {}),
          latest_version_number:
            typeof payload?.version_number === 'number'
              ? payload.version_number
              : snapshot?.metadata.latest_version_number ?? message.metadata.latest_version_number ?? null,
        },
      }
      return { ...(snapshot ?? message), metadata: patchPinMetadata(meta, payload) }
    }

    if (eventType === 'telegram.message.deleted' || eventType === 'telegram.message.visibility_restored') {
      return {
        ...(snapshot ?? message),
        metadata: {
          ...(snapshot?.metadata ?? message.metadata),
          tombstone: {
            reason_class: stringValue(payload?.reason_class),
            tombstone_id: stringValue(payload?.tombstone_id),
            is_visible: eventType === 'telegram.message.visibility_restored',
          },
        },
      }
    }

    if (eventType === 'telegram.media.downloaded' && snapshot) return snapshot
    if (eventType === 'telegram.reaction.changed') {
      const reactionEmoji = stringValue(payload?.reaction_emoji)
      if (!reactionEmoji) return message

      const currentMetadata = snapshot?.metadata ?? message.metadata
      const currentSummary = isRecord(currentMetadata.reaction_summary)
        ? currentMetadata.reaction_summary
        : { reactions: [] as Array<Record<string, unknown>> }
      const currentReactions = Array.isArray(currentSummary.reactions) ? currentSummary.reactions : []
      const existingIndex = currentReactions.findIndex(
        (item) => isRecord(item) && stringValue(item.reaction_emoji) === reactionEmoji
      )
      const isActive = payload?.is_active === true
      const nextReactions = currentReactions.slice()
      if (existingIndex >= 0 && isRecord(nextReactions[existingIndex])) {
        const existing = nextReactions[existingIndex]
        const currentCount = typeof existing.count === 'number' ? existing.count : 0
        nextReactions[existingIndex] = {
          ...existing,
          count: isActive ? currentCount + 1 : Math.max(currentCount - 1, 0),
        }
      } else if (isActive) {
        nextReactions.push({ reaction_emoji: reactionEmoji, count: 1, senders: [] })
      }
      return {
        ...(snapshot ?? message),
        metadata: {
          ...currentMetadata,
          reaction_summary: {
            ...currentSummary,
            reactions: nextReactions.filter(
              (item) => !isRecord(item) || typeof item.count !== 'number' || item.count > 0
            ),
          },
        },
      }
    }

    return message
  })

  const existingIndex = patched.findIndex((message) => message.message_id === targetMessageId)
  if (existingIndex >= 0) return patched
  if ((eventType === 'telegram.message.created' || eventType === 'telegram.media.downloaded') && snapshot) {
    return insertMessageByRecency(messages, snapshot, limit)
  }
  if (eventType === 'telegram.message.updated' && snapshot) {
    return insertMessageByRecency(
      messages,
      { ...snapshot, metadata: patchPinMetadata(snapshot.metadata, payload) },
      limit
    )
  }
  return messages
}

function patchRuntimeStatus(
  queryKey: readonly unknown[],
  status: TelegramRuntimeStatus | null | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined,
  metadata: Record<string, unknown> | undefined
): TelegramRuntimeStatus | null | undefined {
  if (!status) return status
  const queryAccountId = runtimeAccountId(queryKey)
  const eventAccountId = stringValue(metadata?.account_id)
  if (queryAccountId && eventAccountId && queryAccountId !== eventAccountId) return status

  if (eventType.startsWith('telegram.sync.')) {
    if (!payload) return status
    const scope = stringValue(payload.scope)
    if (!scope) return status
    return {
      ...status,
      status:
        eventType === 'telegram.sync.failed'
          ? 'degraded'
          : eventType === 'telegram.sync.started' || eventType === 'telegram.sync.progress'
            ? 'running'
            : status.status,
      last_sync_scope: scope,
      last_sync_status: stringValue(payload.status),
      last_synced_count: typeof payload.synced_count === 'number' ? payload.synced_count : null,
      last_sync_has_more: typeof payload.has_more === 'boolean' ? payload.has_more : null,
      last_sync_provider_chat_id: stringValue(payload.provider_chat_id),
      updated_at: new Date().toISOString(),
    }
  }

  if (eventType !== 'telegram.command.status_changed' || !payload) return status
  return {
    ...status,
    last_command_id: stringValue(payload.command_id),
    last_command_status: stringValue(payload.status),
    last_command_kind: stringValue(payload.command_kind),
    last_command_provider_chat_id: stringValue(payload.provider_chat_id),
    last_command_message_id: stringValue(payload.message_id),
    last_command_telegram_chat_id: stringValue(payload.telegram_chat_id),
    updated_at: new Date().toISOString(),
  }
}

function patchFolderFilters(
  queryKey: readonly unknown[],
  filters: TelegramChatGroupFilter[] | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined,
  metadata: Record<string, unknown> | undefined
): TelegramChatGroupFilter[] | undefined {
  if (!filters || !payload) return filters
  if (queryKey[0] !== 'communications' || queryKey[1] !== 'telegram' || queryKey[2] !== 'folders') return filters
  const queryAccountId = typeof queryKey[3] === 'string' ? queryKey[3] : 'all'
  const eventAccountId = stringValue(payload.account_id) ?? stringValue(metadata?.account_id)
  if (queryAccountId !== 'all' && eventAccountId && queryAccountId !== eventAccountId) {
    return filters
  }
  if (eventType !== 'telegram.folders.updated') return filters
  const items = payload.items
  if (!Array.isArray(items)) return filters
  return items
    .filter((item): item is TelegramChatGroupFilter => isRecord(item))
    .map((item) => ({
      id: stringValue(item.id) ?? 'local:all',
      label: stringValue(item.label) ?? 'All',
      source: (stringValue(item.source) === 'telegram' ? 'telegram' : 'local'),
      count: typeof item.count === 'number' ? item.count : 0,
      icon: stringValue(item.icon) ?? 'tabler:folder',
      provider_folder_id:
        typeof item.provider_folder_id === 'number' ? item.provider_folder_id : null,
    }))
}

function patchChatList(
  queryKey: readonly unknown[],
  chats: TelegramChat[] | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined,
  snapshot: TelegramChat | null,
  occurredAt: string | null
): TelegramChat[] | undefined {
  if (!chats || !payload) return chats

  if (eventType === 'telegram.typing.changed') {
    const [accountId] = chatQueryScope(queryKey)
    const nextChats = chats.map((chat) => {
      if (accountId && chat.account_id !== accountId) return chat
      return matchesTypingChat(chat, payload) ? patchTypingChat(chat, payload, occurredAt) : chat
    })
    return nextChats.some((chat, i) => chat !== chats[i]) ? nextChats : chats
  }

  // Chat flag events: surgical metadata toggle, no snapshot required
  if (
    eventType === 'telegram.chat.pinned' ||
    eventType === 'telegram.chat.archived' ||
    eventType === 'telegram.chat.muted'
  ) {
    const targetChatId = stringValue(payload.telegram_chat_id)
    if (!targetChatId) return chats
    const [accountId] = chatQueryScope(queryKey)
    const metadataPatch =
      eventType === 'telegram.chat.pinned' ? { is_pinned: payload.is_pinned } :
      eventType === 'telegram.chat.archived' ? { is_archived: payload.is_archived } :
      { is_muted: payload.is_muted }
    const nextChats = chats.map((chat) => {
      if (chat.telegram_chat_id !== targetChatId) return chat
      if (accountId && chat.account_id !== accountId) return chat
      return { ...chat, metadata: { ...chat.metadata, ...metadataPatch } }
    })
    return nextChats.some((chat, i) => chat !== chats[i]) ? nextChats : chats
  }

  if (!snapshot) return chats

  if (eventType === 'telegram.chat.updated') {
    const [accountId, limit] = chatQueryScope(queryKey)
    if (!matchesChatScope(snapshot, accountId)) return chats
    const existingIndex = chats.findIndex((chat) => chat.telegram_chat_id === snapshot.telegram_chat_id)
    if (existingIndex < 0) return insertChatByRecency(chats, snapshot, limit)
    return chats.map((chat) => chat.telegram_chat_id === snapshot.telegram_chat_id ? snapshot : chat)
  }

  const supportsRealtimeChatPatch = eventType === 'telegram.command.status_changed'
    || eventType === 'telegram.message.created'
    || eventType === 'telegram.message.updated'
    || eventType === 'telegram.message.deleted'
    || eventType === 'telegram.message.visibility_restored'
  if (!supportsRealtimeChatPatch) return chats

  const [accountId, limit] = chatQueryScope(queryKey)
  if (!matchesChatScope(snapshot, accountId)) return chats

  if (eventType === 'telegram.message.created') return insertChatByRecency(chats, snapshot, limit)

  const existingIndex = chats.findIndex((chat) => chat.telegram_chat_id === snapshot.telegram_chat_id)
  if (existingIndex < 0) return chats
  return chats.map((chat) => chat.telegram_chat_id === snapshot.telegram_chat_id ? snapshot : chat)
}

function patchChatDetail(
  queryKey: readonly unknown[],
  chat: TelegramChat | null | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined,
  snapshot: TelegramChat | null,
  occurredAt: string | null
): TelegramChat | null | undefined {
  if (!chat || !payload) return chat
  if (queryKey[0] !== 'communications' || queryKey[1] !== 'telegram' || queryKey[2] !== 'chat-detail') return chat

  if (eventType === 'telegram.typing.changed') {
    return matchesTypingChat(chat, payload) ? patchTypingChat(chat, payload, occurredAt) : chat
  }

  if (
    eventType === 'telegram.chat.pinned' ||
    eventType === 'telegram.chat.archived' ||
    eventType === 'telegram.chat.muted'
  ) {
    const targetChatId = stringValue(payload.telegram_chat_id)
    if (!targetChatId || queryKey[3] !== targetChatId) return chat
    const metadataPatch =
      eventType === 'telegram.chat.pinned' ? { is_pinned: payload.is_pinned } :
      eventType === 'telegram.chat.archived' ? { is_archived: payload.is_archived } :
      { is_muted: payload.is_muted }
    return { ...chat, metadata: { ...chat.metadata, ...metadataPatch } }
  }

  if (!snapshot) return chat
  if (eventType === 'telegram.chat.updated') {
    if (queryKey[3] !== snapshot.telegram_chat_id) return chat
    return snapshot
  }

  const supportsRealtimeChatPatch = eventType === 'telegram.command.status_changed'
    || eventType === 'telegram.message.created'
    || eventType === 'telegram.message.updated'
    || eventType === 'telegram.message.deleted'
    || eventType === 'telegram.message.visibility_restored'
  if (!supportsRealtimeChatPatch || queryKey[3] !== snapshot.telegram_chat_id) return chat
  return snapshot
}

function matchesTypingChat(chat: TelegramChat, payload: TelegramEventPayload): boolean {
  const telegramChatId = stringValue(payload.telegram_chat_id)
  const providerChatId = stringValue(payload.provider_chat_id)
  if (telegramChatId) return chat.telegram_chat_id === telegramChatId
  if (providerChatId) return chat.provider_chat_id === providerChatId
  return false
}

function patchTypingChat(chat: TelegramChat, payload: TelegramEventPayload, occurredAt: string | null): TelegramChat {
  const senderId = stringValue(payload.sender_id)
  const action = stringValue(payload.action)
  const providerThreadId = stringValue(payload.provider_thread_id)
  const isActive = payload.is_active === true
  const startedAtMs = occurredAt ? Date.parse(occurredAt) : NaN
  const expiresAt = new Date((Number.isFinite(startedAtMs) ? startedAtMs : Date.now()) + TELEGRAM_TYPING_TTL_MS).toISOString()
  return {
    ...chat,
    metadata: {
      ...chat.metadata,
      active_typing: isActive
        ? {
            sender_id: senderId,
            action,
            provider_thread_id: providerThreadId,
            is_active: true,
            expires_at: expiresAt,
          }
        : null,
    },
  }
}

function isTelegramPinnedMessagesQueryKey(queryKey: readonly unknown[]): boolean {
  return queryKey[0] === 'communications' && queryKey[1] === 'telegram' && queryKey[2] === 'chats' && queryKey[4] === 'pinned-messages'
}

function patchPinnedMessages(
  queryKey: readonly unknown[],
  response: TelegramMessageListResponse | undefined,
  eventType: string,
  subjectId: string | null,
  payload: TelegramEventPayload | undefined,
  snapshot: TelegramMessage | null
): TelegramMessageListResponse | undefined {
  if (!response) return response
  const telegramChatId = stringValue(payload?.telegram_chat_id)
  if (!telegramChatId || queryKey[3] !== telegramChatId) return response

  const targetMessageId = subjectId ?? snapshot?.message_id ?? null
  if (!targetMessageId) return response

  const isPinned = typeof payload?.is_pinned === 'boolean'
    ? payload.is_pinned
    : Boolean(snapshot?.metadata.is_pinned ?? snapshot?.metadata.pinned)
  const existing = response.items.filter((item) => item.message_id !== targetMessageId)
  if (!isPinned || eventType === 'telegram.message.deleted') {
    return existing.length === response.items.length ? response : { ...response, items: existing }
  }
  if (!snapshot) return response
  const limit = typeof queryKey[5] === 'number' ? queryKey[5] : null
  return {
    ...response,
    items: insertMessageByRecency(
      existing,
      { ...snapshot, metadata: patchPinMetadata(snapshot.metadata, payload) },
      limit
    ),
  }
}

function patchMessageSearch(
  queryKey: readonly unknown[],
  response: TelegramMessageSearchResponse | undefined,
  eventType: string,
  subjectId: string | null,
  payload: TelegramEventPayload | undefined,
  snapshot: TelegramMessage | null
): TelegramMessageSearchResponse | undefined {
  if (!response || !snapshot) return response
  if (queryKey[0] !== 'communications' || queryKey[1] !== 'telegram' || queryKey[2] !== 'search' || queryKey[3] !== 'messages') return response

  const query = typeof queryKey[4] === 'string' ? queryKey[4].trim().toLowerCase() : ''
  const accountId = typeof queryKey[5] === 'string' && queryKey[5] !== 'all' ? queryKey[5] : null
  const providerChatId = typeof queryKey[6] === 'string' && queryKey[6] !== 'all' ? queryKey[6] : null
  const limit = typeof queryKey[7] === 'number' ? queryKey[7] : null
  const targetMessageId = subjectId ?? snapshot.message_id

  if (!matchesMessageScope(snapshot, accountId, providerChatId)) return response

  const matchesQuery = [snapshot.text, snapshot.sender, snapshot.sender_display_name ?? '', snapshot.provider_message_id]
    .join(' ').toLowerCase().includes(query)
  const nextItems = response.items.filter((item) => item.message_id !== targetMessageId)

  if (!matchesQuery || eventType === 'telegram.message.deleted') {
    if (nextItems.length === response.items.length) return response
    return { ...response, items: nextItems, total: Math.max(response.total - 1, nextItems.length) }
  }

  const inserted = insertMessageByRecency(
    nextItems,
    { ...snapshot, metadata: patchPinMetadata(snapshot.metadata, payload) },
    limit
  )
  return { ...response, items: inserted, total: Math.max(response.total, inserted.length) }
}

function patchReactionDetail(
  queryKey: readonly unknown[],
  detail: TelegramReactionListResponse | undefined,
  eventType: string,
  subjectId: string | null,
  payload: TelegramEventPayload | undefined
): TelegramReactionListResponse | undefined {
  if (!detail || eventType !== 'telegram.reaction.changed' || !payload) return detail
  if (queryKey[0] !== 'communications' || queryKey[1] !== 'telegram' || queryKey[2] !== 'message-reactions') return detail

  const queryMessageId = typeof queryKey[3] === 'string' ? queryKey[3] : null
  const targetMessageId = subjectId ?? stringValue(payload.message_id)
  if (!queryMessageId || !targetMessageId || queryMessageId !== targetMessageId) return detail

  const reactionEmoji = stringValue(payload.reaction_emoji)
  if (!reactionEmoji) return detail
  const isActive = payload.is_active === true

  // Update summary aggregate counts only — individual sender records require a full fetch
  const currentSummary = detail.summary
  const currentReactions = currentSummary.reactions ?? []
  const existingIdx = currentReactions.findIndex((r) => r.reaction_emoji === reactionEmoji)
  const nextReactions = currentReactions.slice()

  if (existingIdx >= 0) {
    const existing = nextReactions[existingIdx]
    const nextCount = isActive ? existing.count + 1 : Math.max(existing.count - 1, 0)
    nextReactions[existingIdx] = { ...existing, count: nextCount }
  } else if (isActive) {
    nextReactions.push({ reaction_emoji: reactionEmoji, count: 1, senders: [] })
  }

  const filteredReactions = nextReactions.filter((r) => r.count > 0)
  const totalActive = filteredReactions.reduce((acc, r) => acc + r.count, 0)

  return {
    ...detail,
    summary: {
      ...currentSummary,
      reactions: filteredReactions,
      active_reactions: totalActive,
      total_reactions: totalActive,
    },
  }
}
