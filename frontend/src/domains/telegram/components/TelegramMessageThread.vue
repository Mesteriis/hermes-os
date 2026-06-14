<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type {
  TelegramChat,
  TelegramMessage,
  TelegramRuntimeStatus
} from '../types/telegram'
import type { MessageAnalyzeResponse } from '../../communications/types/communications'
import type { TelegramThreadTab } from '../types/telegram'
import type { TelegramAttachmentHint } from '../types/telegram'
import {
  telegramMessagesChronological,
  telegramAttachmentHintsForMessages,
  telegramLinkHintsForMessages,
  telegramMessageAttachmentHints,
  telegramPinnedMessages
} from '../stores/telegram'

const { t } = useI18n()

const props = defineProps<{
  selectedTelegramChat: TelegramChat | null
  selectedTelegramMessages: TelegramMessage[]
  aiAnalysisResult: MessageAnalyzeResponse | null
  selectedCommunication: { message_id?: string } | null
  isTelegramLoading: boolean
  isTelegramActionSubmitting: boolean
  activeThreadTab: TelegramThreadTab
  telegramMessageTime: (message: TelegramMessage) => string
  telegramManualSendForm: { text: string }
  selectedTelegramRuntimeStatus: TelegramRuntimeStatus | null
}>()

const emit = defineEmits<{
  'update:activeThreadTab': [tab: TelegramThreadTab]
  'railTabChange': [tab: 'context' | 'members' | 'about']
  'loadWorkspace': []
  'syncHistory': []
  'syncOlderHistory': []
  'sendMessage': []
  'downloadMedia': [attachment: TelegramAttachmentHint, message?: TelegramMessage]
}>()

const threadSearchQuery = ref('')
const isSearchOpen = ref(false)
const isEmojiTrayOpen = ref(false)
const isSendMenuOpen = ref(false)

const chronologicalMessages = computed(() =>
  telegramMessagesChronological(props.selectedTelegramMessages)
)
const fileHints = computed(() => telegramAttachmentHintsForMessages(chronologicalMessages.value))
const linkHints = computed(() => telegramLinkHintsForMessages(chronologicalMessages.value))
const pinnedMessages = computed(() => telegramPinnedMessages(chronologicalMessages.value))
const filteredMessages = computed(() =>
  chronologicalMessages.value.filter((message) => {
    const query = threadSearchQuery.value.trim().toLowerCase()
    if (!query) return true
    return [
      message.text,
      message.sender,
      message.sender_display_name ?? '',
      message.provider_message_id
    ]
      .join(' ')
      .toLowerCase()
      .includes(query)
  })
)

interface TabItem {
  id: TelegramThreadTab
  label: string
  count: number
}

const tabs = computed<TabItem[]>(() => [
  { id: 'messages', label: t('Messages'), count: chronologicalMessages.value.length },
  { id: 'files', label: t('Files'), count: fileHints.value.length },
  { id: 'links', label: t('Links'), count: linkHints.value.length },
  { id: 'topics', label: t('Topics'), count: topicCount(props.selectedTelegramChat) },
  { id: 'pinned', label: t('Pinned'), count: pinnedMessages.value.length },
  { id: 'timeline', label: t('Timeline'), count: chronologicalMessages.value.length }
])

function appendEmoji(value: string) {
  props.telegramManualSendForm.text = `${props.telegramManualSendForm.text}${value}`
  isEmojiTrayOpen.value = false
}

function submitManualSend() {
  isSendMenuOpen.value = false
  emit('sendMessage')
}

function senderName(message: TelegramMessage): string {
  return message.sender_display_name ?? message.sender
}

function senderInitials(message: TelegramMessage): string {
  return (
    senderName(message)
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((part) => part[0]?.toUpperCase())
      .join('') || 'TG'
  )
}

function isOutbound(message: TelegramMessage): boolean {
  return message.delivery_state === 'sent' || message.delivery_state === 'send_dry_run'
}

function topicCount(chat: TelegramChat | null): number {
  const meta = chat?.metadata as Record<string, unknown> | undefined
  const value = meta?.topics_count ?? meta?.topic_count
  return typeof value === 'number' ? value : 0
}

function memberSummary(chat: TelegramChat): string {
  const meta = chat.metadata as Record<string, unknown>
  const memberCount = meta.member_count ?? meta.members_count
  const onlineCount = meta.online_count ?? meta.online_members_count
  if (typeof memberCount === 'number' && typeof onlineCount === 'number') {
    return `${memberCount.toLocaleString('en-US')} ${t('members')}, ${onlineCount.toLocaleString('en-US')} ${t('online')}`
  }
  if (typeof memberCount === 'number') return `${memberCount.toLocaleString('en-US')} ${t('members')}`
  return `${chat.account_id} · ${chat.provider_chat_id}`
}

