export type CommunicationProviderChannelKind = 'telegram_user' | 'telegram_bot' | 'whatsapp_web' | string

export type CommunicationProviderConversation = {
  telegram_chat_id?: string
  conversation_id?: string
  account_id: string
  provider_chat_id: string
  chat_kind?: string
  title: string
  username?: string | null
  sync_state?: string
  last_message_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type CommunicationProviderConversationListResponse = {
  items: CommunicationProviderConversation[]
}

export type CommunicationProviderConversationDetailResponse = {
  item: CommunicationProviderConversation
}

export type CommunicationProviderMessage = {
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
  channel_kind: CommunicationProviderChannelKind
  delivery_state: string
  metadata?: Record<string, unknown>
  message_metadata?: Record<string, unknown>
}

export type CommunicationProviderMessageListResponse = {
  items: CommunicationProviderMessage[]
  next_cursor?: string | null
  has_more?: boolean
}

export type CommunicationProviderMessageSearchResponse = {
  query: string
  items: CommunicationProviderMessage[]
  total: number
}

export type CommunicationProviderTopic = {
  topic_id: string
  telegram_chat_id: string
  account_id: string
  provider_topic_id: number
  provider_chat_id: string
  title: string
  icon_emoji: string | null
  is_pinned: boolean
  is_closed: boolean
  unread_count: number
  last_message_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type CommunicationProviderTopicListResponse = {
  telegram_chat_id: string
  items: CommunicationProviderTopic[]
}

export type CommunicationRawEvidenceResponse = {
  raw_record: {
    raw_record_id: string
    provider_kind: string
    provider_account_id: string
    provider_message_id: string
    source_uri: string | null
    occurred_at: string
    ingested_at: string
    payload: Record<string, unknown>
    headers: Record<string, string>
    provenance: Record<string, unknown>
  }
}
