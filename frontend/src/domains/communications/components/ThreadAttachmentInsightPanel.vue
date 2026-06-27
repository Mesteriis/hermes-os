<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
  useAttachmentArchiveInspectionQuery,
  useAttachmentPreviewQuery,
  useTranslateAttachmentMutation
} from '../queries/useCommunicationsQuery'
import type { CommunicationAttachment } from '../types/communications'
import type { AttachmentTranslationResponse } from '../types/attachments'
import {
  formatAttachmentSize,
  isInspectableArchiveAttachment,
  isPreviewableAttachment,
  isPreviewablePdfAttachment,
  isPreviewableImageAttachment,
  scanStatusClass
} from './attachmentTable'

const props = defineProps<{
  attachment: CommunicationAttachment
}>()

const panelMode = ref<'preview' | 'archive' | ''>('')
const attachmentTranslationTarget = ref('en')
const attachmentTranslationResult = ref<AttachmentTranslationResponse | null>(null)
const attachmentTranslationError = ref('')
const translateAttachmentMutation = useTranslateAttachmentMutation()
const isAttachmentTranslationPending = computed(() => translateAttachmentMutation.isPending.value)

const {
  data: archiveInspectionData,
  error: archiveInspectionError,
  isFetching: isArchiveInspectionFetching
} = useAttachmentArchiveInspectionQuery(
  () => panelMode.value === 'archive' ? props.attachment.attachment_id : null,
  () => panelMode.value === 'archive'
)
const archiveInspection = computed(() => archiveInspectionData.value)
const archiveInspectionErrorMessage = computed(() => {
  if (!archiveInspectionError.value) return ''
  return archiveInspectionError.value instanceof Error
    ? archiveInspectionError.value.message
    : 'Archive inspection failed'
})

const {
  data: attachmentPreviewData,
  error: attachmentPreviewError,
  isFetching: isAttachmentPreviewFetching
} = useAttachmentPreviewQuery(
  () => panelMode.value === 'preview' ? props.attachment.attachment_id : null,
  () => panelMode.value === 'preview'
)
const attachmentPreview = computed(() => attachmentPreviewData.value)
const attachmentPreviewErrorMessage = computed(() => {
  if (!attachmentPreviewError.value) return ''
  return attachmentPreviewError.value instanceof Error
    ? attachmentPreviewError.value.message
    : 'Attachment preview failed'
})

const canPreviewAttachment = computed(() => isPreviewableAttachment(props.attachment))
const canInspectArchive = computed(() => isInspectableArchiveAttachment(props.attachment))

function openPreview(): void {
  panelMode.value = panelMode.value === 'preview' ? '' : 'preview'
  attachmentTranslationResult.value = null
  attachmentTranslationError.value = ''
}

function openArchiveInspection(): void {
  panelMode.value = panelMode.value === 'archive' ? '' : 'archive'
}

async function translateAttachmentPreview(): Promise<void> {
  const preview = attachmentPreview.value
  if (!preview?.text.trim()) return

  attachmentTranslationError.value = ''
  try {
    attachmentTranslationResult.value = await translateAttachmentMutation.mutateAsync({
      attachmentId: props.attachment.attachment_id,
      request: {
        target_language: attachmentTranslationTarget.value,
        source_text: preview.text
      }
    })
  } catch (error) {
    attachmentTranslationError.value = error instanceof Error
      ? error.message
      : 'Attachment translation failed'
  }
}
</script>

