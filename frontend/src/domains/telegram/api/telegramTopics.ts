import { ApiClient } from '../../../platform/api'
import type { TelegramTopicListResponse } from '../types/telegramTopics'

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
    `/api/v1/telegram/topics/search?${params}`,
    'Telegram topic search failed'
  )
}
