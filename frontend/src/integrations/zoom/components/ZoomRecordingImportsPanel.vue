<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useSettingsStore } from '../../../shared/zoom/settingsBridge'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import {
  useRemoveZoomRecordingImportMutation,
  useZoomRecordingImportsQuery,
} from '../queries/useZoomRuntimeQuery'

const { t } = useI18n()
const store = useSettingsStore()

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const recordingImportsQuery = useZoomRecordingImportsQuery(
  computed(() => props.selectedAccount?.account_id ?? null),
  12
)
const removeRecordingImport = useRemoveZoomRecordingImportMutation(
  computed(() => props.selectedAccount?.account_id ?? null)
)
const recordingImports = computed(() => recordingImportsQuery.data.value ?? [])

function formatDate(value: string | null | undefined): string {
  if (!value) return '—'
  const parsed = new Date(value)
  if (Number.isNaN(parsed.getTime())) return '—'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(parsed)
}

function formatBytes(value: number | null | undefined): string {
  if (typeof value !== 'number' || !Number.isFinite(value) || value < 0) return '—'
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`
  return `${(value / (1024 * 1024)).toFixed(1)} MB`
}

function shortHash(value: string): string {
  return value.length > 18 ? `${value.slice(0, 10)}…${value.slice(-6)}` : value
}

function formatRetention(item: { retention_mode: string; retention_days: number }): string {
  if (item.retention_mode === 'delete_after_n_days' && item.retention_days > 0) {
    return `${item.retention_days}d`
  }
  return t('Manual only')
}

async function handleRemoveImport(attachmentId: string, recordingId?: string | null) {
  const accountId = props.selectedAccount?.account_id
  if (!accountId) return
  const confirmed = window.confirm(
    t('Remove this imported Zoom recording blob from local storage and audit views?')
  )
  if (!confirmed) return

  try {
    const result = await removeRecordingImport.mutateAsync({
      attachmentId,
      request: {
        reason: recordingId
          ? `operator_removed_recording:${recordingId}`
          : 'operator_removed_recording_import',
      },
    })
    store.setActionMessage(
      result.blob_file_removed
        ? t('Zoom recording import removed from local storage')
        : t('Zoom recording import metadata removed')
    )
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom recording import removal failed')
  }
}
</script>

<template>
  <section class="integration-section zoom-recording-imports">
    <header class="zoom-recording-imports__header">
      <div>
        <h4>{{ t('Recording import audit') }}</h4>
        <p>{{ t('Imported Zoom recording blobs for the selected account.') }}</p>
      </div>
      <span class="zoom-recording-imports__count">{{ recordingImports.length }}</span>
    </header>

    <div v-if="!selectedAccount?.account_id" class="zoom-recording-imports__placeholder">
      {{ t('Select a Zoom account to inspect imported recording files.') }}
    </div>
    <div
      v-else-if="recordingImportsQuery.isLoading.value"
      class="zoom-recording-imports__placeholder"
    >
      {{ t('Loading imported Zoom recording files...') }}
    </div>
    <div v-else-if="recordingImports.length === 0" class="zoom-recording-imports__placeholder">
      {{ t('No imported Zoom recording files for this account yet.') }}
    </div>
    <div v-else class="zoom-recording-imports__list">
      <article
        v-for="item in recordingImports"
        :key="item.attachment_id"
        class="zoom-recording-imports__item"
      >
        <header>
          <strong>{{ item.recording_id ?? item.attachment_id }}</strong>
          <small>{{ formatDate(item.created_at) }}</small>
        </header>
        <dl class="zoom-recording-imports__meta">
          <div><dt>{{ t('Meeting id') }}</dt><dd>{{ item.meeting_id ?? '—' }}</dd></div>
          <div><dt>{{ t('Source') }}</dt><dd>{{ item.source ?? '—' }}</dd></div>
          <div><dt>{{ t('Filename') }}</dt><dd>{{ item.filename ?? '—' }}</dd></div>
          <div><dt>{{ t('Content type') }}</dt><dd>{{ item.content_type }}</dd></div>
          <div><dt>{{ t('Size') }}</dt><dd>{{ formatBytes(item.size_bytes) }}</dd></div>
          <div><dt>{{ t('Scan') }}</dt><dd>{{ item.scan_status }}</dd></div>
          <div><dt>{{ t('Storage') }}</dt><dd>{{ item.storage_kind }}</dd></div>
          <div><dt>{{ t('Retention') }}</dt><dd>{{ formatRetention(item) }}</dd></div>
          <div><dt>{{ t('Expires') }}</dt><dd>{{ formatDate(item.expires_at) }}</dd></div>
          <div><dt>{{ t('SHA-256') }}</dt><dd>{{ shortHash(item.sha256) }}</dd></div>
        </dl>
        <p v-if="item.scan_summary" class="zoom-recording-imports__summary">{{ item.scan_summary }}</p>
        <div class="zoom-recording-imports__actions">
          <button
            type="button"
            class="zoom-recording-imports__remove"
            :disabled="removeRecordingImport.isPending.value"
            @click="handleRemoveImport(item.attachment_id, item.recording_id)"
          >
            {{ t('Remove local import') }}
          </button>
        </div>
      </article>
    </div>
  </section>
</template>

<style scoped>
.zoom-recording-imports { display: grid; gap: 12px; }
.zoom-recording-imports__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.zoom-recording-imports__header h4,
.zoom-recording-imports__header p,
.zoom-recording-imports__item header {
  margin: 0;
}
.zoom-recording-imports__header p,
.zoom-recording-imports__meta dt,
.zoom-recording-imports__item small {
  color: var(--hh-text-muted);
  font-size: 11px;
}
.zoom-recording-imports__actions {
  display: flex;
  justify-content: flex-end;
}
.zoom-recording-imports__count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  min-height: 24px;
  padding: 0 8px;
  border-radius: 999px;
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
  font-size: 11px;
  font-weight: 600;
}
.zoom-recording-imports__list,
.zoom-recording-imports__item {
  display: grid;
  gap: 8px;
}
.zoom-recording-imports__placeholder,
.zoom-recording-imports__item {
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
  font-size: 12px;
}
.zoom-recording-imports__item header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.zoom-recording-imports__item strong {
  display: block;
  font-size: 12px;
}
.zoom-recording-imports__meta {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin: 0;
}
.zoom-recording-imports__meta div {
  display: grid;
  gap: 2px;
}
.zoom-recording-imports__meta dt,
.zoom-recording-imports__meta dd,
.zoom-recording-imports__summary {
  margin: 0;
  word-break: break-word;
}
.zoom-recording-imports__remove {
  min-height: 28px;
  padding: 0 10px;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--hh-danger, #c84b4b) 55%, var(--hh-border) 45%);
  background: color-mix(in srgb, var(--hh-surface) 85%, white 15%);
  color: var(--hh-text);
  font-size: 12px;
}
.zoom-recording-imports__remove:disabled {
  opacity: 0.6;
}
@media (max-width: 900px) {
  .zoom-recording-imports__meta {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
