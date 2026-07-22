import type { AttachmentTranslationRequest } from '../../types/attachments'

export function buildMailAttachmentTranslationRequest(
  attachmentId: string,
  targetLanguage: string
): { attachmentId: string; request: AttachmentTranslationRequest } {
  return {
    attachmentId,
    request: { target_language: targetLanguage },
  }
}
