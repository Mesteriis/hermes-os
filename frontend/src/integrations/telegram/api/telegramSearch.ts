import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramChatSearchResponse,
  TelegramMediaSearchResponse,
  TelegramMessageListResponse,
  TelegramMessageSearchResponse,
} from '../types/telegram'
import {
  fetchTelegramBusinessPinnedMessages,
  searchTelegramBusinessChats,
  searchTelegramBusinessMedia,
  searchTelegramBusinessMessages,
} from '../../../shared/communications/telegramBusinessApi'

export type TelegramProviderSearchCommandResponse = {
  account_id: string
  provider_chat_id?: string | null
  query: string
  limit: number
  status: string
  error?: string | null
}

export async function searchTelegramChats(params: {
  q: string
  account_id?: string
  limit?: number
}): Promise<TelegramChatSearchResponse> {
  return searchTelegramBusinessChats(params)
}

export async function searchTelegramMessages(params: {
  q: string
  account_id?: string
  provider_chat_id?: string
  limit?: number
}): Promise<TelegramMessageSearchResponse> {
  return searchTelegramBusinessMessages(params)
}

export async function searchTelegramProviderMessages(params: {
  q: string
  account_id: string
  provider_chat_id?: string
  limit?: number
}): Promise<TelegramProviderSearchCommandResponse> {
  const body = {
    q: params.q.trim(),
    account_id: params.account_id.trim(),
    provider_chat_id: params.provider_chat_id?.trim(),
    limit: params.limit,
  }
  return ApiClient.instance.post<TelegramProviderSearchCommandResponse>(
    '/api/v1/integrations/telegram/provider-search',
    body,
    'Telegram provider search trigger failed'
  )
}

export async function searchTelegramMedia(params: {
  q?: string
  account_id?: string
  provider_chat_id?: string
  kind?: string
  limit?: number
}): Promise<TelegramMediaSearchResponse> {
  return searchTelegramBusinessMedia(params)
}

export async function fetchTelegramPinnedMessages(params: {
  telegram_chat_id: string
  limit?: number
}): Promise<TelegramMessageListResponse> {
  return fetchTelegramBusinessPinnedMessages(params)
}
