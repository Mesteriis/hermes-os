<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'

// Components
import WhatsAppSessionList from '../components/WhatsAppSessionList.vue'
import WhatsAppMessageThread from '../components/WhatsAppMessageThread.vue'
import WhatsAppRail from '../components/WhatsAppRail.vue'
import WhatsAppStatusMessages from '../components/WhatsAppStatusMessages.vue'

// Store
import { useWhatsappStore } from '../stores/whatsapp'

// API
import * as whatsappApi from '../api/whatsapp'

// TanStack Query
import { useWhatsappCapabilitiesQuery } from '../queries/useWhatsappQuery'

const { t } = useI18n()
const store = useWhatsappStore()

// Capabilities query
const { data: capabilities } = useWhatsappCapabilitiesQuery()

onMounted(() => {
  void loadWhatsappWebWorkspace()
})

async function loadWhatsappWebWorkspace() {
  store.setWhatsappLoading(true)
  try {
    const result = await whatsappApi.loadWhatsappWebWorkspace(
      store.selectedWhatsappSessionId
    )
    store.setWhatsappData(result)
  } catch (err) {
    store.setWhatsappError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setWhatsappLoading(false)
  }
}

function selectWhatsappSession(session: any) {
  store.selectWhatsappSession(session)
  void loadWhatsappWebWorkspace()
}

async function ingestWhatsappWebMessageFixture() {
  if (store.isWhatsappActionSubmitting) return
  store.setWhatsappActionSubmitting(true)
  store.setWhatsappActionMessage('')
  store.setWhatsappError('')
  try {
    const result = await whatsappApi.ingestWhatsappWebMessageFixture({
      account_id: store.whatsappMessageForm.account_id,
      provider_chat_id: store.whatsappMessageForm.provider_chat_id,
      provider_message_id: store.whatsappMessageForm.provider_message_id,
      chat_title: store.whatsappMessageForm.chat_title,
      sender_id: store.whatsappMessageForm.sender_id,
      sender_display_name: store.whatsappMessageForm.sender_display_name,
      text: store.whatsappMessageForm.text,
      import_batch_id: store.whatsappMessageForm.import_batch_id,
      occurred_at: store.whatsappMessageForm.occurred_at,
      delivery_state: store.whatsappMessageForm.delivery_state
    })
    if (result.error) {
      store.setWhatsappError(result.error)
    } else {
      store.setWhatsappActionMessage(result.message)
      store.whatsappMessageForm = {
        ...store.whatsappMessageForm,
        provider_message_id: result.nextProviderMessageId,
        occurred_at: result.nextOccurredAt
      }
      await loadWhatsappWebWorkspace()
    }
  } catch (err) {
    store.setWhatsappError(err instanceof Error ? err.message : String(err))
  } finally {
    store.setWhatsappActionSubmitting(false)
  }
}

function openAccountDrawer() {
  // Placeholder — account wizard integration TBD
  store.setWhatsappActionMessage(t('Account wizard integration pending.'))
}
</script>

<template>
  <section class="whatsapp-runtime-panel communications-page">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small">
          <Icon icon="tabler:brand-whatsapp" width="28" height="28" />
        </span>
        <div>
          <h1>{{ t('WhatsApp') }}</h1>
          <p>{{ t('WhatsApp Web sessions and messages') }}</p>
        </div>
      </div>
      <button
        type="button"
        class="primary-button"
        @click="openAccountDrawer"
      >
        <Icon icon="tabler:plus" width="16" height="16" />{{ t('Add Account') }}
      </button>
      <button
        type="button"
        class="primary-button"
        :disabled="store.isWhatsappLoading"
        @click="loadWhatsappWebWorkspace"
      >
        <Icon icon="tabler:refresh" width="16" height="16" />{{ t('Refresh') }}
      </button>
    </div>

    <div class="metric-grid">
      <article class="metric-card">
        <span>{{ t('Sessions') }}</span>
        <strong>{{ store.whatsappSessions.length }}</strong>
        <small>{{ store.selectedWhatsappSession?.link_state ?? t('not linked') }}</small>
      </article>
      <article class="metric-card">
        <span>{{ t('Messages') }}</span>
        <strong>{{ store.whatsappMessages.length }}</strong>
        <small>{{ t('Canonical WhatsApp Web records') }}</small>
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

    <div class="three-pane communications-grid whatsapp-grid">
      <WhatsAppSessionList
        :whatsapp-sessions="store.whatsappSessions"
        :selected-whatsapp-session-id="store.selectedWhatsappSessionId"
        :is-whatsapp-loading="store.isWhatsappLoading"
        @select-session="selectWhatsappSession"
      />

      <WhatsAppMessageThread
        :selected-whatsapp-session="store.selectedWhatsappSession"
        :selected-whatsapp-messages="store.selectedWhatsappMessages"
        :is-whatsapp-loading="store.isWhatsappLoading"
        :is-whatsapp-action-submitting="store.isWhatsappActionSubmitting"
        :whatsapp-message-time="whatsappApi.whatsappMessageTime"
        :load-whatsapp-web-workspace="loadWhatsappWebWorkspace"
        :ingest-whatsapp-web-message-fixture="ingestWhatsappWebMessageFixture"
        :whatsapp-message-form="store.whatsappMessageForm"
      />

      <WhatsAppRail
        :whatsapp-capabilities="store.whatsappCapabilities"
        :whatsapp-closure-capabilities="store.whatsappClosureCapabilities"
        :whatsapp-blocked-capabilities="store.whatsappBlockedCapabilities"
        :whatsapp-provider-accounts="0"
        :is-whatsapp-action-submitting="store.isWhatsappActionSubmitting"
        :open-account-drawer="openAccountDrawer"
        :ingest-whatsapp-web-message-fixture="ingestWhatsappWebMessageFixture"
        :whatsapp-message-form="store.whatsappMessageForm"
      />
    </div>
  </section>
</template>
