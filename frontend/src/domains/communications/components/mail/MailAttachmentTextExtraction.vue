<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Button } from '@/shared/ui'
import type { CommunicationConversationAttachmentModel } from '../communicationDomainElements'
import {
  useAttachmentExtractedTextQuery,
  useExtractAttachmentTextMutation,
  useTranslateAttachmentMutation
} from '../../queries/mailWorkspaceQueries'
import {
  canExtractMailAttachmentText,
  extractionStatusLabel
} from './mailAttachmentTextExtractionPresentation'

const props = defineProps<{
  attachment: CommunicationConversationAttachmentModel
}>()

const { t } = useI18n()
const requested = ref(false)
const requestFailed = ref(false)
const targetLanguage = ref('en')
const extractionMutation = useExtractAttachmentTextMutation()
const translationMutation = useTranslateAttachmentMutation()
const extractedTextQuery = useAttachmentExtractedTextQuery(
  () => props.attachment.id,
  () => requested.value && extractionMutation.data.value?.status === 'completed'
)
const canExtract = computed(() => canExtractMailAttachmentText(props.attachment))
const extractionStatus = computed(() => {
  const status = extractionMutation.data.value?.status
  return status ? extractionStatusLabel(status) : null
})

async function extractText(): Promise<void> {
  requestFailed.value = false

  try {
    await extractionMutation.mutateAsync({ attachmentId: props.attachment.id })
    requested.value = true
  } catch {
    requestFailed.value = true
  }
}

async function translateExtractedText(): Promise<void> {
  await translationMutation.mutateAsync({
    attachmentId: props.attachment.id,
    request: { target_language: targetLanguage.value }
  })
}
</script>

<template>
  <div v-if="canExtract" class="communication-email-attachment-extraction">
    <Button
      v-if="!requested"
      variant="ghost"
      size="sm"
      icon="tabler:file-search"
      :loading="extractionMutation.isPending.value"
      @click="extractText"
    >
      {{ t('Extract text locally') }}
    </Button>
    <p v-else-if="extractionStatus === 'unsupported'" class="communication-email-attachment-extraction__status">
      {{ t('Text extraction is not available for this attachment.') }}
    </p>
    <p v-else-if="requestFailed || extractedTextQuery.isError.value" class="communication-email-attachment-extraction__status">
      {{ t('Could not extract this attachment locally.') }}
    </p>
    <details v-else-if="extractedTextQuery.data.value" class="communication-email-attachment-extraction__result">
      <summary>{{ t('Extracted text') }}</summary>
      <pre>{{ extractedTextQuery.data.value.text }}</pre>
      <p v-if="extractedTextQuery.data.value.truncated">{{ t('Preview is truncated.') }}</p>
      <div class="communication-email-attachment-extraction__translation">
        <label>
          {{ t('Translate to') }}
          <select v-model="targetLanguage">
            <option value="en">English</option>
            <option value="ru">Русский</option>
          </select>
        </label>
        <Button
          variant="ghost"
          size="sm"
          icon="tabler:language"
          :loading="translationMutation.isPending.value"
          @click="translateExtractedText"
        >
          {{ t('Translate extracted text') }}
        </Button>
      </div>
      <p v-if="translationMutation.isError.value" class="communication-email-attachment-extraction__status">
        {{ t('Could not translate this attachment.') }}
      </p>
      <pre v-else-if="translationMutation.data.value?.text">{{ translationMutation.data.value.text }}</pre>
    </details>
    <p v-else class="communication-email-attachment-extraction__status">{{ t('Preparing extracted text...') }}</p>
  </div>
</template>
