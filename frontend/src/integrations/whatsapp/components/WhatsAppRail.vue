<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { WhatsappCapabilitiesResponse, WhatsappCapabilityStatus } from '../types/whatsapp'
import type { WhatsappMessageForm } from '../stores/whatsapp'

const props = defineProps<{
  whatsappCapabilities: WhatsappCapabilitiesResponse | null
  whatsappClosureCapabilities: WhatsappCapabilityStatus[]
  whatsappBlockedCapabilities: WhatsappCapabilityStatus[]
  whatsappProviderAccounts: number
  isWhatsappActionSubmitting: boolean
  openAccountDrawer: () => void
  ingestWhatsappWebMessageFixture: () => void
  whatsappMessageForm: WhatsappMessageForm
}>()

const { t } = useI18n()

function capabilityLabel(capability: string): string {
  const labels: Record<string, string> = {
    whatsapp_web_messaging: 'WhatsApp Web Messaging',
    whatsapp_web_sync: 'WhatsApp Web Sync',
    whatsapp_web_link: 'QR Link',
    whatsapp_web_runtime: 'Companion Runtime',
    wa_web_contacts: 'Contacts',
    wa_web_media: 'Media'
  }
  return labels[capability] ?? capability
}
</script>

<template>
  <aside class="stacked-rail whatsapp-rail">
    <section class="panel info-card">
      <h2>{{ t('Accounts') }}</h2>
      <div class="setup-summary-card">
        <span class="round-icon green">
          <Icon icon="tabler:brand-whatsapp" width="22" height="22" />
        </span>
        <div>
          <strong>{{ whatsappProviderAccounts }} {{ t('WhatsApp accounts') }}</strong>
          <p>
            {{ whatsappProviderAccounts
              ? t('Companion session records are available for fixture ingestion.')
              : t('No WhatsApp Web account record is configured yet.')
            }}
          </p>
        </div>
      </div>
      <div class="form-actions wide">
        <button type="button" :disabled="isWhatsappActionSubmitting" @click="openAccountDrawer">
          {{ t('Open Wizard') }}
        </button>
      </div>
    </section>

    <section class="panel info-card">
      <h2>{{ t('Runtime Guardrails') }}</h2>
      <div class="health-row">
        <span>{{ t('Mode') }}</span>
        <strong>{{ whatsappCapabilities?.runtime_mode ?? t('unknown') }}</strong>
      </div>
      <ul v-if="whatsappClosureCapabilities.length" class="detail-list">
        <li v-for="cap in whatsappClosureCapabilities" :key="cap.capability">
          {{ capabilityLabel(cap.capability) }}
          <em>{{ cap.status }}</em>
        </li>
      </ul>
      <template v-if="whatsappBlockedCapabilities.length">
        <div class="evidence-row">
          <strong>{{ t('Live Scope') }}</strong>
          <p>{{ whatsappBlockedCapabilities.map((c) => capabilityLabel(c.capability)).join(', ') }}</p>
        </div>
      </template>
      <template v-if="whatsappCapabilities?.unsupported_features?.length">
        <div class="evidence-row">
          <strong>{{ t('Unsupported') }}</strong>
          <p>{{ whatsappCapabilities.unsupported_features.map(capabilityLabel).join(', ') }}</p>
        </div>
      </template>
    </section>

    <section class="panel info-card">
      <h2>{{ t('Ingest Fixture Message') }}</h2>
      <form class="setup-form compact-form" @submit.prevent="ingestWhatsappWebMessageFixture">
        <label><span>{{ t('Account ID') }}</span><input v-model="whatsappMessageForm.account_id" autocomplete="off" /></label>
        <label><span>{{ t('Chat ID') }}</span><input v-model="whatsappMessageForm.provider_chat_id" autocomplete="off" /></label>
        <label><span>{{ t('Chat title') }}</span><input v-model="whatsappMessageForm.chat_title" autocomplete="off" /></label>
        <label><span>{{ t('Sender ID') }}</span><input v-model="whatsappMessageForm.sender_id" autocomplete="off" /></label>
        <label><span>{{ t('Sender') }}</span><input v-model="whatsappMessageForm.sender_display_name" autocomplete="off" /></label>
        <label class="wide"><span>{{ t('Text') }}</span><textarea v-model="whatsappMessageForm.text" rows="3"></textarea></label>
        <div class="form-actions wide">
          <button type="submit" :disabled="isWhatsappActionSubmitting || !whatsappMessageForm.text.trim()">
            {{ t('Ingest Fixture') }}
          </button>
        </div>
      </form>
    </section>
  </aside>
</template>
