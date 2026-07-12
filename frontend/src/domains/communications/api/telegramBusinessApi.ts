import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramChatDetailResponse,
  TelegramChatGroupFilterListResponse,
  TelegramChatHistoryPolicyRequest,
  TelegramChatListResponse,
  TelegramChatMemberListResponse,
  TelegramChatActionResponse,
  TelegramChatReadReceiptPolicyRequest,
  TelegramChatUnreadCounterPolicyRequest,
  TelegramChatSearchResponse,
  TelegramMediaSearchResponse,
  TelegramMessageListResponse,
  TelegramMessagePageResponse,
  TelegramMessageSearchResponse,
  TelegramTopicListResponse,
} from '../../../shared/communications/types/telegram'
import type {
  TelegramForwardChainResponse,
  TelegramLifecycleResponse,
  TelegramMessageTombstoneListResponse,
  TelegramMessageVersionListResponse,
  TelegramReactionListResponse,
  TelegramReactionRequest,
  TelegramReactionResponse,
  TelegramReplyChainResponse,
  TelegramMessage,
  TelegramProviderKind,
} from '../../../shared/communications/types/telegram'
import type { TelegramRawMessageResponse } from '../../../shared/communications/types/telegramRawEvidence'
import type {
  TelegramTopicCreateRequest,
  TelegramTopicCloseRequest,
  TelegramTopicLifecycleResponse,
} from '../../../shared/communications/types/telegramTopics'
import type { AttachmentPreviewResponse } from '../types/attachments'
import type {
  CommunicationProviderMessageCommandResponse,
  CommunicationMessageSummary,
  CommunicationMessagesResponse,
  MessagePinToggleResponse,
} from '../types/communications'

export async function fetchTelegramBusinessChats(accountId?: string, limit = 50): Promise<TelegramChatListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  return ApiClient.instance.get<TelegramChatListResponse>(
    `/api/v1/communications/conversations?${params.toString()}`,
    'Communication conversations request failed'
  )
}

export async function fetchTelegramBusinessChatFolders(
  accountId?: string
): Promise<TelegramChatGroupFilterListResponse> {
  const params = new URLSearchParams()
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  const suffix = params.size ? `?${params.toString()}` : ''
  return ApiClient.instance.get<TelegramChatGroupFilterListResponse>(
    `/api/v1/communications/conversation-folders${suffix}`,
    'Communication conversation folders request failed'
  )
}

export async function fetchTelegramBusinessChatDetail(conversationId: string): Promise<TelegramChatDetailResponse> {
  return ApiClient.instance.get<TelegramChatDetailResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}`,
    'Communication conversation detail request failed'
  )
}

export function fetchTelegramBusinessChatAvatar(telegramChatId: string): Promise<Blob> {
  return ApiClient.instance.getBlob(
    `/api/v1/communications/conversations/${encodeURIComponent(telegramChatId)}/avatar`,
    'Telegram chat avatar request failed'
  )
}

export function syncTelegramBusinessChatAvatar(telegramChatId: string): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(telegramChatId)}/avatar`,
    {},
    'Telegram chat avatar sync failed'
  )
}

export function updateTelegramBusinessChatHistoryPolicy(
  telegramChatId: string,
  request: TelegramChatHistoryPolicyRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.put<TelegramChatActionResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(telegramChatId)}/history-policy`,
    request,
    'Telegram history policy update failed'
  )
}

export function updateTelegramBusinessChatReadReceiptPolicy(
  telegramChatId: string,
  request: TelegramChatReadReceiptPolicyRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.put<TelegramChatActionResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(telegramChatId)}/read-receipt-policy`,
    request,
    'Telegram read receipt policy update failed'
  )
}

export function updateTelegramBusinessChatUnreadCounterPolicy(
  telegramChatId: string,
  request: TelegramChatUnreadCounterPolicyRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.put<TelegramChatActionResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(telegramChatId)}/unread-counter-policy`,
    request,
    'Telegram unread counter policy update failed'
  )
}

export async function fetchTelegramBusinessChatMembers(
  conversationId: string,
  limit = 50,
  query?: string,
  role?: string,
  cursor?: string
): Promise<TelegramChatMemberListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (query?.trim()) params.set('query', query.trim())
  if (role?.trim()) params.set('role', role.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get<TelegramChatMemberListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}/members?${params.toString()}`,
    'Communication conversation members request failed'
  )
}

export async function fetchTelegramBusinessMessages(
  accountId?: string,
  providerChatId?: string,
  limit = 100,
  cursor?: string
): Promise<TelegramMessagePageResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)), channel_kind: 'telegram' })
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  if (providerChatId?.trim()) params.set('conversation_id', providerChatId.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  const response = await ApiClient.instance.get<CommunicationMessagesResponse>(
    `/api/v1/communications/messages?${params.toString()}`,
    'Communication messages request failed'
  )
  return {
    items: response.items.map(communicationMessageToTelegramMessage),
    next_cursor: response.next_cursor,
    has_more: response.has_more,
  }
}

