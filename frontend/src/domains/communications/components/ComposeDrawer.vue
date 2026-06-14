<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import { useCommunicationsStore } from '../stores/communications'
import type { ComposeFormModel, EmailDraft } from '../types/communications'
import { senderEmail } from '../stores/communications'
import { createDraft, sendEmail, deleteDraft } from '../api/communications'

const store = useCommunicationsStore()

// Draft auto-save timer
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null
const isSaving = ref(false)

// Recipient managing
function splitRecipients(value: string): string[] {
  return value
    .split(',')
    .map((r) => r.trim())
    .filter(Boolean)
}

function buildComposeDraftPayload(): Record<string, unknown> {
  const form = store.composeForm
  return {
    draft_id: form.draftId,
    account_id: form.accountId,
    to_recipients: splitRecipients(form.toText),
    cc_recipients: splitRecipients(form.ccText),
    bcc_recipients: splitRecipients(form.bccText),
    subject: form.subject,
    body_text: form.body,
    in_reply_to: form.inReplyTo,
    status: 'draft',
    metadata: { compose_mode: form.mode }
  }
}

async function handleSaveDraft() {
  const form = store.composeForm
  if (!form.draftId || !form.accountId) return
  isSaving.value = true
  try {
    await createDraft(buildComposeDraftPayload())
    store.setComposeStatusMessage('Draft saved')
  } catch (error) {
    store.setComposeSendError(error instanceof Error ? error.message : 'Draft save failed')
  } finally {
    isSaving.value = false
  }
}

function triggerAutoSave() {
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  autoSaveTimer = setTimeout(() => {
    handleSaveDraft()
  }, 2000)
}

// Subject prefix helpers
function subjectWithPrefix(subject: string, prefix: 'Re:' | 'Fwd:'): string {
  return subject.toLowerCase().startsWith(prefix.toLowerCase())
    ? subject
    : `${prefix} ${subject}`
}

// Send
const isSending = ref(false)

async function handleSend() {
  const form = store.composeForm
  if (isSending.value) return
  isSending.value = true
  store.setComposeSendError('')
  try {
    const result = await sendEmail({
      account_id: form.accountId,
      to: splitRecipients(form.toText),
      cc: splitRecipients(form.ccText),
      bcc: splitRecipients(form.bccText),
      subject: form.subject,
      body_text: form.body,
      in_reply_to: form.inReplyTo,
      confirmed_provider_write: true
    })
    store.setComposeStatusMessage(`Sent via ${result.transport ?? 'provider'}`)
    store.closeCompose()
  } catch (error) {
    store.setComposeSendError(error instanceof Error ? error.message : 'Send failed')
  } finally {
    isSending.value = false
  }
}

async function handleDeleteCurrentDraft() {
  const draftId = store.composeForm.draftId?.trim()
  if (!draftId) return
  store.setComposeSendError('')
  try {
    await deleteDraft(draftId)
    store.closeCompose()
  } catch (error) {
    store.setComposeSendError(error instanceof Error ? error.message : 'Delete failed')
  }
}

function handleClose() {
  // Auto-save before closing
  if (store.composeForm.body || store.composeForm.subject || store.composeForm.toText) {
    handleSaveDraft()
  }
  store.closeCompose()
}

function updateField<K extends keyof ComposeFormModel>(key: K, value: ComposeFormModel[K]) {
  store.updateComposeForm({ [key]: value })
  if (key !== 'draftId' && key !== 'accountId') {
    triggerAutoSave()
  }
}

// Send review
const isReviewOpen = ref(false)

function openSendReview() {
  isReviewOpen.value = true
}

function closeSendReview() {
  isReviewOpen.value = false
}

function confirmSend() {
  isReviewOpen.value = false
  handleSend()
}

// Mode label
const modeLabel = computed(() => {
  switch (store.composeForm.mode) {
    case 'reply': return 'Reply'
    case 'forward': return 'Forward'
    default: return 'New Message'
  }
})
</script>

