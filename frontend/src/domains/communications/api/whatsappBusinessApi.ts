import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  WhatsappWebMessage,
  WhatsappWebMessageListResponse,
} from '../../../shared/communications/types/whatsapp'
import type {
  CommunicationMessageSummary,
  CommunicationMessagesResponse,
} from '../types/communications'

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
  const response = await ApiClient.instance.get<CommunicationMessagesResponse>(
    `/api/v1/communications/messages?${params.toString()}`,
    'Communication WhatsApp messages request failed'
  )
  return { items: response.items.map(communicationMessageToWhatsappWebMessage) }
}

function communicationMessageToWhatsappWebMessage(message: CommunicationMessageSummary): WhatsappWebMessage {
  return {
    message_id: message.message_id,
    raw_record_id: message.raw_record_id,
    account_id: message.account_id,
    provider_message_id: message.provider_record_id,
    provider_chat_id: message.conversation_id,
    chat_title: message.subject,
    sender: message.sender,
    sender_display_name: message.sender_display_name,
    text: message.body_text_preview,
    occurred_at: message.occurred_at,
    projected_at: message.projected_at,
    channel_kind: 'whatsapp_web',
    delivery_state: message.delivery_state,
    metadata: message.message_metadata,
  }
}
