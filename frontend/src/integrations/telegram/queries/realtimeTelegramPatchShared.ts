import type { TelegramChat, TelegramMessage } from '../types/telegram'
import { isRecord, stringValue } from '../../../domains/communications/queries/realtimePatchShared'

export const TELEGRAM_TYPING_TTL_MS = 7000

export type TelegramEventPayload = {
  account_id?: unknown
  provider_chat_id?: unknown
  provider_message_id?: unknown
  delivery_state?: unknown
  runtime_kind?: unknown
  status?: unknown
  version_number?: unknown
  reason_class?: unknown
  tombstone_id?: unknown
  reaction_emoji?: unknown
  is_active?: unknown
  scope?: unknown
  synced_count?: unknown
  has_more?: unknown
  tdlib_file_id?: unknown
  download_state?: unknown
  local_path?: unknown
  provider_attachment_id?: unknown
  expected_size_bytes?: unknown
  downloaded_size_bytes?: unknown
  is_downloading_active?: unknown
  is_downloading_completed?: unknown
  error?: unknown
  attachment_id?: unknown
  blob_id?: unknown
  scan_status?: unknown
  command_id?: unknown
  command_kind?: unknown
  retry_count?: unknown
  max_retries?: unknown
  last_error?: unknown
  result_payload?: unknown
  next_attempt_at?: unknown
  last_attempt_at?: unknown
  provider_observed_at?: unknown
  provider_state?: unknown
  reconciliation_status?: unknown
  reconciled_at?: unknown
  dead_lettered_at?: unknown
  completed_at?: unknown
  idempotency_key?: unknown
  target_ref?: unknown
  capability_state?: unknown
  action_class?: unknown
  confirmation_decision?: unknown
  audit_metadata?: unknown
  actor_id?: unknown
  happened_at?: unknown
  created_at?: unknown
  updated_at?: unknown
  action?: unknown
  list_kind?: unknown
  provider_folder_id?: unknown
  order?: unknown
  message_id?: unknown
  is_pinned?: unknown
  is_archived?: unknown
  is_muted?: unknown
  telegram_chat_id?: unknown
  provider_thread_id?: unknown
  sender_id?: unknown
  topic?: unknown
  chat?: unknown
  message?: unknown
  items?: unknown
  payload?: unknown
}

export type TelegramStoredEventEnvelope = {
  event?: {
    event_type?: unknown
    occurred_at?: unknown
    metadata?: unknown
    subject?: unknown
    payload?: unknown
  }
}

export function eventSubjectId(subject: unknown): string | null {
  if (!isRecord(subject)) return null
  return stringValue(subject.id)
}

export function runtimeAccountId(queryKey: readonly unknown[]): string | null {
  if (queryKey[0] !== 'integrations' || queryKey[1] !== 'telegram' || queryKey[2] !== 'runtime') return null
  return typeof queryKey[3] === 'string' ? queryKey[3] : null
}

export function telegramChatSnapshot(value: unknown): TelegramChat | null {
  if (!isRecord(value)) return null
  const telegramChatId = stringValue(value.telegram_chat_id)
  const accountId = stringValue(value.account_id)
  const providerChatId = stringValue(value.provider_chat_id)
  const chatKind = stringValue(value.chat_kind)
  const title = stringValue(value.title)
  const syncState = stringValue(value.sync_state)
  const createdAt = stringValue(value.created_at)
  const updatedAt = stringValue(value.updated_at)
  if (!telegramChatId || !accountId || !providerChatId || !chatKind || !title || !syncState || !createdAt || !updatedAt) {
    return null
  }
  return {
    telegram_chat_id: telegramChatId,
    account_id: accountId,
    provider_chat_id: providerChatId,
    chat_kind: chatKind as TelegramChat['chat_kind'],
    title,
    username: stringValue(value.username),
    sync_state: syncState as TelegramChat['sync_state'],
    last_message_at: stringValue(value.last_message_at),
    metadata: isRecord(value.metadata) ? value.metadata : {},
    created_at: createdAt,
    updated_at: updatedAt,
  }
}

