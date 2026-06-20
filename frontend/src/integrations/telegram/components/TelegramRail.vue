<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import TelegramAccountManager from './TelegramAccountManager.vue'
import TelegramCallsPanel from './TelegramCallsPanel.vue'
import TelegramCommandAuditPanel from './TelegramCommandAuditPanel.vue'
import TelegramMembersPanel from './TelegramMembersPanel.vue'
import TelegramReadProgressPanel from './TelegramReadProgressPanel.vue'
import type { TelegramCapabilitiesResponse, TelegramChat, TelegramMessage, TelegramRuntimeStatus } from '../types/telegram'
import type { TelegramRailTab } from '../types/telegram'
import {
  telegramAttachmentHintsForMessages,
  telegramChatIsPinned,
  telegramChatMentionCount,
  telegramRuntimeCommandTarget,
  telegramChatUnreadCount,
  telegramLinkHintsForMessages,
} from '../stores/telegram'

const { t } = useI18n()

const props = defineProps<{
  selectedTelegramChat: TelegramChat | null
  selectedTelegramChatDetail: TelegramChat | null
  selectedTelegramRuntimeStatus: TelegramRuntimeStatus | null
  selectedTelegramMessages: TelegramMessage[]
  capabilities: TelegramCapabilitiesResponse | null
  isInspectorLoading: boolean
  activeRailTab: TelegramRailTab
}>()

const emit = defineEmits<{
  'update:activeRailTab': [tab: TelegramRailTab]
  'close': []
}>()

const selectedAccountId = computed(() => props.selectedTelegramChat?.account_id)

function detailValue(chat: TelegramChat | null, key: string): string {
  if (!chat) return '—'
  const value = chat.metadata?.[key]
  if (typeof value === 'string' && value.trim()) return value
  if (typeof value === 'number') return value.toLocaleString('en-US')
  if (typeof value === 'boolean') return value ? 'yes' : 'no'
  return '—'
}

function permissionsSummary(chat: TelegramChat | null): string {
  const permissions = chat?.metadata?.tdlib_permissions
  if (!permissions || typeof permissions !== 'object' || Array.isArray(permissions)) return '—'
  const entries = Object.entries(permissions)
    .filter(([, value]) => typeof value === 'boolean')
    .map(([key, value]) => `${key.replace(/^can_/, '')}:${value ? 'yes' : 'no'}`)
  return entries.length ? entries.join(' · ') : '—'
}

function formatDate(value: string | null | undefined): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

function syncSummary(status: TelegramRuntimeStatus | null): string {
  if (!status?.last_sync_scope) return '—'
  const parts = [status.last_sync_scope]
  if (typeof status.last_synced_count === 'number') parts.push(String(status.last_synced_count))
  if (status.last_sync_status) parts.push(status.last_sync_status)
  if (typeof status.last_sync_has_more === 'boolean') {
    parts.push(status.last_sync_has_more ? t('more available') : t('complete'))
  }
  return parts.join(' · ')
}

function commandSummary(status: TelegramRuntimeStatus | null): string {
  if (!status?.last_command_status) return '—'
  const parts = [status.last_command_status]
  if (status.last_command_id) parts.push(status.last_command_id)
  return parts.join(' · ')
}

function syncTarget(status: TelegramRuntimeStatus | null): string {
  if (!status?.last_sync_scope) return '—'
  if (status.last_sync_scope === 'chats') return t('workspace')
  return status.last_sync_provider_chat_id ?? '—'
}

function commandTarget(status: TelegramRuntimeStatus | null): string {
  return telegramRuntimeCommandTarget(status) ?? '—'
}

function runtimeBlockers(status: TelegramRuntimeStatus | null): string {
  if (!status?.runtime_blockers?.length) return t('none')
  return status.runtime_blockers.join(' · ')
}
</script>

