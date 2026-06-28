<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { ProviderAccount } from '../../../shared/yandexTelemost/settingsBridge'
import {
  completeYandexTelemostRecording,
  createYandexTelemostConference,
  openYandexTelemostCompanion,
  startYandexTelemostRecording,
  stopYandexTelemostRecording,
} from '../api/yandexTelemost'
import {
  useSetupYandexTelemostAccountMutation,
  useYandexTelemostCapabilitiesQuery,
  useYandexTelemostRuntimeStatusQuery,
} from '../queries/useYandexTelemostRuntimeQuery'
import type { YandexTelemostConference, YandexTelemostRecordingSession } from '../types/yandexTelemost'

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const { t } = useI18n()

const setupForm = ref({
  account_id: '',
  display_name: '',
  external_account_id: '',
  oauth_token: '',
  oauth_token_ref: '',
  api_base_url: '',
})

const conferenceForm = ref({
  waiting_room_level: '',
  auto_summary: true,
})

const manualOpenForm = ref({
  join_url: '',
  conference_id: '',
})

const activeAction = ref<string | null>(null)
const actionMessage = ref('')
const errorMessage = ref('')
const lastConference = ref<YandexTelemostConference | null>(null)
const activeRecording = ref<YandexTelemostRecordingSession | null>(null)

const setupAccount = useSetupYandexTelemostAccountMutation()
const { data: capabilities } = useYandexTelemostCapabilitiesQuery()
const selectedTelemostAccountId = computed(() =>
  props.selectedAccount?.provider_kind === 'yandex_telemost_user' ? props.selectedAccount.account_id : null
)
const { data: runtimeStatus } = useYandexTelemostRuntimeStatusQuery(selectedTelemostAccountId)

const isSelected = computed(() => props.selectedAccount?.provider_kind === 'yandex_telemost_user')
const selectedLabel = computed(() => {
  const account = props.selectedAccount
  if (!account) return ''
  return account.display_name || account.external_account_id || account.account_id
})
const canUseSelected = computed(() => Boolean(selectedTelemostAccountId.value))
const safetySummary = computed(() => capabilities.value?.capabilities.find((item) => item.capability === 'telemost.speaker_timeline.webview_hints'))

async function handleSetup() {
  activeAction.value = 'setup'
  errorMessage.value = ''
  actionMessage.value = ''
  try {
    await setupAccount.mutateAsync({
      account_id: setupForm.value.account_id.trim(),
      display_name: setupForm.value.display_name.trim(),
      external_account_id: setupForm.value.external_account_id.trim(),
      oauth_token: valueOrUndefined(setupForm.value.oauth_token),
      oauth_token_ref: valueOrUndefined(setupForm.value.oauth_token_ref),
      api_base_url: valueOrUndefined(setupForm.value.api_base_url),
      metadata: { source: 'settings_panel' },
    })
    setupForm.value.oauth_token = ''
    actionMessage.value = t('Yandex Telemost account connected')
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Yandex Telemost setup failed'
  } finally {
    activeAction.value = null
  }
}

async function handleCreateConference() {
  if (!selectedTelemostAccountId.value) return
  activeAction.value = 'create'
  errorMessage.value = ''
  actionMessage.value = ''
  try {
    const response = await createYandexTelemostConference({
      account_id: selectedTelemostAccountId.value,
      waiting_room_level: valueOrUndefined(conferenceForm.value.waiting_room_level),
      is_auto_summarization_enabled: conferenceForm.value.auto_summary,
      metadata: { source: 'settings_panel' },
    })
    lastConference.value = response.conference
    manualOpenForm.value.join_url = response.conference.join_url
    manualOpenForm.value.conference_id = response.conference.id
    actionMessage.value = t('Yandex Telemost conference created')
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Yandex Telemost conference creation failed'
  } finally {
    activeAction.value = null
  }
}

async function handleOpenWebview() {
  if (!selectedTelemostAccountId.value) return
  activeAction.value = 'open'
  errorMessage.value = ''
  try {
    const manifest = await openYandexTelemostCompanion({
      account_id: selectedTelemostAccountId.value,
      conference_id: valueOrUndefined(manualOpenForm.value.conference_id),
      join_url: manualOpenForm.value.join_url.trim(),
      display_name: selectedLabel.value,
    })
    actionMessage.value = `${t('Yandex Telemost WebView opened')}: ${manifest.window_label}`
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Yandex Telemost WebView open failed'
  } finally {
    activeAction.value = null
  }
}

async function handleStartRecording() {
  if (!selectedTelemostAccountId.value) return
  activeAction.value = 'record'
  errorMessage.value = ''
  try {
    const session = await startYandexTelemostRecording({
      account_id: selectedTelemostAccountId.value,
      conference_id: valueOrUndefined(manualOpenForm.value.conference_id),
      join_url: manualOpenForm.value.join_url.trim(),
      consent_attested: true,
    })
    activeRecording.value = session
    actionMessage.value = `${t('Recording started')}: ${session.audio_path}`
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Yandex Telemost recording start failed'
  } finally {
    activeAction.value = null
  }
}

async function handleStopRecording() {
  const session = activeRecording.value
  if (!session) return
  activeAction.value = 'stop-recording'
  errorMessage.value = ''
  try {
    const receipt = await stopYandexTelemostRecording(session.recording_session_id)
    const bridge = await completeYandexTelemostRecording({
      account_id: session.account_id,
      conference_id: session.conference_id,
      join_url: session.join_url,
      recording_session_id: session.recording_session_id,
      output_dir: session.output_dir,
      audio_path: receipt.audio_path,
      speaker_jsonl_path: receipt.speaker_jsonl_path,
      speaker_txt_path: receipt.speaker_txt_path,
      started_at_epoch_ms: session.started_at_epoch_ms,
      stopped_at_epoch_ms: receipt.stopped_at_epoch_ms,
      consent_attested: session.consent_attested,
    })
    actionMessage.value = `${t('Recording stopped')}: ${bridge.bundle_id}`
    activeRecording.value = null
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Yandex Telemost recording stop failed'
  } finally {
    activeAction.value = null
  }
}

