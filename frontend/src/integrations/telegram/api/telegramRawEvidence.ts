import type { TelegramRawMessageResponse } from '../types/telegramRawEvidence'
import { fetchTelegramBusinessRawEvidence } from '../../../shared/communications/telegramBusinessApi'

export async function fetchTelegramRawMessageEvidence(
  messageId: string
): Promise<TelegramRawMessageResponse> {
  return fetchTelegramBusinessRawEvidence(messageId)
}
