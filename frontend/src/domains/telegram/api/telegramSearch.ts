import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramChatSearchResponse,
  TelegramMediaSearchResponse,
  TelegramMessageListResponse,
  TelegramMessageSearchResponse,
} from '../types/telegram'

export async function searchTelegramChats(params: {
  q: string
  account_id?: string
  limit?: number
}): Promise<TelegramChatSearchResponse> {
  const query = new URLSearchParams({ q: params.q.trim() })
  if (params.account_id?.trim()) {
    query.set('account_id', params.account_id.trim())
  }
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<TelegramChatSearchResponse>(
    `/api/v1/telegram/chats/search?${query.toString()}`,
    'Telegram dialog search failed'
  )
}

export async function searchTelegramMessages(params: {
  q: string
  account_id?: string
  provider_chat_id?: string
  limit?: number
}): Promise<TelegramMessageSearchResponse> {
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
  return ApiClient.instance.get<TelegramMessageSearchResponse>(
    `/api/v1/telegram/search/messages?${query.toString()}`,
    'Telegram message search failed'
  )
}

export async function searchTelegramProviderMessages(params: {
  q: string
  account_id: string
  provider_chat_id?: string
  limit?: number
}): Promise<TelegramMessageSearchResponse> {
  const body = {
    q: params.q.trim(),
    account_id: params.account_id.trim(),
    provider_chat_id: params.provider_chat_id?.trim(),
    limit: params.limit,
  }
  return ApiClient.instance.post<TelegramMessageSearchResponse>(
    '/api/v1/telegram/search/provider',
    body,
    'Telegram provider message search failed'
  )
}

export async function searchTelegramMedia(params: {
  q?: string
  account_id?: string
  provider_chat_id?: string
  kind?: string
  limit?: number
}): Promise<TelegramMediaSearchResponse> {
  const query = new URLSearchParams()
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
  return ApiClient.instance.get<TelegramMediaSearchResponse>(
    `/api/v1/telegram/search/media?${query.toString()}`,
    'Telegram media search failed'
  )
}

export async function fetchTelegramPinnedMessages(params: {
  telegram_chat_id: string
  limit?: number
}): Promise<TelegramMessageListResponse> {
  const query = new URLSearchParams()
  if (params.limit != null) {
    query.set('limit', String(Math.trunc(params.limit)))
  }
  return ApiClient.instance.get<TelegramMessageListResponse>(
    `/api/v1/telegram/chats/${encodeURIComponent(params.telegram_chat_id)}/pinned-messages?${query.toString()}`,
    'Telegram pinned messages query failed'
  )
}
