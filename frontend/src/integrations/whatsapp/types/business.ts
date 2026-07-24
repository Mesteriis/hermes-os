export type WhatsappBusinessConversation = {
  conversation_id?: string
  account_id: string
  provider_chat_id: string
  title: string
  last_message_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type WhatsappBusinessConversationListResponse = {
  items: WhatsappBusinessConversation[]
}

export type WhatsappBusinessConversationDetailResponse = {
  item: WhatsappBusinessConversation
}

export type WhatsappBusinessMessage = {
  message_id: string
  raw_record_id: string
  account_id: string
  provider_record_id: string
  subject: string
  sender: string
  sender_display_name: string | null
  body_text_preview: string
  occurred_at: string | null
  projected_at: string
  conversation_id: string | null
  delivery_state: string
  message_metadata: Record<string, unknown>
}

export type WhatsappBusinessMessagesResponse = {
  items: WhatsappBusinessMessage[]
  next_cursor: string | null
  has_more: boolean
}

export type WhatsappBusinessProviderMessage = {
  message_id: string
  raw_record_id: string
  account_id: string
  provider_record_id?: string
  provider_message_id?: string
  provider_chat_id?: string | null
  conversation_id?: string | null
  chat_title?: string
  sender: string
  sender_display_name: string | null
  text?: string
  body_text_preview?: string
  occurred_at: string | null
  projected_at: string
  channel_kind: 'whatsapp_web'
  delivery_state: string
  metadata?: Record<string, unknown>
}

export type WhatsappBusinessProviderMessageListResponse = {
  items: WhatsappBusinessProviderMessage[]
  next_cursor?: string | null
  has_more?: boolean
}

export type WhatsappBusinessMessageCommandResponse = {
  message_id: string
  raw_record_id: string
  conversation_id: string
  provider_chat_id: string
  provider_message_id: string | null
  channel_kind: string
  status: string
  command_id: string
  provider: string
}

export type WhatsappBusinessMessagePinResponse = {
  message_id: string
  pinned: boolean
}

export type WhatsappBusinessConversationActionResponse = {
  conversation_id: string
  provider_chat_id: string
  channel_kind: string
  action: string
  status: string
  command_id: string
  provider: string
  active: boolean
}
