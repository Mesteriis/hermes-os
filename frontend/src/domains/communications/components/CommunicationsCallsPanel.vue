<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import {
  useProviderCallsQuery,
  useProviderCallTranscriptQuery,
} from '../queries/useCommunicationsQuery'
import type { ProviderCall } from '../types/communications'

type MeetingParticipant = {
  participant_id?: string | null
  display_name?: string | null
  email?: string | null
}

type MeetingRecordingRef = {
  recording_id?: string | null
  recording_type?: string | null
  download_ref?: string | null
  file_extension?: string | null
  file_size_bytes?: number | null
  recorded_at?: string | null
}

const { t } = useI18n()

const props = defineProps<{
  mode: 'calls' | 'meetings'
}>()

const selectedCallId = ref<string | null>(null)
const providerCallsQuery = useProviderCallsQuery(
  undefined,
  50,
  computed(() => (props.mode === 'meetings' ? 'zoom' : undefined))
)
const providerCalls = computed(() => providerCallsQuery.data.value ?? [])
const visibleCalls = computed(() =>
  providerCalls.value.filter((call) => matchesMode(call, props.mode))
)
const selectedCall = computed(
  () => visibleCalls.value.find((call) => call.call_id === selectedCallId.value) ?? visibleCalls.value[0] ?? null
)
const selectedTranscript = useProviderCallTranscriptQuery(
  computed(() => selectedCall.value?.call_id ?? null)
)

watch(
  visibleCalls,
  (nextCalls) => {
    if (nextCalls.length === 0) {
      selectedCallId.value = null
      return
    }
    if (!nextCalls.some((call) => call.call_id === selectedCallId.value)) {
      selectedCallId.value = nextCalls[0]?.call_id ?? null
    }
  },
  { immediate: true }
)

function matchesMode(call: ProviderCall, mode: 'calls' | 'meetings'): boolean {
  if (mode === 'calls') return true
  return meetingProvider(call) === 'zoom'
}

function metadataString(call: ProviderCall | null, key: string): string {
  const value = call?.metadata?.[key]
  return typeof value === 'string' && value.trim() ? value : '—'
}

function metadataOptionalString(call: ProviderCall | null, key: string): string | null {
  const value = call?.metadata?.[key]
  return typeof value === 'string' && value.trim() ? value : null
}

function meetingProvider(call: ProviderCall): string | null {
  const provider = call.metadata?.provider
  return typeof provider === 'string' && provider.trim() ? provider.trim() : null
}

function meetingParticipants(call: ProviderCall | null): MeetingParticipant[] {
  const value = call?.metadata?.participants
  if (!Array.isArray(value)) return []
  return value.filter((item): item is MeetingParticipant => typeof item === 'object' && item !== null)
}

function meetingRecordingRefs(call: ProviderCall | null): MeetingRecordingRef[] {
  const value = call?.metadata?.recording_refs
  if (!Array.isArray(value)) return []
  return value.filter((item): item is MeetingRecordingRef => typeof item === 'object' && item !== null)
}

function participantLabel(participant: MeetingParticipant): string {
  return participant.display_name?.trim() || participant.email?.trim() || participant.participant_id?.trim() || '—'
}

function participantSecondary(participant: MeetingParticipant): string {
  return participant.email?.trim() || participant.participant_id?.trim() || '—'
}

function recordingLabel(recording: MeetingRecordingRef): string {
  const parts = [
    recording.recording_id?.trim(),
    recording.recording_type?.trim(),
    recording.file_extension?.trim(),
  ].filter(Boolean)
  return parts.length ? parts.join(' · ') : '—'
}

function formatFileSize(value: number | null | undefined): string {
  if (typeof value !== 'number' || value <= 0) return '—'
  if (value >= 1024 * 1024 * 1024) return `${(value / (1024 * 1024 * 1024)).toFixed(1)} GB`
  if (value >= 1024 * 1024) return `${(value / (1024 * 1024)).toFixed(1)} MB`
  if (value >= 1024) return `${Math.round(value / 1024)} KB`
  return `${value} B`
}

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

function describeCall(call: ProviderCall): string {
  const primary = call.provider_call_id || call.call_id
  return [primary, call.call_state, call.direction].filter(Boolean).join(' · ')
}
</script>

