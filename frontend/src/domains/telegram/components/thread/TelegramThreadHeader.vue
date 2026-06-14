<script setup lang="ts">
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import type {
  TelegramChat,
  TelegramRailTab,
  TelegramRuntimeStatus,
  TelegramThreadTab
} from '../../types/telegram'

const { t } = useI18n()

defineProps<{
  selectedTelegramChat: TelegramChat
  selectedTelegramRuntimeStatus: TelegramRuntimeStatus | null
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
}>()

function memberSummary(chat: TelegramChat): string {
  const memberCount = chat.metadata.member_count ?? chat.metadata.members_count
  const onlineCount = chat.metadata.online_count ?? chat.metadata.online_members_count
  if (typeof memberCount === 'number' && typeof onlineCount === 'number') {
    return `${memberCount.toLocaleString('en-US')} ${t('members')}, ${onlineCount.toLocaleString('en-US')} ${t('online')}`
  }
  if (typeof memberCount === 'number') return `${memberCount.toLocaleString('en-US')} ${t('members')}`
  return `${chat.account_id} · ${chat.provider_chat_id}`
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
