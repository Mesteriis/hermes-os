export type TelegramRawMessageRecord = {
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

export type TelegramRawMessageResponse = {
  raw_record: TelegramRawMessageRecord
}