<template>
  <div
    v-if="canPreviewAttachment || canInspectArchive || panelMode"
    class="thread-attachment-insight"
  >
    <div class="thread-attachment-actions">
      <button
        v-if="canPreviewAttachment"
        class="thread-attachment-action"
        type="button"
        @click="openPreview"
      >
        {{
          panelMode === 'preview'
            ? 'Hide preview'
            : (
                isPreviewableImageAttachment(attachment)
                  ? 'Preview image'
                  : isPreviewablePdfAttachment(attachment)
                    ? 'Preview PDF'
                    : 'Preview'
              )
        }}
      </button>
      <button
        v-if="canInspectArchive"
        class="thread-attachment-action"
        type="button"
        @click="openArchiveInspection"
      >
        {{ panelMode === 'archive' ? 'Hide archive' : 'Inspect archive' }}
      </button>
    </div>

    <section
      v-if="panelMode === 'preview'"
      class="thread-attachment-panel"
      aria-label="Thread attachment preview"
    >
      <div class="thread-attachment-panel-header">
        <div>
          <h4>Attachment preview</h4>
          <p>{{ attachment.filename || 'Unnamed attachment' }}</p>
        </div>
        <span class="thread-attachment-scan" :class="scanStatusClass(attachment.scan_status)">
          {{ attachment.scan_status }}
        </span>
      </div>
      <p v-if="isAttachmentPreviewFetching" class="thread-attachment-muted">Loading safe attachment preview...</p>
      <p v-else-if="attachmentPreviewErrorMessage" class="thread-attachment-error">{{ attachmentPreviewErrorMessage }}</p>
      <div v-else-if="attachmentPreview" class="thread-attachment-report">
        <div class="thread-attachment-stats">
          <span>{{ formatAttachmentSize(attachmentPreview.byte_count) }}</span>
          <span v-if="attachmentPreview.truncated">
            Truncated to {{ formatAttachmentSize(attachmentPreview.max_preview_bytes) }}
          </span>
          <span v-if="attachmentPreview.preview_kind === 'image'">Image preview</span>
          <span v-else-if="attachmentPreview.preview_kind === 'audio'">Audio preview</span>
          <span v-else-if="attachmentPreview.preview_kind === 'video'">Video preview</span>
          <span v-else-if="attachmentPreview.preview_kind === 'pdf'">PDF preview</span>
          <label v-if="attachmentPreview.preview_kind === 'text'" class="thread-attachment-translation-target">
            <span>Translate</span>
            <select v-model="attachmentTranslationTarget">
              <option value="en">EN</option>
              <option value="ru">RU</option>
              <option value="es">ES</option>
            </select>
          </label>
          <button
            v-if="attachmentPreview.preview_kind === 'text'"
            class="thread-attachment-action"
            type="button"
            :disabled="isAttachmentTranslationPending || !attachmentPreview.text.trim()"
            @click="translateAttachmentPreview"
          >
            {{ isAttachmentTranslationPending ? 'Translating' : 'Translate preview' }}
          </button>
        </div>
        <img
          v-if="attachmentPreview.preview_kind === 'image' && attachmentPreview.data_url"
          class="thread-attachment-image"
          :src="attachmentPreview.data_url"
          :alt="attachment.filename || 'Attachment image preview'"
        >
        <audio
          v-else-if="attachmentPreview.preview_kind === 'audio' && attachmentPreview.data_url"
          class="thread-attachment-media"
          controls
          preload="metadata"
          :src="attachmentPreview.data_url"
        />
        <video
          v-else-if="attachmentPreview.preview_kind === 'video' && attachmentPreview.data_url"
          class="thread-attachment-media"
          controls
          preload="metadata"
          :src="attachmentPreview.data_url"
        />
        <iframe
          v-else-if="attachmentPreview.preview_kind === 'pdf' && attachmentPreview.data_url"
          class="thread-attachment-document"
          :src="attachmentPreview.data_url"
          :title="attachment.filename || 'Attachment PDF preview'"
        />
        <pre v-else class="thread-attachment-text">{{ attachmentPreview.text }}</pre>
        <section
          v-if="attachmentTranslationResult || attachmentTranslationError"
          class="thread-attachment-translation"
          aria-label="Thread attachment translation"
        >
          <div class="thread-attachment-panel-header compact">
            <h4>Attachment translation</h4>
            <span v-if="attachmentTranslationResult">
              {{ attachmentTranslationResult.translated ? 'Translated' : 'Fallback' }}
            </span>
          </div>
          <p v-if="attachmentTranslationError" class="thread-attachment-error">{{ attachmentTranslationError }}</p>
          <p v-else-if="attachmentTranslationResult?.text" class="thread-attachment-translation-text">
            {{ attachmentTranslationResult.text }}
          </p>
          <p v-else class="thread-attachment-muted">
            {{ attachmentTranslationResult?.reason ?? 'Translation unavailable' }}
          </p>
        </section>
      </div>
    </section>

    <section
      v-if="panelMode === 'archive'"
      class="thread-attachment-panel"
      aria-label="Thread archive inspection"
    >
      <div class="thread-attachment-panel-header">
        <div>
          <h4>Archive inspection</h4>
          <p>{{ attachment.filename || 'Unnamed archive' }}</p>
        </div>
        <span class="thread-attachment-scan" :class="scanStatusClass(attachment.scan_status)">
          {{ attachment.scan_status }}
        </span>
      </div>
      <p v-if="isArchiveInspectionFetching" class="thread-attachment-muted">Inspecting archive metadata...</p>
      <p v-else-if="archiveInspectionErrorMessage" class="thread-attachment-error">{{ archiveInspectionErrorMessage }}</p>
      <div v-else-if="archiveInspection" class="thread-attachment-report">
        <div class="thread-attachment-stats">
          <span>{{ archiveInspection.report.entry_count }} entries</span>
          <span>{{ formatAttachmentSize(archiveInspection.report.total_uncompressed_bytes) }}</span>
          <span v-if="archiveInspection.report.has_nested_archive">Nested archive</span>
        </div>
        <ul class="thread-attachment-archive-list">
          <li v-for="entry in archiveInspection.report.entries" :key="entry.normalized_path">
            <span>{{ entry.normalized_path }}</span>
            <span>{{ formatAttachmentSize(entry.uncompressed_size) }}</span>
          </li>
        </ul>
      </div>
    </section>
  </div>
