import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  CommunicationProviderConversationDetailResponse,
  CommunicationProviderConversationListResponse,
  CommunicationProviderMessageListResponse,
  CommunicationProviderMessageSearchResponse,
  CommunicationProviderTopicListResponse,
  CommunicationRawEvidenceResponse,
} from '../types/providerChannels'

export async function fetchCommunicationConversations(
  accountId?: string,
  limit = 50
): Promise<CommunicationProviderConversationListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  return ApiClient.instance.get<CommunicationProviderConversationListResponse>(
    `/api/v1/communications/conversations?${params.toString()}`,
    'Communication conversations request failed'
  )
}

export async function searchCommunicationConversations(params: {
  q: string
  account_id?: string
  limit?: number
}): Promise<CommunicationProviderConversationListResponse & { query: string; total: number }> {
  const query = new URLSearchParams({ q: params.q.trim() })
  if (params.account_id?.trim()) {
    query.set('account_id', params.account_id.trim())
  }
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<CommunicationProviderConversationListResponse & { query: string; total: number }>(
    `/api/v1/communications/conversations/search?${query.toString()}`,
    'Communication conversation search failed'
  )
}

export async function fetchCommunicationConversationDetail(
  conversationId: string
): Promise<CommunicationProviderConversationDetailResponse> {
  return ApiClient.instance.get<CommunicationProviderConversationDetailResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}`,
    'Communication conversation detail request failed'
  )
}

export async function fetchCommunicationConversationMembers(
  conversationId: string,
  limit = 50,
  query?: string,
  role?: string,
  cursor?: string
) {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  if (query?.trim()) params.set('query', query.trim())
  if (role?.trim()) params.set('role', role.trim())
  if (cursor?.trim()) params.set('cursor', cursor.trim())
  return ApiClient.instance.get(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}/members?${params.toString()}`,
    'Communication conversation members request failed'
  )
}

export async function fetchCommunicationMessages(params: {
  accountId?: string
  conversationId?: string
  channelKind?: string
  limit?: number
} = {}): Promise<CommunicationProviderMessageListResponse> {
  const query = new URLSearchParams({ limit: String(Math.trunc(params.limit ?? 50)) })
  if (params.accountId?.trim()) {
    query.set('account_id', params.accountId.trim())
  }
  if (params.conversationId?.trim()) {
    query.set('conversation_id', params.conversationId.trim())
  }
  if (params.channelKind?.trim()) {
    query.set('channel_kind', params.channelKind.trim())
  }
  return ApiClient.instance.get<CommunicationProviderMessageListResponse>(
    `/api/v1/communications/messages?${query.toString()}`,
    'Communication messages request failed'
  )
}

export async function searchCommunicationMessages(params: {
  q: string
  account_id?: string
  provider_chat_id?: string
  limit?: number
}): Promise<CommunicationProviderMessageSearchResponse> {
  const query = new URLSearchParams({ q: params.q.trim() })
  if (params.account_id?.trim()) {
    query.set('account_id', params.account_id.trim())
  }
  if (params.provider_chat_id?.trim()) {
    query.set('provider_chat_id', params.provider_chat_id.trim())
  }
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<CommunicationProviderMessageSearchResponse>(
    `/api/v1/communications/search/messages?${query.toString()}`,
    'Communication message search failed'
  )
}

export async function fetchCommunicationPinnedMessages(params: {
  conversationId: string
  limit?: number
}): Promise<CommunicationProviderMessageListResponse> {
  const query = new URLSearchParams()
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<CommunicationProviderMessageListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(params.conversationId)}/pinned-messages?${query.toString()}`,
    'Communication pinned messages request failed'
  )
}

export async function fetchCommunicationRawEvidence(
  messageId: string
): Promise<CommunicationRawEvidenceResponse> {
  return ApiClient.instance.get<CommunicationRawEvidenceResponse>(
    `/api/v1/communications/messages/${encodeURIComponent(messageId)}/raw-evidence`,
    'Communication raw evidence request failed'
  )
}

export async function fetchCommunicationTopics(
  conversationId: string,
  limit = 100
): Promise<CommunicationProviderTopicListResponse> {
  return ApiClient.instance.get<CommunicationProviderTopicListResponse>(
    `/api/v1/communications/conversations/${encodeURIComponent(conversationId)}/topics?limit=${limit}`,
    'Communication topics request failed'
  )
}

export async function fetchCommunicationTopicMessages(
  topicId: string,
  limit = 50
): Promise<CommunicationProviderMessageListResponse> {
  return ApiClient.instance.get<CommunicationProviderMessageListResponse>(
    `/api/v1/communications/topics/${encodeURIComponent(topicId)}/messages?limit=${limit}`,
    'Communication topic messages request failed'
  )
}

export async function searchCommunicationTopics(
  conversationId: string,
  q: string,
  limit = 50
): Promise<CommunicationProviderTopicListResponse> {
  const params = new URLSearchParams({
    q: q.trim(),
    telegram_chat_id: conversationId.trim(),
    limit: String(limit),
  })
  return ApiClient.instance.get<CommunicationProviderTopicListResponse>(
    `/api/v1/communications/topics/search?${params}`,
    'Communication topic search failed'
  )
}
