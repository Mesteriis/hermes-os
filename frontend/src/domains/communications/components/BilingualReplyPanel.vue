<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import { usePrepareBilingualReplyFlowMutation } from '../queries/useCommunicationsQuery'
import {
  bilingualReplyFlowFormDefaults,
  bilingualReplyFlowFormToRequest,
  bilingualReplyFlowVeeValidationSchema,
  bilingualReplyToneOptions,
  type BilingualReplyFlowFormValues
} from '../forms/bilingualReplyFlowForm'
import type {
  BilingualReplyFlowResponse,
  BilingualReplyTone,
  BilingualTranslationStep
} from '../types/bilingualReplyFlow'

const props = defineProps<{
  messageId: string
}>()

const emit = defineEmits<{
  sendBilingualReply: [payload: BilingualReplyFlowResponse]
}>()

const result = ref<BilingualReplyFlowResponse | null>(null)
const prepareMutation = usePrepareBilingualReplyFlowMutation()

const {
  errors,
  handleSubmit,
  setFieldValue,
  values: formValues
} = useForm<BilingualReplyFlowFormValues>({
  validationSchema: bilingualReplyFlowVeeValidationSchema,
  initialValues: bilingualReplyFlowFormDefaults()
})

const isPreparing = computed(() => prepareMutation.isPending.value)
const errorMessage = computed(() => {
  const error = prepareMutation.error.value
  if (!error) return ''
  return error instanceof Error ? error.message : 'Bilingual reply preparation failed'
})

const submitBilingualReply = handleSubmit(async (values) => {
  result.value = await prepareMutation.mutateAsync({
    messageId: props.messageId,
    request: bilingualReplyFlowFormToRequest(values)
  })
})

function updateReplyText(event: Event): void {
  const input = event.target as HTMLTextAreaElement
  setFieldValue('replyTextRu', input.value)
}

function updateTone(event: Event): void {
  const input = event.target as HTMLSelectElement
  setFieldValue('tone', input.value as BilingualReplyTone)
}

function stepText(step: BilingualTranslationStep): string {
  return step.text ?? step.reason ?? 'Pending'
}

function stepStatus(step: BilingualTranslationStep): string {
  return step.translated ? 'translated' : 'review required'
}

function sendPreparedReply(): void {
  if (!result.value?.send_ready) return
  emit('sendBilingualReply', result.value)
}
</script>

<template>
  <section class="bilingual-reply-panel">
    <form class="bilingual-reply-form" @submit.prevent="submitBilingualReply">
      <div class="bilingual-reply-header">
        <span class="bilingual-reply-title">
          <Icon icon="tabler:language-hiragana" />
          Bilingual reply
        </span>
        <span class="bilingual-reply-state">{{ result?.send_ready ? 'Ready' : 'Review' }}</span>
      </div>

      <label class="bilingual-reply-field">
        <span>Reply in Russian</span>
        <textarea
          :value="formValues.replyTextRu"
          rows="5"
          autocomplete="off"
          @input="updateReplyText"
        />
        <small v-if="errors.replyTextRu">{{ errors.replyTextRu }}</small>
      </label>

      <label class="bilingual-reply-field bilingual-reply-tone">
        <span>Tone</span>
        <select :value="formValues.tone" @change="updateTone">
          <option v-for="tone in bilingualReplyToneOptions" :key="tone" :value="tone">
            {{ tone }}
          </option>
        </select>
        <small v-if="errors.tone">{{ errors.tone }}</small>
      </label>

      <div class="bilingual-reply-actions">
        <Button type="submit" size="sm" :loading="isPreparing" :disabled="isPreparing">
          <Icon icon="tabler:arrows-exchange" />
          Prepare
        </Button>
        <span v-if="errorMessage" class="bilingual-reply-error">{{ errorMessage }}</span>
      </div>
    </form>

    <div v-if="result" class="bilingual-reply-review">
      <section class="bilingual-reply-step">
        <div class="bilingual-reply-step-header">
          <span>Original</span>
          <span>{{ result.original.language }} {{ Math.round(result.original.confidence * 100) }}%</span>
        </div>
        <p>{{ result.original.text }}</p>
      </section>

      <section class="bilingual-reply-step">
        <div class="bilingual-reply-step-header">
          <span>Translation</span>
          <span>{{ stepStatus(result.translation) }}</span>
        </div>
        <p>{{ stepText(result.translation) }}</p>
      </section>

      <section class="bilingual-reply-step">
        <div class="bilingual-reply-step-header">
          <span>Reply in Russian</span>
          <span>{{ result.reply.tone }}</span>
        </div>
        <p>{{ result.reply.text }}</p>
      </section>

      <section class="bilingual-reply-step">
        <div class="bilingual-reply-step-header">
          <span>Back Translation</span>
          <span>{{ stepStatus(result.back_translation) }}</span>
        </div>
        <p>{{ stepText(result.back_translation) }}</p>
      </section>

      <div class="bilingual-reply-send">
        <Button
          type="button"
          size="sm"
          variant="outline"
          :disabled="!result.send_ready"
          @click="sendPreparedReply"
        >
          <Icon icon="tabler:send" />
          Open Compose
        </Button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.bilingual-reply-panel {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, rgba(148, 163, 184, 0.28));
  border-radius: var(--hh-radius-sm, 0.375rem);
  background: var(--hh-surface-glass, rgba(255, 255, 255, 0.72));
  backdrop-filter: blur(18px);
  box-shadow: var(--hh-shadow-sm, 0 10px 28px rgba(15, 23, 42, 0.08));
}

.bilingual-reply-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(8rem, 12rem);
  gap: 0.75rem;
}

.bilingual-reply-header,
.bilingual-reply-actions {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.bilingual-reply-title {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  font-weight: 600;
}

.bilingual-reply-state {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  text-transform: uppercase;
}

.bilingual-reply-field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  min-width: 0;
}

.bilingual-reply-field span {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  font-weight: 500;
}

.bilingual-reply-field textarea,
.bilingual-reply-field select {
  width: 100%;
  border: 1px solid var(--hh-border, #d1d5db);
  border-radius: var(--hh-radius-xs, 0.25rem);
  background: var(--hh-bg-surface, #fff);
  color: var(--hh-text-primary, #1f2937);
  font: inherit;
  font-size: 0.8125rem;
}

.bilingual-reply-field textarea {
  min-height: 7rem;
  padding: 0.625rem;
  resize: vertical;
}

.bilingual-reply-field select {
  height: 2.125rem;
  padding: 0 0.5rem;
  text-transform: capitalize;
}

.bilingual-reply-field small,
.bilingual-reply-error {
  color: var(--hh-color-danger, #dc2626);
  font-size: 0.75rem;
}

.bilingual-reply-send {
  grid-column: 1 / -1;
}

.bilingual-reply-review {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.625rem;
}

.bilingual-reply-step {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  min-width: 0;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, rgba(148, 163, 184, 0.24));
  border-radius: var(--hh-radius-xs, 0.25rem);
  background: var(--hh-bg-elevated, rgba(255, 255, 255, 0.62));
}

.bilingual-reply-step-header {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.bilingual-reply-step p {
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  line-height: 1.45;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

@media (max-width: 760px) {
  .bilingual-reply-form,
  .bilingual-reply-review {
    grid-template-columns: 1fr;
  }
}
</style>
