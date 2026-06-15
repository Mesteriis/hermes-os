<script setup lang="ts">
import { ref, computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import Sheet from '../../../shared/ui/Sheet.vue'
import ComposeSignaturePicker from './ComposeSignaturePicker.vue'
import ComposeTemplatePicker from './ComposeTemplatePicker.vue'
import RichComposeEditor from './RichComposeEditor.vue'
import { useCommunicationsStore } from '../stores/communications'
import type { ComposeFormModel } from '../types/communications'
import {
  useDeleteDraftMutation,
  useSaveDraftMutation,
  useSendMailMutation
} from '../queries/useCommunicationsQuery'
import { useComposeDraftAutosave } from '../forms/composeDraftAutosave'
import { datetimeLocalToIso } from '../forms/composeDraftAutosave'
import { splitComposeRecipients, useComposeValidation } from '../forms/composeValidation'
import {
  appendHtmlSignature,
  appendPlainTextSignature,
  htmlToComposePlainText,
  plainTextToComposeHtml
} from './richComposeHtml'
import './ComposeDrawer.css'

const store = useCommunicationsStore()
const sendMailMutation = useSendMailMutation()
const saveDraftMutation = useSaveDraftMutation()
const deleteDraftMutation = useDeleteDraftMutation()

const isSaving = computed(() => saveDraftMutation.isPending.value)
const isSending = computed(() => sendMailMutation.isPending.value)
const { errors: composeValidationErrors, validateForSend } = useComposeValidation(() => store.composeForm)
const htmlEditorMode = ref<'rich' | 'source'>('rich')
const attachmentInput = ref<HTMLInputElement | null>(null)
type StagedComposeAttachment = {
  id: string
  file: File
  name: string
  size: number
  type: string
}
const stagedAttachments = ref<StagedComposeAttachment[]>([])
const hasStagedAttachments = computed(() => stagedAttachments.value.length > 0)

const draftAutosave = useComposeDraftAutosave({
  formSource: () => store.composeForm,
  saveDraft: (payload) => saveDraftMutation.mutateAsync(payload),
  onSaved: () => store.setComposeStatusMessage('Draft saved'),
  onError: (error) => {
    store.setComposeSendError(error instanceof Error ? error.message : 'Draft save failed')
  }
})

async function handleSaveDraft() {
  store.setComposeSendError('')
  await draftAutosave.saveNow()
}

function triggerAutoSave() {
  draftAutosave.schedule()
}

async function handleSend() {
  const form = store.composeForm
  if (isSending.value) return
  store.setComposeSendError('')
  if (hasStagedAttachments.value) {
    store.setComposeSendError('Attachment upload is not connected to provider send yet; remove staged attachments before sending')
    return
  }
  if (!(await validateForSend())) {
    store.setComposeSendError('Fix compose validation errors before sending')
    return
  }
  try {
    const result = await sendMailMutation.mutateAsync({
      account_id: form.accountId,
      to: splitComposeRecipients(form.toText),
      cc: splitComposeRecipients(form.ccText),
      bcc: splitComposeRecipients(form.bccText),
      subject: form.subject,
      body_text: form.body,
      body_html: form.bodyFormat === 'html' ? form.bodyHtml : null,
      in_reply_to: form.inReplyTo,
      draft_id: form.draftId,
      scheduled_send_at: datetimeLocalToIso(form.scheduledSendAt),
      undo_send_seconds: form.undoSendSeconds,
      confirmed_provider_write: true
    })
    store.setComposeStatusMessage(`Sent via ${result.transport ?? 'provider'}`)
    store.closeCompose()
  } catch (error) {
    store.setComposeSendError(error instanceof Error ? error.message : 'Send failed')
  }
}

async function handleDeleteCurrentDraft() {
  const draftId = store.composeForm.draftId?.trim()
  if (!draftId) return
  store.setComposeSendError('')
  draftAutosave.cancel()
  try {
    await deleteDraftMutation.mutateAsync(draftId)
    store.closeCompose()
  } catch (error) {
    store.setComposeSendError(error instanceof Error ? error.message : 'Delete failed')
  }
}

async function handleClose() {
  await draftAutosave.flush()
  stagedAttachments.value = []
  store.closeCompose()
}

function handleSheetOpenChange(open: boolean) {
  if (open) return
  void handleClose()
}

function updateField<K extends keyof ComposeFormModel>(key: K, value: ComposeFormModel[K]) {
  store.updateComposeForm({ [key]: value })
  if (key !== 'draftId' && key !== 'accountId') {
    triggerAutoSave()
  }
}

function setBodyFormat(format: ComposeFormModel['bodyFormat'], htmlMode: 'rich' | 'source' = 'rich') {
  if (format === 'html') {
    htmlEditorMode.value = htmlMode
  }
  updateField('bodyFormat', format)
  if (format === 'html' && store.composeForm.bodyHtml === null) {
    updateField('bodyHtml', htmlMode === 'rich'
      ? plainTextToComposeHtml(store.composeForm.body)
      : store.composeForm.body)
  }
}

function updateHtmlBody(value: string) {
  store.updateComposeForm({
    bodyHtml: value,
    body: htmlToComposePlainText(value)
  })
  triggerAutoSave()
}

function applyRenderedTemplate(payload: { subject: string; bodyHtml: string }) {
  htmlEditorMode.value = 'rich'
  store.updateComposeForm({
    subject: payload.subject,
    bodyFormat: 'html',
    bodyHtml: payload.bodyHtml,
    body: htmlToComposePlainText(payload.bodyHtml)
  })
  store.setComposeStatusMessage('Template applied')
  triggerAutoSave()
}

function applySignature(signature: string) {
  const trimmed = signature.trim()
  if (!trimmed) return

  if (store.composeForm.bodyFormat === 'html') {
    updateHtmlBody(appendHtmlSignature(store.composeForm.bodyHtml, trimmed))
  } else {
    updateField('body', appendPlainTextSignature(store.composeForm.body, trimmed))
  }
  store.setComposeStatusMessage('Signature inserted')
}

function handleAttachmentFiles(files: File[] | FileList) {
  const nextFiles = Array.from(files).filter((file) => file.size >= 0)
  if (nextFiles.length === 0) return
  const existingKeys = new Set(stagedAttachments.value.map((attachment) => attachment.id))
  const nextAttachments = nextFiles
    .map((file) => ({
      id: composeAttachmentId(file),
      file,
      name: file.name || 'Untitled attachment',
      size: file.size,
      type: file.type || 'application/octet-stream'
    }))
    .filter((attachment) => !existingKeys.has(attachment.id))
  if (nextAttachments.length === 0) {
    store.setComposeStatusMessage('Attachment already staged')
    return
  }
  stagedAttachments.value = [...stagedAttachments.value, ...nextAttachments]
  store.setComposeStatusMessage(`${nextAttachments.length} attachment${nextAttachments.length === 1 ? '' : 's'} staged locally`)
}

function handleAttachmentInput(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files) handleAttachmentFiles(input.files)
  input.value = ''
}

