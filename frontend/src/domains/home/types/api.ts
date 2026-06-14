export type CommunicationMessageSummary = {
  message_id: string
  raw_record_id: string
  account_id: string
  provider_record_id: string
  subject: string
  sender: string
  recipients: string[]
  body_text_preview: string
  occurred_at: string | null
  projected_at: string
  channel_kind: string
  conversation_id: string | null
  sender_display_name: string | null
  delivery_state: string
  message_metadata: Record<string, unknown>
  attachment_count: number
  local_state: LocalMessageState
  local_state_changed_at: string | null
}

export type LocalMessageState = 'active' | 'trash' | 'all'

export type CommunicationMessagesResponse = {
  items: CommunicationMessageSummary[]
}

export type MailboxHealth = {
  total_messages: number
  unread: number
  needs_action: number
  waiting: number
  done: number
  archived: number
  spam: number
  important: number
  with_attachments: number
  average_importance: number
  oldest_message_days: number | null
}
