import { ApiClient } from '../../../platform/api'
import type {
  TelegramTopicCloseRequest,
  TelegramTopicCreateRequest,
  TelegramTopicLifecycleResponse,
  TelegramTopicListResponse,
} from '../types/telegramTopics'

export async function fetchTelegramTopicSearch(
  telegramChatId: string,
  q: string,
  limit = 50
): Promise<TelegramTopicListResponse> {
  const params = new URLSearchParams({
    q: q.trim(),
    telegram_chat_id: telegramChatId.trim(),
    limit: String(limit),
  })
  return ApiClient.instance.get<TelegramTopicListResponse>(
    `/api/v1/communications/telegram/topics/search?${params}`,
    'Telegram topic search failed'
  )
}

export async function createTelegramTopic(
  telegramChatId: string,
  request: TelegramTopicCreateRequest
): Promise<TelegramTopicLifecycleResponse> {
  return ApiClient.instance.post<TelegramTopicLifecycleResponse>(
    `/api/v1/communications/telegram/chats/${encodeURIComponent(telegramChatId)}/topics`,
    request,
    'Telegram topic create failed'
  )
}

export async function toggleTelegramTopicClosed(
  topicId: string,
  request: TelegramTopicCloseRequest
): Promise<TelegramTopicLifecycleResponse> {
  return ApiClient.instance.post<TelegramTopicLifecycleResponse>(
    `/api/v1/communications/telegram/topics/${encodeURIComponent(topicId)}/close`,
    request,
    'Telegram topic close failed'
  )
}
