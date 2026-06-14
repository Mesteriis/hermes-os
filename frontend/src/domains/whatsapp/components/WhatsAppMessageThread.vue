<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { WhatsappWebSession, WhatsappWebMessage } from '../types/whatsapp'
import type { WhatsappMessageForm } from '../stores/whatsapp'

const props = defineProps<{
  selectedWhatsappSession: WhatsappWebSession | null
  selectedWhatsappMessages: WhatsappWebMessage[]
  isWhatsappLoading: boolean
  isWhatsappActionSubmitting: boolean
  whatsappMessageTime: (msg: WhatsappWebMessage) => string
  loadWhatsappWebWorkspace: () => void
  ingestWhatsappWebMessageFixture: () => void
  whatsappMessageForm: WhatsappMessageForm
}>()

const { t } = useI18n()
</script>

<template>
  <section class="panel chat-pane whatsapp-chat-pane">
    <template v-if="selectedWhatsappSession">
      <header>
        <span class="round-icon cyan">
          <Icon icon="tabler:brand-whatsapp" width="24" height="24" />
        </span>
        <div>
          <h2>{{ selectedWhatsappSession.device_name }}</h2>
          <p>{{ selectedWhatsappSession.account_id }} · {{ selectedWhatsappSession.link_state }}</p>
        </div>
      </header>

      <div class="message-scroll">
        <article
          v-for="msg in selectedWhatsappMessages"
          :key="msg.message_id"
          class="message-bubble"
        >
          <strong>{{ msg.sender_display_name ?? msg.sender }}</strong>
          <p>{{ msg.text }}</p>
          <time>{{ whatsappMessageTime(msg) }}</time>
        </article>
        <div v-if="selectedWhatsappMessages.length === 0" class="empty-panel">
          {{ t('No messages for this session.') }}
        </div>
      </div>

      <form class="telegram-inline-form" @submit.prevent="ingestWhatsappWebMessageFixture">
        <input
          v-model="whatsappMessageForm.provider_message_id"
          :placeholder="t('Provider message ID')"
          autocomplete="off"
        />
        <input
          v-model="whatsappMessageForm.sender_display_name"
          :placeholder="t('Sender')"
          autocomplete="off"
        />
        <input
          v-model="whatsappMessageForm.text"
          :placeholder="t('Fixture message text')"
          autocomplete="off"
        />
        <button
          type="submit"
          :disabled="isWhatsappActionSubmitting || !whatsappMessageForm.text.trim()"
        >
          <Icon icon="tabler:send" width="17" height="17" />{{ t('Ingest') }}
        </button>
      </form>
    </template>

    <div v-else class="empty-panel">
      {{ t('No WhatsApp session selected.') }}
    </div>
  </section>
</template>

<style scoped>
.chat-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.message-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.message-bubble {
  background: var(--bg-card);
  border-radius: var(--radius-sm);
  padding: 0.5rem 0.75rem;
  max-width: 85%;
}
.message-bubble strong {
  display: block;
  font-size: 0.75rem;
  color: var(--accent);
  margin-bottom: 0.2rem;
}
.message-bubble p {
  margin: 0;
  font-size: 0.875rem;
}
.message-bubble time {
  display: block;
  font-size: 0.7rem;
  color: var(--text-secondary);
  margin-top: 0.25rem;
}
.telegram-inline-form {
  display: flex;
  gap: 0.4rem;
  padding: 0.5rem;
  border-top: 1px solid var(--border);
  flex-wrap: wrap;
}
.telegram-inline-form input {
  flex: 1;
  min-width: 100px;
  padding: 0.35rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-primary);
  font-size: 0.8rem;
}
.telegram-inline-form button {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  padding: 0.35rem 0.75rem;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--accent);
  color: var(--accent-fg);
  cursor: pointer;
  font-size: 0.8rem;
}
.telegram-inline-form button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