function formatDate(value: string): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric' }).format(date)
}

function handleThreadScroll(event: Event) {
  if (props.activeThreadTab !== 'messages' || props.isTelegramActionSubmitting) return
  const target = event.currentTarget as HTMLElement | null
  if (!target || target.scrollTop > 48) return
  emit('syncOlderHistory')
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function downloadAttachment(attachment: TelegramAttachmentHint, message?: TelegramMessage) {
  emit('downloadMedia', attachment, message)
}
</script>

<template>
  <section class="panel chat-pane telegram-chat-pane">
    <template v-if="selectedTelegramChat">
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
        <span v-if="selectedTelegramRuntimeStatus" class="state-badge" :class="selectedTelegramRuntimeStatus.status">
          {{ selectedTelegramRuntimeStatus.status }}
        </span>
        <div class="telegram-thread-actions">
          <button
            type="button"
            :class="{ active: isSearchOpen }"
            :title="t('Search')"
            @click="isSearchOpen = !isSearchOpen"
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
            :title="t('Members')"
            @click="emit('railTabChange', 'members')"
          >
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

      <label v-if="isSearchOpen" class="telegram-thread-search">
        <Icon icon="tabler:search" width="16" height="16" />
        <input v-model="threadSearchQuery" :placeholder="t('Search in this chat...')" autocomplete="off" />
      </label>

      <nav class="message-context-tabs telegram-thread-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          :class="{ active: activeThreadTab === tab.id }"
          @click="emit('update:activeThreadTab', tab.id)"
        >
          {{ tab.label }}
          <em v-if="tab.count > 0">{{ tab.count }}</em>
        </button>
      </nav>

      <div class="chat-body telegram-thread-body" @scroll="handleThreadScroll">
        <article
          v-if="aiAnalysisResult && aiAnalysisResult.message_id === selectedCommunication?.message_id"
          class="ai-analysis-card telegram-ai-card"
        >
          <strong><Icon icon="tabler:sparkles" width="16" height="16" />{{ t('AI Analysis') }}</strong>
          <p v-if="aiAnalysisResult.category"><em>{{ t('Category:') }}</em> {{ aiAnalysisResult.category }}</p>
          <p v-if="aiAnalysisResult.summary"><em>{{ t('Summary:') }}</em> {{ aiAnalysisResult.summary }}</p>
          <p v-if="aiAnalysisResult.importance_score != null"><em>{{ t('Importance:') }}</em> {{ aiAnalysisResult.importance_score }}/100</p>
        </article>

        <!-- Messages tab -->
        <template v-if="activeThreadTab === 'messages'">
          <div v-if="selectedTelegramChat.chat_kind !== 'private'" class="telegram-history-actions">
            <button
              type="button"
              :disabled="isTelegramActionSubmitting"
              @click="emit('syncOlderHistory')"
            >
              <Icon icon="tabler:arrow-up" width="16" height="16" />
              {{ t('Load older') }}
            </button>
          </div>
          <div v-if="filteredMessages.length === 0" class="empty-panel fill">
            {{ threadSearchQuery ? t('No Telegram messages match this search.') : isTelegramActionSubmitting ? t('Syncing selected Telegram history...') : t('No messages for this chat.') }}
          </div>
          <template v-else>
            <div class="telegram-date-chip">{{ t('Today') }}</div>
            <article
              v-for="message in filteredMessages"
              :key="message.message_id"
              class="telegram-message-row"
              :class="{ outbound: isOutbound(message) }"
            >
              <span class="telegram-message-avatar">{{ senderInitials(message) }}</span>
              <div
                class="bubble telegram-bubble"
                :class="{ outbound: isOutbound(message), inbound: !isOutbound(message) }"
              >
                <strong>{{ senderName(message) }}</strong>
                <p>{{ message.text }}</p>
                <div v-if="telegramMessageAttachmentHints(message).length" class="telegram-bubble-files">
                  <div
                    v-for="attachment in telegramMessageAttachmentHints(message)"
                    :key="attachment.messageId + attachment.fileName"
                    class="telegram-file-card compact"
                  >
                    <span><Icon icon="tabler:file" width="18" height="18" /></span>
                    <div>
                      <strong>{{ attachment.fileName }}</strong>
                      <small>{{ attachment.sizeBytes == null ? attachment.downloadState : `${formatBytes(attachment.sizeBytes)} · ${attachment.downloadState}` }}</small>
                    </div>
                    <button
                      type="button"
                      :disabled="isTelegramActionSubmitting || attachment.tdlibFileId === null"
                      :title="attachment.tdlibFileId === null ? t('Download requires TDLib file metadata') : t('Download media')"
                      @click="downloadAttachment(attachment, message)"
                    >
                      <Icon icon="tabler:download" width="16" height="16" />
                    </button>
                  </div>
                </div>
                <time>
                  {{ telegramMessageTime(message) }}
                  <span>{{ message.delivery_state }}</span>
                </time>
              </div>
            </article>
          </template>
        </template>

        <!-- Files tab -->
        <template v-else-if="activeThreadTab === 'files'">
          <div v-if="fileHints.length === 0" class="empty-panel fill">
            {{ t('No files in selected Telegram history.') }}
          </div>
          <div v-else class="telegram-file-list">
            <div v-for="file in fileHints" :key="file.fileName" class="telegram-file-card">
              <span>
                <Icon
                  :icon="file.kind === 'photo' ? 'tabler:photo' : file.kind === 'video' ? 'tabler:video' : 'tabler:file-description'"
                  width="20" height="20"
                />
              </span>
              <div>
                <strong>{{ file.fileName }}</strong>
                <small>{{ file.mimeType ?? file.kind }} · {{ file.sizeBytes == null ? file.downloadState : formatBytes(file.sizeBytes) }}</small>
              </div>
              <button
                type="button"
                :disabled="isTelegramActionSubmitting || file.tdlibFileId === null"
                :title="file.tdlibFileId === null ? t('Download requires TDLib file metadata') : t('Download media')"
                @click="downloadAttachment(file)"
              >
                <Icon icon="tabler:download" width="17" height="17" />
              </button>
            </div>
          </div>
        </template>

        <!-- Links tab -->
        <template v-else-if="activeThreadTab === 'links'">
          <div v-if="linkHints.length === 0" class="empty-panel fill">
            {{ t('No links in selected Telegram history.') }}
          </div>
          <div v-else class="telegram-link-list">
            <a
              v-for="(link, idx) in linkHints"
              :key="idx"
              :href="link.url"
              target="_blank"
              rel="noreferrer"
            >
              <Icon icon="tabler:link" width="17" height="17" />
              <span>{{ link.label }}</span>
              <em>{{ link.occurredAt ? formatDate(link.occurredAt) : '' }}</em>
            </a>
          </div>
        </template>

        <!-- Pinned tab -->
        <template v-else-if="activeThreadTab === 'pinned'">
          <div v-if="pinnedMessages.length === 0" class="empty-panel fill">
            {{ t('No pinned messages in selected Telegram history.') }}
          </div>
          <template v-else>
            <article
              v-for="message in pinnedMessages"
              :key="message.message_id"
              class="telegram-timeline-row"
            >
              <Icon icon="tabler:pin" width="16" height="16" />
              <div><strong>{{ senderName(message) }}</strong><p>{{ message.text }}</p></div>
              <time>{{ telegramMessageTime(message) }}</time>
            </article>
          </template>
        </template>

        <!-- Timeline tab -->
        <template v-else-if="activeThreadTab === 'timeline'">
          <div v-if="chronologicalMessages.length === 0" class="empty-panel fill">
            {{ t('No timeline events in selected Telegram history.') }}
          </div>
          <template v-else>
            <article
              v-for="message in chronologicalMessages"
              :key="message.message_id"
              class="telegram-timeline-row"
            >
              <Icon :icon="isOutbound(message) ? 'tabler:send' : 'tabler:message'" width="16" height="16" />
              <div><strong>{{ senderName(message) }}</strong><p>{{ message.text }}</p></div>
              <time>{{ telegramMessageTime(message) }}</time>
            </article>
          </template>
        </template>

        <!-- Topics placeholder -->
        <div v-else class="empty-panel fill">
          {{ t('Telegram topics are available after TDLib forum topic sync is implemented.') }}
        </div>
      </div>

      <form
        class="telegram-compose-bar"
        @submit.prevent="submitManualSend"
      >
        <button type="button" disabled :title="t('Attachment upload is not available in this slice')">
          <Icon icon="tabler:paperclip" width="18" height="18" />
        </button>
        <textarea
          :value="telegramManualSendForm.text"
          rows="1"
          :placeholder="t('Write a message...')"
          autocomplete="off"
          @input="telegramManualSendForm.text = ($event.target as HTMLTextAreaElement).value"
        ></textarea>
        <div class="telegram-compose-menu">
          <button
            type="button"
            :title="t('Emoji')"
            @click="isEmojiTrayOpen = !isEmojiTrayOpen"
          >
            <Icon icon="tabler:mood-smile" width="18" height="18" />
          </button>
          <div v-if="isEmojiTrayOpen" class="telegram-emoji-popover">
            <button v-for="emoji in ['👍', '🔥', '🎉', '✅', '🙏']" :key="emoji" type="button" @click="appendEmoji(emoji)">
              {{ emoji }}
            </button>
          </div>
        </div>
        <button type="button" disabled :title="t('Voice messages require media runtime')">
          <Icon icon="tabler:microphone" width="18" height="18" />
        </button>
        <button
          type="submit"
          class="send"
          :disabled="isTelegramActionSubmitting || !telegramManualSendForm.text.trim()"
          :title="t('Send')"
        >
          <Icon icon="tabler:send" width="18" height="18" />
        </button>
        <div class="telegram-compose-menu">
          <button
            type="button"
            class="send-more"
            :title="t('More')"
            @click="isSendMenuOpen = !isSendMenuOpen"
          >
            <Icon icon="tabler:chevron-down" width="17" height="17" />
          </button>
          <div v-if="isSendMenuOpen" class="command-popover telegram-send-popover">
            <button
              type="button"
              :disabled="isTelegramActionSubmitting || !telegramManualSendForm.text.trim()"
              @click="submitManualSend"
            >
              <Icon icon="tabler:send" width="15" height="15" />{{ t('Send now') }}
            </button>
            <button
              type="button"
              :disabled="isTelegramActionSubmitting"
              @click="isSendMenuOpen = false; emit('syncHistory')"
            >
              <Icon icon="tabler:history" width="15" height="15" />{{ t('Sync History') }}
            </button>
          </div>
        </div>
      </form>
    </template>
    <div v-else class="empty-panel fill">
      {{ t('Select a Telegram chat to inspect messages and compose replies.') }}
    </div>
  </section>
</template>

<style scoped>
.telegram-chat-pane {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  background: var(--color-bg, #f9f9f9);
}
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
.telegram-thread-search {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
}
.telegram-thread-search input {
  border: none;
  background: transparent;
  font-size: 12px;
  outline: none;
  flex: 1;
  color: var(--color-text, #333);
}
.message-context-tabs {
  display: flex;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
  overflow-x: auto;
}
.message-context-tabs button {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  border: none;
  background: transparent;
  font-size: 11px;
  cursor: pointer;
  color: var(--color-text-secondary, #777);
  border-bottom: 2px solid transparent;
  white-space: nowrap;
}
.message-context-tabs button.active {
  color: var(--color-primary, #0066cc);
  border-bottom-color: var(--color-primary, #0066cc);
  font-weight: 500;
}
.message-context-tabs button:hover {
  background: var(--color-bg, #f5f5f5);
}
.message-context-tabs button em {
  font-style: normal;
  font-size: 10px;
  opacity: 0.6;
  background: var(--color-bg, #f0f0f0);
  border-radius: 8px;
  padding: 0 5px;
}
.chat-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 16px;
}
.empty-panel.fill {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  font-size: 13px;
  color: var(--color-text-secondary, #999);
}
.ai-analysis-card {
  padding: 12px;
  background: var(--color-surface, #fff);
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  margin-bottom: 12px;
  font-size: 12px;
}
.ai-analysis-card strong {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 6px;
}
.ai-analysis-card p {
  margin: 2px 0;
  color: var(--color-text-secondary, #666);
}
.telegram-history-actions {
  text-align: center;
  padding: 8px 0;
}
.telegram-history-actions button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  background: var(--color-surface, #fff);
  font-size: 11px;
  cursor: pointer;
  color: var(--color-text-secondary, #777);
}
.telegram-date-chip {
  text-align: center;
  font-size: 11px;
  color: var(--color-text-secondary, #999);
  padding: 6px 0;
}
.telegram-message-row {
  display: flex;
  gap: 8px;
  margin-bottom: 4px;
}
.telegram-message-row.outbound {
  flex-direction: row-reverse;
}
.telegram-message-avatar {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--color-avatar-bg, #e0e0e0);
  font-size: 10px;
  font-weight: 600;
  flex-shrink: 0;
  color: var(--color-text-secondary, #555);
}
.bubble {
  max-width: 75%;
  padding: 8px 12px;
  border-radius: 12px;
  font-size: 12px;
  line-height: 1.4;
}
.bubble.inbound {
  background: var(--color-surface, #fff);
  border: 1px solid var(--color-border, #e0e0e0);
  color: var(--color-text, #333);
  border-bottom-left-radius: 4px;
}
.bubble.outbound {
  background: var(--color-primary-subtle, #e3f2fd);
  border: 1px solid var(--color-primary-light, #bbdefb);
  color: var(--color-text, #333);
  border-bottom-right-radius: 4px;
}
.bubble strong {
  display: block;
  font-size: 11px;
  margin-bottom: 2px;
  color: var(--color-primary, #0066cc);
}
.bubble p {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}
.bubble time {
  display: block;
  font-size: 10px;
  color: var(--color-text-secondary, #aaa);
  margin-top: 4px;
  text-align: right;
}
.telegram-bubble-files {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 6px;
}
.telegram-file-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  background: var(--color-bg, #f9f9f9);
  font-size: 11px;
}
.telegram-file-card.compact {
  padding: 4px 6px;
  font-size: 10px;
}
.telegram-file-card strong {
  display: block;
  font-size: 11px;
}
.telegram-file-card small {
  display: block;
  color: var(--color-text-secondary, #999);
}
.telegram-file-card button {
  margin-left: auto;
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 4px;
  color: var(--color-text-secondary, #777);
  border-radius: 4px;
  flex-shrink: 0;
}
.telegram-file-card button:hover:not(:disabled) {
  background: var(--color-primary-subtle, #e3f2fd);
}
.telegram-file-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px 0;
}
.telegram-link-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 0;
}
.telegram-link-list a {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  text-decoration: none;
  color: var(--color-primary, #0066cc);
  font-size: 12px;
  border-radius: 6px;
}
.telegram-link-list a:hover {
  background: var(--color-primary-subtle, #e3f2fd);
}
.telegram-link-list a span {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.telegram-link-list a em {
  font-style: normal;
  font-size: 10px;
  color: var(--color-text-secondary, #aaa);
}
.telegram-timeline-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border, #f0f0f0);
  font-size: 12px;
}
.telegram-timeline-row div {
  flex: 1;
}
.telegram-timeline-row div strong {
  display: block;
  font-size: 11px;
  color: var(--color-text-secondary, #555);
}
.telegram-timeline-row div p {
  margin: 2px 0 0;
  color: var(--color-text, #333);
}
.telegram-timeline-row time {
  font-size: 10px;
  color: var(--color-text-secondary, #aaa);
  white-space: nowrap;
}
.telegram-compose-bar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  border-top: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
}
.telegram-compose-bar textarea {
  flex: 1;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  padding: 8px 12px;
  font-size: 12px;
  font-family: inherit;
  resize: none;
  outline: none;
  min-height: 36px;
  color: var(--color-text, #333);
  background: var(--color-bg, #f9f9f9);
}
.telegram-compose-bar button {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 6px;
  color: var(--color-text-secondary, #777);
  border-radius: 4px;
}
.telegram-compose-bar button:hover:not(:disabled) {
  background: var(--color-bg, #f5f5f5);
}
.telegram-compose-bar button.send {
  background: var(--color-primary, #0066cc);
  color: #fff;
  border-radius: 8px;
  padding: 8px;
}
.telegram-compose-bar button.send:disabled {
  opacity: 0.5;
}
.telegram-compose-menu {
  position: relative;
}
.telegram-emoji-popover {
  position: absolute;
  bottom: 100%;
  right: 0;
  display: flex;
  gap: 2px;
  padding: 6px;
  background: var(--color-surface, #fff);
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  margin-bottom: 4px;
  z-index: 10;
}
.telegram-emoji-popover button {
  padding: 4px 8px;
  font-size: 18px;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: 4px;
}
.telegram-emoji-popover button:hover {
  background: var(--color-bg, #f5f5f5);
}
.command-popover {
  position: absolute;
  bottom: 100%;
  right: 0;
  min-width: 140px;
  background: var(--color-surface, #fff);
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 8px;
  padding: 4px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  margin-bottom: 4px;
  z-index: 10;
}
.command-popover button {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 6px 10px;
  border: none;
  border-radius: 4px;
  background: transparent;
  font-size: 11px;
  cursor: pointer;
  color: var(--color-text, #333);
}
.command-popover button:hover {
  background: var(--color-bg, #f5f5f5);
}
</style>
