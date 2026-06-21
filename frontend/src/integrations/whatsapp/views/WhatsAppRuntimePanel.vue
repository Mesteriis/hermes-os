<script setup lang="ts">
import { computed, watch } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import WhatsAppSessionList from '../components/WhatsAppSessionList.vue'
import WhatsAppRail from '../components/WhatsAppRail.vue'
import WhatsAppStatusMessages from '../components/WhatsAppStatusMessages.vue'
import { useWhatsappStore } from '../stores/whatsapp'
import {
  ingestWhatsappWebMessageFixture,
  setupWhatsappWebFixture,
} from '../api/whatsapp'
import {
  useWhatsappCapabilitiesQuery,
  useWhatsappSessionsQuery,
} from '../queries/useWhatsappQuery'

const { t } = useI18n()
const store = useWhatsappStore()
const capabilitiesQuery = useWhatsappCapabilitiesQuery()
const sessionsQuery = useWhatsappSessionsQuery(undefined, 100)
const capabilities = computed(() => capabilitiesQuery.data.value ?? null)
const sessions = computed(() => sessionsQuery.data.value ?? [])

watch(
  [sessions, capabilities],
  ([nextSessions, nextCapabilities]) => {
    const selectedSessionId = nextSessions.some((session) => session.session_id === store.selectedWhatsappSessionId)
      ? store.selectedWhatsappSessionId
      : nextSessions[0]?.session_id ?? ''
    store.setWhatsappData({
      sessions: nextSessions,
      capabilities: nextCapabilities,
      selectedSessionId,
      error: '',
    })
  },
  { immediate: true }
)

async function refreshRuntime() {
  await Promise.all([capabilitiesQuery.refetch(), sessionsQuery.refetch()])
}

async function ingestFixtureMessage() {
  if (store.isWhatsappActionSubmitting) return
  store.setWhatsappActionSubmitting(true)
  store.setWhatsappActionMessage('')
  store.setWhatsappError('')
  try {
    const result = await ingestWhatsappWebMessageFixture({
      account_id: store.whatsappMessageForm.account_id,
      provider_chat_id: store.whatsappMessageForm.provider_chat_id,
      provider_message_id: store.whatsappMessageForm.provider_message_id,
      chat_title: store.whatsappMessageForm.chat_title,
      sender_id: store.whatsappMessageForm.sender_id,
      sender_display_name: store.whatsappMessageForm.sender_display_name,
      text: store.whatsappMessageForm.text,
      import_batch_id: store.whatsappMessageForm.import_batch_id,
      occurred_at: store.whatsappMessageForm.occurred_at,
      delivery_state: store.whatsappMessageForm.delivery_state as 'received' | 'sent' | 'send_dry_run' | 'send_blocked',
    })
    store.setWhatsappActionMessage(result.message)
    store.whatsappMessageForm = {
      ...store.whatsappMessageForm,
      provider_message_id: `wa-fixture-msg-${crypto.randomUUID()}`,
      occurred_at: new Date().toISOString(),
    }
  } catch (error) {
    store.setWhatsappError(error instanceof Error ? error.message : String(error))
  } finally {
    store.setWhatsappActionSubmitting(false)
  }
}

async function setupFixtureAccount() {
  if (store.isWhatsappActionSubmitting) return
  store.setWhatsappActionSubmitting(true)
  store.setWhatsappActionMessage('')
  store.setWhatsappError('')
  try {
    const result = await setupWhatsappWebFixture({
      account_id: store.whatsappMessageForm.account_id,
      display_name: 'WhatsApp Fixture',
      external_account_id: store.whatsappMessageForm.account_id,
      device_name: 'Local fixture device',
      local_state_path: 'docker/data/whatsapp/fixture',
    })
    if (result.error) {
      store.setWhatsappError(result.error)
    } else {
      store.setWhatsappActionMessage(result.message)
      await refreshRuntime()
    }
  } catch (error) {
    store.setWhatsappError(error instanceof Error ? error.message : String(error))
  } finally {
    store.setWhatsappActionSubmitting(false)
  }
}
</script>

<template>
  <section class="whatsapp-runtime-panel communications-page">
    <header class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small">
          <Icon icon="tabler:brand-whatsapp" width="28" height="28" />
        </span>
        <div>
          <h1>{{ t('WhatsApp Runtime') }}</h1>
          <p>{{ t('Provider sessions, capabilities and fixture controls') }}</p>
        </div>
      </div>
      <button type="button" class="primary-button" :disabled="sessionsQuery.isFetching.value" @click="refreshRuntime">
        <Icon icon="tabler:refresh" width="16" height="16" />{{ t('Refresh') }}
      </button>
    </header>

    <div class="metric-grid">
      <article class="metric-card">
        <span>{{ t('Sessions') }}</span>
        <strong>{{ sessions.length }}</strong>
        <small>{{ store.selectedWhatsappSession?.link_state ?? t('not linked') }}</small>
      </article>
      <article class="metric-card">
        <span>{{ t('Runtime') }}</span>
        <strong>{{ capabilities?.runtime_mode ?? t('unknown') }}</strong>
        <small>{{ t('Fixture/manual foundation') }}</small>
      </article>
      <article class="metric-card">
        <span>{{ t('Blocked') }}</span>
        <strong>{{ store.whatsappBlockedCapabilities.length }}</strong>
        <small>{{ t('Live runtime remains blocked') }}</small>
      </article>
    </div>

    <WhatsAppStatusMessages
      :action-message="store.whatsappActionMessage"
      :error="store.whatsappError"
    />

    <div class="whatsapp-runtime-grid">
      <WhatsAppSessionList
        :whatsapp-sessions="sessions"
        :selected-whatsapp-session-id="store.selectedWhatsappSessionId"
        :is-whatsapp-loading="sessionsQuery.isLoading.value"
        @select-session="store.selectWhatsappSession"
      />

      <WhatsAppRail
        :whatsapp-capabilities="capabilities"
        :whatsapp-closure-capabilities="store.whatsappClosureCapabilities"
        :whatsapp-blocked-capabilities="store.whatsappBlockedCapabilities"
        :whatsapp-provider-accounts="sessions.length"
        :is-whatsapp-action-submitting="store.isWhatsappActionSubmitting"
        :open-account-drawer="setupFixtureAccount"
        :ingest-whatsapp-web-message-fixture="ingestFixtureMessage"
        :whatsapp-message-form="store.whatsappMessageForm"
      />
    </div>
  </section>
</template>

<style scoped>
.whatsapp-runtime-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: auto;
}
.view-header,
.view-title-with-icon {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.view-header {
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--hh-border, #d9e2ec);
}
.whatsapp-runtime-grid {
  display: grid;
  grid-template-columns: minmax(280px, 420px) minmax(320px, 1fr);
  gap: 1rem;
  padding: 1rem;
  min-height: 0;
}
</style>
