import { describe, expect, it } from 'vitest'
import type { AttachmentSearchResult } from '../types/attachments'
import {
  attachmentSearchTableColumns,
  attachmentSearchTableRowId
} from './attachmentSearchTable'

function result(overrides: Partial<AttachmentSearchResult> = {}): AttachmentSearchResult {
  return {
    attachment_id: 'attachment-1',
    message_id: 'msg-1',
    raw_record_id: 'raw-1',
    account_id: 'account-1',
    message_subject: 'Invoice',
    sender: 'billing@example.com',
    occurred_at: '2026-06-15T00:00:00Z',
    blob_id: 'blob-1',
    provider_attachment_id: 'provider-1',
    filename: 'invoice.pdf',
    content_type: 'application/pdf',
    size_bytes: 2048,
    sha256: 'hash',
    disposition: 'attachment',
    scan_status: 'not_scanned',
    scan_engine: null,
    scan_checked_at: null,
    scan_summary: null,
    storage_kind: 'local',
    storage_path: 'mail/blob',
    extracted_text_match: false,
    created_at: '2026-06-15T00:00:00Z',
    updated_at: '2026-06-15T00:00:00Z',
    ...overrides
  }
}

describe('attachment search table helpers', () => {
  it('defines stable TanStack Table columns for attachment search results', () => {
    expect(attachmentSearchTableColumns.map((column) => column.id)).toEqual([
      'filename',
      'message_subject',
      'sender',
      'size',
      'scan_status'
    ])
  })

  it('uses attachment ids as stable search table row ids', () => {
    expect(attachmentSearchTableRowId(result({ attachment_id: 'attachment-42' }))).toBe('attachment-42')
  })
})
