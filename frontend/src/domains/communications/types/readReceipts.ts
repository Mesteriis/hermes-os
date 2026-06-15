export type MailReadReceiptKind = 'read'

export type MailReadReceipt = {
  receipt_id: string
  account_id: string
  outbox_id: string | null
  provider_message_id: string
  recipient: string
  receipt_kind: MailReadReceiptKind
  read_at: string
  source_kind: string
  provider_record_id: string | null
  raw_record_id: string | null
  metadata: Record<string, unknown>
  created_at: string
}

export type NewMailReadReceipt = {
  receipt_id?: string
  account_id: string
  provider_message_id: string
  recipient: string
  read_at: string
  source_kind?: string
  provider_record_id?: string
  raw_record_id?: string
  metadata?: Record<string, unknown>
}
