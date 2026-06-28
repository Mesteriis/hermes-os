<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'
import { useI18n } from '../../../platform/i18n'
import { settingsKeys, useSettingsStore } from '../../../shared/zoom/settingsBridge'
import type { ProviderAccount } from '../../../shared/zoom/settingsBridge'
import ZoomAuditEventsPanel from './ZoomAuditEventsPanel.vue'
import ZoomBridgeLab from './ZoomBridgeLab.vue'
import ZoomRecordingMaintenancePanel from './ZoomRecordingMaintenancePanel.vue'
import ZoomObservedCallsPanel from './ZoomObservedCallsPanel.vue'
import ZoomRecordingImportsPanel from './ZoomRecordingImportsPanel.vue'
import {
  useAuthorizeZoomServerToServerMutation,
  useCompleteZoomOAuthMutation,
  useMaintainZoomTokensMutation,
  useRefreshZoomTokenMutation,
  useRemoveZoomRuntimeMutation,
  useSetupZoomFixtureAccountMutation,
  useSetupZoomLiveAccountMutation,
  useStartZoomOAuthMutation,
  useStartZoomRuntimeMutation,
  useStopZoomRuntimeMutation,
  useZoomCapabilitiesQuery,
  useZoomRuntimeStatusQuery,
} from '../queries/useZoomRuntimeQuery'

type ZoomAuthShape = 'oauth_user' | 'server_to_server'

const props = defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

const emit = defineEmits<{
  removed: []
}>()

const { t } = useI18n()
const store = useSettingsStore()
const queryClient = useQueryClient()

const fixtureForm = ref({
  account_id: '',
  display_name: '',
  external_account_id: '',
  account_email: '',
})

const liveForm = ref({
  account_id: '',
  display_name: '',
  external_account_id: '',
  auth_shape: 'oauth_user' as ZoomAuthShape,
  client_id: '',
  token_secret_ref: '',
  client_secret_ref: '',
  webhook_secret_ref: '',
})

const oauthStartForm = ref({
  redirect_uri: 'http://127.0.0.1:8080/api/v1/integrations/zoom/oauth/callback',
  client_secret: '',
  client_secret_ref: '',
  webhook_secret_ref: '',
  scopes: 'meeting:read recording:read',
  authorization_endpoint: '',
  token_endpoint: '',
})

const oauthCompleteForm = ref({
  setup_id: '',
  state: '',
  authorization_code: '',
  external_account_id: '',
})

const s2sAuthorizeForm = ref({
  client_secret: '',
  client_secret_ref: '',
  zoom_account_id: '',
  token_endpoint: '',
})

const tokenRefreshForm = ref({
  refresh_expiring_within_seconds: '60',
})

const tokenMaintenanceForm = ref({
  refresh_expiring_within_seconds: '300',
})

const activeAction = ref<string | null>(null)
const pendingOAuthAuthorizationUrl = ref<string>('')

const setupZoomFixtureAccount = useSetupZoomFixtureAccountMutation()
const setupZoomLiveAccount = useSetupZoomLiveAccountMutation()
const startZoomOAuth = useStartZoomOAuthMutation()
const completeZoomOAuth = useCompleteZoomOAuthMutation()
const authorizeZoomServerToServer = useAuthorizeZoomServerToServerMutation()
const refreshZoomToken = useRefreshZoomTokenMutation()
const maintainZoomTokens = useMaintainZoomTokensMutation()
const startZoomRuntime = useStartZoomRuntimeMutation()
const stopZoomRuntime = useStopZoomRuntimeMutation()
const removeZoomRuntime = useRemoveZoomRuntimeMutation()

const selectedZoomAccountId = computed(() => props.selectedAccount?.account_id ?? null)
const { data: selectedZoomRuntime } = useZoomRuntimeStatusQuery(selectedZoomAccountId)
const { data: zoomCapabilities } = useZoomCapabilitiesQuery()

const selectedZoomConfig = computed<Record<string, unknown>>(() =>
  asRecord(props.selectedAccount?.config) ?? {}
)

const selectedZoomAuthShape = computed(() => {
  if (typeof selectedZoomRuntime.value?.auth_shape === 'string') return selectedZoomRuntime.value.auth_shape
  if (props.selectedAccount?.provider_kind === 'zoom_server_to_server') return 'server_to_server'
  if (props.selectedAccount?.provider_kind === 'zoom_user') return 'oauth_user'
  return null
})

