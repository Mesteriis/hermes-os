<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { FlexRender, getCoreRowModel, useVueTable } from '@tanstack/vue-table'
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationMessageDetailResponse } from '../types/communications'
import { attachmentIcon } from '../stores/communications'
import {
  useAttachmentArchiveInspectionQuery,
  useAttachmentPreviewQuery,
  useTranslateAttachmentMutation
} from '../queries/useCommunicationsQuery'
import type { AttachmentTranslationResponse } from '../types/attachments'
import {
  attachmentTableColumns,
  attachmentTableRowId,
  formatAttachmentSize,
  isInspectableArchiveAttachment,
  isPreviewableAttachment,
  isPreviewableImageAttachment,
  scanStatusClass
} from './attachmentTable'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
}>()

const attachments = computed(() => props.detail?.attachments ?? [])
const selectedArchiveAttachmentId = ref<string | null>(null)
const selectedArchiveAttachment = computed(() =>
  attachments.value.find((attachment) => attachment.attachment_id === selectedArchiveAttachmentId.value) ?? null
)
const selectedPreviewAttachmentId = ref<string | null>(null)
const selectedPreviewAttachment = computed(() =>
  attachments.value.find((attachment) => attachment.attachment_id === selectedPreviewAttachmentId.value) ?? null
)
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
  () => selectedArchiveAttachmentId.value,
  () => Boolean(selectedArchiveAttachmentId.value)
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
  () => selectedPreviewAttachmentId.value,
  () => Boolean(selectedPreviewAttachmentId.value)
)
const attachmentPreview = computed(() => attachmentPreviewData.value)
const attachmentPreviewErrorMessage = computed(() => {
  if (!attachmentPreviewError.value) return ''
  return attachmentPreviewError.value instanceof Error
    ? attachmentPreviewError.value.message
    : 'Attachment preview failed'
})

const table = useVueTable({
  get data() {
    return attachments.value
  },
  columns: attachmentTableColumns,
  getCoreRowModel: getCoreRowModel(),
  getRowId: attachmentTableRowId
})

watch(attachments, (items) => {
  if (
    selectedArchiveAttachmentId.value
    && !items.some((attachment) => attachment.attachment_id === selectedArchiveAttachmentId.value)
  ) {
    selectedArchiveAttachmentId.value = null
  }
  if (
    selectedPreviewAttachmentId.value
    && !items.some((attachment) => attachment.attachment_id === selectedPreviewAttachmentId.value)
  ) {
    selectedPreviewAttachmentId.value = null
    attachmentTranslationResult.value = null
    attachmentTranslationError.value = ''
  }
})

function inspectArchive(attachmentId: string) {
  selectedArchiveAttachmentId.value = attachmentId
}

function showAttachmentPreview(attachmentId: string) {
  selectedPreviewAttachmentId.value = attachmentId
  attachmentTranslationResult.value = null
  attachmentTranslationError.value = ''
}

async function translateSelectedAttachment() {
  const attachmentId = selectedPreviewAttachmentId.value
  const preview = attachmentPreview.value
  if (!attachmentId || !preview?.text.trim()) return

  attachmentTranslationError.value = ''
  try {
    attachmentTranslationResult.value = await translateAttachmentMutation.mutateAsync({
      attachmentId,
      request: {
        target_language: attachmentTranslationTarget.value,
        source_text: preview.text
      }
    })
  } catch (e) {
    attachmentTranslationError.value = e instanceof Error ? e.message : 'Attachment translation failed'
  }
}
</script>

