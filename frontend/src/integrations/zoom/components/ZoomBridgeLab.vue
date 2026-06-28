<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useSettingsStore } from '../../../shared/zoom/settingsBridge'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import type { ZoomParticipantSnapshot, ZoomRecordingRef } from '../types/zoom'
import {
  useBridgeZoomMeetingMutation,
  useBridgeZoomRecordingMutation,
  useBridgeZoomTranscriptMutation,
  useImportZoomTranscriptFileMutation,
} from '../queries/useZoomRuntimeQuery'

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const { t } = useI18n()
const store = useSettingsStore()

const activeBridgeAction = ref<string | null>(null)
const bridgeResult = ref('')
const exampleMeetingParticipants = '[{"display_name":"Alice"}]'
const exampleMeetingRecordingRefs = '[{"recording_id":"rec-1"}]'
const exampleFixtureMetadata = '{"source":"fixture"}'
const exampleRecordingMetadata = '{"channel":"cloud"}'
const exampleTranscriptSegments = '[{"text":"Hello","start_ms":0,"end_ms":1000}]'

const meetingForm = ref({
  meeting_id: '',
  meeting_uuid: '',
  topic: '',
  host_email: '',
  join_url: '',
  started_at: '',
  ended_at: '',
  duration_seconds: '',
  transcript_ref: '',
  participants_json: '[]',
  recording_refs_json: '[]',
  metadata_json: '{}',
})

const recordingForm = ref({
  meeting_id: '',
  recording_id: '',
  recording_type: '',
  download_ref: '',
  file_extension: '',
  file_size_bytes: '',
  recorded_at: '',
  metadata_json: '{}',
  request_metadata_json: '{}',
})

const transcriptForm = ref({
  transcript_id: '',
  meeting_id: '',
  meeting_uuid: '',
  source_recording_ref: '',
  language_code: '',
  transcript_text: '',
  segments_json: '[]',
  metadata_json: '{}',
})

const transcriptFileForm = ref({
  transcript_id: '',
  meeting_id: '',
  meeting_uuid: '',
  source_recording_ref: '',
  language_code: '',
  file_name: '',
  content_type: 'text/vtt',
  file_text: '',
  metadata_json: '{}',
})

const bridgeMeeting = useBridgeZoomMeetingMutation()
const bridgeRecording = useBridgeZoomRecordingMutation()
const bridgeTranscript = useBridgeZoomTranscriptMutation()
const importTranscriptFile = useImportZoomTranscriptFileMutation()

const selectedZoomAccountId = computed(() => props.selectedAccount?.account_id ?? '')

function selectedAccountIdOrError(): string | null {
  const accountId = selectedZoomAccountId.value.trim()
  if (!accountId) {
    store.setError(t('Select a Zoom account before using the bridge lab'))
    return null
  }
  return accountId
}

function parseJsonObject(raw: string, label: string): Record<string, unknown> {
  const trimmed = raw.trim()
  if (!trimmed) return {}
  const parsed = JSON.parse(trimmed)
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error(`${label} must be a JSON object`)
  }
  return parsed as Record<string, unknown>
}

function parseJsonArray<T>(raw: string, label: string): T[] {
  const trimmed = raw.trim()
  if (!trimmed) return []
  const parsed = JSON.parse(trimmed)
  if (!Array.isArray(parsed)) {
    throw new Error(`${label} must be a JSON array`)
  }
  return parsed as T[]
}

function optionalNumber(raw: string): number | undefined {
  const trimmed = raw.trim()
  if (!trimmed) return undefined
  const parsed = Number.parseInt(trimmed, 10)
  if (!Number.isFinite(parsed)) throw new Error('Numeric field must be a valid integer')
  return parsed
}

function optionalString(raw: string): string | undefined {
  const trimmed = raw.trim()
  return trimmed ? trimmed : undefined
}