export async function searchTelegramBusinessChats(params: {
  q: string
  account_id?: string
  limit?: number
}): Promise<TelegramChatSearchResponse> {
  const query = new URLSearchParams({ q: params.q.trim() })
  if (params.account_id?.trim()) query.set('account_id', params.account_id.trim())
  if (params.limit != null) query.set('limit', String(Math.trunc(params.limit)))
  return ApiClient.instance.get<TelegramChatSearchResponse>(
    `/api/v1/communications/conversations/search?${query.toString()}`,
    'Communication conversation search failed'
  )
}

export async function searchTelegramBusinessMessages(params: {
  q: string
  account_id?: string
  provider_chat_id?: string
  limit?: number
}): Promise<TelegramMessageSearchResponse> {
  const query = new URLSearchParams({ q: params.q.trim() })
  if (params.account_id?.trim()) query.set('account_id', params.account_id.trim())
  if (params.provider_chat_id?.trim()) query.set('provider_chat_id', params.provider_chat_id.trim())
  if (params.limit != null) query.set('limit', String(Math.trunc(params.limit)))
  return ApiClient.instance.get<TelegramMessageSearchResponse>(
    `/api/v1/communications/search/messages?${query.toString()}`,
    'Communication message search failed'
  )
}

export async function searchTelegramBusinessMedia(params: {
  q?: string
  account_id?: string
  provider_chat_id?: string
  kind?: string
  limit?: number
}): Promise<TelegramMediaSearchResponse> {
  const query = new URLSearchParams()
  if (params.q?.trim()) query.set('q', params.q.trim())
  if (params.account_id?.trim()) query.set('account_id', params.account_id.trim())
  if (params.provider_chat_id?.trim()) query.set('provider_chat_id', params.provider_chat_id.trim())
  if (params.kind?.trim()) query.set('kind', params.kind.trim())
  if (params.limit != null) query.set('limit', String(Math.trunc(params.limit)))
  return ApiClient.instance.get<TelegramMediaSearchResponse>(
    `/api/v1/communications/search/media?${query.toString()}`,
    'Communication media search failed'
  )
}

export async function fetchTelegramBusinessPinnedMessages(params: {
  telegram_chat_id: string
  limit?: number
}): Promise<TelegramMessageListResponse> {
  const query = new URLSearchParams()
  if (params.limit != null) query.set('limit', String(Math.trunc(params.limit)))
  return ApiClient.instance.get<TelegramMessageListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.telegram_chat_id)}/pinned-messages?${query.toString()}`,
    'Communication pinned messages request failed'
  )
}

export async function sendTelegramBusinessMessage(request: {
  account_id: string
  provider_chat_id: string
  text: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(request.provider_chat_id)}/messages`,
    { account_id: request.account_id, text: request.text },
    'Communication message send failed'
  )
}

export async function replyToTelegramBusinessMessage(params: {
  message_id: string
  text: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/reply`,
    { text: params.text },
    'Communication reply failed'
  )
}

export async function forwardTelegramBusinessMessage(params: {
  message_id: string
  provider_chat_id: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/forward`,
    { conversation_id: params.provider_chat_id },
    'Communication forward failed'
  )
}

export async function editTelegramBusinessMessage(params: {
  message_id: string
  command_id?: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  new_text: string
}): Promise<TelegramLifecycleResponse> {
  return ApiClient.instance.patch<TelegramLifecycleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}`,
    params,
    'Communication message edit failed'
  )
}

export async function deleteTelegramBusinessMessage(params: {
  message_id: string
  command_id?: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason_class?: string
  actor_class?: string
  is_provider_delete?: boolean
}): Promise<TelegramLifecycleResponse> {
  return ApiClient.instance.deleteWithBody<TelegramLifecycleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}`,
    params,
    'Communication message delete failed'
  )
}

export async function restoreTelegramBusinessMessageVisibility(params: {
  message_id: string
  command_id?: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason?: string
}): Promise<TelegramLifecycleResponse> {
  return ApiClient.instance.post<TelegramLifecycleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/restore-visibility`,
    params,
    'Communication message restore failed'
  )
}

export async function pinTelegramBusinessMessage(params: {
  message_id: string
}): Promise<MessagePinToggleResponse> {
  return ApiClient.instance.post<MessagePinToggleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/pin`,
    {},
    'Communication message pin failed'
  )
}

export async function markTelegramBusinessMessageRead(params: {
  message_id: string
  account_id: string
  provider_chat_id: string
}): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/mark-read`,
    { account_id: params.account_id, provider_chat_id: params.provider_chat_id },
    'Communication message mark read failed'
  )
}

export async function fetchTelegramBusinessMessageVersions(messageId: string): Promise<TelegramMessageVersionListResponse> {
  return ApiClient.instance.get<TelegramMessageVersionListResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/versions`,
    'Communication message versions failed'
  )
}