<template>
  <aside class="stacked-rail telegram-rail">
    <section class="panel telegram-context-panel">
      <header class="telegram-inspector-head">
        <div>
          <h2>{{ t('Details') }}</h2>
          <p>{{ selectedTelegramChat?.title ?? t('No chat selected') }}</p>
        </div>
        <button type="button" :title="t('Close')" @click="emit('close')">
          <Icon icon="tabler:x" width="17" height="17" />
        </button>
      </header>

      <nav class="inspector-tabs telegram-rail-tabs">
        <button
          type="button"
          :class="{ active: activeRailTab === 'context' }"
          @click="emit('update:activeRailTab', 'context')"
        >{{ t('Context') }}</button>
        <button
          type="button"
          :class="{ active: activeRailTab === 'members' }"
          @click="emit('update:activeRailTab', 'members')"
        >{{ t('Members') }}</button>
        <button
          type="button"
          :class="{ active: activeRailTab === 'about' }"
          @click="emit('update:activeRailTab', 'about')"
        >{{ t('About') }}</button>
      </nav>

      <div v-if="isInspectorLoading" class="telegram-inspector-placeholder">
        {{ t('Loading Telegram state...') }}
      </div>

      <div v-else-if="activeRailTab === 'context'" class="telegram-rail-section">
        <article class="telegram-rail-card">
          <h3>{{ t('Context') }}</h3>
          <dl>
            <div><dt>{{ t('Messages') }}</dt><dd>{{ selectedTelegramMessages.length }}</dd></div>
            <div><dt>{{ t('Files') }}</dt><dd>{{ telegramAttachmentHintsForMessages(selectedTelegramMessages).length }}</dd></div>
            <div><dt>{{ t('Links') }}</dt><dd>{{ telegramLinkHintsForMessages(selectedTelegramMessages).length }}</dd></div>
            <div><dt>{{ t('Unread') }}</dt><dd>{{ selectedTelegramChatDetail ? telegramChatUnreadCount(selectedTelegramChatDetail) : 0 }}</dd></div>
            <div><dt>{{ t('Mentions') }}</dt><dd>{{ selectedTelegramChatDetail ? telegramChatMentionCount(selectedTelegramChatDetail, selectedTelegramMessages) : 0 }}</dd></div>
            <div><dt>{{ t('Last sync') }}</dt><dd>{{ syncSummary(selectedTelegramRuntimeStatus) }}</dd></div>
            <div><dt>{{ t('Sync target') }}</dt><dd>{{ syncTarget(selectedTelegramRuntimeStatus) }}</dd></div>
            <div><dt>{{ t('Last command') }}</dt><dd>{{ commandSummary(selectedTelegramRuntimeStatus) }}</dd></div>
            <div><dt>{{ t('Command target') }}</dt><dd>{{ commandTarget(selectedTelegramRuntimeStatus) }}</dd></div>
          </dl>
        </article>
        <TelegramReadProgressPanel
          :selectedChat="selectedTelegramChatDetail"
          :selectedMessages="selectedTelegramMessages"
        />
        <TelegramCallsPanel :selectedAccountId="selectedTelegramChat?.account_id ?? null" />
        <TelegramCommandAuditPanel
          :selectedAccountId="selectedTelegramChat?.account_id ?? null"
          :selectedProviderChatId="selectedTelegramChat?.provider_chat_id ?? null"
        />
      </div>

      <div v-else-if="activeRailTab === 'members'" class="telegram-rail-section">
        <TelegramMembersPanel
          :telegramChatId="selectedTelegramChat?.telegram_chat_id ?? null"
          :accountId="selectedTelegramChat?.account_id ?? null"
          :providerChatId="selectedTelegramChat?.provider_chat_id ?? null"
          :capabilities="capabilities"
        />
      </div>

      <div v-else class="telegram-rail-section">
        <article class="telegram-rail-card">
          <h3>{{ t('About') }}</h3>
          <dl>
            <div><dt>ID</dt><dd>{{ selectedTelegramChatDetail?.provider_chat_id ?? '—' }}</dd></div>
            <div><dt>{{ t('Type') }}</dt><dd>{{ selectedTelegramChatDetail?.chat_kind ?? '—' }}</dd></div>
            <div><dt>Username</dt><dd>{{ selectedTelegramChatDetail?.username ?? '—' }}</dd></div>
            <div><dt>{{ t('TDLib chat type') }}</dt><dd>{{ detailValue(selectedTelegramChatDetail, 'tdlib_chat_type') }}</dd></div>
            <div><dt>{{ t('Supergroup ID') }}</dt><dd>{{ detailValue(selectedTelegramChatDetail, 'tdlib_supergroup_id') }}</dd></div>
            <div><dt>{{ t('Supergroup') }}</dt><dd>{{ detailValue(selectedTelegramChatDetail, 'is_supergroup') }}</dd></div>
            <div><dt>{{ t('Forum') }}</dt><dd>{{ detailValue(selectedTelegramChatDetail, 'is_forum') }}</dd></div>
            <div><dt>{{ t('Channel supergroup') }}</dt><dd>{{ detailValue(selectedTelegramChatDetail, 'is_channel_supergroup') }}</dd></div>
            <div><dt>{{ t('Permissions') }}</dt><dd>{{ permissionsSummary(selectedTelegramChatDetail) }}</dd></div>
            <div><dt>{{ t('State') }}</dt><dd>{{ selectedTelegramChatDetail?.sync_state ?? '—' }}</dd></div>
            <div><dt>{{ t('Runtime') }}</dt><dd>{{ selectedTelegramRuntimeStatus?.status ?? '—' }}</dd></div>
            <div><dt>TDLib path</dt><dd>{{ selectedTelegramRuntimeStatus?.tdjson_path ?? '—' }}</dd></div>
            <div><dt>TDLib</dt><dd>{{ selectedTelegramRuntimeStatus?.tdjson_runtime_available ? t('available') : t('unavailable') }}</dd></div>
            <div><dt>{{ t('API ID') }}</dt><dd>{{ selectedTelegramRuntimeStatus?.telegram_api_id_configured ? t('configured') : t('missing') }}</dd></div>
            <div><dt>{{ t('API hash') }}</dt><dd>{{ selectedTelegramRuntimeStatus?.telegram_api_hash_configured ? t('configured') : t('missing') }}</dd></div>
            <div><dt>{{ t('Runtime blockers') }}</dt><dd>{{ runtimeBlockers(selectedTelegramRuntimeStatus) }}</dd></div>
            <div><dt>{{ t('Runtime probe') }}</dt><dd>{{ selectedTelegramRuntimeStatus?.tdjson_probe_error ?? '—' }}</dd></div>
            <div><dt>{{ t('Pinned') }}</dt><dd>{{ selectedTelegramChatDetail && telegramChatIsPinned(selectedTelegramChatDetail) ? 'yes' : 'no' }}</dd></div>
            <div><dt>{{ t('Archived') }}</dt><dd>{{ detailValue(selectedTelegramChatDetail, 'is_archived') }}</dd></div>
            <div><dt>{{ t('Muted') }}</dt><dd>{{ detailValue(selectedTelegramChatDetail, 'is_muted') }}</dd></div>
            <div><dt>{{ t('Last activity') }}</dt><dd>{{ formatDate(selectedTelegramChatDetail?.last_message_at ?? selectedTelegramChatDetail?.updated_at) }}</dd></div>
          </dl>
        </article>
        <TelegramAccountManager
          :selectedAccountId="selectedTelegramChat?.account_id ?? null"
        />
      </div>
    </section>
  </aside>