async function handleBridgeMeeting() {
  const accountId = selectedAccountIdOrError()
  if (!accountId) return
  activeBridgeAction.value = 'meeting'
  try {
    const result = await bridgeMeeting.mutateAsync({
      account_id: accountId,
      meeting_id: meetingForm.value.meeting_id.trim(),
      meeting_uuid: optionalString(meetingForm.value.meeting_uuid),
      topic: optionalString(meetingForm.value.topic),
      host_email: optionalString(meetingForm.value.host_email),
      join_url: optionalString(meetingForm.value.join_url),
      started_at: optionalString(meetingForm.value.started_at),
      ended_at: optionalString(meetingForm.value.ended_at),
      duration_seconds: optionalNumber(meetingForm.value.duration_seconds),
      transcript_ref: optionalString(meetingForm.value.transcript_ref),
      participants: parseJsonArray<ZoomParticipantSnapshot>(meetingForm.value.participants_json, 'Participants'),
      recording_refs: parseJsonArray<ZoomRecordingRef>(meetingForm.value.recording_refs_json, 'Recording refs'),
      metadata: parseJsonObject(meetingForm.value.metadata_json, 'Meeting metadata'),
    })
    bridgeResult.value = `meeting:${result.call_id}:${result.event_id}`
    store.setActionMessage(t('Zoom meeting observation ingested'))
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom meeting bridge ingest failed')
  } finally {
    activeBridgeAction.value = null
  }
}

async function handleBridgeRecording() {
  const accountId = selectedAccountIdOrError()
  if (!accountId) return
  activeBridgeAction.value = 'recording'
  try {
    const result = await bridgeRecording.mutateAsync({
      account_id: accountId,
      meeting_id: recordingForm.value.meeting_id.trim(),
      recording: {
        recording_id: recordingForm.value.recording_id.trim(),
        recording_type: optionalString(recordingForm.value.recording_type),
        download_ref: optionalString(recordingForm.value.download_ref),
        file_extension: optionalString(recordingForm.value.file_extension),
        file_size_bytes: optionalNumber(recordingForm.value.file_size_bytes),
        recorded_at: optionalString(recordingForm.value.recorded_at),
        metadata: parseJsonObject(recordingForm.value.metadata_json, 'Recording metadata'),
      },
      metadata: parseJsonObject(recordingForm.value.request_metadata_json, 'Recording request metadata'),
    })
    bridgeResult.value = `recording:${result.recording_id}:${result.event_id}`
    store.setActionMessage(t('Zoom recording observation ingested'))
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom recording bridge ingest failed')
  } finally {
    activeBridgeAction.value = null
  }
}

async function handleBridgeTranscript() {
  const accountId = selectedAccountIdOrError()
  if (!accountId) return
  activeBridgeAction.value = 'transcript'
  try {
    const result = await bridgeTranscript.mutateAsync({
      account_id: accountId,
      transcript_id: transcriptForm.value.transcript_id.trim(),
      meeting_id: transcriptForm.value.meeting_id.trim(),
      meeting_uuid: optionalString(transcriptForm.value.meeting_uuid),
      source_recording_ref: optionalString(transcriptForm.value.source_recording_ref),
      language_code: optionalString(transcriptForm.value.language_code),
      transcript_text: transcriptForm.value.transcript_text.trim(),
      segments: parseJsonArray(transcriptForm.value.segments_json, 'Transcript segments'),
      metadata: parseJsonObject(transcriptForm.value.metadata_json, 'Transcript metadata'),
    })
    bridgeResult.value = `transcript:${result.transcript_id}:${result.event_id}`
    store.setActionMessage(t('Zoom transcript observation ingested'))
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom transcript bridge ingest failed')
  } finally {
    activeBridgeAction.value = null
  }
}

async function handleImportTranscriptFile() {
  const accountId = selectedAccountIdOrError()
  if (!accountId) return
  activeBridgeAction.value = 'transcript-file'
  try {
    const result = await importTranscriptFile.mutateAsync({
      account_id: accountId,
      transcript_id: transcriptFileForm.value.transcript_id.trim(),
      meeting_id: transcriptFileForm.value.meeting_id.trim(),
      meeting_uuid: optionalString(transcriptFileForm.value.meeting_uuid),
      source_recording_ref: optionalString(transcriptFileForm.value.source_recording_ref),
      language_code: optionalString(transcriptFileForm.value.language_code),
      file_name: optionalString(transcriptFileForm.value.file_name),
      content_type: optionalString(transcriptFileForm.value.content_type),
      file_text: transcriptFileForm.value.file_text,
      metadata: parseJsonObject(transcriptFileForm.value.metadata_json, 'Transcript file metadata'),
    })
    bridgeResult.value = `transcript-file:${result.transcript_id}:${result.event_id}:${result.import_format}`
    store.setActionMessage(t('Zoom transcript file imported'))
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom transcript file import failed')
  } finally {
    activeBridgeAction.value = null
  }
}
</script>

