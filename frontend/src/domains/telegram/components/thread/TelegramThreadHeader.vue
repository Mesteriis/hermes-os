<script setup lang="ts">
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import type {
  TelegramCapabilitiesResponse,
  TelegramChat,
  TelegramOperationCapability,
  TelegramRailTab,
  TelegramRuntimeStatus,
  TelegramThreadTab
} from '../../types/telegram'
import { telegramChatIsSavedMessages, telegramChatMentionCountValue, telegramChatTypingLabel } from '../../stores/telegram'

const { t } = useI18n()

const props = defineProps<{
  selectedTelegramChat: TelegramChat
  selectedTelegramRuntimeStatus: TelegramRuntimeStatus | null
  capabilities?: TelegramCapabilitiesResponse | null
  isSearchOpen: boolean
  isTelegramActionSubmitting: boolean
  isTelegramLoading: boolean
}>()

const emit = defineEmits<{
  'update:isSearchOpen': [value: boolean]
  'update:activeThreadTab': [tab: TelegramThreadTab]
  railTabChange: [tab: TelegramRailTab]
  loadWorkspace: []
  syncHistory: []
  togglePinChat: []
  toggleArchiveChat: []
  toggleMuteChat: []
  toggleReadChat: []
}>()

function memberSummary(chat: TelegramChat): string {
  if (telegramChatIsSavedMessages(chat)) return t('Saved Messages')
  const memberCount = chat.metadata.member_count ?? chat.metadata.members_count
  const onlineCount = chat.metadata.online_count ?? chat.metadata.online_members_count
  if (typeof memberCount === 'number' && typeof onlineCount === 'number') {
    return `${memberCount.toLocaleString('en-US')} ${t('members')}, ${onlineCount.toLocaleString('en-US')} ${t('online')}`
  }
  if (typeof memberCount === 'number') return `${memberCount.toLocaleString('en-US')} ${t('members')}`
  return `${chat.account_id} · ${chat.provider_chat_id}`
}

function capability(operation: string): TelegramOperationCapability | null {
  return props.capabilities?.capabilities.find((item) => item.operation === operation) ?? null
}

function capabilityEnabled(operation: string): boolean {
  const item = capability(operation)
  return item?.status === 'available' || item?.status === 'degraded'
}

function capabilityTitle(operation: string, fallback: string): string {
  const item = capability(operation)
  return item?.reason || fallback
}

function isPinned(chat: TelegramChat): boolean {
  return Boolean(chat.metadata.is_pinned ?? chat.metadata.pinned)
}

function isArchived(chat: TelegramChat): boolean {
  return Boolean(chat.metadata.is_archived)
}

function isMuted(chat: TelegramChat): boolean {
  return Boolean(chat.metadata.is_muted ?? chat.metadata.muted)
}

function unreadCount(chat: TelegramChat): number {
  const value = chat.metadata.unread_count
  return typeof value === 'number' ? value : 0
}

function mentionCount(chat: TelegramChat): number {
  return telegramChatMentionCountValue(chat)
}

function typingLabel(chat: TelegramChat): string {
  return telegramChatTypingLabel(chat)
}

function syncStateMatchesChat(
  status: TelegramRuntimeStatus | null,
  chat: TelegramChat
): boolean {
  if (!status?.last_sync_scope) return false
  if (status.last_sync_scope === 'chats') return true
  return status.last_sync_provider_chat_id === chat.provider_chat_id
}

function syncStateLabel(
  status: TelegramRuntimeStatus | null,
  chat: TelegramChat
): string {
  if (!syncStateMatchesChat(status, chat)) return ''
  const parts = [status?.last_sync_scope ?? t('sync')]
  if (typeof status?.last_synced_count === 'number') parts.push(String(status.last_synced_count))
  if (status?.last_sync_status) parts.push(status.last_sync_status)
  return parts.join(' · ')
}

function commandStateMatchesChat(
  status: TelegramRuntimeStatus | null,
  chat: TelegramChat
): boolean {
  return Boolean(status?.last_command_status && status.last_command_provider_chat_id === chat.provider_chat_id)
}

function commandStateLabel(
  status: TelegramRuntimeStatus | null,
  chat: TelegramChat
): string {
  if (!commandStateMatchesChat(status, chat)) return ''
  const parts = [status?.last_command_status ?? t('command')]
  if (status?.last_command_message_id) parts.push(status.last_command_message_id)
  return parts.join(' · ')
}
</script>