export function telegramMessageSnapshot(value: unknown): TelegramMessage | null {
  if (!isRecord(value)) return null
  const messageId = stringValue(value.message_id)
  const accountId = stringValue(value.account_id)
  const providerMessageId = stringValue(value.provider_message_id)
  const chatTitle = stringValue(value.chat_title)
  const sender = stringValue(value.sender)
  const projectedAt = stringValue(value.projected_at)
  const channelKind = stringValue(value.channel_kind)
  const deliveryState = stringValue(value.delivery_state)
  if (!messageId || !accountId || !providerMessageId || !chatTitle || !sender || !projectedAt || !channelKind || !deliveryState) {
    return null
  }
  return {
    message_id: messageId,
    raw_record_id: stringValue(value.raw_record_id) ?? '',
    account_id: accountId,
    provider_message_id: providerMessageId,
    provider_chat_id: stringValue(value.provider_chat_id),
    chat_title: chatTitle,
    sender,
    sender_display_name: stringValue(value.sender_display_name),
    text: stringValue(value.text) ?? '',
    occurred_at: stringValue(value.occurred_at),
    projected_at: projectedAt,
    channel_kind: channelKind as TelegramMessage['channel_kind'],
    delivery_state: deliveryState,
    metadata: isRecord(value.metadata) ? value.metadata : {},
  }
}

export function messageQueryScope(queryKey: readonly unknown[]): [string | null, string | null, number | null] {
  if (queryKey[0] !== 'integrations' || queryKey[1] !== 'telegram' || queryKey[2] !== 'messages') return [null, null, null]
  const accountId = typeof queryKey[3] === 'string' && queryKey[3] !== 'all' && queryKey[3] !== 'none'
    ? queryKey[3]
    : null
  const providerChatId = typeof queryKey[4] === 'string' && queryKey[4] !== 'all' && queryKey[4] !== 'none'
    ? queryKey[4]
    : null
  const limit = typeof queryKey[5] === 'number' ? queryKey[5] : null
  return [accountId, providerChatId, limit]
}

export function chatQueryScope(queryKey: readonly unknown[]): [string | null, number | null] {
  if (queryKey[0] !== 'integrations' || queryKey[1] !== 'telegram' || queryKey[2] !== 'chats') return [null, null]
  const accountId = typeof queryKey[3] === 'string' && queryKey[3] !== 'all' ? queryKey[3] : null
  const limit = typeof queryKey[4] === 'number' ? queryKey[4] : null
  return [accountId, limit]
}

export function matchesMessageScope(message: TelegramMessage, accountId: string | null, providerChatId: string | null): boolean {
  if (accountId && message.account_id !== accountId) return false
  if (providerChatId && message.provider_chat_id !== providerChatId) return false
  return true
}

export function matchesChatScope(chat: TelegramChat, accountId: string | null): boolean {
  if (accountId && chat.account_id !== accountId) return false
  return true
}

export function insertMessageByRecency(
  messages: TelegramMessage[],
  nextMessage: TelegramMessage,
  limit: number | null
): TelegramMessage[] {
  const items = [nextMessage, ...messages.filter((m) => m.message_id !== nextMessage.message_id)]
  items.sort((l, r) => messageRecencyKey(r).localeCompare(messageRecencyKey(l)))
  return typeof limit === 'number' ? items.slice(0, limit) : items
}

export function insertChatByRecency(
  chats: TelegramChat[],
  nextChat: TelegramChat,
  limit: number | null
): TelegramChat[] {
  const items = [nextChat, ...chats.filter((c) => c.telegram_chat_id !== nextChat.telegram_chat_id)]
  items.sort((l, r) => chatRecencyKey(r).localeCompare(chatRecencyKey(l)))
  return typeof limit === 'number' ? items.slice(0, limit) : items
}

export function patchPinMetadata(
  metadata: Record<string, unknown>,
  payload: TelegramEventPayload | undefined
): Record<string, unknown> {
  if (typeof payload?.is_pinned !== 'boolean') return metadata
  return { ...metadata, pinned: payload.is_pinned, is_pinned: payload.is_pinned }
}

function messageRecencyKey(m: TelegramMessage): string { return m.occurred_at ?? m.projected_at ?? '' }
function chatRecencyKey(c: TelegramChat): string { return c.last_message_at ?? c.updated_at ?? '' }
