import { ApiClient } from '../../../platform/api/ApiClient'
import type { AttachmentPreviewResponse } from '../../../domains/communications/types/attachments'

export type TelegramAttachmentPreviewResponse = AttachmentPreviewResponse

export async function previewTelegramAttachment(
  attachmentId: string
): Promise<TelegramAttachmentPreviewResponse> {
  return ApiClient.instance.get<TelegramAttachmentPreviewResponse>(
    `/api/v1/communications/attachments/${encodeURIComponent(attachmentId)}/preview`,
    'Telegram attachment preview failed'
  )
}
