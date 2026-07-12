import { describe, expect, it } from 'vitest'
import {
  bytesToBase64,
  composeAttachmentSendError,
  importedComposeAttachment
} from './composeAttachmentUpload'

describe('compose attachment upload', () => {
  it('encodes binary bytes without corrupting non-text content', () => {
    expect(bytesToBase64(new Uint8Array([0, 255, 16, 32]))).toBe('AP8QIA==')
  })

  it('blocks imported attachments until a clean scanner verdict exists', () => {
    const blocked = importedComposeAttachment({
      attachment_id: 'attachment-1',
      blob_id: 'blob-1',
      filename: 'report.pdf',
      content_type: 'application/pdf',
      size_bytes: 42,
      sha256: 'sha256:fixture',
      scan_status: 'not_scanned',
      storage_kind: 'local_fs',
      storage_path: 'sha256/fixture'
    })
    expect(blocked.uploadStatus).toBe('blocked')
    expect(blocked.error).toContain('not_scanned')
  })

  it('explains why attachments cannot be sent yet', () => {
    const blocked = importedComposeAttachment({
      attachment_id: 'attachment-1',
      blob_id: 'blob-1',
      filename: 'report.pdf',
      content_type: 'application/pdf',
      size_bytes: 42,
      sha256: 'sha256:fixture',
      scan_status: 'suspicious',
      storage_kind: 'local_fs',
      storage_path: 'sha256/fixture'
    })

    expect(composeAttachmentSendError([blocked])).toBe(
      'Attachment "report.pdf" is blocked by its security scan (suspicious)'
    )
  })
})