function handleAttachmentDrop(event: DragEvent) {
  handleAttachmentFiles(event.dataTransfer?.files ?? [])
}

function openAttachmentPicker() {
  attachmentInput.value?.click()
}

function removeStagedAttachment(id: string) {
  stagedAttachments.value = stagedAttachments.value.filter((attachment) => attachment.id !== id)
}

function composeAttachmentId(file: File): string {
  return `${file.name}:${file.size}:${file.lastModified}`
}

function formatAttachmentSize(size: number): string {
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / (1024 * 1024)).toFixed(1)} MB`
}

// Send review
const isReviewOpen = ref(false)

async function openSendReview() {
  store.setComposeSendError('')
  if (hasStagedAttachments.value) {
    store.setComposeSendError('Attachment upload is not connected to provider send yet; remove staged attachments before sending')
    return
  }
  if (!(await validateForSend())) {
    store.setComposeSendError('Fix compose validation errors before sending')
    return
  }
  isReviewOpen.value = true
}

function closeSendReview() {
  isReviewOpen.value = false
}

function confirmSend() {
  isReviewOpen.value = false
  void handleSend()
}

// Mode label
const modeLabel = computed(() => {
  switch (store.composeForm.mode) {
    case 'reply': return 'Reply'
    case 'forward': return 'Forward'
    default: return 'New Message'
  }
})

const deliveryActionLabel = computed(() => {
  return store.composeForm.scheduledSendAt ? 'Schedule' : 'Send'
})
const scheduledSendReviewLabel = computed(() => {
  if (!store.composeForm.scheduledSendAt) return ''
  const timestamp = Date.parse(store.composeForm.scheduledSendAt)
  if (!Number.isFinite(timestamp)) return store.composeForm.scheduledSendAt
  return new Intl.DateTimeFormat('en-US', {
    dateStyle: 'medium',
    timeStyle: 'short'
  }).format(new Date(timestamp))
})
const undoSendReviewLabel = computed(() => {
  return store.composeForm.undoSendSeconds
    ? `${store.composeForm.undoSendSeconds} seconds`
    : 'Off'
})
</script>

<template>
  <Sheet
    :open="true"
    side="right"
    :title="modeLabel"
    content-class="compose-drawer"
    @update:open="handleSheetOpenChange"
  >
    <template #header>
      <div class="compose-header-actions">
        <span v-if="isSaving" class="saving-indicator">
          <Icon icon="tabler:loader-2" class="spin-icon" /> Saving...
        </span>
      </div>
    </template>

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
      <div v-if="scheduledSendReviewLabel" class="review-field">
        <span class="review-label">Schedule:</span>
        <span class="review-value">{{ scheduledSendReviewLabel }}</span>
      </div>
      <div class="review-field">
        <span class="review-label">Undo:</span>
        <span class="review-value">{{ undoSendReviewLabel }}</span>
      </div>
      <div class="review-actions">
        <Button variant="default" @click="confirmSend" :disabled="isSending">
          <Icon icon="tabler:send" /> {{ isSending ? 'Sending...' : deliveryActionLabel }}
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
          <span v-if="composeValidationErrors.toText" class="field-error">
            {{ composeValidationErrors.toText }}
          </span>
        </div>
        <div class="form-field">
          <label>CC</label>
          <input
            type="text"
            placeholder="cc@example.com"
            :value="store.composeForm.ccText"
            @input="updateField('ccText', ($event.target as HTMLInputElement).value)"
          />
          <span v-if="composeValidationErrors.ccText" class="field-error">
            {{ composeValidationErrors.ccText }}
          </span>
        </div>
        <div class="form-field">
          <label>BCC</label>
          <input
            type="text"
            placeholder="bcc@example.com"
            :value="store.composeForm.bccText"
            @input="updateField('bccText', ($event.target as HTMLInputElement).value)"
          />
          <span v-if="composeValidationErrors.bccText" class="field-error">
            {{ composeValidationErrors.bccText }}
          </span>
        </div>
        <div class="form-field">
          <label>Subject</label>
          <input
            type="text"
            placeholder="Subject"
            :value="store.composeForm.subject"
            @input="updateField('subject', ($event.target as HTMLInputElement).value)"
          />
          <span v-if="composeValidationErrors.subject" class="field-error">
            {{ composeValidationErrors.subject }}
          </span>
        </div>
        <ComposeTemplatePicker
          :to-text="store.composeForm.toText"
          :cc-text="store.composeForm.ccText"
          :bcc-text="store.composeForm.bccText"
          :subject="store.composeForm.subject"
          :body="store.composeForm.body"
          :body-html="store.composeForm.bodyHtml"
          @apply="applyRenderedTemplate"
          @saved="(name) => store.setComposeStatusMessage(`Template saved: ${name}`)"
          @deleted="(name) => store.setComposeStatusMessage(`Template deleted: ${name}`)"
          @error="store.setComposeSendError"
        />
        <ComposeSignaturePicker
          @apply="applySignature"
        />
        <div class="form-field body-field">
          <div class="body-toolbar">
            <label>Message</label>
            <div class="body-mode-toggle" role="group" aria-label="Message format">
              <button
                type="button"
                :class="{ active: store.composeForm.bodyFormat === 'plain' }"
                @click="setBodyFormat('plain')"
              >
                Text
              </button>
              <button
                type="button"
                :class="{ active: store.composeForm.bodyFormat === 'html' && htmlEditorMode === 'rich' }"
                @click="setBodyFormat('html', 'rich')"
              >
                Rich
              </button>
              <button
                type="button"
                :class="{ active: store.composeForm.bodyFormat === 'html' && htmlEditorMode === 'source' }"
                @click="setBodyFormat('html', 'source')"
              >
                HTML
              </button>
            </div>
          </div>
          <textarea
            v-if="store.composeForm.bodyFormat === 'plain'"
            placeholder="Write your message..."
            :value="store.composeForm.body"
            @input="updateField('body', ($event.target as HTMLTextAreaElement).value)"
          />
          <RichComposeEditor
            v-else-if="htmlEditorMode === 'rich'"
            :model-value="store.composeForm.bodyHtml ?? ''"
            placeholder="Write your message..."
            @update:model-value="updateHtmlBody"
            @attachments-dropped="handleAttachmentFiles"
            @blur="triggerAutoSave"
          />
          <textarea
            v-else
            class="html-body-editor"
            placeholder="<p>Write your message...</p>"
            :value="store.composeForm.bodyHtml ?? ''"
            spellcheck="false"
            @input="updateHtmlBody(($event.target as HTMLTextAreaElement).value)"
          />
          <span v-if="composeValidationErrors.body" class="field-error">
            {{ composeValidationErrors.body }}
          </span>
        </div>

        <div class="compose-attachments">
          <div class="attachment-header">
            <span>Attachments</span>
            <button type="button" @click="openAttachmentPicker">
              <Icon icon="tabler:paperclip" size="16" /> Add
            </button>
            <input
              ref="attachmentInput"
              class="attachment-input"
              type="file"
              multiple
              @change="handleAttachmentInput"
            />
          </div>
          <div class="attachment-drop-zone" @dragover.prevent @drop.prevent="handleAttachmentDrop">
            <Icon icon="tabler:paperclip" size="16" />
            <span>Drop files here or use Add</span>
          </div>
          <ul v-if="stagedAttachments.length > 0" class="attachment-list">
            <li v-for="attachment in stagedAttachments" :key="attachment.id">
              <span class="attachment-name">{{ attachment.name }}</span>
              <span class="attachment-meta">{{ formatAttachmentSize(attachment.size) }}</span>
              <button type="button" title="Remove attachment" @click="removeStagedAttachment(attachment.id)">
                <Icon icon="tabler:x" size="14" />
              </button>
            </li>
          </ul>
          <p v-if="hasStagedAttachments" class="attachment-warning">
            Attachment upload is not connected to provider send yet. Remove staged attachments before sending.
          </p>
        </div>

        <div class="delivery-options">
          <label class="delivery-field">
            <span>Schedule</span>
            <input
              type="datetime-local"
              :value="store.composeForm.scheduledSendAt"
              @input="updateField('scheduledSendAt', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <label class="delivery-field">
            <span>Undo</span>
            <select
              :value="store.composeForm.undoSendSeconds ?? ''"
              @change="updateField('undoSendSeconds', ($event.target as HTMLSelectElement).value ? Number(($event.target as HTMLSelectElement).value) : null)"
            >
              <option value="">Off</option>
              <option value="10">10 seconds</option>
              <option value="30">30 seconds</option>
              <option value="60">60 seconds</option>
            </select>
          </label>
        </div>

        <div v-if="store.composeSendError" class="compose-error">
          {{ store.composeSendError }}
        </div>
        <div v-if="store.composeStatusMessage" class="compose-status">
          {{ store.composeStatusMessage }}
        </div>

        <div class="compose-actions">
          <Button variant="default" @click="openSendReview" :disabled="!store.composeForm.toText">
            <Icon icon="tabler:send" /> {{ deliveryActionLabel }}
          </Button>
          <Button variant="ghost" @click="handleSaveDraft" :disabled="isSaving">
            <Icon icon="tabler:edit" /> Save Draft
          </Button>
          <Button variant="ghost" @click="handleDeleteCurrentDraft" class="delete-btn">
            <Icon icon="tabler:trash" /> Discard
          </Button>
        </div>
    </div>
  </Sheet>
</template>
