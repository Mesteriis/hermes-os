export type TelegramBusinessMessageSummary = {
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
  channel_kind: string
  conversation_id: string | null
  delivery_state: string
  message_metadata: Record<string, unknown>
}

export type TelegramBusinessMessagesResponse = {
  items: TelegramBusinessMessageSummary[]
  next_cursor: string | null
  has_more: boolean
}

export type TelegramBusinessMessageCommandResponse = {
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

export type TelegramBusinessMessagePinResponse = {
  message_id: string
  pinned: boolean
}

export type TelegramBusinessAttachmentPreviewResponse = {
  attachment_id: string
  message_id: string
  filename: string | null
  content_type: string
  scan_status: string
  preview_kind: 'text' | 'image' | 'audio' | 'video' | 'pdf'
  text: string
  data_url: string | null
  truncated: boolean
  byte_count: number
  max_preview_bytes: number
}
