import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  WhatsappWebMessage,
  WhatsAppLifecycleResponse,
  WhatsappWebMessageListResponse,
  WhatsappWebMessageSearchResponse,
  WhatsappWebMediaSearchResponse,
} from '../../../shared/communications/types/whatsapp'
import type { TelegramChatMemberListResponse } from '../../../shared/communications/types/telegramMembers'
import type {
  CommunicationMessageSummary,
  CommunicationMessagesResponse,
  CommunicationProviderMessageCommandResponse,
  ConversationPinToggleResponse,
  MessagePinToggleResponse,
} from '../types/communications'
import type {
  TelegramReactionListResponse,
  TelegramReactionRequest,
  TelegramReactionResponse,
} from '../../../shared/communications/types/telegram'
import type {
  CommunicationProviderConversation,
  CommunicationProviderConversationDetailResponse,
  CommunicationProviderConversationListResponse,
  CommunicationProviderMessageListResponse,
} from '../types/providerChannels'

export async function fetchWhatsappWebBusinessConversations(
  accountId?: string,
  limit = 50
): Promise<CommunicationProviderConversationListResponse> {
  const params = new URLSearchParams({
    limit: String(Math.trunc(limit)),
    channel_kind: 'whatsapp_web',
  })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  const response = await ApiClient.instance.get<CommunicationProviderConversationListResponse>(
    `/api/v1/communications/conversations?${params.toString()}`,
    'Communication WhatsApp conversations request failed'
  )
  return {
    items: response.items.filter(isWhatsappConversation),
  }
}

export async function fetchWhatsappWebBusinessConversationDetail(
  conversationId: string
): Promise<CommunicationProviderConversationDetailResponse> {
  return ApiClient.instance.get<CommunicationProviderConversationDetailResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}`,
    'Communication WhatsApp conversation detail request failed'
  )
}

export async function fetchWhatsappWebBusinessConversationMembers(
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
    'Communication WhatsApp conversation members request failed'
  )
}

export async function fetchWhatsappWebBusinessMessages(
  accountId?: string,
  providerChatId?: string,
  limit = 50
): Promise<WhatsappWebMessageListResponse> {
  const params = new URLSearchParams({
    limit: String(Math.trunc(limit)),
    channel_kind: 'whatsapp_web',
  })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  if (providerChatId?.trim()) {
    params.set('conversation_id', providerChatId.trim())
  }
  const response = await ApiClient.instance.get<CommunicationMessagesResponse>(
    `/api/v1/communications/messages?${params.toString()}`,
    'Communication WhatsApp messages request failed'
  )
  return { items: response.items.map(communicationMessageToWhatsappWebMessage) }
}

export async function searchWhatsappWebBusinessMessages(params: {
  q: string
  account_id?: string
  provider_chat_id?: string
  limit?: number
}): Promise<WhatsappWebMessageSearchResponse> {
  const query = new URLSearchParams({
    q: params.q.trim(),
    channel_kind: 'whatsapp_web',
  })
  if (params.account_id?.trim()) {
    query.set('account_id', params.account_id.trim())
  }
  if (params.provider_chat_id?.trim()) {
    query.set('provider_chat_id', params.provider_chat_id.trim())
  }
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<WhatsappWebMessageSearchResponse>(
    `/api/v1/communications/search/messages?${query.toString()}`,
    'Communication WhatsApp message search failed'
  )
}

export async function searchWhatsappWebBusinessMedia(params: {
  q?: string
  account_id?: string
  provider_chat_id?: string
  kind?: string
  limit?: number
}): Promise<WhatsappWebMediaSearchResponse> {
  const query = new URLSearchParams({ channel_kind: 'whatsapp_web' })
  if (params.q?.trim()) {
    query.set('q', params.q.trim())
  }
  if (params.account_id?.trim()) {
    query.set('account_id', params.account_id.trim())
  }
  if (params.provider_chat_id?.trim()) {
    query.set('provider_chat_id', params.provider_chat_id.trim())
  }
  if (params.kind?.trim()) {
    query.set('kind', params.kind.trim())
  }
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<WhatsappWebMediaSearchResponse>(
    `/api/v1/communications/search/media?${query.toString()}`,
    'Communication WhatsApp media search failed'
  )
}

export async function fetchWhatsappWebBusinessPinnedMessages(params: {
  conversation_id: string
  limit?: number
}): Promise<CommunicationProviderMessageListResponse> {
  const query = new URLSearchParams()
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<CommunicationProviderMessageListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/pinned-messages?${query.toString()}`,
    'Communication WhatsApp pinned messages request failed'
  )
}

