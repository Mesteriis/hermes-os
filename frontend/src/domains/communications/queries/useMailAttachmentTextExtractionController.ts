import { computed, ref, type Ref } from 'vue'
import type { CommunicationConversationAttachmentModel } from '../components/communicationDomainElements'
import {
  useAttachmentExtractedTextQuery,
  useExtractAttachmentTextMutation,
  useTranslateAttachmentMutation,
} from './mailWorkspaceQueries'
import {
  canExtractMailAttachmentText,
  extractionStatusLabel,
} from '../components/mail/mailAttachmentTextExtractionPresentation'
import { buildMailAttachmentTranslationRequest } from '../components/mail/mailAttachmentTextExtractionActions'

export function useMailAttachmentTextExtractionController(
  attachment: Ref<CommunicationConversationAttachmentModel>,
) {
  const requested = ref(false)
  const requestFailed = ref(false)
  const targetLanguage = ref('en')
  const extractionMutation = useExtractAttachmentTextMutation()
  const translationMutation = useTranslateAttachmentMutation()
  const extractedTextQuery = useAttachmentExtractedTextQuery(
    () => attachment.value.id,
    () => requested.value && extractionMutation.data.value?.status === 'completed',
  )
  const canExtract = computed(() => canExtractMailAttachmentText(attachment.value))
  const extractionStatus = computed(() => {
    const status = extractionMutation.data.value?.status
    return status ? extractionStatusLabel(status) : null
  })

  async function extractText(): Promise<void> {
    requestFailed.value = false
    try {
      await extractionMutation.mutateAsync({ attachmentId: attachment.value.id })
      requested.value = true
    } catch {
      requestFailed.value = true
    }
  }

  async function translateExtractedText(): Promise<void> {
    await translationMutation.mutateAsync(buildMailAttachmentTranslationRequest(
      attachment.value.id,
      targetLanguage.value,
    ))
  }

  return {
    requested,
    requestFailed,
    targetLanguage,
    extractionMutation,
    translationMutation,
    extractedTextQuery,
    canExtract,
    extractionStatus,
    extractText,
    translateExtractedText,
  }
}
