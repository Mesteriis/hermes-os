import { ApiClient } from '../../../platform/api/ApiClient'

export type CommunicationAttachmentImportRequest = {
  account_id?: string
  channel_kind?: string
  filename?: string
  content_type?: string
  content_base64: string
  source_kind?: string
  metadata?: Record<string, unknown>
}

export type CommunicationAttachmentImportResponse = {
  attachment_id: string
  account_id?: string | null
  channel_kind?: string | null
  blob_id: string
  filename?: string | null
  content_type: string
  size_bytes: number
  sha256: string
  scan_status: string
  storage_kind: string
  storage_path: string
}

export type TelegramMediaUploadRequest = {
  command_id?: string
  account_id: string
  provider_chat_id: string
  attachment_id?: string
  blob_id?: string
  media_type: TelegramMediaUploadKind
  caption?: string
  filename?: string
}

export type TelegramMediaUploadResponse = {
  command_id: string
  account_id: string
  provider_chat_id: string
  attachment_id?: string | null
  blob_id: string
  media_type: TelegramMediaUploadKind
  status: string
  reconciliation_status: string
}

export type TelegramMediaUploadKind =
  | 'photo'
  | 'video'
  | 'document'
  | 'audio'
  | 'voice'
  | 'sticker'
  | 'animation'

export async function importCommunicationAttachment(
  request: CommunicationAttachmentImportRequest
): Promise<CommunicationAttachmentImportResponse> {
  return ApiClient.instance.post<CommunicationAttachmentImportResponse>(
    '/api/v1/communications/attachments/import',
    request,
    'Communication attachment import failed'
  )
}

export async function uploadTelegramMedia(
  request: TelegramMediaUploadRequest
): Promise<TelegramMediaUploadResponse> {
  return ApiClient.instance.post<TelegramMediaUploadResponse>(
    '/api/v1/integrations/telegram/provider-media/upload',
    request,
    'Telegram media upload failed'
  )
}
