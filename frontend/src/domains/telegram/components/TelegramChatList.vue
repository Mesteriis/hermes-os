<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { TelegramChat, TelegramMessage } from '../types/telegram'
import {
  telegramChatIsPinned,
  telegramChatMentionCountValue,
  telegramChatUnreadCount,
  telegramChatPreview,
  telegramMessageAttachmentHints
} from '../stores/telegram'

const { t } = useI18n()

const props = defineProps<{
  telegramChats: TelegramChat[]
  telegramMessages: TelegramMessage[]
  selectedTelegramChatId: string
  isTelegramLoading: boolean
  formatDateTime: (date: string | null) => string
}>()

const emit = defineEmits<{
  'selectChat': [chat: TelegramChat]
}>()

const parentRef = ref<HTMLDivElement | null>(null)

const virtualOptions = computed(() => ({
  count: props.telegramChats.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 68,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())

const visibleRange = computed(() =>
  props.telegramChats.length ? `${Math.min(virtualItems.value.length ? virtualItems.value[0].index + 1 : 0, props.telegramChats.length)}-${Math.min(virtualItems.value.length ? virtualItems.value[virtualItems.value.length - 1].index + 1 : 0, props.telegramChats.length)}` : '0'
)

function chatMessages(chat: TelegramChat): TelegramMessage[] {
  return props.telegramMessages.filter(
    (message) => message.provider_chat_id === chat.provider_chat_id
  )
}

function chatTime(chat: TelegramChat): string {
  return props.formatDateTime(chat.last_message_at ?? chat.updated_at)
}

function chatIcon(chat: TelegramChat): string {
  if (chat.chat_kind === 'bot' || chat.title.toLowerCase().includes('bot')) return 'tabler:robot'
  if (chat.chat_kind === 'channel') return 'tabler:speakerphone'
  if (chat.chat_kind === 'private') return 'tabler:user'
  return 'tabler:users-group'
}

function chatInitials(chat: TelegramChat): string {
  const parts = chat.title.split(/\s+/).filter(Boolean)
  if (!parts.length) return 'TG'
  return parts.slice(0, 2).map((part) => part[0]?.toUpperCase()).join('')
}

function isMuted(chat: TelegramChat): boolean {
  return Boolean(chat.metadata?.muted ?? chat.metadata?.is_muted)
}

function hasAttachment(chat: TelegramChat): boolean {
  return chatMessages(chat).some((message) => telegramMessageAttachmentHints(message).length > 0)
}
</script>

<template>
  <section class="panel conversation-list telegram-conversation-list">
    <header class="telegram-panel-header">
      <h2>{{ t('Conversations') }}</h2>
      <button type="button" :title="t('Filters')" :disabled="isTelegramLoading">
        <Icon icon="tabler:adjustments-horizontal" width="17" height="17" />
      </button>
    </header>

    <div ref="parentRef" class="telegram-chat-scroll">
      <div v-if="isTelegramLoading && telegramChats.length === 0" class="empty-panel">
        {{ t('Loading Telegram state...') }}
      </div>
      <div v-else-if="telegramChats.length === 0" class="empty-panel">
        {{ t('No Telegram chats projected yet.') }}
      </div>
      <div v-else :style="{ height: `${totalSize}px` }">
        <button
          v-for="vitem in virtualItems"
          :key="telegramChats[vitem.index].provider_chat_id"
          type="button"
          class="telegram-chat-row"
          :style="{ transform: `translateY(${vitem.start}px)`, height: `${vitem.size}px` }"
          :class="{ active: selectedTelegramChatId === telegramChats[vitem.index].provider_chat_id }"
          @click="emit('selectChat', telegramChats[vitem.index])"
        >
          <span class="telegram-avatar" :data-kind="telegramChats[vitem.index].chat_kind">
            <Icon v-if="telegramChats[vitem.index].chat_kind === 'group' || telegramChats[vitem.index].chat_kind === 'channel'" :icon="chatIcon(telegramChats[vitem.index])" width="19" height="19" />
            <span v-else>{{ chatInitials(telegramChats[vitem.index]) }}</span>
          </span>
          <span class="telegram-chat-copy">
            <span class="telegram-chat-title-line">
              <strong>{{ telegramChats[vitem.index].title }}</strong>
              <Icon v-if="telegramChatIsPinned(telegramChats[vitem.index])" icon="tabler:pin" width="13" height="13" />
              <Icon v-if="isMuted(telegramChats[vitem.index])" icon="tabler:bell-off" width="13" height="13" />
            </span>
            <small>{{ telegramChatPreview(telegramChats[vitem.index], telegramMessages) }}</small>
            <span class="telegram-chat-state">
              <em>{{ telegramChats[vitem.index].sync_state }}</em>
              <Icon v-if="hasAttachment(telegramChats[vitem.index])" icon="tabler:paperclip" width="13" height="13" />
            </span>
          </span>
          <span class="telegram-chat-side">
            <time>{{ chatTime(telegramChats[vitem.index]) }}</time>
            <span
              v-if="telegramChatMentionCountValue(telegramChats[vitem.index]) > 0"
              class="telegram-chat-mention-badge"
              :title="t('Unread mentions')"
            >
              @{{ telegramChatMentionCountValue(telegramChats[vitem.index]) }}
            </span>
            <b v-if="telegramChatUnreadCount(telegramChats[vitem.index]) > 0">{{ telegramChatUnreadCount(telegramChats[vitem.index]) }}</b>
            <Icon v-else-if="telegramChats[vitem.index].metadata?.delivery_state === 'sent'" icon="tabler:checks" width="15" height="15" />
          </span>
        </button>
      </div>
    </div>

    <footer class="telegram-list-footer">
      <span class="telegram-list-range">{{ visibleRange }} {{ t('of') }} {{ telegramChats.length }}</span>
      <div>
        <button type="button" disabled :title="t('Previous')">
          <Icon icon="tabler:chevron-left" width="17" height="17" />
        </button>
        <button type="button" disabled :title="t('Next')">
          <Icon icon="tabler:chevron-right" width="17" height="17" />
        </button>
      </div>
    </footer>
  </section>
</template>

<style scoped>
.telegram-conversation-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  border-right: 1px solid var(--color-border, #e0e0e0);
  min-width: 260px;
  max-width: 320px;
  background: var(--color-surface, #fff);
}
.telegram-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.telegram-panel-header h2 {
  font-size: 13px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text, #333);
}
.telegram-panel-header button {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 4px;
  color: var(--color-text-secondary, #777);
  border-radius: 4px;
}
.telegram-panel-header button:hover:not(:disabled) {
  background: var(--color-bg, #f5f5f5);
}
.telegram-chat-scroll {
  flex: 1;
  overflow-y: auto;
}
.empty-panel {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 16px;
  font-size: 13px;
  color: var(--color-text-secondary, #999);
}
.telegram-chat-row {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  font-family: inherit;
  color: var(--color-text, #333);
}
.telegram-chat-row:hover {
  background: var(--color-bg, #f5f5f5);
}
.telegram-chat-row.active {
  background: var(--color-primary-subtle, #e3f2fd);
}
.telegram-avatar {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 50%;
  background: var(--color-avatar-bg, #e0e0e0);
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
  color: var(--color-text-secondary, #555);
}
.telegram-chat-copy {
  flex: 1;
  min-width: 0;
}
.telegram-chat-title-line {
  display: flex;
  align-items: center;
  gap: 4px;
}
.telegram-chat-title-line strong {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.telegram-chat-copy small {
  display: block;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}
.telegram-chat-state {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 2px;
}
.telegram-chat-state em {
  font-style: normal;
  font-size: 10px;
  color: var(--color-text-secondary, #aaa);
}
.telegram-chat-side {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
  flex-shrink: 0;
}
.telegram-chat-side time {
  font-size: 10px;
  color: var(--color-text-secondary, #aaa);
}
.telegram-chat-side b {
  font-size: 11px;
  background: var(--color-primary, #0066cc);
  color: #fff;
  border-radius: 10px;
  padding: 1px 6px;
  min-width: 18px;
  text-align: center;
}
.telegram-chat-mention-badge {
  font-size: 10px;
  background: #fff4e5;
  color: #9a5b00;
  border: 1px solid #f4c37d;
  border-radius: 10px;
  padding: 1px 6px;
  min-width: 22px;
  text-align: center;
}
.telegram-list-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 14px;
  border-top: 1px solid var(--color-border, #e0e0e0);
  font-size: 11px;
  color: var(--color-text-secondary, #999);
}
.telegram-list-footer button {
  border: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
  border-radius: 4px;
  padding: 2px 6px;
  cursor: pointer;
  color: var(--color-text-secondary, #777);
}
</style>
