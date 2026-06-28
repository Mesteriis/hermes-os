<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'
import { useI18n } from '../../../platform/i18n'
import { settingsKeys, useApplicationSettingsQuery, useSettingsStore } from '../../../shared/zoom/settingsBridge'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import {
  useCleanupZoomRetentionMutation,
  useSyncZoomRecordingsMutation,
} from '../queries/useZoomRuntimeQuery'

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const { t } = useI18n()
const store = useSettingsStore()
const queryClient = useQueryClient()
const { data: applicationSettingsData } = useApplicationSettingsQuery()

const recordingSyncForm = ref({
  user_id: '',
  from: '2026-06-01',
  to: '2026-06-30',
  page_size: '30',
  max_meetings: '100',
  api_base_url: '',
})
const activeAction = ref<string | null>(null)

const selectedZoomAccountId = computed(() => props.selectedAccount?.account_id ?? null)
const syncZoomRecordings = useSyncZoomRecordingsMutation()
const cleanupZoomRetention = useCleanupZoomRetentionMutation(selectedZoomAccountId)

const zoomRecordingRetentionDays = computed(() =>
  applicationIntegerSetting('privacy.zoom_recording_import_retention_days')
)
const zoomTranscriptRetentionDays = computed(() =>
  applicationIntegerSetting('privacy.zoom_transcript_retention_days')
)

function isZoomProvider(providerKind: string): boolean {
  return providerKind === 'zoom_user' || providerKind === 'zoom_server_to_server'
}

function valueOrUndefined(input: string): string | undefined {
  const trimmed = input.trim()
  return trimmed.length ? trimmed : undefined
}

function positiveIntegerOrUndefined(input: string): number | undefined {
  const trimmed = input.trim()
  if (!trimmed) return undefined
  const parsed = Number.parseInt(trimmed, 10)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined
}

function applicationIntegerSetting(settingKey: string): number {
  const setting = applicationSettingsData.value?.items?.find((item) => item.setting_key === settingKey)
  return typeof setting?.value === 'number' ? setting.value : 0
}

async function refreshSettings() {
  await queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
}

async function handleSyncZoomRecordings() {
  if (!props.selectedAccount || !isZoomProvider(props.selectedAccount.provider_kind)) return

  activeAction.value = `recording-sync:${props.selectedAccount.account_id}`
  try {
    const result = await syncZoomRecordings.mutateAsync({
      account_id: props.selectedAccount.account_id,
      user_id: valueOrUndefined(recordingSyncForm.value.user_id),
      from: recordingSyncForm.value.from.trim(),
      to: recordingSyncForm.value.to.trim(),
      page_size: positiveIntegerOrUndefined(recordingSyncForm.value.page_size),
      max_meetings: positiveIntegerOrUndefined(recordingSyncForm.value.max_meetings),
      api_base_url: valueOrUndefined(recordingSyncForm.value.api_base_url),
    })
    store.setActionMessage(
      `Zoom recording sync completed: ${result.meetings_recorded} meetings, ${result.recordings_recorded} recordings, ${result.media_downloads_recorded} media downloads, ${result.transcripts_recorded} transcripts`
    )
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom recording sync failed')
  } finally {
    activeAction.value = null
  }
}

async function handleCleanupZoomRetention() {
  if (!selectedZoomAccountId.value) return

  activeAction.value = `retention-cleanup:${selectedZoomAccountId.value}`
  try {
    const result = await cleanupZoomRetention.mutateAsync({
      remove_recordings: true,
      remove_transcripts: true,
      limit: 100,
    })
    store.setActionMessage(
      t('Zoom retention cleanup removed') +
        ` ${result.recordings_removed} ${t('recordings')} / ${result.transcripts_removed} ${t('transcripts')}`
    )
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom retention cleanup failed')
  } finally {
    activeAction.value = null
  }
}
</script>

<template>
  <section
    v-if="selectedAccount && isZoomProvider(selectedAccount.provider_kind)"
    class="integration-section compact"
  >
    <h4>{{ t('Manual recording sync') }}</h4>
    <p class="integration-section-description">
      {{ t('Provider-side recording media downloads require privacy.zoom_remote_recording_download_enabled. Provider-side transcript downloads require privacy.zoom_remote_transcript_download_enabled.') }}
    </p>
    <p class="integration-section-description">
      {{ t('Imported recording blobs follow privacy.zoom_recording_import_retention_days') }} =
      {{ zoomRecordingRetentionDays }}.
      {{ t('Transcript evidence follows privacy.zoom_transcript_retention_days') }} =
      {{ zoomTranscriptRetentionDays }}.
      {{ t('0 means explicit removal only.') }}
    </p>
    <form class="integration-form" @submit.prevent="handleSyncZoomRecordings">
      <input
        v-model="recordingSyncForm.user_id"
        class="hermes-input-control"
        type="text"
        :placeholder="t('Zoom user id override (optional)')"
      />
      <div class="maintenance-grid">
        <input v-model="recordingSyncForm.from" class="hermes-input-control" type="date" required />
        <input v-model="recordingSyncForm.to" class="hermes-input-control" type="date" required />
      </div>
      <div class="maintenance-grid">
        <input
          v-model="recordingSyncForm.page_size"
          class="hermes-input-control"
          type="number"
          min="1"
          max="100"
          :placeholder="t('30')"
        />
        <input
          v-model="recordingSyncForm.max_meetings"
          class="hermes-input-control"
          type="number"
          min="1"
          max="500"
          :placeholder="t('100')"
        />
      </div>
      <input
        v-model="recordingSyncForm.api_base_url"
        class="hermes-input-control"
        type="text"
        :placeholder="t('https://api.zoom.us/v2')"
      />
      <button
        type="submit"
        class="hermes-btn hermes-btn--outline"
        :disabled="activeAction === `recording-sync:${selectedAccount.account_id}` || syncZoomRecordings.isPending.value"
      >
        {{ syncZoomRecordings.isPending.value ? t('Syncing...') : t('Sync cloud recordings') }}
      </button>
    </form>
    <div class="inspector-actions">
      <button
        type="button"
        class="hermes-btn hermes-btn--outline"
        :disabled="activeAction === `retention-cleanup:${selectedAccount.account_id}` || cleanupZoomRetention.isPending.value"
        @click="handleCleanupZoomRetention"
      >
        {{ cleanupZoomRetention.isPending.value ? t('Cleaning...') : t('Run retention cleanup') }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.integration-section { border: 1px solid var(--hh-border); border-radius: var(--hh-radius-md); background: var(--hh-surface-deep); padding: 12px; }
.integration-section.compact { margin-top: 12px; }
.integration-section h4 { margin: 0 0 6px; }
.integration-section-description { margin: 0 0 8px; font-size: 12px; color: var(--hh-text-muted); }
.integration-form { display: grid; gap: 8px; }
.integration-form button { margin-top: 6px; }
.maintenance-grid { display: grid; gap: 12px; grid-template-columns: repeat(2, minmax(0, 1fr)); }
.inspector-actions { display: flex; gap: 8px; margin-top: 12px; flex-wrap: wrap; }
@media (max-width: 960px) {
  .maintenance-grid { grid-template-columns: minmax(0, 1fr); }
}
</style>
