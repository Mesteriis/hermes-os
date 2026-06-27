import type { ColumnDef } from '@tanstack/vue-table'
import type { CommunicationAttachment } from '../types/communications'

export const attachmentTableColumns: ColumnDef<CommunicationAttachment>[] = [
  {
    id: 'filename',
    header: 'File',
    accessorFn: (attachment) => attachment.filename || 'Unnamed'
  },
  {
    id: 'content_type',
    header: 'Type',
    accessorKey: 'content_type'
  },
  {
    id: 'size',
    header: 'Size',
    accessorFn: (attachment) => attachment.size_bytes
  },
  {
    id: 'scan_status',
    header: 'Scan',
    accessorKey: 'scan_status'
  }
]

export function attachmentTableRowId(attachment: CommunicationAttachment): string {
  return attachment.attachment_id
}

export function formatAttachmentSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function scanStatusClass(status: string): string {
  switch (status) {
    case 'clean': return 'att-scan--clean'
    case 'suspicious': return 'att-scan--suspicious'
    case 'malicious': return 'att-scan--danger'
    case 'failed': return 'att-scan--danger'
    default: return 'att-scan--unknown'
  }
}

export function isInspectableArchiveAttachment(attachment: CommunicationAttachment): boolean {
  const contentType = attachment.content_type.trim().toLowerCase()
  if (contentType === 'application/zip' || contentType === 'application/x-zip-compressed') {
    return true
  }
  return attachment.filename?.trim().toLowerCase().endsWith('.zip') ?? false
}

export function isPreviewableTextAttachment(attachment: CommunicationAttachment): boolean {
  if (!isPreviewAllowedByScanStatus(attachment)) {
    return false
  }
  const contentType = attachment.content_type.trim().toLowerCase()
  if (contentType.startsWith('text/')) {
    return true
  }
  if (['application/json', 'application/xml', 'application/yaml', 'application/x-yaml'].includes(contentType)) {
    return true
  }
  const filename = attachment.filename?.trim().toLowerCase()
  return Boolean(
    filename
    && (
      filename.endsWith('.txt')
      || filename.endsWith('.md')
      || filename.endsWith('.csv')
      || filename.endsWith('.json')
      || filename.endsWith('.xml')
      || filename.endsWith('.yaml')
      || filename.endsWith('.yml')
    )
  )
}

export function isPreviewableImageAttachment(attachment: CommunicationAttachment): boolean {
  if (!isPreviewAllowedByScanStatus(attachment)) {
    return false
  }
  const contentType = attachment.content_type.trim().toLowerCase()
  if (['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(contentType)) {
    return true
  }
  const filename = attachment.filename?.trim().toLowerCase()
  return Boolean(
    filename
    && (
      filename.endsWith('.png')
      || filename.endsWith('.jpg')
      || filename.endsWith('.jpeg')
      || filename.endsWith('.gif')
      || filename.endsWith('.webp')
    )
  )
}

export function isPreviewableAudioAttachment(attachment: CommunicationAttachment): boolean {
  if (!isPreviewAllowedByScanStatus(attachment)) {
    return false
  }
  const contentType = attachment.content_type.trim().toLowerCase()
  if (contentType.startsWith('audio/')) {
    return true
  }
  const filename = attachment.filename?.trim().toLowerCase()
  return Boolean(
    filename
    && (
      filename.endsWith('.mp3')
      || filename.endsWith('.m4a')
      || filename.endsWith('.aac')
      || filename.endsWith('.ogg')
      || filename.endsWith('.opus')
      || filename.endsWith('.wav')
      || filename.endsWith('.webm')
    )
  )
}

export function isPreviewableVideoAttachment(attachment: CommunicationAttachment): boolean {
  if (!isPreviewAllowedByScanStatus(attachment)) {
    return false
  }
  const contentType = attachment.content_type.trim().toLowerCase()
  if (contentType.startsWith('video/')) {
    return true
  }
  const filename = attachment.filename?.trim().toLowerCase()
  return Boolean(
    filename
    && (
      filename.endsWith('.mp4')
      || filename.endsWith('.webm')
      || filename.endsWith('.mov')
    )
  )
}

export function isPreviewablePdfAttachment(attachment: CommunicationAttachment): boolean {
  if (!isPreviewAllowedByScanStatus(attachment)) {
    return false
  }
  const contentType = attachment.content_type.trim().toLowerCase()
  if (contentType === 'application/pdf') {
    return true
  }
  const filename = attachment.filename?.trim().toLowerCase()
  return Boolean(filename && filename.endsWith('.pdf'))
}

export function isPreviewableAttachment(attachment: CommunicationAttachment): boolean {
  return (
    isPreviewableTextAttachment(attachment) ||
    isPreviewableImageAttachment(attachment) ||
    isPreviewableAudioAttachment(attachment) ||
    isPreviewableVideoAttachment(attachment) ||
    isPreviewablePdfAttachment(attachment)
  )
}

function isPreviewAllowedByScanStatus(attachment: CommunicationAttachment): boolean {
  return ['not_scanned', 'clean'].includes(attachment.scan_status)
}
