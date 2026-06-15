<script setup lang="ts">
import { ref } from 'vue'
import Button from '../../../shared/ui/Button.vue'
import RichComposeEditor from './RichComposeEditor.vue'
import type { ThreadMessage } from '../types/communications'
import { senderEmail, senderLabel } from '../stores/communications'

const props = defineProps<{
  message: ThreadMessage
  bodyHtml: string
  isSendingReply: boolean
}>()

const emit = defineEmits<{
  'update:bodyHtml': [bodyHtml: string]
  cancel: []
  saveDraft: []
  continueInCompose: []
  send: []
}>()

const reviewingReply = ref(false)

function updateBodyHtml(bodyHtml: string): void {
  emit('update:bodyHtml', bodyHtml)
  if (!bodyHtml.trim()) {
    reviewingReply.value = false
  }
}

function openSendReview(): void {
  if (!props.bodyHtml.trim()) return
  reviewingReply.value = true
}

function closeSendReview(): void {
  reviewingReply.value = false
}

function confirmSend(): void {
  if (!props.bodyHtml.trim()) return
  reviewingReply.value = false
  emit('send')
}

function replyReviewRecipient(message: ThreadMessage): string {
  const label = senderLabel(message.sender)
  const email = senderEmail(message.sender)
  return label && label !== email ? `${label} <${email}>` : email
}

function replyReviewSubject(message: ThreadMessage): string {
  return message.subject.startsWith('Re:') ? message.subject : `Re: ${message.subject}`
}
</script>

<template>
  <div class="inline-reply">
    <div class="inline-reply-header">
      <span>Replying to {{ senderLabel(message.sender) }}</span>
      <button type="button" @click="emit('cancel')">Discard</button>
    </div>
    <RichComposeEditor
      :model-value="bodyHtml"
      placeholder="Write a reply..."
      @update:model-value="updateBodyHtml"
      @blur="emit('saveDraft')"
    />
    <div class="inline-reply-actions">
      <Button variant="secondary" size="sm" @click="emit('cancel')">
        Cancel
      </Button>
      <Button
        variant="secondary"
        size="sm"
        :disabled="!bodyHtml.trim()"
        @click="emit('saveDraft')"
      >
        Save Draft
      </Button>
      <Button variant="secondary" size="sm" @click="emit('continueInCompose')">
        Continue in Compose
      </Button>
      <Button
        variant="default"
        size="sm"
        :disabled="!bodyHtml.trim() || isSendingReply"
        @click="openSendReview"
      >
        Review & Send
      </Button>
    </div>
    <div v-if="reviewingReply" class="inline-send-review">
      <div class="review-title">Review reply before sending</div>
      <div class="review-grid">
        <span>To</span>
        <strong>{{ replyReviewRecipient(message) }}</strong>
        <span>Subject</span>
        <strong>{{ replyReviewSubject(message) }}</strong>
        <span>Delivery</span>
        <strong>Immediate provider send</strong>
        <span>Undo</span>
        <strong>Off</strong>
      </div>
      <div class="review-actions">
        <Button variant="default" size="sm" :disabled="isSendingReply" @click="confirmSend">
          {{ isSendingReply ? 'Sending...' : 'Send' }}
        </Button>
        <Button variant="ghost" size="sm" @click="closeSendReview">
          Edit
        </Button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.inline-reply {
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
  margin-top: 0.875rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 82%, transparent);
}

.inline-reply :deep(.rich-compose-editor) {
  min-height: 160px;
}

.inline-reply :deep(.rich-compose-surface) {
  min-height: 120px;
}

.inline-reply-header,
.inline-reply-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.inline-reply-header {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
}

.inline-reply-header button {
  border: 0;
  padding: 0;
  background: transparent;
  color: var(--hh-accent, #3b82f6);
  cursor: pointer;
  font: inherit;
}

.inline-reply-actions {
  justify-content: flex-end;
}

.inline-send-review {
  display: grid;
  gap: 0.625rem;
  padding: 0.75rem;
  border: 1px solid color-mix(in srgb, var(--hh-accent, #3b82f6) 32%, var(--hh-border, #e5e7eb));
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-accent, #3b82f6) 8%, var(--hh-bg-primary, #ffffff));
}

.review-title {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--hh-text-primary, #1f2937);
}

.review-grid {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: 0.375rem 0.75rem;
  font-size: 0.75rem;
}

.review-grid span {
  color: var(--hh-text-secondary, #6b7280);
}

.review-grid strong {
  min-width: 0;
  overflow-wrap: anywhere;
  color: var(--hh-text-primary, #1f2937);
}

.review-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