export async function fetchTelegramBusinessMessageTombstones(messageId: string): Promise<TelegramMessageTombstoneListResponse> {
  return ApiClient.instance.get<TelegramMessageTombstoneListResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/tombstones`,
    'Communication message tombstones failed'
  )
}

export async function fetchTelegramBusinessReplyChain(messageId: string): Promise<TelegramReplyChainResponse> {
  return ApiClient.instance.get<TelegramReplyChainResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reply-chain`,
    'Communication reply chain failed'
  )
}

export async function fetchTelegramBusinessForwardChain(messageId: string): Promise<TelegramForwardChainResponse> {
  return ApiClient.instance.get<TelegramForwardChainResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/forward-chain`,
    'Communication forward chain failed'
  )
}

export async function fetchTelegramBusinessReactions(messageId: string): Promise<TelegramReactionListResponse> {
  return ApiClient.instance.get<TelegramReactionListResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reactions`,
    'Communication reactions failed'
  )
}

export async function addTelegramBusinessReaction(messageId: string, request: TelegramReactionRequest): Promise<TelegramReactionResponse> {
  return ApiClient.instance.post<TelegramReactionResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reactions`,
    request,
    'Communication reaction add failed'
  )
}

export async function removeTelegramBusinessReaction(messageId: string, request: TelegramReactionRequest): Promise<TelegramReactionResponse> {
  const params = new URLSearchParams({
    account_id: request.account_id,
    provider_chat_id: request.provider_chat_id,
    provider_message_id: request.provider_message_id,
    reaction_emoji: request.reaction_emoji,
  })
  if (request.sender_id) params.set('sender_id', request.sender_id)
  if (request.sender_display_name) params.set('sender_display_name', request.sender_display_name)
  if (request.command_id) params.set('command_id', request.command_id)
  return ApiClient.instance.delete<TelegramReactionResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reactions?${params.toString()}`,
    'Communication reaction remove failed'
  )
}

export async function fetchTelegramBusinessRawEvidence(messageId: string): Promise<TelegramRawMessageResponse> {
  return ApiClient.instance.get<TelegramRawMessageResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/raw-evidence`,
    'Communication raw evidence failed'
  )
}

export async function previewTelegramBusinessAttachment(attachmentId: string): Promise<AttachmentPreviewResponse> {
  return ApiClient.instance.get<AttachmentPreviewResponse>(
    `/api/v1/communications/attachments/${encodeURIComponent(attachmentId)}/preview`,
    'Communication attachment preview failed'
  )
}

export async function fetchTelegramBusinessTopics(conversationId: string, limit = 100): Promise<TelegramTopicListResponse> {
  return ApiClient.instance.get<TelegramTopicListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}/topics?limit=${limit}`,
    'Communication topics failed'
  )
}

export async function fetchTelegramBusinessTopicMessages(topicId: string, limit = 50): Promise<TelegramMessageListResponse> {
  return ApiClient.instance.get<TelegramMessageListResponse>(
    `/api/v1/communications/topics/${encodeURIComponent(topicId)}/messages?limit=${limit}`,
    'Communication topic messages failed'
  )
}

export async function searchTelegramBusinessTopics(
  conversationId: string,
  q: string,
  limit = 50
): Promise<TelegramTopicListResponse> {
  const params = new URLSearchParams({
    q: q.trim(),
    telegram_chat_id: conversationId.trim(),
    limit: String(Math.trunc(limit)),
  })
  return ApiClient.instance.get<TelegramTopicListResponse>(
    `/api/v1/communications/topics/search?${params.toString()}`,
    'Communication topic search failed'
  )
}

export async function createTelegramBusinessTopic(
  conversationId: string,
  request: TelegramTopicCreateRequest
): Promise<TelegramTopicLifecycleResponse> {
  return ApiClient.instance.post<TelegramTopicLifecycleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}/topics`,
    request,
    'Communication topic create failed'
  )
}

export async function closeTelegramBusinessTopic(
  topicId: string,
  request: TelegramTopicCloseRequest
): Promise<TelegramTopicLifecycleResponse> {
  return ApiClient.instance.post<TelegramTopicLifecycleResponse>(
    `/api/v1/communications/topics/${encodeURIComponent(topicId)}/close`,
    request,
    'Communication topic lifecycle update failed'
  )
}

function communicationMessageToTelegramMessage(message: CommunicationMessageSummary): TelegramMessage {
  return {
    message_id: message.message_id,
    raw_record_id: message.raw_record_id,
    account_id: message.account_id,
    provider_message_id: message.provider_record_id,
    provider_chat_id: message.conversation_id,
    chat_title: message.subject,
    sender: message.sender,
    sender_display_name: message.sender_display_name,
    text: message.body_text_preview,
    occurred_at: message.occurred_at,
    projected_at: message.projected_at,
    channel_kind: telegramChannelKind(message.channel_kind),
    delivery_state: message.delivery_state,
    metadata: message.message_metadata,
  }
}

function telegramChannelKind(channelKind: string): TelegramProviderKind {
  return channelKind.trim() === 'telegram_bot' ? 'telegram_bot' : 'telegram_user'
}