const selectedTokenRotationPolicy = computed<Record<string, unknown>>(() => {
  return asRecord(selectedZoomRuntime.value?.metadata?.token_rotation_policy) ?? {}
})
const selectedTokenRotationPolicyConfig = computed<Record<string, unknown>>(
  () => asRecord(selectedTokenRotationPolicy.value.policy) ?? {}
)

const selectedZoomRuntimeBlockers = computed(() => selectedZoomRuntime.value?.runtime_blockers?.join(', ') || t('None'))
const plannedZoomFeatures = computed(() => zoomCapabilities.value?.planned_features?.join(', ') || t('None'))
const unsupportedZoomFeatures = computed(() => zoomCapabilities.value?.unsupported_features?.join(', ') || t('None'))

function isZoomProvider(providerKind: string): boolean {
  return providerKind === 'zoom_user' || providerKind === 'zoom_server_to_server'
}

function selectedAccountEmail(): string {
  if (!props.selectedAccount) return ''
  return props.selectedAccount.email || selectedString(selectedZoomConfig.value, 'email') || ''
}

function selectedDisplayName(): string {
  if (!props.selectedAccount) return ''
  return (
    props.selectedAccount.display_name ||
    props.selectedAccount.label ||
    selectedAccountEmail() ||
    props.selectedAccount.external_account_id ||
    props.selectedAccount.account_id
  )
}

function selectedClientId(): string {
  return selectedString(selectedZoomConfig.value, 'client_id') || ''
}

function selectedRuntimePolicyLabel(key: string): string {
  const value = selectedTokenRotationPolicy.value[key]
  if (typeof value === 'boolean') return value ? t('Yes') : t('No')
  if (typeof value === 'number') return String(value)
  if (typeof value === 'string') return value
  return '-'
}

function selectedRuntimePolicyDate(): string {
  const value = selectedTokenRotationPolicy.value.expires_at
  return typeof value === 'string' ? value : '-'
}

function selectedRuntimePolicyThreshold(key: string): string {
  const value = selectedTokenRotationPolicyConfig.value[key]
  return typeof value === 'number' ? `${value}s` : '-'
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

function splitScopes(input: string): string[] {
  return input
    .split(/[\s,]+/)
    .map((value) => value.trim())
    .filter(Boolean)
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null
  return value as Record<string, unknown>
}

function selectedString(record: Record<string, unknown>, key: string): string | null {
  const value = record[key]
  return typeof value === 'string' && value.trim() ? value.trim() : null
}

async function refreshSettings() {
  await queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() })
}

