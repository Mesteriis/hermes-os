import { ApiClient } from '../../../platform/api'
import type { TelegramRawMessageResponse } from '../types/telegramRawEvidence'

export async function fetchTelegramRawMessageEvidence(
  messageId: string
): Promise<TelegramRawMessageResponse> {
  return ApiClient.instance.get<TelegramRawMessageResponse>(
    `/api/v1/integrations/telegram/provider-messages/${encodeURIComponent(messageId)}/raw-evidence`,
    'Telegram raw message evidence request failed'
  )
}
