import type { AttachmentScanStatus } from '../../types/attachments'

export function canExtractMailAttachmentText(attachment: { scanStatus?: AttachmentScanStatus }): boolean {
  return attachment.scanStatus === 'clean'
}

export function extractionStatusLabel(status: 'completed' | 'unsupported'): 'ready' | 'unsupported' {
  return status === 'completed' ? 'ready' : 'unsupported'
}