async function handleCreateZoomFixture() {
  const account_id = fixtureForm.value.account_id.trim()
  const display_name = fixtureForm.value.display_name.trim()
  const external_account_id = fixtureForm.value.external_account_id.trim()
  const account_email = fixtureForm.value.account_email.trim()
  if (!account_id || !display_name || !external_account_id) {
    store.setError(t('Account id, display name and external account id are required'))
    return
  }

  activeAction.value = 'fixture'
  try {
    await setupZoomFixtureAccount.mutateAsync({
      account_id,
      display_name,
      external_account_id,
      account_email: account_email || undefined,
    })
    fixtureForm.value = { account_id: '', display_name: '', external_account_id: '', account_email: '' }
    store.setActionMessage(t('Zoom fixture account created'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom fixture setup failed')
  } finally {
    activeAction.value = null
  }
}

async function handleCreateZoomLive() {
  const account_id = liveForm.value.account_id.trim()
  const display_name = liveForm.value.display_name.trim()
  const external_account_id = liveForm.value.external_account_id.trim()
  const client_id = liveForm.value.client_id.trim()
  if (!account_id || !display_name || !external_account_id || !client_id) {
    store.setError(t('Account id, display name, external account id and client id are required'))
    return
  }

  activeAction.value = 'live'
  try {
    await setupZoomLiveAccount.mutateAsync({
      account_id,
      display_name,
      external_account_id,
      auth_shape: liveForm.value.auth_shape,
      client_id,
      token_secret_ref: valueOrUndefined(liveForm.value.token_secret_ref),
      client_secret_ref: valueOrUndefined(liveForm.value.client_secret_ref),
      webhook_secret_ref: valueOrUndefined(liveForm.value.webhook_secret_ref),
    })
    liveForm.value = {
      account_id: '',
      display_name: '',
      external_account_id: '',
      auth_shape: liveForm.value.auth_shape,
      client_id: '',
      token_secret_ref: '',
      client_secret_ref: '',
      webhook_secret_ref: '',
    }
    store.setActionMessage(t('Zoom live account metadata registered'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom live setup failed')
  } finally {
    activeAction.value = null
  }
}

async function handleStartZoomOAuth() {
  if (!props.selectedAccount || !isZoomProvider(props.selectedAccount.provider_kind)) return
  if (selectedZoomAuthShape.value !== 'oauth_user') {
    store.setError(t('Selected Zoom account is not an OAuth user account'))
    return
  }
  const client_id = selectedClientId()
  if (!client_id) {
    store.setError(t('Selected Zoom account has no client id in metadata'))
    return
  }

  activeAction.value = `oauth-start:${props.selectedAccount.account_id}`
  try {
    const response = await startZoomOAuth.mutateAsync({
      account_id: props.selectedAccount.account_id,
      display_name: selectedDisplayName(),
      external_account_id: props.selectedAccount.external_account_id,
      account_email: selectedAccountEmail() || undefined,
      client_id,
      client_secret: valueOrUndefined(oauthStartForm.value.client_secret),
      client_secret_ref: valueOrUndefined(oauthStartForm.value.client_secret_ref),
      webhook_secret_ref: valueOrUndefined(oauthStartForm.value.webhook_secret_ref),
      redirect_uri: oauthStartForm.value.redirect_uri.trim(),
      scopes: splitScopes(oauthStartForm.value.scopes),
      authorization_endpoint: valueOrUndefined(oauthStartForm.value.authorization_endpoint),
      token_endpoint: valueOrUndefined(oauthStartForm.value.token_endpoint),
    })
    oauthCompleteForm.value.setup_id = response.setup_id
    oauthCompleteForm.value.state = response.state
    oauthCompleteForm.value.external_account_id = props.selectedAccount.external_account_id
    pendingOAuthAuthorizationUrl.value = response.authorization_url
    window.open(response.authorization_url, '_blank', 'noopener,noreferrer')
    store.setActionMessage(t('Zoom OAuth authorization started'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom OAuth start failed')
  } finally {
    activeAction.value = null
  }
}

async function handleCompleteZoomOAuth() {
  if (!props.selectedAccount || !isZoomProvider(props.selectedAccount.provider_kind)) return
  activeAction.value = `oauth-complete:${props.selectedAccount.account_id}`
  try {
    await completeZoomOAuth.mutateAsync({
      setup_id: oauthCompleteForm.value.setup_id.trim(),
      state: oauthCompleteForm.value.state.trim(),
      authorization_code: oauthCompleteForm.value.authorization_code.trim(),
      external_account_id: valueOrUndefined(oauthCompleteForm.value.external_account_id),
    })
    oauthCompleteForm.value.authorization_code = ''
    pendingOAuthAuthorizationUrl.value = ''
    store.setActionMessage(t('Zoom OAuth authorization completed'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom OAuth completion failed')
  } finally {
    activeAction.value = null
  }
}

async function handleAuthorizeZoomServerToServer() {
  if (!props.selectedAccount || !isZoomProvider(props.selectedAccount.provider_kind)) return
  if (selectedZoomAuthShape.value !== 'server_to_server') {
    store.setError(t('Selected Zoom account is not a server-to-server account'))
    return
  }
  const client_id = selectedClientId()
  if (!client_id) {
    store.setError(t('Selected Zoom account has no client id in metadata'))
    return
  }

  activeAction.value = `s2s-authorize:${props.selectedAccount.account_id}`
  try {
    await authorizeZoomServerToServer.mutateAsync({
      account_id: props.selectedAccount.account_id,
      client_id,
      client_secret: valueOrUndefined(s2sAuthorizeForm.value.client_secret),
      client_secret_ref: valueOrUndefined(s2sAuthorizeForm.value.client_secret_ref),
      zoom_account_id: valueOrUndefined(s2sAuthorizeForm.value.zoom_account_id),
      token_endpoint: valueOrUndefined(s2sAuthorizeForm.value.token_endpoint),
    })
    store.setActionMessage(t('Zoom server to server authorization completed'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom server to server authorization failed')
  } finally {
    activeAction.value = null
  }
}

async function handleRefreshZoomToken() {
  if (!props.selectedAccount || !isZoomProvider(props.selectedAccount.provider_kind)) return
  activeAction.value = `refresh:${props.selectedAccount.account_id}`
  try {
    await refreshZoomToken.mutateAsync({
      account_id: props.selectedAccount.account_id,
      force: true,
      refresh_expiring_within_seconds: positiveIntegerOrUndefined(
        tokenRefreshForm.value.refresh_expiring_within_seconds
      ),
    })
    store.setActionMessage(t('Zoom token refresh requested'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom token refresh failed')
  } finally {
    activeAction.value = null
  }
}

async function handleMaintainZoomTokens() {
  if (!props.selectedAccount || !isZoomProvider(props.selectedAccount.provider_kind)) return
  activeAction.value = `maintenance:${props.selectedAccount.account_id}`
  try {
    await maintainZoomTokens.mutateAsync({
      account_id: props.selectedAccount.account_id,
      refresh_expiring_within_seconds: positiveIntegerOrUndefined(
        tokenMaintenanceForm.value.refresh_expiring_within_seconds
      ),
    })
    store.setActionMessage(t('Zoom token maintenance requested'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom token maintenance failed')
  } finally {
    activeAction.value = null
  }
}

async function handleStartZoomRuntime(accountId: string) {
  activeAction.value = `start:${accountId}`
  try {
    await startZoomRuntime.mutateAsync({ account_id: accountId, force: true })
    store.setActionMessage(t('Zoom runtime start requested'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom runtime start failed')
  } finally {
    activeAction.value = null
  }
}

async function handleStopZoomRuntime(accountId: string) {
  activeAction.value = `stop:${accountId}`
  try {
    await stopZoomRuntime.mutateAsync({ account_id: accountId, reason: 'Manual stop from settings' })
    store.setActionMessage(t('Zoom runtime stop requested'))
    await refreshSettings()
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom runtime stop failed')
  } finally {
    activeAction.value = null
  }
}

async function handleRemoveZoomRuntime(accountId: string) {
  activeAction.value = `remove:${accountId}`
  try {
    await removeZoomRuntime.mutateAsync({ account_id: accountId, reason: 'Manual removal from settings' })
    store.setActionMessage(t('Zoom runtime account removed'))
    await refreshSettings()
    emit('removed')
  } catch (err) {
    store.setError(err instanceof Error ? err.message : 'Zoom runtime remove failed')
  } finally {
    activeAction.value = null
  }
}
</script>

<template>
  <div class="zoom-settings-panel">
    <div v-if="selectedAccount && isZoomProvider(selectedAccount.provider_kind)" class="integration-inspector">
      <header>
        <h3>{{ selectedDisplayName() }}</h3>
      </header>

      <div class="inspector-grid">
        <section class="inspector-card">
          <h4>{{ t('Zoom runtime') }}</h4>
          <div class="detail-row"><span>{{ t('Runtime') }}</span><strong>{{ selectedZoomRuntime?.status ?? t('Unknown') }}</strong></div>
          <div class="detail-row"><span>{{ t('Healthy') }}</span><strong>{{ selectedZoomRuntime?.healthy ? t('Yes') : t('No') }}</strong></div>
          <div class="detail-row"><span>{{ t('Auth shape') }}</span><strong>{{ selectedZoomAuthShape ?? '-' }}</strong></div>
          <div class="detail-row"><span>{{ t('Blockers') }}</span><strong>{{ selectedZoomRuntimeBlockers }}</strong></div>
          <div class="detail-row"><span>{{ t('Last error') }}</span><strong>{{ selectedZoomRuntime?.last_error ?? '-' }}</strong></div>
          <div class="detail-row"><span>{{ t('Planned features') }}</span><strong>{{ plannedZoomFeatures }}</strong></div>
          <div class="detail-row"><span>{{ t('Unsupported') }}</span><strong>{{ unsupportedZoomFeatures }}</strong></div>
          <div class="inspector-actions">
            <button type="button" class="hermes-btn hermes-btn--outline"
              :disabled="activeAction===`start:${selectedAccount.account_id}` || startZoomRuntime.isPending.value"
              @click="handleStartZoomRuntime(selectedAccount.account_id)">
              {{ t('Start runtime') }}
            </button>
            <button type="button" class="hermes-btn hermes-btn--outline"
              :disabled="activeAction===`stop:${selectedAccount.account_id}` || stopZoomRuntime.isPending.value"
              @click="handleStopZoomRuntime(selectedAccount.account_id)">
              {{ t('Stop runtime') }}
            </button>
            <button type="button" class="hermes-btn hermes-btn--destructive"
              :disabled="activeAction===`remove:${selectedAccount.account_id}` || removeZoomRuntime.isPending.value"
              @click="handleRemoveZoomRuntime(selectedAccount.account_id)">
              {{ t('Remove runtime account') }}
            </button>
          </div>
        </section>

        <section class="inspector-card">
          <h4>{{ t('Token rotation policy') }}</h4>
          <div class="detail-row"><span>{{ t('Policy status') }}</span><strong>{{ selectedRuntimePolicyLabel('status') }}</strong></div>
          <div class="detail-row"><span>{{ t('Rotation required') }}</span><strong>{{ selectedRuntimePolicyLabel('rotation_required') }}</strong></div>
          <div class="detail-row"><span>{{ t('Refresh due') }}</span><strong>{{ selectedRuntimePolicyLabel('refresh_due') }}</strong></div>
          <div class="detail-row"><span>{{ t('Expired') }}</span><strong>{{ selectedRuntimePolicyLabel('expired') }}</strong></div>
          <div class="detail-row"><span>{{ t('Last refresh status') }}</span><strong>{{ selectedRuntimePolicyLabel('last_refresh_status') }}</strong></div>
          <div class="detail-row"><span>{{ t('Expires at') }}</span><strong>{{ selectedRuntimePolicyDate() }}</strong></div>
          <div class="detail-row"><span>{{ t('Explicit threshold') }}</span><strong>{{ selectedRuntimePolicyThreshold('explicit_refresh_threshold_seconds') }}</strong></div>
          <div class="detail-row"><span>{{ t('Maintenance threshold') }}</span><strong>{{ selectedRuntimePolicyThreshold('maintenance_refresh_threshold_seconds') }}</strong></div>
        </section>
      </div>

      <section v-if="selectedZoomAuthShape === 'oauth_user'" class="integration-section compact">
        <h4>{{ t('OAuth authorization') }}</h4>
        <form class="integration-form" @submit.prevent="handleStartZoomOAuth">
          <input v-model="oauthStartForm.redirect_uri" class="hermes-input-control" type="text" :placeholder="t('http://127.0.0.1:8080/api/v1/integrations/zoom/oauth/callback')" required />
          <input v-model="oauthStartForm.client_secret" class="hermes-input-control" type="password" :placeholder="t('Zoom client secret (optional when using secret ref)')" />
          <input v-model="oauthStartForm.client_secret_ref" class="hermes-input-control" type="text" :placeholder="t('secret:zoom-client-secret')" />
          <input v-model="oauthStartForm.webhook_secret_ref" class="hermes-input-control" type="text" :placeholder="t('secret:zoom-webhook-secret')" />
          <input v-model="oauthStartForm.scopes" class="hermes-input-control" type="text" :placeholder="t('meeting:read recording:read')" />
          <input v-model="oauthStartForm.authorization_endpoint" class="hermes-input-control" type="text" :placeholder="t('https://zoom.us/oauth/authorize')" />
          <input v-model="oauthStartForm.token_endpoint" class="hermes-input-control" type="text" :placeholder="t('https://zoom.us/oauth/token')" />
          <button type="submit" class="hermes-btn hermes-btn--outline"
            :disabled="activeAction===`oauth-start:${selectedAccount.account_id}` || startZoomOAuth.isPending.value">
            {{ startZoomOAuth.isPending.value ? t('Starting...') : t('Start OAuth authorization') }}
          </button>
        </form>
        <form class="integration-form" @submit.prevent="handleCompleteZoomOAuth">
          <input v-model="oauthCompleteForm.setup_id" class="hermes-input-control" type="text" :placeholder="t('OAuth setup id')" required />
          <input v-model="oauthCompleteForm.state" class="hermes-input-control" type="text" :placeholder="t('OAuth state')" required />
          <input v-model="oauthCompleteForm.authorization_code" class="hermes-input-control" type="text" :placeholder="t('Authorization code')" required />
          <input v-model="oauthCompleteForm.external_account_id" class="hermes-input-control" type="text" :placeholder="t('zoom-live-external-id')" />
          <a v-if="pendingOAuthAuthorizationUrl" class="oauth-link" :href="pendingOAuthAuthorizationUrl" target="_blank" rel="noreferrer">
            {{ t('Open Zoom authorization URL') }}
          </a>
          <button type="submit" class="hermes-btn hermes-btn--outline"
            :disabled="activeAction===`oauth-complete:${selectedAccount.account_id}` || completeZoomOAuth.isPending.value">
            {{ completeZoomOAuth.isPending.value ? t('Completing...') : t('Complete OAuth authorization') }}
          </button>
        </form>
      </section>

      <section v-if="selectedZoomAuthShape === 'server_to_server'" class="integration-section compact">
        <h4>{{ t('Server to server authorization') }}</h4>
        <form class="integration-form" @submit.prevent="handleAuthorizeZoomServerToServer">
          <input v-model="s2sAuthorizeForm.client_secret" class="hermes-input-control" type="password" :placeholder="t('Zoom client secret (optional when using secret ref)')" />
          <input v-model="s2sAuthorizeForm.client_secret_ref" class="hermes-input-control" type="text" :placeholder="t('secret:zoom-client-secret')" />
          <input v-model="s2sAuthorizeForm.zoom_account_id" class="hermes-input-control" type="text" :placeholder="t('Zoom account id override (optional)')" />
          <input v-model="s2sAuthorizeForm.token_endpoint" class="hermes-input-control" type="text" :placeholder="t('https://zoom.us/oauth/token')" />
          <button type="submit" class="hermes-btn hermes-btn--outline"
            :disabled="activeAction===`s2s-authorize:${selectedAccount.account_id}` || authorizeZoomServerToServer.isPending.value">
            {{ authorizeZoomServerToServer.isPending.value ? t('Authorizing...') : t('Authorize server to server account') }}
          </button>
        </form>
      </section>

      <section class="integration-section compact">
        <h4>{{ t('Credential maintenance') }}</h4>
        <div class="maintenance-grid">
          <form class="integration-form" @submit.prevent="handleRefreshZoomToken">
            <input v-model="tokenRefreshForm.refresh_expiring_within_seconds" class="hermes-input-control" type="number" min="60" max="86400" :placeholder="t('60')" />
            <button type="submit" class="hermes-btn hermes-btn--outline"
              :disabled="activeAction===`refresh:${selectedAccount.account_id}` || refreshZoomToken.isPending.value">
              {{ refreshZoomToken.isPending.value ? t('Refreshing...') : t('Refresh token now') }}
            </button>
          </form>
          <form class="integration-form" @submit.prevent="handleMaintainZoomTokens">
            <input v-model="tokenMaintenanceForm.refresh_expiring_within_seconds" class="hermes-input-control" type="number" min="60" max="86400" :placeholder="t('300')" />
            <button type="submit" class="hermes-btn hermes-btn--outline"
              :disabled="activeAction===`maintenance:${selectedAccount.account_id}` || maintainZoomTokens.isPending.value">
              {{ maintainZoomTokens.isPending.value ? t('Running...') : t('Run token maintenance') }}
            </button>
          </form>
        </div>
      </section>
    </div>

    <section class="integration-section">
      <h3>{{ t('Create Zoom fixture account') }}</h3>
      <form class="integration-form" @submit.prevent="handleCreateZoomFixture">
        <input v-model="fixtureForm.account_id" class="hermes-input-control" type="text" :placeholder="t('zoom-fixture-001')" required />
        <input v-model="fixtureForm.display_name" class="hermes-input-control" type="text" :placeholder="t('Zoom fixture')" required />
        <input v-model="fixtureForm.external_account_id" class="hermes-input-control" type="text" :placeholder="t('zoom-user-id')" required />
        <input v-model="fixtureForm.account_email" class="hermes-input-control" type="email" :placeholder="t('fixture@example.test')" />
        <button type="submit" class="hermes-btn hermes-btn--outline" :disabled="setupZoomFixtureAccount.isPending.value || activeAction==='fixture'">
          {{ setupZoomFixtureAccount.isPending.value ? t('Creating...') : t('Create fixture account') }}
        </button>
      </form>
    </section>

    <section class="integration-section">
      <h3>{{ t('Register Zoom live account metadata') }}</h3>
      <p class="integration-section-description">{{ t('Use secret references only. No raw tokens are sent from this form.') }}</p>
      <form class="integration-form" @submit.prevent="handleCreateZoomLive">
        <input v-model="liveForm.account_id" class="hermes-input-control" type="text" :placeholder="t('zoom-live-001')" required />
        <input v-model="liveForm.display_name" class="hermes-input-control" type="text" :placeholder="t('Zoom live account')" required />
        <input v-model="liveForm.external_account_id" class="hermes-input-control" type="text" :placeholder="t('zoom-live-external-id')" required />
        <label>
          <span>{{ t('Auth shape') }}</span>
          <select v-model="liveForm.auth_shape" class="hermes-select-control">
            <option value="oauth_user">{{ t('OAuth user') }}</option>
            <option value="server_to_server">{{ t('Server to server') }}</option>
          </select>
        </label>
        <input v-model="liveForm.client_id" class="hermes-input-control" type="text" :placeholder="t('zoom-client-id')" required />
        <input v-model="liveForm.token_secret_ref" class="hermes-input-control" type="text" :placeholder="t('secret:zoom-token')" />
        <input v-model="liveForm.client_secret_ref" class="hermes-input-control" type="text" :placeholder="t('secret:zoom-client-secret')" />
        <input v-model="liveForm.webhook_secret_ref" class="hermes-input-control" type="text" :placeholder="t('secret:zoom-webhook-secret')" />
        <button type="submit" class="hermes-btn hermes-btn--outline" :disabled="setupZoomLiveAccount.isPending.value || activeAction==='live'">
          {{ setupZoomLiveAccount.isPending.value ? t('Registering...') : t('Register live account') }}
        </button>
      </form>
    </section>

    <ZoomBridgeLab :selected-account="selectedAccount" />
    <ZoomRecordingMaintenancePanel :selected-account="selectedAccount" />
    <ZoomObservedCallsPanel :selected-account="selectedAccount" />
    <ZoomRecordingImportsPanel :selected-account="selectedAccount" />
    <ZoomAuditEventsPanel :selected-account="selectedAccount" />
  </div>
</template>

<style scoped>
.zoom-settings-panel { display: grid; gap: 12px; }
.integration-inspector { margin-top: 12px; padding: 16px; border: 1px solid var(--hh-border); border-radius: var(--hh-radius-md); background: var(--hh-surface-deep); }
.integration-inspector header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
.integration-inspector h3 { margin: 0; font-size: 14px; }
.integration-inspector h4 { margin: 0 0 8px; color: var(--hh-text-secondary); font-size: 12px; }
.inspector-grid { display: grid; gap: 12px; grid-template-columns: repeat(2, minmax(0, 1fr)); }
.inspector-card { border: 1px solid var(--hh-border); border-radius: var(--hh-radius-sm); background: color-mix(in srgb, var(--hh-surface-deep) 88%, white 12%); padding: 12px; }
.detail-row { display: flex; justify-content: space-between; gap: 12px; font-size: 12px; }
.detail-row span { color: var(--hh-text-muted); }
.detail-row strong { color: var(--hh-text-primary); text-align: right; }
.inspector-actions { display: flex; gap: 8px; margin-top: 12px; flex-wrap: wrap; }
.integration-section { border: 1px solid var(--hh-border); border-radius: var(--hh-radius-md); background: var(--hh-surface-deep); padding: 12px; }
.integration-section.compact { margin-top: 12px; }
.integration-section h3,.integration-section h4 { margin: 0 0 6px; }
.integration-section-description { margin: 0 0 8px; font-size: 12px; color: var(--hh-text-muted); }
.integration-form { display: grid; gap: 8px; }
.integration-form label { display: grid; gap: 4px; font-size: 11px; color: var(--hh-text-muted); }
.integration-form button { margin-top: 6px; }
.maintenance-grid { display: grid; gap: 12px; grid-template-columns: repeat(2, minmax(0, 1fr)); }
.oauth-link { font-size: 12px; color: var(--hh-accent); text-decoration: none; }
.oauth-link:hover { text-decoration: underline; }
@media (max-width: 960px) {
  .inspector-grid,
  .maintenance-grid { grid-template-columns: minmax(0, 1fr); }
}
</style>