function valueOrUndefined(input: string): string | undefined {
  const trimmed = input.trim()
  return trimmed.length ? trimmed : undefined
}
</script>

<template>
  <section class="integration-section telemost-panel">
    <header class="panel-title-row">
      <div>
        <h3>{{ t('Yandex Telemost') }}</h3>
        <p class="integration-section-description">
          {{ t('Visible WebView, provider API, local MP3 recorder and speaker timeline hints.') }}
        </p>
      </div>
    </header>

    <div v-if="actionMessage" class="setup-state success">{{ actionMessage }}</div>
    <div v-if="errorMessage" class="inline-error">{{ errorMessage }}</div>

    <form class="integration-form" @submit.prevent="handleSetup">
      <h4>{{ t('Connect account') }}</h4>
      <label>{{ t('Account id') }}<input v-model="setupForm.account_id" /></label>
      <label>{{ t('Display name') }}<input v-model="setupForm.display_name" /></label>
      <label>{{ t('External account id') }}<input v-model="setupForm.external_account_id" /></label>
      <label>{{ t('OAuth token') }}<input v-model="setupForm.oauth_token" type="password" autocomplete="off" /></label>
      <label>{{ t('Existing token secret ref') }}<input v-model="setupForm.oauth_token_ref" /></label>
      <label>{{ t('API base URL') }}<input v-model="setupForm.api_base_url" placeholder="https://cloud-api.yandex.net/v1/telemost-api" /></label>
      <button type="submit" class="hermes-btn hermes-btn--primary" :disabled="activeAction==='setup'">
        {{ t('Connect Yandex Telemost') }}
      </button>
    </form>

    <div v-if="isSelected" class="integration-section nested">
      <h4>{{ t('Selected Telemost account') }}: {{ selectedLabel }}</h4>
      <p class="integration-section-description">
        {{ t('Runtime') }}: {{ runtimeStatus?.lifecycle_state || '-' }} · {{ t('Blockers') }}: {{ runtimeStatus?.blockers?.length ?? 0 }}
      </p>

      <div class="integration-form split">
        <label>{{ t('Waiting room level') }}<input v-model="conferenceForm.waiting_room_level" placeholder="PUBLIC" /></label>
        <label class="inline-check"><input v-model="conferenceForm.auto_summary" type="checkbox" /> {{ t('Request provider auto-summary') }}</label>
        <button type="button" class="hermes-btn hermes-btn--secondary" :disabled="!canUseSelected || activeAction==='create'" @click="handleCreateConference">
          {{ t('Create conference') }}
        </button>
      </div>

      <div class="integration-form split">
        <label>{{ t('Join URL') }}<input v-model="manualOpenForm.join_url" placeholder="https://telemost.yandex.ru/j/..." /></label>
        <label>{{ t('Conference id') }}<input v-model="manualOpenForm.conference_id" /></label>
        <button type="button" class="hermes-btn hermes-btn--secondary" :disabled="!manualOpenForm.join_url.trim() || activeAction==='open'" @click="handleOpenWebview">
          {{ t('Open in Hermes WebView') }}
        </button>
        <button type="button" class="hermes-btn hermes-btn--outline" :disabled="!manualOpenForm.join_url.trim() || Boolean(activeRecording) || activeAction==='record'" @click="handleStartRecording">
          {{ t('Start local MP3 recording') }}
        </button>
        <button type="button" class="hermes-btn hermes-btn--outline" :disabled="!activeRecording || activeAction==='stop-recording'" @click="handleStopRecording">
          {{ t('Stop recording') }}
        </button>
      </div>

      <div v-if="lastConference" class="telemost-result">
        <strong>{{ t('Last conference') }}</strong>
        <code>{{ lastConference.id }}</code>
        <span>{{ lastConference.join_url }}</span>
      </div>
      <div v-if="activeRecording" class="telemost-result">
        <strong>{{ t('Active recording') }}</strong>
        <code>{{ activeRecording.recording_session_id }}</code>
        <span>{{ activeRecording.audio_path }}</span>
        <span>{{ activeRecording.speaker_txt_path }}</span>
      </div>
    </div>

    <div v-if="safetySummary" class="telemost-safety">
      <strong>{{ t('Safety boundary') }}</strong>
      <span>{{ safetySummary.status }} · {{ safetySummary.source }}</span>
    </div>
  </section>
</template>

<style scoped>
.telemost-panel { display: grid; gap: 12px; }
.integration-form.split { margin-top: 12px; }
.integration-form input { width: 100%; padding: 8px; border: 1px solid var(--hh-border); border-radius: var(--hh-radius-sm); background: var(--hh-surface-deep); color: var(--hh-text-primary); }
.inline-check { display: flex !important; grid-template-columns: auto 1fr; align-items: center; gap: 8px; }
.inline-check input { width: auto; }
.nested { background: color-mix(in srgb, var(--hh-surface-deep) 88%, transparent); }
.telemost-result, .telemost-safety { display: grid; gap: 4px; margin-top: 10px; padding: 10px; border: 1px solid var(--hh-border); border-radius: var(--hh-radius-sm); font-size: 12px; color: var(--hh-text-secondary); }
.telemost-result code { color: var(--hh-text-primary); }
</style>
