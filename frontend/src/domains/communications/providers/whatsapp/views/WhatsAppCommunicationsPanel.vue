<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../../../platform/i18n'
import Icon from '../../../../../shared/ui/Icon.vue'
import type { WhatsappWebMessage } from '../../../../../shared/communications/types/whatsapp'
import { useWhatsappBusinessMessagesQuery } from '../../../queries/whatsappBusinessQueries'

type WhatsappConversationSummary = WhatsappWebMessage & {
  provider_chat_id: string
}

const { t } = useI18n()
const selectedConversationId = ref('')
const messagesQuery = useWhatsappBusinessMessagesQuery(undefined, () => selectedConversationId.value || undefined, 200)
const allMessagesQuery = useWhatsappBusinessMessagesQuery(undefined, undefined, 200)
const allMessages = computed(() => allMessagesQuery.data.value ?? [])
const selectedMessages = computed(() => messagesQuery.data.value ?? [])
const conversations = computed(() => {
  const byId = new Map<string, WhatsappConversationSummary>()
  for (const message of allMessages.value) {
    if (!message.provider_chat_id) continue
    if (!byId.has(message.provider_chat_id)) {
      byId.set(message.provider_chat_id, {
        ...message,
        provider_chat_id: message.provider_chat_id,
      })
    }
  }
  return Array.from(byId.values())
})

function selectConversation(providerChatId: string) {
  selectedConversationId.value = providerChatId
}

function messageTime(message: WhatsappWebMessage): string {
  const value = message.occurred_at ?? message.projected_at
  if (!value) return ''
  const date = new Date(value)
  return Number.isNaN(date.getTime())
    ? ''
    : new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date)
}
</script>

<template>
  <section class="whatsapp-communications-panel communications-page">
    <header class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small">
          <Icon icon="tabler:brand-whatsapp" width="28" height="28" />
        </span>
        <div>
          <h1>{{ t('WhatsApp') }}</h1>
          <p>{{ t('Projected Communication messages') }}</p>
        </div>
      </div>
      <button type="button" class="primary-button" :disabled="allMessagesQuery.isFetching.value" @click="allMessagesQuery.refetch()">
        <Icon icon="tabler:refresh" width="16" height="16" />{{ t('Refresh') }}
      </button>
    </header>

    <div class="three-pane communications-grid whatsapp-grid">
      <section class="panel conversation-list">
        <header class="provider-panel-header">
          <h2>{{ t('Conversations') }}</h2>
        </header>
        <div class="provider-list-scroll">
          <button
            v-for="conversation in conversations"
            :key="conversation.provider_chat_id"
            type="button"
            class="provider-row"
            :class="{ active: selectedConversationId === conversation.provider_chat_id }"
            @click="selectConversation(conversation.provider_chat_id)"
          >
            <strong>{{ conversation.chat_title }}</strong>
            <span>{{ conversation.sender_display_name ?? conversation.sender }}</span>
          </button>
          <div v-if="!conversations.length" class="empty-panel">{{ t('No projected WhatsApp conversations yet.') }}</div>
        </div>
      </section>

      <section class="panel chat-pane">
        <div class="message-scroll">
          <article v-for="message in selectedMessages" :key="message.message_id" class="message-bubble">
            <strong>{{ message.sender_display_name ?? message.sender }}</strong>
            <p>{{ message.text }}</p>
            <time>{{ messageTime(message) }}</time>
          </article>
          <div v-if="!selectedMessages.length" class="empty-panel">{{ t('Select a WhatsApp conversation.') }}</div>
        </div>
      </section>
    </div>
  </section>
</template>

<style scoped>
.whatsapp-communications-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}
.view-header,
.view-title-with-icon,
.provider-panel-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.view-header,
.provider-panel-header {
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--hh-border, #d9e2ec);
}
.provider-list-scroll,
.message-scroll {
  flex: 1;
  overflow: auto;
  min-height: 0;
}
.provider-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  width: 100%;
  padding: 0.7rem 0.85rem;
  border: 0;
  border-bottom: 1px solid var(--hh-border, #eef2f6);
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}
.provider-row.active,
.provider-row:hover {
  background: var(--hh-bg-muted, #f5f8fb);
}
.provider-row span,
.message-bubble time {
  color: var(--hh-text-muted, #667085);
  font-size: 0.78rem;
}
.message-bubble {
  margin: 0.75rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-primary, #fff);
}
.empty-panel {
  padding: 0.75rem;
}
</style>
