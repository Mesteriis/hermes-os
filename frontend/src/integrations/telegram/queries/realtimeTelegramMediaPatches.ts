import type {
  TelegramMediaItem,
  TelegramMediaSearchResponse,
  TelegramMessage,
} from '../types/telegram'
import { isRecord, stringValue } from '../../../shared/communications/queries/realtimePatchShared'
import {
  type TelegramEventPayload,
  matchesMessageScope,
} from './realtimeTelegramPatchShared'

export function isTelegramMediaDownloadEvent(eventType: string): boolean {
  return (
    eventType === 'telegram.media.download.started' ||
    eventType === 'telegram.media.download.progress' ||
    eventType === 'telegram.media.download.failed' ||
    eventType === 'telegram.media.downloaded'
  )
}

export function patchTelegramMessageMediaDownloadState(
  message: TelegramMessage,
  eventType: string,
  payload: TelegramEventPayload | undefined,
  snapshot: TelegramMessage | null
): TelegramMessage {
  if (!isTelegramMediaDownloadEvent(eventType) || !payload) return message
  if (eventType === 'telegram.media.downloaded' && snapshot) return snapshot
  if (message.provider_chat_id && stringValue(payload.provider_chat_id) !== message.provider_chat_id) {
    return message
  }
  if (stringValue(payload.provider_message_id) !== message.provider_message_id) return message

  const nextMetadata = patchAttachmentCollection(message.metadata, payload)
  return nextMetadata === message.metadata ? message : { ...message, metadata: nextMetadata }
}

export function patchTelegramMediaSearch(
  queryKey: readonly unknown[],
  response: TelegramMediaSearchResponse | undefined,
  eventType: string,
  payload: TelegramEventPayload | undefined,
  snapshot: TelegramMessage | null
): TelegramMediaSearchResponse | undefined {
  if (!response || !isTelegramMediaDownloadEvent(eventType)) return response
  if (queryKey[0] !== 'communications' || queryKey[1] !== 'telegram' || queryKey[2] !== 'search' || queryKey[3] !== 'media') {
    return response
  }

  if (eventType === 'telegram.media.downloaded' && snapshot) {
    return upsertDownloadedMediaSnapshot(queryKey, response, snapshot)
  }

  if (!payload) return response
  const providerMessageId = stringValue(payload.provider_message_id)
  const providerChatId = stringValue(payload.provider_chat_id)
  if (!providerMessageId || !providerChatId) return response

  const nextItems = response.items.map((item) =>
    patchMediaItemDownloadState(item, providerChatId, providerMessageId, payload)
  )
  return nextItems.some((item, index) => item !== response.items[index])
    ? { ...response, items: nextItems }
    : response
}

function upsertDownloadedMediaSnapshot(
  queryKey: readonly unknown[],
  response: TelegramMediaSearchResponse,
  snapshot: TelegramMessage
): TelegramMediaSearchResponse {
  const query = typeof queryKey[4] === 'string' ? queryKey[4].trim().toLowerCase() : ''
  const accountId = typeof queryKey[5] === 'string' && queryKey[5] !== 'all' ? queryKey[5] : null
  const providerChatId = typeof queryKey[6] === 'string' && queryKey[6] !== 'all' ? queryKey[6] : null
  const kindFilter = typeof queryKey[7] === 'string' && queryKey[7] !== 'all' ? queryKey[7] : null
  const limit = typeof queryKey[8] === 'number' ? queryKey[8] : null

  if (!matchesMessageScope(snapshot, accountId, providerChatId)) return response

  const nextItemsById = new Map(
    response.items.map((item) => [`${item.message_id}:${item.file_name}`, item] as const)
  )
  for (const item of telegramMediaItemsFromMessageSnapshot(snapshot)) {
    if (kindFilter && item.kind !== kindFilter) continue
    if (query && !matchesMediaQuery(item, query)) continue
    nextItemsById.set(`${item.message_id}:${item.file_name}`, item)
  }

  const nextItems = Array.from(nextItemsById.values()).sort(
    (left, right) => (right.occurred_at ?? '').localeCompare(left.occurred_at ?? '')
  )
  return { ...response, items: typeof limit === 'number' ? nextItems.slice(0, limit) : nextItems }
}

function patchMediaItemDownloadState(
  item: TelegramMediaItem,
  providerChatId: string,
  providerMessageId: string,
  payload: TelegramEventPayload
): TelegramMediaItem {
  if (item.provider_chat_id !== providerChatId || item.provider_message_id !== providerMessageId) {
    return item
  }
  if (!attachmentMatchesPayload(item, payload)) return item

  return {
    ...item,
    download_state: stringValue(payload.download_state) ?? item.download_state,
    tdlib_file_id:
      typeof payload.tdlib_file_id === 'number' ? payload.tdlib_file_id : item.tdlib_file_id,
    provider_attachment_id:
      stringValue(payload.provider_attachment_id) ?? item.provider_attachment_id,
    local_path: stringValue(payload.local_path) ?? item.local_path,
    expected_size_bytes:
      typeof payload.expected_size_bytes === 'number'
        ? payload.expected_size_bytes
        : item.expected_size_bytes,
    downloaded_size_bytes:
      typeof payload.downloaded_size_bytes === 'number'
        ? payload.downloaded_size_bytes
        : item.downloaded_size_bytes,
    is_downloading_active:
      typeof payload.is_downloading_active === 'boolean'
        ? payload.is_downloading_active
        : item.is_downloading_active,
    is_downloading_completed:
      typeof payload.is_downloading_completed === 'boolean'
        ? payload.is_downloading_completed
        : item.is_downloading_completed,
    last_error: stringValue(payload.error) ?? item.last_error,
  }
}

