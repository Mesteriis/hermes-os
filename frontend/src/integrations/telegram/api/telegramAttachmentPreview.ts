import type { AttachmentPreviewResponse } from '../../../shared/communications/types/attachments'
import { previewTelegramBusinessAttachment } from '../../../shared/communications/telegramBusinessApi'

export type TelegramAttachmentPreviewResponse = AttachmentPreviewResponse

export async function previewTelegramAttachment(
  attachmentId: string
): Promise<TelegramAttachmentPreviewResponse> {
  return previewTelegramBusinessAttachment(attachmentId)
}
