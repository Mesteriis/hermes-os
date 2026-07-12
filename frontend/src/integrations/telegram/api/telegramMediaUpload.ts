import { ApiClient } from '../../../platform/api/ApiClient'

export type TelegramMediaUploadRequest = {
  command_id?: string
  account_id: string
  provider_chat_id: string
  attachment_id: string
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

export async function uploadTelegramMedia(
  request: TelegramMediaUploadRequest
): Promise<TelegramMediaUploadResponse> {
  return ApiClient.instance.post<TelegramMediaUploadResponse>(
    '/api/v1/integrations/telegram/provider-media/upload',
    request,
    'Telegram media upload failed'
  )
}
