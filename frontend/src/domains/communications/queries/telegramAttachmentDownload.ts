import type { MessengerAttachmentModel } from '../components/messengers/messengerElements'
import type { TelegramConversationRuntimeActionRequest } from '@/shared/communications/types/telegramRuntimeActions'

export type TelegramAttachmentDownloadExtras = Pick<
  TelegramConversationRuntimeActionRequest,
  'contentType' | 'filename' | 'providerAttachmentId' | 'providerMessageId' | 'tdlibFileId'
>

export function telegramAttachmentDownloadExtras(
  attachment: MessengerAttachmentModel,
): TelegramAttachmentDownloadExtras | null {
  if (!attachment.downloadable || !attachment.providerMessageId || attachment.tdlibFileId == null) {
    return null
  }

  return {
    providerAttachmentId: attachment.providerAttachmentId,
    providerMessageId: attachment.providerMessageId,
    tdlibFileId: attachment.tdlibFileId,
    filename: attachment.name,
    contentType: attachment.contentType,
  }
}