</template>

<style scoped>
.telegram-rail {
  border-left: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
  min-width: 280px;
  max-width: 360px;
  display: flex;
  flex-direction: column;
}
.telegram-context-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.telegram-inspector-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.telegram-inspector-head h2 {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text, #333);
}
.telegram-inspector-head p {
  font-size: 11px;
  margin: 2px 0 0;
  color: var(--color-text-secondary, #777);
}
.telegram-inspector-head button {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 4px;
  color: var(--color-text-secondary, #777);
  border-radius: 4px;
}
.telegram-inspector-head button:hover {
  background: var(--color-bg, #f5f5f5);
}
.inspector-tabs {
  display: flex;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.inspector-tabs button {
  flex: 1;
  padding: 8px 12px;
  border: none;
  background: transparent;
  font-size: 12px;
  cursor: pointer;
  color: var(--color-text-secondary, #777);
  border-bottom: 2px solid transparent;
}
.inspector-tabs button.active {
  color: var(--color-primary, #0066cc);
  border-bottom-color: var(--color-primary, #0066cc);
  font-weight: 500;
}
.inspector-tabs button:hover {
  background: var(--color-bg, #f5f5f5);
}
.telegram-inspector-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 16px;
  color: var(--color-text-secondary, #aaa);
  font-size: 13px;
}
.telegram-rail-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px 16px 16px;
  overflow-y: auto;
}
.telegram-rail-card {
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  padding: 12px;
  background: var(--color-bg, #fafafa);
}
.telegram-rail-card h3 {
  margin: 0 0 8px;
  font-size: 13px;
}
.telegram-rail-card dl {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 0;
}
.telegram-rail-card dl div {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  font-size: 12px;
}
.telegram-rail-card dt {
  color: var(--color-text-secondary, #777);
}
.telegram-rail-card dd {
  margin: 0;
  text-align: right;
  color: var(--color-text, #333);
}
.telegram-call-placeholder {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}
</style>
