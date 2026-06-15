export type AttachmentScanStatus = 'not_scanned' | 'clean' | 'suspicious' | 'malicious' | 'failed'

export type AttachmentSearchRequest = {
  account_id?: string
  q?: string
  content_type?: string
  scan_status?: AttachmentScanStatus
  cursor?: string | null
  limit?: number
}

export type AttachmentSearchResult = {
  attachment_id: string
  message_id: string
  raw_record_id: string
  account_id: string
  message_subject: string
  sender: string
  occurred_at: string | null
  blob_id: string
  provider_attachment_id: string
  filename: string | null
  content_type: string
  size_bytes: number
  sha256: string
  disposition: 'attachment' | 'inline' | 'unknown'
  scan_status: AttachmentScanStatus
  scan_engine: string | null
  scan_checked_at: string | null
  scan_summary: string | null
  storage_kind: string
  storage_path: string
  created_at: string
  updated_at: string
}

export type AttachmentSearchResponse = {
  items: AttachmentSearchResult[]
  next_cursor: string | null
  has_more: boolean
}

export type ArchiveInspectionEntry = {
  name: string
  normalized_path: string
  compressed_size: number
  uncompressed_size: number
  is_dir: boolean
  is_nested_archive: boolean
}

export type ArchiveInspectionReport = {
  archive_kind: 'zip'
  entry_count: number
  total_uncompressed_bytes: number
  has_nested_archive: boolean
  entries: ArchiveInspectionEntry[]
}

export type AttachmentArchiveInspectionResponse = {
  attachment_id: string
  message_id: string
  filename: string | null
  content_type: string
  scan_status: AttachmentScanStatus
  report: ArchiveInspectionReport
}

export type AttachmentPreviewResponse = {
  attachment_id: string
  message_id: string
  filename: string | null
  content_type: string
  scan_status: AttachmentScanStatus
  preview_kind: 'text' | 'image'
  text: string
  data_url: string | null
  truncated: boolean
  byte_count: number
  max_preview_bytes: number
}

export type AttachmentTranslationRequest = {
  target_language: string
  source_text: string
}

export type AttachmentTranslationResponse = {
  attachment_id: string
  message_id: string
  filename: string | null
  original_language: string
  confidence: number
  translated: boolean
  text: string | null
  target: string
  model: string | null
  reason: string | null
  source: 'caller_provided_extracted_text'
}