<template>
  <section class="communications-calls-panel">
    <header class="communications-calls-panel__header">
      <div>
        <h3>{{ mode === 'meetings' ? t('Meetings') : t('Calls') }}</h3>
        <p>
          {{
            mode === 'meetings'
              ? t('Provider-neutral meeting evidence projected from calls, including Zoom meetings.')
              : t('Provider-neutral call evidence across integrations.')
          }}
        </p>
      </div>
      <span class="communications-calls-panel__count">{{ visibleCalls.length }}</span>
    </header>

    <div v-if="providerCallsQuery.isLoading.value" class="communications-calls-panel__placeholder">
      {{ t('Loading call evidence...') }}
    </div>
    <div
      v-else-if="visibleCalls.length === 0"
      class="communications-calls-panel__placeholder"
    >
      {{
        mode === 'meetings'
          ? t('No projected meetings yet.')
          : t('No projected calls yet.')
      }}
    </div>
    <div v-else class="communications-calls-panel__content">
      <div class="communications-calls-panel__list">
        <button
          v-for="call in visibleCalls"
          :key="call.call_id"
          type="button"
          class="communications-calls-panel__row"
          :class="{ 'communications-calls-panel__row--active': call.call_id === selectedCallId }"
          @click="selectedCallId = call.call_id"
        >
          <div>
            <strong>{{ describeCall(call) }}</strong>
            <p>{{ metadataString(call, 'topic') }}</p>
          </div>
          <small>{{ formatDate(call.started_at ?? call.created_at) }}</small>
        </button>
      </div>

      <div v-if="selectedCall" class="communications-calls-panel__detail">
        <header>
          <strong>{{ metadataString(selectedCall, 'topic') }}</strong>
          <small>{{ selectedCall.call_id }}</small>
        </header>
        <dl class="communications-calls-panel__meta">
          <div><dt>{{ t('Provider') }}</dt><dd>{{ meetingProvider(selectedCall) ?? '—' }}</dd></div>
          <div><dt>{{ t('Provider id') }}</dt><dd>{{ selectedCall.provider_call_id }}</dd></div>
          <div><dt>{{ t('Meeting id') }}</dt><dd>{{ metadataString(selectedCall, 'meeting_id') }}</dd></div>
          <div><dt>{{ t('Direction') }}</dt><dd>{{ selectedCall.direction }}</dd></div>
          <div><dt>{{ t('State') }}</dt><dd>{{ selectedCall.call_state }}</dd></div>
          <div><dt>{{ t('Started') }}</dt><dd>{{ formatDate(selectedCall.started_at) }}</dd></div>
          <div><dt>{{ t('Ended') }}</dt><dd>{{ formatDate(selectedCall.ended_at) }}</dd></div>
          <div><dt>{{ t('Host email') }}</dt><dd>{{ metadataString(selectedCall, 'host_email') }}</dd></div>
          <div><dt>{{ t('Transcript ref') }}</dt><dd>{{ metadataString(selectedCall, 'transcript_ref') }}</dd></div>
          <div><dt>{{ t('Join url') }}</dt><dd>{{ metadataString(selectedCall, 'join_url') }}</dd></div>
          <div><dt>{{ t('Participants') }}</dt><dd>{{ meetingParticipants(selectedCall).length || '—' }}</dd></div>
          <div><dt>{{ t('Recording refs') }}</dt><dd>{{ meetingRecordingRefs(selectedCall).length || '—' }}</dd></div>
        </dl>

        <div
          v-if="meetingParticipants(selectedCall).length > 0"
          class="communications-calls-panel__evidence"
        >
          <strong>{{ t('Participants') }}</strong>
          <div class="communications-calls-panel__chips">
            <div
              v-for="participant in meetingParticipants(selectedCall)"
              :key="participant.participant_id || participant.email || participant.display_name || 'participant'"
              class="communications-calls-panel__chip"
            >
              <span>{{ participantLabel(participant) }}</span>
              <small>{{ participantSecondary(participant) }}</small>
            </div>
          </div>
        </div>

        <div
          v-if="meetingRecordingRefs(selectedCall).length > 0"
          class="communications-calls-panel__evidence"
        >
          <strong>{{ t('Recording references') }}</strong>
          <div class="communications-calls-panel__chips">
            <div
              v-for="recording in meetingRecordingRefs(selectedCall)"
              :key="recording.recording_id || recording.download_ref || 'recording'"
              class="communications-calls-panel__chip"
            >
              <span>{{ recordingLabel(recording) }}</span>
              <small>{{ formatDate(recording.recorded_at) }} · {{ formatFileSize(recording.file_size_bytes) }}</small>
            </div>
          </div>
          <a
            v-if="metadataOptionalString(selectedCall, 'join_url')"
            class="communications-calls-panel__link"
            :href="metadataOptionalString(selectedCall, 'join_url') ?? '#'"
            target="_blank"
            rel="noreferrer"
          >
            {{ t('Open join URL') }}
          </a>
        </div>

        <div
          v-if="selectedTranscript.isLoading.value"
          class="communications-calls-panel__placeholder"
        >
          {{ t('Loading transcript evidence...') }}
        </div>
        <div
          v-else-if="!selectedTranscript.data.value"
          class="communications-calls-panel__placeholder"
        >
          {{ t('No transcript evidence projected for this call yet.') }}
        </div>
        <div v-else class="communications-calls-panel__transcript">
          <dl class="communications-calls-panel__meta">
            <div><dt>{{ t('Transcript status') }}</dt><dd>{{ selectedTranscript.data.value.transcript_status }}</dd></div>
            <div><dt>{{ t('Provider') }}</dt><dd>{{ selectedTranscript.data.value.stt_provider }}</dd></div>
            <div><dt>{{ t('Language') }}</dt><dd>{{ selectedTranscript.data.value.language_code ?? '—' }}</dd></div>
            <div><dt>{{ t('Audio ref') }}</dt><dd>{{ selectedTranscript.data.value.source_audio_ref ?? '—' }}</dd></div>
          </dl>
          <p>{{ selectedTranscript.data.value.transcript_text }}</p>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.communications-calls-panel {
  display: grid;
  gap: 12px;
  padding: 16px;
}
.communications-calls-panel__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
.communications-calls-panel__header h3,
.communications-calls-panel__header p,
.communications-calls-panel__detail header,
.communications-calls-panel__row p {
  margin: 0;
}
.communications-calls-panel__header p,
.communications-calls-panel__meta dt,
.communications-calls-panel__row small,
.communications-calls-panel__detail small {
  color: var(--hh-text-muted);
  font-size: 11px;
}
.communications-calls-panel__count {
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
.communications-calls-panel__content {
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(280px, 0.8fr) minmax(0, 1.2fr);
}
.communications-calls-panel__list,
.communications-calls-panel__detail,
.communications-calls-panel__transcript {
  display: grid;
  gap: 8px;
}
.communications-calls-panel__row,
.communications-calls-panel__placeholder {
  padding: 10px 12px;
  border-radius: var(--hh-radius-sm);
  border: 1px solid var(--hh-border);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
  font-size: 12px;
}
.communications-calls-panel__row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  text-align: left;
  color: var(--hh-text-primary);
}
.communications-calls-panel__row--active {
  border-color: var(--hh-accent);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--hh-accent) 32%, transparent);
}
.communications-calls-panel__row strong,
.communications-calls-panel__detail strong {
  display: block;
  font-size: 12px;
}
.communications-calls-panel__detail strong,
.communications-calls-panel__evidence strong {
  color: var(--hh-text-primary);
}
.communications-calls-panel__detail {
  align-content: start;
}
.communications-calls-panel__detail header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.communications-calls-panel__meta {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin: 0;
}
.communications-calls-panel__meta div {
  display: grid;
  gap: 2px;
}
.communications-calls-panel__meta dt,
.communications-calls-panel__meta dd,
.communications-calls-panel__transcript p {
  margin: 0;
  word-break: break-word;
}
.communications-calls-panel__evidence {
  display: grid;
  gap: 8px;
}
.communications-calls-panel__chips {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}
.communications-calls-panel__chip {
  display: grid;
  gap: 2px;
  padding: 8px 10px;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%);
}
.communications-calls-panel__chip span,
.communications-calls-panel__link {
  color: var(--hh-text-primary);
  font-size: 12px;
}
.communications-calls-panel__chip small {
  color: var(--hh-text-muted);
  font-size: 11px;
}
.communications-calls-panel__link {
  text-decoration: none;
}
.communications-calls-panel__link:hover {
  text-decoration: underline;
}
@media (max-width: 900px) {
  .communications-calls-panel__content,
  .communications-calls-panel__meta {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
