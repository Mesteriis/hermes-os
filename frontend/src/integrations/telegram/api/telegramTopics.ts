import { ApiClient } from '../../../platform/api'
import type {
  TelegramTopicCloseRequest,
  TelegramTopicCreateRequest,
  TelegramTopicLifecycleResponse,
  TelegramTopicListResponse,
} from '../types/telegramTopics'
import {
  createTelegramBusinessTopic,
  searchTelegramBusinessTopics,
} from '../../../shared/communications/telegramBusinessApi'

export async function fetchTelegramTopicSearch(
  telegramChatId: string,
  q: string,
  limit = 50
): Promise<TelegramTopicListResponse> {
  return searchTelegramBusinessTopics(telegramChatId, q, limit)
}

export async function createTelegramTopic(
  telegramChatId: string,
  request: TelegramTopicCreateRequest
): Promise<TelegramTopicLifecycleResponse> {
  return createTelegramBusinessTopic(telegramChatId, request)
}

export async function toggleTelegramTopicClosed(
  topicId: string,
  request: TelegramTopicCloseRequest
): Promise<TelegramTopicLifecycleResponse> {
  return ApiClient.instance.post<TelegramTopicLifecycleResponse>(
    `/api/v1/integrations/telegram/provider-commands/topics/${encodeURIComponent(topicId)}/close`,
    request,
    'Telegram topic close failed'
  )
}
