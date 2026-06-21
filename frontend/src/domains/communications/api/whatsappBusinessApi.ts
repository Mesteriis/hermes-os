import { ApiClient } from '../../../platform/api/ApiClient'
import type { WhatsappWebMessageListResponse } from '../../../shared/communications/types/whatsapp'

export async function fetchWhatsappWebBusinessMessages(
  accountId?: string,
  providerChatId?: string,
  limit = 50
): Promise<WhatsappWebMessageListResponse> {
  const params = new URLSearchParams({
    limit: String(Math.trunc(limit)),
    channel_kind: 'whatsapp_web',
  })
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  if (providerChatId?.trim()) {
    params.set('conversation_id', providerChatId.trim())
  }
  return ApiClient.instance.get<WhatsappWebMessageListResponse>(
    `/api/v1/communications/messages?${params.toString()}`,
    'Communication WhatsApp messages request failed'
  )
}