export async function sendWhatsappBusinessMessage(request: {
  account_id: string
  provider_chat_id: string
  text: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(request.provider_chat_id)}/messages`,
    { account_id: request.account_id, text: request.text },
    'Communication WhatsApp message send failed'
  )
}

export async function replyToWhatsappBusinessMessage(params: {
  message_id: string
  text: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/reply`,
    { text: params.text },
    'Communication WhatsApp reply failed'
  )
}

export async function forwardWhatsappBusinessMessage(params: {
  message_id: string
  provider_chat_id: string
}): Promise<CommunicationProviderMessageCommandResponse> {
  return ApiClient.instance.post<CommunicationProviderMessageCommandResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/forward`,
    { conversation_id: params.provider_chat_id },
    'Communication WhatsApp forward failed'
  )
}

export async function editWhatsappBusinessMessage(params: {
  message_id: string
  command_id?: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  new_text: string
}): Promise<WhatsAppLifecycleResponse> {
  return ApiClient.instance.patch<WhatsAppLifecycleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}`,
    params,
    'Communication WhatsApp message edit failed'
  )
}

export async function deleteWhatsappBusinessMessage(params: {
  message_id: string
  command_id?: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason_class?: string
  actor_class?: string
  is_provider_delete?: boolean
}): Promise<WhatsAppLifecycleResponse> {
  return ApiClient.instance.deleteWithBody<WhatsAppLifecycleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}`,
    params,
    'Communication WhatsApp message delete failed'
  )
}

export async function pinWhatsappBusinessMessage(params: {
  message_id: string
}): Promise<MessagePinToggleResponse> {
  return ApiClient.instance.post<MessagePinToggleResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(params.message_id)}/pin`,
    {},
    'Communication WhatsApp message pin failed'
  )
}

export async function pinWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/pin`,
    {},
    'Communication WhatsApp conversation pin failed'
  )
}

export async function unpinWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/unpin`,
    {},
    'Communication WhatsApp conversation unpin failed'
  )
}

export async function archiveWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/archive`,
    {},
    'Communication WhatsApp conversation archive failed'
  )
}

export async function unarchiveWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/unarchive`,
    {},
    'Communication WhatsApp conversation unarchive failed'
  )
}

export async function muteWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/mute`,
    {},
    'Communication WhatsApp conversation mute failed'
  )
}

export async function unmuteWhatsappBusinessConversation(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/unmute`,
    {},
    'Communication WhatsApp conversation unmute failed'
  )
}

export async function markWhatsappBusinessConversationRead(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/read`,
    {},
    'Communication WhatsApp conversation mark-read failed'
  )
}

export async function markWhatsappBusinessConversationUnread(params: {
  conversation_id: string
}): Promise<ConversationPinToggleResponse> {
  return ApiClient.instance.post<ConversationPinToggleResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversation_id)}/unread`,
    {},
    'Communication WhatsApp conversation mark-unread failed'
  )
}

export async function fetchWhatsappBusinessReactions(
  messageId: string
): Promise<TelegramReactionListResponse> {
  return ApiClient.instance.get<TelegramReactionListResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reactions`,
    'Communication WhatsApp reactions failed'
  )
}

export async function addWhatsappBusinessReaction(
  messageId: string,
  request: TelegramReactionRequest
): Promise<TelegramReactionResponse> {
  return ApiClient.instance.post<TelegramReactionResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reactions`,
    request,
    'Communication WhatsApp reaction add failed'
  )
}

export async function removeWhatsappBusinessReaction(
  messageId: string,
  request: TelegramReactionRequest
): Promise<TelegramReactionResponse> {
  const params = new URLSearchParams({
    account_id: request.account_id,
    provider_chat_id: request.provider_chat_id,
    provider_message_id: request.provider_message_id,
    reaction_emoji: request.reaction_emoji,
    sender_id: request.sender_id,
  })
  if (request.sender_display_name) params.set('sender_display_name', request.sender_display_name)
  if (request.command_id) params.set('command_id', request.command_id)
  return ApiClient.instance.delete<TelegramReactionResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/reactions?${params.toString()}`,
    'Communication WhatsApp reaction remove failed'
  )
}

function communicationMessageToWhatsappWebMessage(message: CommunicationMessageSummary): WhatsappWebMessage {
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
    channel_kind: 'whatsapp_web',
    delivery_state: message.delivery_state,
    metadata: message.message_metadata,
  }
}

function isWhatsappConversation(conversation: CommunicationProviderConversation): boolean {
  const channelKind =
    typeof conversation.metadata?.channel_kind === 'string'
      ? conversation.metadata.channel_kind
      : null
  return channelKind === null || channelKind === 'whatsapp_web'
}
