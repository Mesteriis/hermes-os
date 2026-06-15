<script setup lang="ts">
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import type { MessageAnalyzeResponse } from '../../../communications/types/communications'
import type { TelegramAttachmentHint, TelegramChat, TelegramMessage } from '../../types/telegram'
import { telegramMessageAttachmentHints } from '../../stores/telegram'

const { t } = useI18n()

const props = defineProps<{
  selectedTelegramChat: TelegramChat
  filteredMessages: TelegramMessage[]
  threadSearchQuery: string
  isTelegramActionSubmitting: boolean
  aiAnalysisResult: MessageAnalyzeResponse | null
  selectedCommunication: { message_id?: string } | null
  telegramMessageTime: (message: TelegramMessage) => string
}>()

const emit = defineEmits<{
  syncOlderHistory: []
  downloadMedia: [attachment: TelegramAttachmentHint, message?: TelegramMessage]
}>()

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

function handleThreadScroll(event: Event) {
  if (props.isTelegramActionSubmitting) return
  const target = event.currentTarget as HTMLElement | null
  if (!target || target.scrollTop > 48) return
  emit('syncOlderHistory')
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
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

    <div v-if="selectedTelegramChat.chat_kind !== 'private'" class="telegram-history-actions">
      <button type="button" :disabled="isTelegramActionSubmitting" @click="emit('syncOlderHistory')">
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
        <div class="bubble telegram-bubble" :class="{ outbound: isOutbound(message), inbound: !isOutbound(message) }">
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
                @click="emit('downloadMedia', attachment, message)"
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
  </div>
</template>

<style scoped>
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
.telegram-history-actions,
.telegram-date-chip {
  text-align: center;
  padding: 8px 0;
}
.telegram-history-actions button,
.telegram-file-card button {
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--color-text-secondary, #777);
  border-radius: 4px;
}
.telegram-history-actions button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  border: 1px solid var(--color-border, #e0e0e0);
  background: var(--color-surface, #fff);
  font-size: 11px;
}
.telegram-date-chip {
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
  border-bottom-left-radius: 4px;
}
.bubble.outbound {
  background: var(--color-primary-subtle, #e3f2fd);
  border: 1px solid var(--color-primary-light, #bbdefb);
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
  padding: 4px 6px;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 6px;
  background: var(--color-bg, #f9f9f9);
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
  padding: 4px;
  flex-shrink: 0;
}
.telegram-file-card button:hover:not(:disabled) {
  background: var(--color-primary-subtle, #e3f2fd);
}
</style>
