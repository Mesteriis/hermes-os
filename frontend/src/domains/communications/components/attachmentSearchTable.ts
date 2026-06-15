import type { ColumnDef } from '@tanstack/vue-table'
import type { AttachmentSearchResult } from '../types/attachments'

export const attachmentSearchTableColumns: ColumnDef<AttachmentSearchResult>[] = [
  {
    id: 'filename',
    header: 'File',
    accessorFn: (attachment) => attachment.filename || 'Unnamed'
  },
  {
    id: 'message_subject',
    header: 'Message',
    accessorKey: 'message_subject'
  },
  {
    id: 'sender',
    header: 'Sender',
    accessorKey: 'sender'
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

export function attachmentSearchTableRowId(result: AttachmentSearchResult): string {
  return result.attachment_id
}
