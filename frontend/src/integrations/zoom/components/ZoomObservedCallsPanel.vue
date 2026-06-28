<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import { useZoomCallTranscriptQuery, useZoomProviderCallsQuery } from '../queries/useZoomRuntimeQuery'
import { extractZoomRecordingRefs, formatZoomTranscriptProvenance } from './zoomEvidence'

const { t } = useI18n()

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const selectedCallId = ref<string | null>(null)
const providerCallsQuery = useZoomProviderCallsQuery(
  computed(() => props.selectedAccount?.account_id ?? null),
  12
)
const calls = computed(() => providerCallsQuery.data.value ?? [])
const transcriptQuery = useZoomCallTranscriptQuery(selectedCallId)
const selectedCall = computed(
  () => calls.value.find((call) => call.call_id === selectedCallId.value) ?? calls.value[0] ?? null
)
const selectedTranscript = computed(() => transcriptQuery.data.value)
const selectedRecordingRefs = computed(() => extractZoomRecordingRefs(selectedCall.value?.metadata))
const selectedTranscriptProvenance = computed(() =>
  formatZoomTranscriptProvenance(selectedTranscript.value?.provenance)
)

watch(
  calls,
  (nextCalls) => {
    if (nextCalls.length === 0) {
      selectedCallId.value = null
      return
    }
    const hasSelectedCall = nextCalls.some((call) => call.call_id === selectedCallId.value)
    if (!hasSelectedCall) selectedCallId.value = nextCalls[0]?.call_id ?? null
  },
  { immediate: true }
)

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

function describeCall(call: { provider_call_id: string; call_state: string; direction: string }): string {
  return [call.provider_call_id, call.call_state, call.direction].filter(Boolean).join(' · ')
}

function metadataString(key: string): string {
  const value = selectedCall.value?.metadata?.[key]
  return typeof value === 'string' && value.trim().length > 0 ? value : '—'
}

