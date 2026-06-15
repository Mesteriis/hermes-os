<script setup lang="ts">
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import type {
  TelegramAttachmentHint,
  TelegramMessage,
  TelegramThreadTab
} from '../../types/telegram'

const { t } = useI18n()

defineProps<{
  activeThreadTab: TelegramThreadTab
  chronologicalMessages: TelegramMessage[]
  fileHints: TelegramAttachmentHint[]
  linkHints: Array<{ url: string; label: string; occurredAt: string | null }>
  pinnedMessages: TelegramMessage[]
  isTelegramActionSubmitting: boolean
  telegramMessageTime: (message: TelegramMessage) => string
}>()

const emit = defineEmits<{
  downloadMedia: [attachment: TelegramAttachmentHint, message?: TelegramMessage]
}>()

function senderName(message: TelegramMessage): string {
  return message.sender_display_name ?? message.sender
}

function isOutbound(message: TelegramMessage): boolean {
  return message.delivery_state === 'sent' || message.delivery_state === 'send_dry_run'
}

function formatDate(value: string): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric' }).format(date)
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <div class="chat-body telegram-thread-body">
    <template v-if="activeThreadTab === 'files'">
      <div v-if="fileHints.length === 0" class="empty-panel fill">
        {{ t('No files in selected Telegram history.') }}
      </div>
      <div v-else class="telegram-file-list">
        <div v-for="file in fileHints" :key="file.fileName" class="telegram-file-card">
          <span>
            <Icon
              :icon="file.kind === 'photo' ? 'tabler:photo' : file.kind === 'video' ? 'tabler:video' : 'tabler:file-description'"
              width="20"
              height="20"
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
            @click="emit('downloadMedia', file)"
          >
            <Icon icon="tabler:download" width="17" height="17" />
          </button>
        </div>
      </div>
    </template>

    <template v-else-if="activeThreadTab === 'links'">
      <div v-if="linkHints.length === 0" class="empty-panel fill">
        {{ t('No links in selected Telegram history.') }}
      </div>
      <div v-else class="telegram-link-list">
        <a v-for="(link, idx) in linkHints" :key="idx" :href="link.url" target="_blank" rel="noreferrer">
          <Icon icon="tabler:link" width="17" height="17" />
          <span>{{ link.label }}</span>
          <em>{{ link.occurredAt ? formatDate(link.occurredAt) : '' }}</em>
        </a>
      </div>
    </template>

    <template v-else-if="activeThreadTab === 'pinned'">
      <div v-if="pinnedMessages.length === 0" class="empty-panel fill">
        {{ t('No pinned messages in selected Telegram history.') }}
      </div>
      <template v-else>
        <article v-for="message in pinnedMessages" :key="message.message_id" class="telegram-timeline-row">
          <Icon icon="tabler:pin" width="16" height="16" />
          <div><strong>{{ senderName(message) }}</strong><p>{{ message.text }}</p></div>
          <time>{{ telegramMessageTime(message) }}</time>
        </article>
      </template>
    </template>

    <template v-else-if="activeThreadTab === 'timeline'">
      <div v-if="chronologicalMessages.length === 0" class="empty-panel fill">
        {{ t('No timeline events in selected Telegram history.') }}
      </div>
      <template v-else>
        <article v-for="message in chronologicalMessages" :key="message.message_id" class="telegram-timeline-row">
          <Icon :icon="isOutbound(message) ? 'tabler:send' : 'tabler:message'" width="16" height="16" />
          <div><strong>{{ senderName(message) }}</strong><p>{{ message.text }}</p></div>
          <time>{{ telegramMessageTime(message) }}</time>
        </article>
      </template>
    </template>

    <div v-else class="empty-panel fill">
      {{ t('Telegram topics are available after TDLib forum topic sync is implemented.') }}
    </div>
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
.telegram-file-list,
.telegram-link-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px 0;
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
.telegram-file-card button:hover:not(:disabled),
.telegram-link-list a:hover {
  background: var(--color-primary-subtle, #e3f2fd);
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
.telegram-link-list a span {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.telegram-link-list a em,
.telegram-timeline-row time {
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
  white-space: nowrap;
}
</style>
