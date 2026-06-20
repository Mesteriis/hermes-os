<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { TelegramCall } from '../types/telegram'
import { useTelegramCallTranscriptQuery } from '../queries/useTelegramQuery'

const { t } = useI18n()

const props = defineProps<{
  calls: TelegramCall[]
}>()

const selectedCallId = ref<string | null>(null)
const transcriptQuery = useTelegramCallTranscriptQuery(selectedCallId)

const selectedCall = computed(() =>
  props.calls.find((call) => call.call_id === selectedCallId.value) ?? props.calls[0] ?? null
)

const selectedTranscript = computed(() => transcriptQuery.data.value)

watch(
  () => props.calls,
  (calls) => {
    if (calls.length === 0) {
      selectedCallId.value = null
      return
    }
    const hasCurrent = calls.some((call) => call.call_id === selectedCallId.value)
    if (!hasCurrent) {
      selectedCallId.value = calls[0]?.call_id ?? null
    }
  },
  { immediate: true }
)

function formatDate(value: string | null | undefined): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}
</script>

<template>
  <div class="telegram-call-transcripts">
    <div class="telegram-call-list">
      <button
        v-for="call in calls"
        :key="call.call_id"
        type="button"
        class="telegram-call-row"
        :class="{ 'telegram-call-row--active': call.call_id === selectedCallId }"
        @click="selectedCallId = call.call_id"
      >
        <div>
          <strong>{{ call.status }}</strong>
          <p>{{ call.provider_chat_id }}</p>
        </div>
        <small>{{ formatDate(call.occurred_at) }}</small>
      </button>
    </div>

    <div v-if="selectedCall" class="telegram-call-transcript-card">
      <header>
        <strong>{{ t('Transcript') }}</strong>
        <small>{{ selectedCall.call_id }}</small>
      </header>
      <div v-if="transcriptQuery.isLoading.value" class="telegram-call-placeholder">
        {{ t('Loading Telegram transcript...') }}
      </div>
      <div v-else-if="!selectedTranscript" class="telegram-call-placeholder">
        {{ t('No transcript projected for this call yet.') }}
      </div>
      <div v-else class="telegram-call-transcript-copy">
        <dl>
          <div><dt>{{ t('Status') }}</dt><dd>{{ selectedTranscript.transcript_status }}</dd></div>
          <div><dt>{{ t('Provider') }}</dt><dd>{{ selectedTranscript.stt_provider }}</dd></div>
          <div><dt>{{ t('Language') }}</dt><dd>{{ selectedTranscript.language_code ?? '—' }}</dd></div>
          <div><dt>{{ t('Audio Ref') }}</dt><dd>{{ selectedTranscript.source_audio_ref ?? '—' }}</dd></div>
        </dl>
        <p>{{ selectedTranscript.transcript_text }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.telegram-call-transcripts {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.telegram-call-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.telegram-call-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-bg, #fafafa);
  text-align: left;
  cursor: pointer;
}

.telegram-call-row--active {
  border-color: var(--color-primary, #0066cc);
  background: var(--color-primary-subtle, #e3f2fd);
}

.telegram-call-row strong,
.telegram-call-transcript-card strong {
  display: block;
  font-size: 12px;
  color: var(--color-text, #333);
}

.telegram-call-row p,
.telegram-call-transcript-copy p {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-call-row small,
.telegram-call-transcript-card small,
.telegram-call-transcript-copy dt {
  color: var(--color-text-secondary, #777);
  font-size: 11px;
}

.telegram-call-transcript-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  border-top: 1px solid var(--color-border, #e0e0e0);
  padding-top: 12px;
}

.telegram-call-transcript-card header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}

.telegram-call-transcript-copy {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.telegram-call-transcript-copy dl {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px 12px;
  margin: 0;
}

.telegram-call-transcript-copy dl div {
  min-width: 0;
}

.telegram-call-transcript-copy dd {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}

.telegram-call-placeholder {
  padding: 10px 12px;
  border-radius: 10px;
  background: var(--color-bg, #fafafa);
  color: var(--color-text-secondary, #777);
  font-size: 12px;
}
</style>
