import { describe, expect, it } from 'vitest'
import type { CommunicationAttachment } from '../types/communications'
import {
  attachmentTableColumns,
  attachmentTableRowId,
  formatAttachmentSize,
  isInspectableArchiveAttachment,
  isPreviewableImageAttachment,
  isPreviewableTextAttachment,
  scanStatusClass
} from './attachmentTable'

function attachment(overrides: Partial<CommunicationAttachment> = {}): CommunicationAttachment {
  return {
    attachment_id: 'attachment-1',
    message_id: 'msg-1',
    raw_record_id: 'raw-1',
    blob_id: 'blob-1',
    provider_attachment_id: 'provider-attachment-1',
    filename: 'invoice.pdf',
    content_type: 'application/pdf',
    size_bytes: 2048,
    sha256: 'hash',
    disposition: 'attachment',
    scan_status: 'not_scanned',
    scan_engine: null,
    scan_checked_at: null,
    scan_summary: null,
    scan_metadata: {},
    storage_kind: 'local',
    storage_path: 'mail/blob',
    created_at: '2026-06-14T10:00:00Z',
    updated_at: '2026-06-14T10:00:00Z',
    ...overrides
  }
}

describe('attachment table helpers', () => {
  it('defines stable TanStack Table columns for attachment metadata', () => {
    expect(attachmentTableColumns.map((column) => column.id)).toEqual([
      'filename',
      'content_type',
      'size',
      'scan_status'
    ])
  })

  it('uses attachment ids as stable table row ids', () => {
    expect(attachmentTableRowId(attachment({ attachment_id: 'attachment-42' }))).toBe('attachment-42')
  })

  it('formats sizes and scan status classes consistently', () => {
    expect(formatAttachmentSize(512)).toBe('512 B')
    expect(formatAttachmentSize(2048)).toBe('2.0 KB')
    expect(formatAttachmentSize(3 * 1024 * 1024)).toBe('3.0 MB')
    expect(scanStatusClass('clean')).toBe('att-scan--clean')
    expect(scanStatusClass('suspicious')).toBe('att-scan--suspicious')
    expect(scanStatusClass('malicious')).toBe('att-scan--danger')
    expect(scanStatusClass('not_scanned')).toBe('att-scan--unknown')
  })

  it('recognizes ZIP attachments as inspectable archives', () => {
    expect(isInspectableArchiveAttachment(attachment({
      filename: 'evidence.zip',
      content_type: 'application/octet-stream'
    }))).toBe(true)
    expect(isInspectableArchiveAttachment(attachment({
      filename: 'evidence.bin',
      content_type: 'application/zip'
    }))).toBe(true)
    expect(isInspectableArchiveAttachment(attachment({
      filename: 'invoice.pdf',
      content_type: 'application/pdf'
    }))).toBe(false)
  })

  it('recognizes safe text attachments as previewable', () => {
    expect(isPreviewableTextAttachment(attachment({
      filename: 'notes.txt',
      content_type: 'text/plain',
      scan_status: 'not_scanned'
    }))).toBe(true)
    expect(isPreviewableTextAttachment(attachment({
      filename: 'payload.json',
      content_type: 'application/json',
      scan_status: 'clean'
    }))).toBe(true)
    expect(isPreviewableTextAttachment(attachment({
      filename: 'danger.txt',
      content_type: 'text/plain',
      scan_status: 'malicious'
    }))).toBe(false)
    expect(isPreviewableTextAttachment(attachment({
      filename: 'invoice.pdf',
      content_type: 'application/pdf',
      scan_status: 'clean'
    }))).toBe(false)
  })

  it('recognizes safe raster image attachments as previewable', () => {
    expect(isPreviewableImageAttachment(attachment({
      filename: 'photo.png',
      content_type: 'image/png',
      scan_status: 'not_scanned'
    }))).toBe(true)
    expect(isPreviewableImageAttachment(attachment({
      filename: 'avatar.webp',
      content_type: 'application/octet-stream',
      scan_status: 'clean'
    }))).toBe(true)
    expect(isPreviewableImageAttachment(attachment({
      filename: 'unsafe.svg',
      content_type: 'image/svg+xml',
      scan_status: 'clean'
    }))).toBe(false)
    expect(isPreviewableImageAttachment(attachment({
      filename: 'danger.png',
      content_type: 'image/png',
      scan_status: 'malicious'
    }))).toBe(false)
  })
})
