import { ApiClient } from '../../../platform/api'
import type { TelegramRawMessageResponse } from '../types/telegramRawEvidence'

export async function fetchTelegramRawMessageEvidence(
  messageId: string
): Promise<TelegramRawMessageResponse> {
  return ApiClient.instance.get<TelegramRawMessageResponse>(
    `/api/v1/communications/provider-messages/${encodeURIComponent(messageId)}/raw`,
    'Telegram raw message evidence request failed'
  )
}