<template>
  <header class="telegram-thread-header">
    <div class="telegram-thread-title">
      <span class="telegram-avatar large" :data-kind="selectedTelegramChat.chat_kind">
        <Icon icon="tabler:brand-telegram" width="24" height="24" />
      </span>
      <div>
        <h2>{{ selectedTelegramChat.title }}</h2>
        <p>{{ memberSummary(selectedTelegramChat) }}</p>
        <div class="telegram-thread-stats">
          <span v-if="unreadCount(selectedTelegramChat) > 0" class="telegram-thread-stat">
            {{ unreadCount(selectedTelegramChat) }} {{ t('unread') }}
          </span>
          <span v-if="mentionCount(selectedTelegramChat) > 0" class="telegram-thread-stat telegram-thread-stat-mention">
            @{{ mentionCount(selectedTelegramChat) }} {{ t('mentions') }}
          </span>
          <span v-if="typingLabel(selectedTelegramChat)" class="telegram-thread-stat telegram-thread-stat-typing">
            {{ typingLabel(selectedTelegramChat) }}
          </span>
          <span
            v-if="syncStateMatchesChat(selectedTelegramRuntimeStatus, selectedTelegramChat)"
            class="telegram-thread-stat telegram-thread-stat-sync"
          >
            {{ syncStateLabel(selectedTelegramRuntimeStatus, selectedTelegramChat) }}
          </span>
          <span
            v-if="commandStateMatchesChat(selectedTelegramRuntimeStatus, selectedTelegramChat)"
            class="telegram-thread-stat telegram-thread-stat-command"
          >
            {{ commandStateLabel(selectedTelegramRuntimeStatus, selectedTelegramChat) }}
          </span>
        </div>
      </div>
    </div>
    <span
      v-if="selectedTelegramRuntimeStatus"
      class="state-badge"
      :class="selectedTelegramRuntimeStatus.status"
    >
      {{ selectedTelegramRuntimeStatus.status }}
    </span>
    <div class="telegram-thread-actions">
      <button
        type="button"
        :class="{ active: isSearchOpen }"
        :title="t('Search')"
        @click="emit('update:isSearchOpen', !isSearchOpen)"
      >
        <Icon icon="tabler:search" width="18" height="18" />
      </button>
      <button
        type="button"
        :title="t('Pinned')"
        @click="emit('update:activeThreadTab', 'pinned')"
      >
        <Icon icon="tabler:pin" width="18" height="18" />
      </button>
      <button
        type="button"
        :disabled="isTelegramActionSubmitting || !capabilityEnabled('dialogs.mark_read')"
        :title="capabilityTitle('dialogs.mark_read', unreadCount(selectedTelegramChat) > 0 ? t('Mark read') : t('Mark unread'))"
        :class="{ active: unreadCount(selectedTelegramChat) > 0 }"
        @click="emit('toggleReadChat')"
      >
        <Icon :icon="unreadCount(selectedTelegramChat) > 0 ? 'tabler:mail-opened' : 'tabler:mail'" width="18" height="18" />
      </button>
      <button
        type="button"
        :disabled="isTelegramActionSubmitting || !capabilityEnabled('dialogs.pin')"
        :title="capabilityTitle('dialogs.pin', isPinned(selectedTelegramChat) ? t('Unpin chat') : t('Pin chat'))"
        :class="{ active: isPinned(selectedTelegramChat) }"
        @click="emit('togglePinChat')"
      >
        <Icon icon="tabler:pin" width="18" height="18" />
      </button>
      <button
        type="button"
        :disabled="isTelegramActionSubmitting || !capabilityEnabled('dialogs.archive')"
        :title="capabilityTitle('dialogs.archive', isArchived(selectedTelegramChat) ? t('Unarchive chat') : t('Archive chat'))"
        :class="{ active: isArchived(selectedTelegramChat) }"
        @click="emit('toggleArchiveChat')"
      >
        <Icon icon="tabler:archive" width="18" height="18" />
      </button>
      <button
        type="button"
        :disabled="isTelegramActionSubmitting || !capabilityEnabled('dialogs.mute')"
        :title="capabilityTitle('dialogs.mute', isMuted(selectedTelegramChat) ? t('Unmute chat') : t('Mute chat'))"
        :class="{ active: isMuted(selectedTelegramChat) }"
        @click="emit('toggleMuteChat')"
      >
        <Icon :icon="isMuted(selectedTelegramChat) ? 'tabler:bell' : 'tabler:bell-off'" width="18" height="18" />
      </button>
      <button type="button" :title="t('Members')" @click="emit('railTabChange', 'members')">
        <Icon icon="tabler:users" width="18" height="18" />
      </button>
      <button
        type="button"
        :disabled="isTelegramActionSubmitting"
        :title="t('Sync History')"
        @click="emit('syncHistory')"
      >
        <Icon icon="tabler:history" width="18" height="18" />
      </button>
      <button
        type="button"
        :disabled="isTelegramLoading"
        :title="t('Refresh')"
        @click="emit('loadWorkspace')"
      >
        <Icon icon="tabler:refresh" width="18" height="18" />
      </button>
    </div>
  </header>
</template>

<style scoped>
.telegram-thread-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
}
.telegram-thread-title {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}
.telegram-thread-title h2 {
  font-size: 15px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text, #333);
}
.telegram-thread-title p {
  font-size: 11px;
  margin: 0;
  color: var(--color-text-secondary, #777);
}
.telegram-thread-stats {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 4px;
  flex-wrap: wrap;
}
.telegram-thread-stat {
  display: inline-flex;
  align-items: center;
  min-height: 20px;
  padding: 0 8px;
  border-radius: 999px;
  font-size: 10px;
  background: #e3f2fd;
  color: #0b5394;
}
.telegram-thread-stat-mention {
  background: #fff4e5;
  color: #9a5b00;
}
.telegram-thread-stat-sync {
  background: #e8f5e9;
  color: #1b5e20;
}
.telegram-thread-stat-command {
  background: #ede7f6;
  color: #512da8;
}
.telegram-thread-stat-typing {
  background: #e0f2fe;
  color: #075985;
}
.telegram-avatar.large {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--color-avatar-bg, #e0e0e0);
  flex-shrink: 0;
}
.state-badge {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 8px;
  text-transform: uppercase;
}
.state-badge.running { background: #e6f7e6; color: #2e7d32; }
.state-badge.stopped { background: #f5f5f5; color: #999; }
.state-badge.error { background: #fdecea; color: #c62828; }
.state-badge.degraded { background: #fff3e0; color: #e65100; }
.telegram-thread-actions {
  display: flex;
  gap: 2px;
}
.telegram-thread-actions button {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 6px;
  color: var(--color-text-secondary, #777);
  border-radius: 4px;
}
.telegram-thread-actions button:hover:not(:disabled) {
  background: var(--color-bg, #f5f5f5);
}
.telegram-thread-actions button.active {
  background: var(--color-primary-subtle, #e3f2fd);
  color: var(--color-primary, #0066cc);
}
</style>
