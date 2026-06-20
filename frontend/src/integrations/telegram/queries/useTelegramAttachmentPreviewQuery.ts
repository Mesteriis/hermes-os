import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  previewTelegramAttachment,
  type TelegramAttachmentPreviewResponse,
} from '../api/telegramAttachmentPreview'

export function useTelegramAttachmentPreviewQuery(
  attachmentId: MaybeRefOrGetter<string | null | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true
) {
  return useQuery<TelegramAttachmentPreviewResponse>({
    queryKey: computed(() => ['communications', 'messages', toValue(attachmentId) ?? 'none', 'attachment-preview']),
    queryFn: () => previewTelegramAttachment(toValue(attachmentId) as string),
    enabled: computed(() => Boolean(toValue(attachmentId)) && Boolean(toValue(enabled))),
  })
}