</template>

<style scoped>
.thread-attachment-insight {
  display: grid;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.thread-attachment-actions,
.thread-attachment-stats,
.thread-attachment-panel-header,
.thread-attachment-archive-list li {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.thread-attachment-panel-header {
  justify-content: space-between;
}

.thread-attachment-panel-header.compact {
  justify-content: space-between;
}

.thread-attachment-action {
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.25rem;
  background: color-mix(in srgb, var(--hh-accent, #3b82f6) 12%, transparent);
  color: var(--hh-accent, #3b82f6);
  cursor: pointer;
  font: inherit;
  font-size: 0.6875rem;
  padding: 0.1875rem 0.375rem;
  white-space: nowrap;
}

.thread-attachment-action:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.thread-attachment-panel {
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 82%, transparent);
  padding: 0.75rem;
}

.thread-attachment-panel h4 {
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.thread-attachment-panel p,
.thread-attachment-muted,
.thread-attachment-error {
  margin: 0.25rem 0 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.thread-attachment-error {
  color: var(--hh-status-danger-text, #ef4444);
}

.thread-attachment-scan {
  font-size: 0.6875rem;
  font-weight: 500;
  white-space: nowrap;
}

.thread-attachment-report {
  display: grid;
  gap: 0.625rem;
  margin-top: 0.625rem;
}

.thread-attachment-stats {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.thread-attachment-translation-target {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
}

.thread-attachment-translation-target select {
  min-height: 1.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.25rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  color: var(--hh-text-primary, #1f2937);
  font: inherit;
}

.thread-attachment-image {
  max-width: min(100%, 28rem);
  max-height: 20rem;
  border-radius: 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
}

.thread-attachment-media {
  width: 100%;
  max-width: min(100%, 28rem);
  max-height: 20rem;
  border-radius: 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 92%, transparent);
}

.thread-attachment-document {
  width: 100%;
  max-width: min(100%, 36rem);
  min-height: 20rem;
  border-radius: 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 92%, transparent);
}

.thread-attachment-text {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font: 0.75rem/1.45 ui-monospace, SFMono-Regular, Menlo, monospace;
  color: var(--hh-text-primary, #1f2937);
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  padding: 0.625rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 92%, transparent);
  max-height: 14rem;
  overflow: auto;
}

.thread-attachment-translation {
  display: grid;
  gap: 0.5rem;
  border-top: 1px solid var(--hh-border, #e5e7eb);
  padding-top: 0.625rem;
}

.thread-attachment-translation-text {
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  line-height: 1.5;
}

.thread-attachment-archive-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 10rem;
  overflow: auto;
}

.thread-attachment-archive-list li {
  justify-content: space-between;
  border-top: 1px solid var(--hh-border, #e5e7eb);
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
  min-width: 0;
  padding: 0.4375rem 0;
}

.thread-attachment-archive-list li span:first-child {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