<template>
  <section class="integration-section bridge-lab">
    <header class="bridge-lab-header">
      <div>
        <h4>{{ t('Runtime bridge lab') }}</h4>
        <p>{{ t('Use the selected Zoom account to validate meetings, recordings and transcript ingestion.') }}</p>
      </div>
      <div class="bridge-account">
        <span>{{ t('Account') }}</span>
        <strong>{{ selectedZoomAccountId || '-' }}</strong>
      </div>
    </header>

    <div v-if="bridgeResult" class="bridge-result">
      <span>{{ t('Last result') }}</span>
      <code>{{ bridgeResult }}</code>
    </div>

    <div class="bridge-grid">
      <form class="bridge-card" @submit.prevent="handleBridgeMeeting">
        <h5>{{ t('Meeting observation') }}</h5>
        <input v-model="meetingForm.meeting_id" class="hermes-input-control" type="text" :placeholder="t('Meeting id')" required />
        <input v-model="meetingForm.meeting_uuid" class="hermes-input-control" type="text" :placeholder="t('Meeting UUID')" />
        <input v-model="meetingForm.topic" class="hermes-input-control" type="text" :placeholder="t('Topic')" />
        <input v-model="meetingForm.host_email" class="hermes-input-control" type="email" :placeholder="t('host@example.test')" />
        <input v-model="meetingForm.join_url" class="hermes-input-control" type="text" :placeholder="t('https://example.zoom.us/j/...')" />
        <input v-model="meetingForm.started_at" class="hermes-input-control" type="text" :placeholder="t('2026-06-28T12:00:00Z')" />
        <input v-model="meetingForm.ended_at" class="hermes-input-control" type="text" :placeholder="t('2026-06-28T12:30:00Z')" />
        <input v-model="meetingForm.duration_seconds" class="hermes-input-control" type="number" min="0" :placeholder="t('1800')" />
        <input v-model="meetingForm.transcript_ref" class="hermes-input-control" type="text" :placeholder="t('provider transcript ref')" />
        <textarea v-model="meetingForm.participants_json" class="bridge-textarea" rows="4" :placeholder="exampleMeetingParticipants" />
        <textarea v-model="meetingForm.recording_refs_json" class="bridge-textarea" rows="4" :placeholder="exampleMeetingRecordingRefs" />
        <textarea v-model="meetingForm.metadata_json" class="bridge-textarea" rows="4" :placeholder="exampleFixtureMetadata" />
        <button type="submit" class="hermes-btn hermes-btn--outline"
          :disabled="activeBridgeAction === 'meeting' || bridgeMeeting.isPending.value">
          {{ bridgeMeeting.isPending.value ? t('Ingesting...') : t('Ingest meeting') }}
        </button>
      </form>

      <form class="bridge-card" @submit.prevent="handleBridgeRecording">
        <h5>{{ t('Recording observation') }}</h5>
        <input v-model="recordingForm.meeting_id" class="hermes-input-control" type="text" :placeholder="t('Meeting id')" required />
        <input v-model="recordingForm.recording_id" class="hermes-input-control" type="text" :placeholder="t('Recording id')" required />
        <input v-model="recordingForm.recording_type" class="hermes-input-control" type="text" :placeholder="t('shared_screen_with_speaker_view')" />
        <input v-model="recordingForm.download_ref" class="hermes-input-control" type="text" :placeholder="t('provider download ref')" />
        <input v-model="recordingForm.file_extension" class="hermes-input-control" type="text" :placeholder="t('mp4')" />
        <input v-model="recordingForm.file_size_bytes" class="hermes-input-control" type="number" min="0" :placeholder="t('1048576')" />
        <input v-model="recordingForm.recorded_at" class="hermes-input-control" type="text" :placeholder="t('2026-06-28T12:30:00Z')" />
        <textarea v-model="recordingForm.metadata_json" class="bridge-textarea" rows="4" :placeholder="exampleRecordingMetadata" />
        <textarea v-model="recordingForm.request_metadata_json" class="bridge-textarea" rows="4" :placeholder="exampleFixtureMetadata" />
        <button type="submit" class="hermes-btn hermes-btn--outline"
          :disabled="activeBridgeAction === 'recording' || bridgeRecording.isPending.value">
          {{ bridgeRecording.isPending.value ? t('Ingesting...') : t('Ingest recording') }}
        </button>
      </form>

      <form class="bridge-card" @submit.prevent="handleBridgeTranscript">
        <h5>{{ t('Transcript observation') }}</h5>
        <input v-model="transcriptForm.transcript_id" class="hermes-input-control" type="text" :placeholder="t('Transcript id')" required />
        <input v-model="transcriptForm.meeting_id" class="hermes-input-control" type="text" :placeholder="t('Meeting id')" required />
        <input v-model="transcriptForm.meeting_uuid" class="hermes-input-control" type="text" :placeholder="t('Meeting UUID')" />
        <input v-model="transcriptForm.source_recording_ref" class="hermes-input-control" type="text" :placeholder="t('Recording ref')" />
        <input v-model="transcriptForm.language_code" class="hermes-input-control" type="text" :placeholder="t('en-US')" />
        <textarea v-model="transcriptForm.transcript_text" class="bridge-textarea" rows="6" :placeholder="t('Transcript text')" />
        <textarea v-model="transcriptForm.segments_json" class="bridge-textarea" rows="4" :placeholder="exampleTranscriptSegments" />
        <textarea v-model="transcriptForm.metadata_json" class="bridge-textarea" rows="4" :placeholder="exampleFixtureMetadata" />
        <button type="submit" class="hermes-btn hermes-btn--outline"
          :disabled="activeBridgeAction === 'transcript' || bridgeTranscript.isPending.value">
          {{ bridgeTranscript.isPending.value ? t('Ingesting...') : t('Ingest transcript') }}
        </button>
      </form>

      <form class="bridge-card" @submit.prevent="handleImportTranscriptFile">
        <h5>{{ t('Transcript file import') }}</h5>
        <input v-model="transcriptFileForm.transcript_id" class="hermes-input-control" type="text" :placeholder="t('Transcript id')" required />
        <input v-model="transcriptFileForm.meeting_id" class="hermes-input-control" type="text" :placeholder="t('Meeting id')" required />
        <input v-model="transcriptFileForm.meeting_uuid" class="hermes-input-control" type="text" :placeholder="t('Meeting UUID')" />
        <input v-model="transcriptFileForm.source_recording_ref" class="hermes-input-control" type="text" :placeholder="t('Recording ref')" />
        <input v-model="transcriptFileForm.language_code" class="hermes-input-control" type="text" :placeholder="t('en-US')" />
        <input v-model="transcriptFileForm.file_name" class="hermes-input-control" type="text" :placeholder="t('meeting.vtt')" />
        <input v-model="transcriptFileForm.content_type" class="hermes-input-control" type="text" :placeholder="t('text/vtt')" />
        <textarea v-model="transcriptFileForm.file_text" class="bridge-textarea" rows="8" :placeholder="t('WEBVTT')" />
        <textarea v-model="transcriptFileForm.metadata_json" class="bridge-textarea" rows="4" :placeholder="exampleFixtureMetadata" />
        <button type="submit" class="hermes-btn hermes-btn--outline"
          :disabled="activeBridgeAction === 'transcript-file' || importTranscriptFile.isPending.value">
          {{ importTranscriptFile.isPending.value ? t('Importing...') : t('Import transcript file') }}
        </button>
      </form>
    </div>
  </section>