<template>
  <div class="attachments-tab">
    <div v-if="attachments.length === 0" class="no-data">No attachments</div>
    <div v-else class="attachment-table-shell">
      <table class="attachment-table">
        <thead>
          <tr v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id">
            <th v-for="header in headerGroup.headers" :key="header.id">
              <FlexRender
                v-if="!header.isPlaceholder"
                :render="header.column.columnDef.header"
                :props="header.getContext()"
              />
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in table.getRowModel().rows" :key="row.id">
            <td
              v-for="cell in row.getVisibleCells()"
              :key="cell.id"
              :class="`attachment-cell attachment-cell--${cell.column.id}`"
            >
              <div v-if="cell.column.id === 'filename'" class="att-file">
                <Icon :icon="attachmentIcon(row.original.content_type)" class="att-icon" />
                <span class="att-filename">{{ row.original.filename || 'Unnamed' }}</span>
                <div class="att-actions">
                  <button
                    v-if="isPreviewableAttachment(row.original)"
                    class="att-inspect"
                    type="button"
                    :disabled="isAttachmentPreviewFetching && selectedPreviewAttachmentId === row.original.attachment_id"
                    @click="showAttachmentPreview(row.original.attachment_id)"
                  >
                    {{ isPreviewableImageAttachment(row.original) ? 'Preview image' : 'Preview' }}
                  </button>
                  <button
                    v-if="isInspectableArchiveAttachment(row.original)"
                    class="att-inspect"
                    type="button"
                    :disabled="isArchiveInspectionFetching && selectedArchiveAttachmentId === row.original.attachment_id"
                    @click="inspectArchive(row.original.attachment_id)"
                  >
                    Inspect archive
                  </button>
                </div>
              </div>
              <span v-else-if="cell.column.id === 'size'">
                {{ formatAttachmentSize(row.original.size_bytes) }}
              </span>
              <span
                v-else-if="cell.column.id === 'scan_status'"
                class="att-scan"
                :class="scanStatusClass(row.original.scan_status)"
              >
                {{ row.original.scan_status }}
              </span>
              <span v-else>{{ cell.getValue() }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <section
      v-if="selectedPreviewAttachment"
      class="attachment-preview-panel"
      aria-label="Attachment preview"
    >
      <div class="attachment-preview-header">
        <div>
          <h4>Attachment preview</h4>
          <p>{{ selectedPreviewAttachment.filename || 'Unnamed attachment' }}</p>
        </div>
        <span class="att-scan" :class="scanStatusClass(selectedPreviewAttachment.scan_status)">
          {{ selectedPreviewAttachment.scan_status }}
        </span>
      </div>
      <p v-if="isAttachmentPreviewFetching" class="attachment-preview-muted">Loading safe attachment preview...</p>
      <p v-else-if="attachmentPreviewErrorMessage" class="attachment-preview-error">
        {{ attachmentPreviewErrorMessage }}
      </p>
      <div v-else-if="attachmentPreview" class="attachment-preview-report">
        <div class="attachment-preview-stats">
          <span>{{ formatAttachmentSize(attachmentPreview.byte_count) }}</span>
          <span v-if="attachmentPreview.truncated">
            Truncated to {{ formatAttachmentSize(attachmentPreview.max_preview_bytes) }}
          </span>
          <span v-if="attachmentPreview.preview_kind === 'image'">Image preview</span>
          <label v-if="attachmentPreview.preview_kind === 'text'" class="attachment-translation-target">
            <span>Translate</span>
            <select v-model="attachmentTranslationTarget">
              <option value="en">EN</option>
              <option value="ru">RU</option>
              <option value="es">ES</option>
            </select>
          </label>
          <button
            v-if="attachmentPreview.preview_kind === 'text'"
            class="att-inspect"
            type="button"
            :disabled="isAttachmentTranslationPending || !attachmentPreview.text.trim()"
            @click="translateSelectedAttachment"
          >
            {{ isAttachmentTranslationPending ? 'Translating' : 'Translate preview' }}
          </button>
        </div>
        <img
          v-if="attachmentPreview.preview_kind === 'image' && attachmentPreview.data_url"
          class="attachment-preview-image"
          :src="attachmentPreview.data_url"
          :alt="selectedPreviewAttachment.filename || 'Attachment image preview'"
        >
        <pre v-else class="attachment-preview-text">{{ attachmentPreview.text }}</pre>
        <section
          v-if="attachmentTranslationResult || attachmentTranslationError"
          class="attachment-translation-panel"
          aria-label="Attachment translation"
        >
          <div class="attachment-translation-header">
            <h4>Attachment translation</h4>
            <span v-if="attachmentTranslationResult">
              {{ attachmentTranslationResult.translated ? 'Translated' : 'Fallback' }}
            </span>
          </div>
          <p v-if="attachmentTranslationError" class="attachment-preview-error">
            {{ attachmentTranslationError }}
          </p>
          <p v-else-if="attachmentTranslationResult?.text" class="attachment-translation-text">
            {{ attachmentTranslationResult.text }}
          </p>
          <p v-else class="attachment-preview-muted">
            {{ attachmentTranslationResult?.reason ?? 'Translation unavailable' }}
          </p>
        </section>
      </div>
    </section>
    <section
      v-if="selectedArchiveAttachment"
      class="archive-inspection-panel"
      aria-label="Archive inspection"
    >
      <div class="archive-inspection-header">
        <div>
          <h4>Archive inspection</h4>
          <p>{{ selectedArchiveAttachment.filename || 'Unnamed archive' }}</p>
        </div>
        <span class="att-scan" :class="scanStatusClass(selectedArchiveAttachment.scan_status)">
          {{ selectedArchiveAttachment.scan_status }}
        </span>
      </div>
      <p v-if="isArchiveInspectionFetching" class="archive-inspection-muted">Inspecting archive metadata...</p>
      <p v-else-if="archiveInspectionErrorMessage" class="archive-inspection-error">
        {{ archiveInspectionErrorMessage }}
      </p>
      <div v-else-if="archiveInspection" class="archive-inspection-report">
        <div class="archive-inspection-stats">
          <span>{{ archiveInspection.report.entry_count }} entries</span>
          <span>{{ formatAttachmentSize(archiveInspection.report.total_uncompressed_bytes) }}</span>
          <span v-if="archiveInspection.report.has_nested_archive">Nested archive</span>
        </div>
        <ul>
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
.attachments-tab {
  padding: 0.75rem;
  min-width: 0;
}

.attachment-table-shell {
  overflow-x: auto;
  background: var(--hh-bg-secondary, #f9fafb);
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
}

.attachment-table {
  width: 100%;
  min-width: 520px;
  border-collapse: collapse;
  font-size: 0.75rem;
}

.attachment-table th,
.attachment-table td {
  padding: 0.5rem 0.625rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  text-align: left;
  vertical-align: middle;
}

.attachment-table th {
  color: var(--hh-text-secondary, #6b7280);
  font-weight: 600;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 70%, transparent);
}

.attachment-table tbody tr:last-child td {
  border-bottom: none;
}

.attachment-cell--filename {
  width: 44%;
}

.attachment-cell--content_type {
  color: var(--hh-text-tertiary, #9ca3af);
}

.attachment-cell--size {
  white-space: nowrap;
  color: var(--hh-text-secondary, #6b7280);
}

.att-file {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.att-icon {
  width: 20px;
  height: 20px;
  color: var(--hh-accent, #3b82f6);
  flex-shrink: 0;
}

.att-filename {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.att-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 0.375rem;
  flex-shrink: 0;
}

.att-inspect {
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

.att-inspect:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.att-scan {
  font-size: 0.6875rem;
  font-weight: 500;
  white-space: nowrap;
  flex-shrink: 0;
}

.att-scan--clean {
  color: var(--hh-status-success-text, #16a34a);
}

.att-scan--suspicious {
  color: var(--hh-status-warning-text, #f59e0b);
}

.att-scan--danger {
  color: var(--hh-status-danger-text, #ef4444);
}

.att-scan--unknown {
  color: var(--hh-text-muted, #9ca3af);
}

.no-data {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
  padding: 2rem;
  text-align: center;
}

.archive-inspection-panel,
.attachment-preview-panel {
  margin-top: 0.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 82%, transparent);
  padding: 0.75rem;
}

.archive-inspection-header,
.attachment-preview-header,
.archive-inspection-stats,
.attachment-preview-stats,
.archive-inspection-report li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.archive-inspection-header h4,
.attachment-preview-header h4 {
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.archive-inspection-header p,
.attachment-preview-header p,
.archive-inspection-muted,
.attachment-preview-muted,
.archive-inspection-error,
.attachment-preview-error {
  margin: 0.25rem 0 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.archive-inspection-error,
.attachment-preview-error {
  color: var(--hh-status-danger-text, #ef4444);
}

.archive-inspection-report,
.attachment-preview-report {
  margin-top: 0.625rem;
}

.archive-inspection-stats,
.attachment-preview-stats {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  justify-content: flex-start;
  flex-wrap: wrap;
}

.attachment-translation-target {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
}

.attachment-translation-target select {
  min-height: 1.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.25rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  color: var(--hh-text-primary, #1f2937);
  font: inherit;
}

.archive-inspection-report ul {
  list-style: none;
  margin: 0.625rem 0 0;
  max-height: 10rem;
  overflow: auto;
  padding: 0;
}

.archive-inspection-report li {
  border-top: 1px solid var(--hh-border, #e5e7eb);
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
  min-width: 0;
  padding: 0.4375rem 0;
}

.archive-inspection-report li span:first-child {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.attachment-preview-text {
  margin: 0.625rem 0 0;
  max-height: 12rem;
  overflow: auto;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.25rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 78%, transparent);
  color: var(--hh-text-primary, #1f2937);
  font-family: var(--hh-font-mono, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace);
  font-size: 0.75rem;
  line-height: 1.45;
  padding: 0.625rem;
  white-space: pre-wrap;
  word-break: break-word;
}

.attachment-preview-image {
  display: block;
  width: 100%;
  max-height: 18rem;
  margin-top: 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.25rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 78%, transparent);
  object-fit: contain;
}

.attachment-translation-panel {
  margin-top: 0.625rem;
  border: 1px solid color-mix(in srgb, var(--hh-accent, #3b82f6) 24%, var(--hh-border, #e5e7eb));
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-accent, #3b82f6) 7%, transparent);
  padding: 0.625rem;
}

.attachment-translation-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.attachment-translation-header h4 {
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.attachment-translation-header span {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0;
}

.attachment-translation-text {
  margin: 0.5rem 0 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  line-height: 1.5;
  white-space: pre-wrap;
}
</style>