function patchAttachmentCollection(
  metadata: Record<string, unknown>,
  payload: TelegramEventPayload
): Record<string, unknown> {
  const attachmentKey = Array.isArray(metadata.attachments)
    ? 'attachments'
    : Array.isArray(metadata.files)
      ? 'files'
      : null
  if (!attachmentKey) return metadata

  const rawAttachments = metadata[attachmentKey]
  if (!Array.isArray(rawAttachments)) return metadata
  const nextAttachments = rawAttachments.map((attachment) =>
    patchAttachmentRecord(attachment, payload)
  )
  return nextAttachments.some((attachment, index) => attachment !== rawAttachments[index])
    ? { ...metadata, [attachmentKey]: nextAttachments }
    : metadata
}

function patchAttachmentRecord(attachment: unknown, payload: TelegramEventPayload): unknown {
  if (!isRecord(attachment) || !attachmentMatchesPayload(attachment, payload)) return attachment

  const nextAttachment = { ...attachment }
  if (typeof payload.tdlib_file_id === 'number') nextAttachment.tdlib_file_id = payload.tdlib_file_id
  if (typeof payload.download_state === 'string') nextAttachment.download_state = payload.download_state
  if (typeof payload.local_path === 'string') nextAttachment.local_path = payload.local_path
  if (typeof payload.provider_attachment_id === 'string') {
    nextAttachment.provider_attachment_id = payload.provider_attachment_id
  }
  if (typeof payload.expected_size_bytes === 'number') {
    nextAttachment.expected_size_bytes = payload.expected_size_bytes
  }
  if (typeof payload.downloaded_size_bytes === 'number') {
    nextAttachment.downloaded_size_bytes = payload.downloaded_size_bytes
  }
  if (typeof payload.is_downloading_active === 'boolean') {
    nextAttachment.is_downloading_active = payload.is_downloading_active
  }
  if (typeof payload.is_downloading_completed === 'boolean') {
    nextAttachment.is_downloading_completed = payload.is_downloading_completed
  }
  if (typeof payload.error === 'string') nextAttachment.last_error = payload.error
  return nextAttachment
}

function attachmentMatchesPayload(
  attachment: Record<string, unknown> | TelegramMediaItem,
  payload: TelegramEventPayload
): boolean {
  const payloadAttachmentId = stringValue(payload.attachment_id) ?? stringValue(payload.provider_attachment_id)
  const attachmentId =
    stringValue((attachment as Record<string, unknown>).attachment_id)
    ?? stringValue((attachment as Record<string, unknown>).provider_attachment_id)
  if (payloadAttachmentId && attachmentId) return payloadAttachmentId === attachmentId

  const payloadTdlibFileId = typeof payload.tdlib_file_id === 'number' ? payload.tdlib_file_id : null
  const attachmentTdlibFileId =
    typeof (attachment as Record<string, unknown>).tdlib_file_id === 'number'
      ? ((attachment as Record<string, unknown>).tdlib_file_id as number)
      : null
  return payloadTdlibFileId !== null && attachmentTdlibFileId === payloadTdlibFileId
}

function telegramMediaItemsFromMessageSnapshot(message: TelegramMessage): TelegramMediaItem[] {
  const rawAttachments = message.metadata?.attachments ?? message.metadata?.files
  if (!Array.isArray(rawAttachments)) return []
  return rawAttachments.flatMap((attachment): TelegramMediaItem[] => {
    if (!isRecord(attachment)) return []
    const fileName = stringValue(attachment.filename) ?? stringValue(attachment.file_name)
    const kind = stringValue(attachment.attachment_type) ?? stringValue(attachment.kind) ?? 'file'
    if (!fileName || !message.provider_chat_id) return []
    return [{
      message_id: message.message_id,
      provider_message_id: message.provider_message_id,
      provider_chat_id: message.provider_chat_id,
      file_name: fileName,
      kind,
      mime_type: stringValue(attachment.content_type) ?? stringValue(attachment.mime_type),
      size_bytes: typeof attachment.size === 'number'
        ? attachment.size
        : typeof attachment.size_bytes === 'number' ? attachment.size_bytes : null,
      occurred_at: message.occurred_at,
      download_state: stringValue(attachment.download_state) ?? 'unknown',
      tdlib_file_id: typeof attachment.tdlib_file_id === 'number' ? attachment.tdlib_file_id : null,
      provider_attachment_id: stringValue(attachment.attachment_id) ?? stringValue(attachment.provider_attachment_id),
      local_path: stringValue(attachment.local_path),
      expected_size_bytes: typeof attachment.expected_size_bytes === 'number' ? attachment.expected_size_bytes : null,
      downloaded_size_bytes: typeof attachment.downloaded_size_bytes === 'number' ? attachment.downloaded_size_bytes : null,
      is_downloading_active: typeof attachment.is_downloading_active === 'boolean' ? attachment.is_downloading_active : null,
      is_downloading_completed: typeof attachment.is_downloading_completed === 'boolean' ? attachment.is_downloading_completed : null,
      last_error: stringValue(attachment.last_error),
    }]
  })
}

function matchesMediaQuery(item: TelegramMediaItem, query: string): boolean {
  return [item.file_name, item.kind, item.provider_message_id, item.mime_type ?? '']
    .join(' ')
    .toLowerCase()
    .includes(query)
}
