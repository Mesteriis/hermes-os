export type CommunicationDraftAttachmentRef = {
  attachment_id: string
  filename: string | null
  content_type: string
  size_bytes: number
  scan_status: string
  scan_engine: string | null
  scan_checked_at: string | null
  scan_summary: string | null
}

export type CommunicationDraft = {
  draft_id: string
  account_id: string
  persona_id: string | null
  to_recipients: string[]
  cc_recipients: string[]
  bcc_recipients: string[]
  subject: string
  body_text: string
  body_html: string | null
  in_reply_to: string | null
  references: string[]
  attachment_ids: string[]
  attachments: CommunicationDraftAttachmentRef[]
  status: 'draft' | 'scheduled' | 'sending' | 'sent' | 'failed'
  scheduled_send_at: string | null
  send_attempts: number
  last_error: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type DraftListResponse = {
  items: CommunicationDraft[]
  next_cursor: string | null
  has_more: boolean
}

export type DraftUpsertRequest = {
  draft_id: string
  account_id: string
  persona_id?: string | null
  to_recipients: string[]
  cc_recipients?: string[]
  bcc_recipients?: string[]
  subject: string
  body_text: string
  body_html?: string | null
  in_reply_to?: string | null
  references?: string[]
  attachment_ids?: string[]
  status?: 'draft' | 'scheduled' | 'sending' | 'sent' | 'failed'
  scheduled_send_at?: string | null
  metadata?: Record<string, unknown>
}

export type DraftDeleteResponse = { deleted: boolean }

export type SendCommunicationRequest = {
  account_id: string
  to: string[]
  cc?: string[]
  bcc?: string[]
  subject: string
  body_text: string
  body_html?: string | null
  in_reply_to?: string | null
  references?: string[]
  draft_id?: string | null
  scheduled_send_at?: string | null
  undo_send_seconds?: number | null
  confirmed_provider_write: boolean
}

export type SendCommunicationResponse = {
  message_id: string
  outbox_id: string | null
  accepted: string[]
  accepted_recipients: string[]
  transport: 'smtp' | 'local' | 'outbox' | string
  status: 'sent' | 'queued' | 'scheduled' | string
  scheduled_send_at: string | null
  undo_deadline_at: string | null
  failure_reason: string | null
}

export type RedirectMessageRequest = {
  to: string[]
  cc?: string[]
  bcc?: string[]
  confirmed_provider_write?: boolean
}

export type CommunicationOutboxStatus = 'queued' | 'scheduled' | 'sending' | 'sent' | 'failed' | 'canceled'

export type CommunicationOutboxItem = {
  outbox_id: string
  account_id: string
  draft_id: string | null
  to_recipients: string[]
  cc_recipients: string[]
  bcc_recipients: string[]
  subject: string
  body_text: string
  body_html: string | null
  status: CommunicationOutboxStatus
  scheduled_send_at: string | null
  undo_deadline_at: string | null
  send_attempts: number
  claimed_at: string | null
  sent_at: string | null
  provider_message_id: string | null
  last_error: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type OutboxListResponse = {
  items: CommunicationOutboxItem[]
  next_cursor: string | null
  has_more: boolean
}

export type BulkMessageAction =
  | 'mark_read'
  | 'mark_unread'
  | 'archive'
  | 'trash'
  | 'restore'
  | 'pin'
  | 'unpin'
  | 'important'
  | 'not_important'
  | 'star'
  | 'unstar'
  | 'add_label'
  | 'remove_label'
  | 'snooze'

export type BulkMessageActionRequest = {
  action: BulkMessageAction
  message_ids: string[]
  label?: string
  snooze_until?: string
}

export type BulkMessageActionResponse = {
  action: BulkMessageAction
  requested_count: number
  matched_count: number
  updated_count: number
  not_found: string[]
}