function formatBytes(value: number | null | undefined): string {
  if (typeof value !== 'number' || !Number.isFinite(value) || value < 0) return '—'
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`
  return `${(value / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <section class="integration-section zoom-observed-calls">
    <header class="zoom-observed-calls__header">
      <div>
        <h4>{{ t('Observed calls') }}</h4>
        <p>{{ t('Projected provider call evidence for the selected Zoom account.') }}</p>
      </div>
      <span class="zoom-observed-calls__count">{{ calls.length }}</span>
    </header>

    <div v-if="!selectedAccount?.account_id" class="zoom-observed-calls__placeholder">
      {{ t('Select a Zoom account to inspect observed calls.') }}
    </div>
    <div v-else-if="providerCallsQuery.isLoading.value" class="zoom-observed-calls__placeholder">
      {{ t('Loading Zoom call evidence...') }}
    </div>
    <div v-else-if="calls.length === 0" class="zoom-observed-calls__placeholder">
      {{ t('No Zoom calls projected for this account yet.') }}
    </div>
    <div v-else class="zoom-observed-calls__content">
      <div class="zoom-observed-calls__list">
        <button
          v-for="call in calls"
          :key="call.call_id"
          type="button"
          class="zoom-observed-calls__row"
          :class="{ 'zoom-observed-calls__row--active': call.call_id === selectedCallId }"
          @click="selectedCallId = call.call_id"
        >
          <div>
            <strong>{{ describeCall(call) }}</strong>
            <p>{{ call.provider_chat_id || '—' }}</p>
          </div>
          <small>{{ formatDate(call.started_at ?? call.created_at) }}</small>
        </button>
      </div>

      <div v-if="selectedCall" class="zoom-observed-calls__detail">
        <header>
          <strong>{{ t('Transcript evidence') }}</strong>
          <small>{{ selectedCall.call_id }}</small>
        </header>
        <dl class="zoom-observed-calls__meta">
          <div><dt>{{ t('Meeting id') }}</dt><dd>{{ selectedCall.provider_call_id }}</dd></div>
          <div><dt>{{ t('Direction') }}</dt><dd>{{ selectedCall.direction }}</dd></div>
          <div><dt>{{ t('State') }}</dt><dd>{{ selectedCall.call_state }}</dd></div>
          <div><dt>{{ t('Started') }}</dt><dd>{{ formatDate(selectedCall.started_at) }}</dd></div>
          <div><dt>{{ t('Topic') }}</dt><dd>{{ metadataString('topic') }}</dd></div>
          <div><dt>{{ t('Host email') }}</dt><dd>{{ metadataString('host_email') }}</dd></div>
        </dl>

        <div class="zoom-observed-calls__section">
          <header>
            <strong>{{ t('Recording references') }}</strong>
            <small>{{ selectedRecordingRefs.length }}</small>
          </header>
          <div v-if="selectedRecordingRefs.length === 0" class="zoom-observed-calls__placeholder">
            {{ t('No recording references projected for this call yet.') }}
          </div>
          <div v-else class="zoom-observed-calls__recordings">
            <article v-for="recording in selectedRecordingRefs" :key="recording.recording_id" class="zoom-observed-calls__recording">
              <dl class="zoom-observed-calls__meta">
                <div><dt>{{ t('Recording id') }}</dt><dd>{{ recording.recording_id }}</dd></div>
                <div><dt>{{ t('Type') }}</dt><dd>{{ recording.recording_type ?? '—' }}</dd></div>
                <div><dt>{{ t('Format') }}</dt><dd>{{ recording.file_extension ?? '—' }}</dd></div>
                <div><dt>{{ t('Size') }}</dt><dd>{{ formatBytes(recording.file_size_bytes) }}</dd></div>
                <div><dt>{{ t('Recorded') }}</dt><dd>{{ formatDate(recording.recorded_at) }}</dd></div>
                <div><dt>{{ t('Download ref') }}</dt><dd>{{ recording.download_ref ?? '—' }}</dd></div>
              </dl>
            </article>
          </div>
        </div>

        <div v-if="transcriptQuery.isLoading.value" class="zoom-observed-calls__placeholder">
          {{ t('Loading Zoom transcript evidence...') }}
        </div>
        <div v-else-if="!selectedTranscript" class="zoom-observed-calls__placeholder">
          {{ t('No transcript evidence projected for this call yet.') }}
        </div>
        <div v-else class="zoom-observed-calls__transcript">
          <dl class="zoom-observed-calls__meta">
            <div><dt>{{ t('Status') }}</dt><dd>{{ selectedTranscript.transcript_status }}</dd></div>
            <div><dt>{{ t('Provider') }}</dt><dd>{{ selectedTranscript.stt_provider }}</dd></div>
            <div><dt>{{ t('Language') }}</dt><dd>{{ selectedTranscript.language_code ?? '—' }}</dd></div>
            <div><dt>{{ t('Audio ref') }}</dt><dd>{{ selectedTranscript.source_audio_ref ?? '—' }}</dd></div>
          </dl>
          <p>{{ selectedTranscript.transcript_text }}</p>
          <div class="zoom-observed-calls__section">
            <header>
              <strong>{{ t('Transcript provenance') }}</strong>
              <small>{{ selectedTranscript.transcript_id }}</small>
            </header>
            <pre class="zoom-observed-calls__provenance">{{ selectedTranscriptProvenance }}</pre>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.zoom-observed-calls { display: grid; gap: 12px; }
.zoom-observed-calls__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.zoom-observed-calls__header h4,
.zoom-observed-calls__header p {
  margin: 0;
}
.zoom-observed-calls__header p,
.zoom-observed-calls__meta dt,
.zoom-observed-calls__row small,
.zoom-observed-calls__detail small {
  color: var(--hh-text-muted);
  font-size: 11px;
}
.zoom-observed-calls__count {
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
.zoom-observed-calls__content {
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr);
}
.zoom-observed-calls__list,
.zoom-observed-calls__detail,
.zoom-observed-calls__transcript,
.zoom-observed-calls__recordings,
.zoom-observed-calls__section {
  display: grid;
  gap: 8px;
}
.zoom-observed-calls__row,
.zoom-observed-calls__placeholder {
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
  font-size: 12px;
}
.zoom-observed-calls__row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  text-align: left;
  cursor: pointer;
  color: var(--hh-text-primary);
}
.zoom-observed-calls__row--active {
  border-color: var(--hh-accent);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--hh-accent) 32%, transparent);
}
.zoom-observed-calls__row strong,
.zoom-observed-calls__detail strong {
  display: block;
  font-size: 12px;
}
.zoom-observed-calls__row p,
.zoom-observed-calls__transcript p {
  margin: 2px 0 0;
  font-size: 12px;
  word-break: break-word;
}
.zoom-observed-calls__detail {
  align-content: start;
}
.zoom-observed-calls__detail header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.zoom-observed-calls__section header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.zoom-observed-calls__meta {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px 12px;
  margin: 0;
}
.zoom-observed-calls__meta dd {
  margin: 2px 0 0;
  font-size: 12px;
  word-break: break-word;
}
.zoom-observed-calls__recording,
.zoom-observed-calls__provenance {
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
}
.zoom-observed-calls__provenance {
  margin: 0;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 11px;
  line-height: 1.45;
  color: var(--hh-text-muted);
}
@media (max-width: 1100px) {
  .zoom-observed-calls__content { grid-template-columns: minmax(0, 1fr); }
}
</style>