</template>

<style scoped>
.bridge-lab { display: grid; gap: 12px; }
.bridge-lab-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; }
.bridge-lab-header h4 { margin: 0 0 4px; }
.bridge-lab-header p { margin: 0; font-size: 12px; color: var(--hh-text-muted); }
.bridge-account { display: grid; gap: 4px; text-align: right; font-size: 12px; }
.bridge-account span { color: var(--hh-text-muted); }
.bridge-result { display: grid; gap: 4px; padding: 10px 12px; border-radius: var(--hh-radius-sm); border: 1px solid var(--hh-border); background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%); }
.bridge-result span { font-size: 11px; color: var(--hh-text-muted); }
.bridge-result code { white-space: pre-wrap; word-break: break-word; }
.bridge-grid { display: grid; gap: 12px; grid-template-columns: repeat(2, minmax(0, 1fr)); }
.bridge-card { display: grid; gap: 8px; padding: 12px; border: 1px solid var(--hh-border); border-radius: var(--hh-radius-sm); background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%); }
.bridge-card h5 { margin: 0; font-size: 12px; color: var(--hh-text-secondary); }
.bridge-textarea { width: 100%; padding: 8px; background: var(--hh-surface-deep); border: 1px solid var(--hh-border); border-radius: var(--hh-radius-sm); color: var(--hh-text-primary); font-size: 12px; font-family: monospace; resize: vertical; outline: none; }
.bridge-textarea:focus-visible { box-shadow: 0 0 0 2px var(--hh-focus-ring); border-color: var(--hh-accent); }
@media (max-width: 1100px) {
  .bridge-grid { grid-template-columns: minmax(0, 1fr); }
  .bridge-lab-header { flex-direction: column; }
  .bridge-account { text-align: left; }
}
</style>
