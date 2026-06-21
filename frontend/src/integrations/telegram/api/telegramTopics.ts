import { ApiClient } from '../../../platform/api'
import type {
  TelegramTopicCloseRequest,
  TelegramTopicCreateRequest,
  TelegramTopicLifecycleResponse,
  TelegramTopicListResponse,
} from '../types/telegramTopics'

function communicationBusinessApiMoved<T>(operation: string): Promise<T> {
  return Promise.reject(
    new Error(
      `${operation} moved to frontend/src/domains/communications/api/providerChannels; integration clients own runtime/control only`
    )
  )
}

export async function fetchTelegramTopicSearch(
  telegramChatId: string,
  q: string,
  limit = 50
): Promise<TelegramTopicListResponse> {
  void telegramChatId
  void q
  void limit
  return communicationBusinessApiMoved('Telegram topic search')
}

export async function createTelegramTopic(
  telegramChatId: string,
  request: TelegramTopicCreateRequest
): Promise<TelegramTopicLifecycleResponse> {
  void telegramChatId
  void request
  return communicationBusinessApiMoved('Telegram topic create')
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