<template>
  <div class="compose-drawer-overlay" @click.self="handleClose">
    <div class="compose-drawer">
      <div class="compose-header">
        <span class="compose-title">{{ modeLabel }}</span>
        <div class="compose-header-actions">
          <span v-if="isSaving" class="saving-indicator">
            <Icon icon="tabler:loader-2" class="spin-icon" /> Saving...
          </span>
          <Button variant="ghost" size="sm" @click="handleClose">
            <Icon icon="tabler:x" />
          </Button>
        </div>
      </div>

      <!-- Send review step -->
      <div v-if="isReviewOpen" class="send-review">
        <h3>Review before sending</h3>
        <div class="review-field">
          <span class="review-label">To:</span>
          <span class="review-value">{{ store.composeForm.toText }}</span>
        </div>
        <div v-if="store.composeForm.ccText" class="review-field">
          <span class="review-label">CC:</span>
          <span class="review-value">{{ store.composeForm.ccText }}</span>
        </div>
        <div v-if="store.composeForm.bccText" class="review-field">
          <span class="review-label">BCC:</span>
          <span class="review-value">{{ store.composeForm.bccText }}</span>
        </div>
        <div class="review-field">
          <span class="review-label">Subject:</span>
          <span class="review-value">{{ store.composeForm.subject || '(No subject)' }}</span>
        </div>
        <div class="review-actions">
          <Button variant="default" @click="confirmSend" :disabled="isSending">
            <Icon icon="tabler:send" /> {{ isSending ? 'Sending...' : 'Send' }}
          </Button>
          <Button variant="ghost" @click="closeSendReview">Edit</Button>
        </div>
      </div>

      <!-- Compose form -->
      <div v-else class="compose-form">
        <div class="form-field">
          <label>To</label>
          <input
            type="text"
            placeholder="recipient@example.com"
            :value="store.composeForm.toText"
            @input="updateField('toText', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="form-field">
          <label>CC</label>
          <input
            type="text"
            placeholder="cc@example.com"
            :value="store.composeForm.ccText"
            @input="updateField('ccText', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="form-field">
          <label>BCC</label>
          <input
            type="text"
            placeholder="bcc@example.com"
            :value="store.composeForm.bccText"
            @input="updateField('bccText', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="form-field">
          <label>Subject</label>
          <input
            type="text"
            placeholder="Subject"
            :value="store.composeForm.subject"
            @input="updateField('subject', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="form-field body-field">
          <label>Message</label>
          <textarea
            placeholder="Write your message..."
            :value="store.composeForm.body"
            @input="updateField('body', ($event.target as HTMLTextAreaElement).value)"
          />
        </div>

        <div v-if="store.composeSendError" class="compose-error">
          {{ store.composeSendError }}
        </div>
        <div v-if="store.composeStatusMessage" class="compose-status">
          {{ store.composeStatusMessage }}
        </div>

        <div class="compose-actions">
          <Button variant="default" @click="openSendReview" :disabled="!store.composeForm.toText">
            <Icon icon="tabler:send" /> Send
          </Button>
          <Button variant="ghost" @click="handleSaveDraft" :disabled="isSaving">
            <Icon icon="tabler:edit" /> Save Draft
          </Button>
          <Button variant="ghost" @click="handleDeleteCurrentDraft" class="delete-btn">
            <Icon icon="tabler:trash" /> Discard
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.compose-drawer-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 100;
  display: flex;
  justify-content: flex-end;
  align-items: flex-end;
}

.compose-drawer {
  width: 560px;
  max-height: 85vh;
  background: var(--hh-bg-primary, #ffffff);
  border-radius: 0.75rem 0.75rem 0 0;
  box-shadow: 0 -4px 24px rgba(0, 0, 0, 0.12);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.compose-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: var(--hh-bg-primary, #ffffff);
}

.compose-title {
  font-weight: 600;
  font-size: 0.9375rem;
  color: var(--hh-text-primary, #1f2937);
}

.compose-header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.saving-indicator {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.spin-icon {
  animation: spin 1s linear infinite;
  width: 14px;
  height: 14px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.compose-form {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding: 0.75rem 1rem;
  gap: 0.5rem;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.form-field label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-secondary, #6b7280);
}

.form-field input {
  padding: 0.4375rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  background: var(--hh-bg-primary, #ffffff);
  outline: none;
  transition: border-color 0.15s;
}

.form-field input:focus {
  border-color: var(--hh-accent, #3b82f6);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.body-field {
  flex: 1;
}

.body-field textarea {
  flex: 1;
  min-height: 200px;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
  font-family: inherit;
  color: var(--hh-text-primary, #1f2937);
  background: var(--hh-bg-primary, #ffffff);
  resize: vertical;
  outline: none;
  line-height: 1.6;
}

.body-field textarea:focus {
  border-color: var(--hh-accent, #3b82f6);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.compose-error {
  padding: 0.5rem 0.75rem;
  background: var(--hh-bg-error-light, #fef2f2);
  color: var(--hh-text-error, #ef4444);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
}

.compose-status {
  padding: 0.375rem 0.75rem;
  background: var(--hh-bg-success-light, #f0fdf4);
  color: var(--hh-text-success, #16a34a);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
}

.compose-actions {
  display: flex;
  gap: 0.5rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--hh-border, #e5e7eb);
}

.delete-btn {
  margin-left: auto;
}

/* Send review */
.send-review {
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.send-review h3 {
  font-size: 1rem;
  font-weight: 600;
  margin: 0;
}

.review-field {
  display: flex;
  gap: 0.5rem;
}

.review-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-secondary, #6b7280);
  min-width: 60px;
  flex-shrink: 0;
}

.review-value {
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
}

.review-actions {
  display: flex;
  gap: 0.5rem;
  padding-top: 0.5rem;
}
</style>
