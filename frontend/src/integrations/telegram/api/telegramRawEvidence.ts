import type { TelegramRawMessageResponse } from '../types/telegramRawEvidence'

export async function fetchTelegramRawMessageEvidence(
  messageId: string
): Promise<TelegramRawMessageResponse> {
  void messageId
  return Promise.reject(
    new Error(
      'Telegram raw evidence moved to frontend/src/domains/communications/api/providerChannels; integration clients own runtime/control only'
    )
  )
}
