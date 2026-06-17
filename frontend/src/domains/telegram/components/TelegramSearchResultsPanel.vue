<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { TelegramChat, TelegramMediaItem, TelegramMessage } from '../types/telegram'
import { telegramMediaReadiness } from '../stores/telegramMediaSearch'

const { t } = useI18n()

const props = defineProps<{
  query: string
  chats: TelegramChat[]
  results: TelegramMessage[]
  total: number
  mediaItems: TelegramMediaItem[]
  isLoading: boolean
  sourceLabel: string
  mediaSourceLabel: string
}>()

const emit = defineEmits<{
  openChat: [chat: TelegramChat]
  openMessage: [message: TelegramMessage]
  openMedia: [item: TelegramMediaItem]
}>()

function formatDate(value: string | null): string {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

function senderName(message: TelegramMessage): string {
  return message.sender_display_name ?? message.sender
}

function mediaLabel(item: TelegramMediaItem): string {
  return item.mime_type ?? item.kind
}

function totalResultsCount(): number {
  return props.chats.length + props.total + props.mediaItems.length
}
</script>

<template>
  <section class="panel telegram-search-results">
    <header class="telegram-search-results__header">
      <div>
        <h2>{{ t('Search Results') }}</h2>
        <p>{{ query }}</p>
      </div>
      <span class="telegram-search-results__count">
        {{ totalResultsCount() }} {{ t('results') }}
      </span>
    </header>
    <p class="telegram-search-results__source">{{ t(sourceLabel) }}</p>

    <div v-if="isLoading" class="empty-panel fill">
      {{ t('Searching Telegram workspace...') }}
    </div>
    <div
      v-else-if="chats.length === 0 && results.length === 0 && mediaItems.length === 0"
      class="empty-panel fill"
    >
      {{ t('No Telegram search results for this query.') }}
    </div>
    <div v-else class="telegram-search-results__body">
      <section v-if="chats.length > 0" class="telegram-search-results__section">
        <header>
          <h3>{{ t('Dialogs') }}</h3>
        </header>
        <button
          v-for="chat in chats"
          :key="chat.telegram_chat_id"
          type="button"
          class="telegram-search-results__message"
          @click="emit('openChat', chat)"
        >
          <span class="telegram-search-results__icon">
            <Icon icon="tabler:messages" width="18" height="18" />
          </span>
          <span class="telegram-search-results__copy">
            <strong>{{ chat.title }}</strong>
            <small>{{ chat.chat_kind }} · {{ formatDate(chat.last_message_at) }}</small>
            <p>{{ chat.provider_chat_id }}</p>
          </span>
        </button>
      </section>

      <section v-if="results.length > 0" class="telegram-search-results__section">
        <header>
          <h3>{{ t('Messages') }}</h3>
        </header>
        <button
          v-for="message in results"
          :key="message.message_id"
          type="button"
          class="telegram-search-results__message"
          @click="emit('openMessage', message)"
        >
          <span class="telegram-search-results__icon">
            <Icon icon="tabler:message-search" width="18" height="18" />
          </span>
          <span class="telegram-search-results__copy">
            <strong>{{ message.chat_title || senderName(message) }}</strong>
            <small>{{ senderName(message) }} · {{ formatDate(message.occurred_at ?? message.projected_at) }}</small>
            <p>{{ message.text || t('No text content') }}</p>
          </span>
        </button>
      </section>

      <section v-if="mediaItems.length > 0" class="telegram-search-results__section">
        <header>
          <h3>{{ t('Media In Current Chat') }}</h3>
          <p v-if="mediaSourceLabel" class="telegram-search-results__media-source">
            {{ t(mediaSourceLabel) }}
          </p>
        </header>
        <div class="telegram-search-results__media-grid">
          <button
            v-for="item in mediaItems"
            :key="`${item.message_id}:${item.file_name}`"
            type="button"
            class="telegram-search-results__media-card"
            @click="emit('openMedia', item)"
          >
            <span class="telegram-search-results__icon">
              <Icon :icon="item.kind === 'photo' ? 'tabler:photo' : item.kind === 'video' ? 'tabler:video' : item.kind === 'audio' || item.kind === 'voice' ? 'tabler:wave-sine' : 'tabler:file-description'" width="18" height="18" />
            </span>
            <div>
              <strong>{{ item.file_name }}</strong>
              <small>{{ mediaLabel(item) }} · {{ item.download_state }}</small>
              <small>{{ telegramMediaReadiness(item).label }} · {{ telegramMediaReadiness(item).detail }}</small>
              <small>{{ formatDate(item.occurred_at) }}</small>
            </div>
          </button>
        </div>
      </section>
    </div>
  </section>
</template>

<style scoped>
.telegram-search-results {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  background: var(--color-surface, #fff);
}
.telegram-search-results__header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.telegram-search-results__header h2,
.telegram-search-results__section h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text, #333);
}
.telegram-search-results__header p,
.telegram-search-results__count,
.telegram-search-results__source,
.telegram-search-results__media-source,
.telegram-search-results__message small,
.telegram-search-results__media-card small {
  color: var(--color-text-secondary, #777);
  font-size: 11px;
}
.telegram-search-results__source {
  margin: 0;
  padding: 8px 16px 0;
}
.telegram-search-results__media-source {
  margin: 4px 0 0;
}
.telegram-search-results__body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 12px 16px 16px;
  overflow-y: auto;
}
.telegram-search-results__section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.telegram-search-results__message {
  display: flex;
  gap: 10px;
  width: 100%;
  text-align: left;
  padding: 10px 12px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  background: var(--color-bg, #fafafa);
  cursor: pointer;
}
.telegram-search-results__message:hover {
  background: var(--color-primary-subtle, #e3f2fd);
}
.telegram-search-results__icon {
  color: var(--color-text-secondary, #777);
  flex-shrink: 0;
}
.telegram-search-results__copy {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}
.telegram-search-results__copy strong,
.telegram-search-results__media-card strong {
  font-size: 12px;
  color: var(--color-text, #333);
}
.telegram-search-results__copy p {
  margin: 0;
  font-size: 12px;
  color: var(--color-text, #333);
  word-break: break-word;
}
.telegram-search-results__media-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 8px;
}
.telegram-search-results__media-card {
  display: flex;
  gap: 10px;
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  background: var(--color-bg, #fafafa);
  text-align: left;
  cursor: pointer;
}
.telegram-search-results__media-card:hover {
  background: var(--color-primary-subtle, #e3f2fd);
}
</style>
