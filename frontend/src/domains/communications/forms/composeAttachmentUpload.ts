import type { CommunicationAttachmentImportResponse } from '../api/attachmentImportApi'
import type { ComposeAttachmentModel } from '../types/communications'

const MAX_COMPOSE_ATTACHMENT_BYTES = 50 * 1024 * 1024

export function pendingComposeAttachment(file: File, temporaryId: string): ComposeAttachmentModel {
  return {
    attachmentId: temporaryId,
    filename: file.name || 'attachment.bin',
    contentType: file.type || 'application/octet-stream',
    sizeBytes: file.size,
    scanStatus: 'uploading',
    uploadStatus: 'uploading',
    error: ''
  }
}

export async function composeFileContentBase64(file: File): Promise<string> {
  if (file.size <= 0) throw new Error('Empty attachments are not supported')
  if (file.size > MAX_COMPOSE_ATTACHMENT_BYTES) {
    throw new Error('Attachment exceeds the 50 MiB limit')
  }
  return bytesToBase64(new Uint8Array(await file.arrayBuffer()))
}

export function importedComposeAttachment(
  imported: CommunicationAttachmentImportResponse
): ComposeAttachmentModel {
  const clean = imported.scan_status === 'clean'
  return {
    attachmentId: imported.attachment_id,
    filename: imported.filename?.trim() || imported.attachment_id,
    contentType: imported.content_type,
    sizeBytes: imported.size_bytes,
    scanStatus: imported.scan_status,
    uploadStatus: clean ? 'ready' : 'blocked',
    error: clean ? '' : `Waiting for a clean scan verdict (${imported.scan_status})`
  }
}

export function failedComposeAttachment(
  pending: ComposeAttachmentModel,
  error: unknown
): ComposeAttachmentModel {
  return {
    ...pending,
    scanStatus: 'failed',
    uploadStatus: 'failed',
    error: error instanceof Error ? error.message : 'Attachment upload failed'
  }
}

export function composeAttachmentSendError(attachments: readonly ComposeAttachmentModel[]): string {
  const uploading = attachments.find((attachment) => attachment.uploadStatus === 'uploading')
  if (uploading) return `Attachment "${uploading.filename}" is still uploading`

  const failed = attachments.find((attachment) => attachment.uploadStatus === 'failed')
  if (failed) return failed.error || `Attachment "${failed.filename}" failed to upload`

  const blocked = attachments.find((attachment) => attachment.uploadStatus === 'blocked')
  if (blocked) {
    return `Attachment "${blocked.filename}" is blocked by its security scan (${blocked.scanStatus})`
  }
  return ''
}

export function bytesToBase64(bytes: Uint8Array): string {
  let binary = ''
  const chunkSize = 0x8000
  for (let offset = 0; offset < bytes.length; offset += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + chunkSize))
  }
  return btoa(binary)
}
