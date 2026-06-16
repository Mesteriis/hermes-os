<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import type {
  TelegramAttachmentHint,
  TelegramMediaItem,
  TelegramMessage,
  TelegramThreadTab
} from '../../types/telegram'
import { mergeTelegramAttachmentHints } from '../../stores/telegram'
import TelegramMediaViewer from './TelegramMediaViewer.vue'

const { t } = useI18n()

const emit = defineEmits<{
  downloadMedia: [attachment: TelegramAttachmentHint, message?: TelegramMessage]
  openMessage: [message: TelegramMessage]
}>()

const props = defineProps<{
  activeThreadTab: TelegramThreadTab
  chronologicalMessages: TelegramMessage[]
  fileHints: TelegramAttachmentHint[]
  voiceHints: TelegramAttachmentHint[]
  mediaGalleryItems: TelegramMediaItem[]
  linkHints: Array<{ url: string; label: string; occurredAt: string | null }>
  pinnedMessages: TelegramMessage[]
  isTelegramActionSubmitting: boolean
  telegramMessageTime: (message: TelegramMessage) => string
}>()

const activeViewerAttachment = ref<TelegramAttachmentHint | null>(null)

const mergedFileHints = computed(() =>
  mergeTelegramAttachmentHints(props.mediaGalleryItems, props.fileHints)
)

function senderName(message: TelegramMessage): string {
  return message.sender_display_name ?? message.sender
}

function messageForAttachment(attachment: TelegramAttachmentHint): TelegramMessage | null {
  return props.chronologicalMessages.find((message) => message.message_id === attachment.messageId) ?? null
}

function openAttachmentMessage(attachment: TelegramAttachmentHint) {
  const message = messageForAttachment(attachment)
  if (message) {
    emit('openMessage', message)
  }
}

function attachmentSender(attachment: TelegramAttachmentHint): string {
  const message = messageForAttachment(attachment)
  return message ? senderName(message) : ''
}

function attachmentTime(attachment: TelegramAttachmentHint): string {
  const message = messageForAttachment(attachment)
  return message ? props.telegramMessageTime(message) : ''
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

function openViewer(attachment: TelegramAttachmentHint) {
  activeViewerAttachment.value = attachment
}
</script>

<template>
  <div class="chat-body telegram-thread-body">
    <template v-if="activeThreadTab === 'files'">
      <div v-if="mediaGalleryItems.length === 0 && fileHints.length === 0" class="empty-panel fill">
        {{ t('No files in selected Telegram history.') }}
      </div>
      <div v-else class="telegram-file-list">
        <div
          v-for="attachment in mergedFileHints"
          :key="attachment.id"
          class="telegram-file-card"
        >
          <span>
            <Icon
              :icon="attachment.kind === 'photo' ? 'tabler:photo' : attachment.kind === 'video' ? 'tabler:video' : attachment.kind === 'audio' || attachment.kind === 'voice' ? 'tabler:wave-sine' : 'tabler:file-description'"
              width="20"
              height="20"
            />
          </span>
          <div>
            <strong>{{ attachment.fileName }}</strong>
            <small>{{ attachment.mimeType ?? attachment.kind }} · {{ attachment.sizeBytes == null ? attachment.downloadState : formatBytes(attachment.sizeBytes) }}</small>
          </div>
          <button
            v-if="messageForAttachment(attachment)"
            type="button"
            :title="t('Open message')"
            @click="openAttachmentMessage(attachment)"
          >
            <Icon icon="tabler:arrow-up-right" width="17" height="17" />
          </button>
          <button
            type="button"
            :title="t('Preview media')"
            @click="openViewer(attachment)"
          >
            <Icon icon="tabler:eye" width="17" height="17" />
          </button>
          <button
            type="button"
            :disabled="isTelegramActionSubmitting || attachment.tdlibFileId === null"
            :title="attachment.tdlibFileId === null ? t('Download requires TDLib file metadata') : t('Download media')"
            @click="emit('downloadMedia', attachment, messageForAttachment(attachment) ?? undefined)"
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

    <template v-else-if="activeThreadTab === 'voice'">
      <div v-if="voiceHints.length === 0" class="empty-panel fill">
        {{ t('No voice notes or audio files in selected Telegram history.') }}
      </div>
      <div v-else class="telegram-voice-list">
        <article
          v-for="voice in voiceHints"
          :key="voice.id"
          class="telegram-voice-card"
        >
          <div class="telegram-voice-card__meta">
            <span class="telegram-voice-card__icon">
              <Icon :icon="voice.kind === 'voice' ? 'tabler:microphone-2' : 'tabler:wave-sine'" width="18" height="18" />
            </span>
            <div>
              <strong>{{ voice.fileName }}</strong>
              <small>
                {{ voice.mimeType ?? voice.kind }} · {{ voice.sizeBytes == null ? voice.downloadState : formatBytes(voice.sizeBytes) }}
              </small>
            </div>
            <button
              v-if="messageForAttachment(voice)"
              type="button"
              class="telegram-voice-card__jump"
              :title="t('Open message')"
              @click="openAttachmentMessage(voice)"
            >
              <Icon icon="tabler:arrow-up-right" width="16" height="16" />
            </button>
          </div>
          <audio
            v-if="voice.localPath"
            class="telegram-voice-card__player"
            :src="voice.localPath"
            controls
            preload="metadata"
          ></audio>
          <div v-else class="telegram-voice-card__empty">
            <span>{{ t('Voice playback is available after local download.') }}</span>
            <button
              type="button"
              :disabled="isTelegramActionSubmitting || voice.tdlibFileId === null"
              :title="voice.tdlibFileId === null ? t('Download requires TDLib file metadata') : t('Download voice file')"
              @click="emit('downloadMedia', voice, messageForAttachment(voice) ?? undefined)"
            >
              <Icon icon="tabler:download" width="16" height="16" />
              {{ t('Download') }}
            </button>
          </div>
          <footer v-if="messageForAttachment(voice)" class="telegram-voice-card__footer">
            <span>{{ attachmentSender(voice) }}</span>
            <time>{{ attachmentTime(voice) }}</time>
          </footer>
        </article>
      </div>
    </template>

    <template v-else-if="activeThreadTab === 'pinned'">
      <div v-if="pinnedMessages.length === 0" class="empty-panel fill">
        {{ t('No pinned messages in selected Telegram history.') }}
      </div>
      <template v-else>
        <article
          v-for="message in pinnedMessages"
          :key="message.message_id"
          class="telegram-timeline-row telegram-timeline-row-action"
          @click="emit('openMessage', message)"
        >
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
  <TelegramMediaViewer
    :attachment="activeViewerAttachment"
    @close="activeViewerAttachment = null"
  />
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
.telegram-link-list,
.telegram-voice-list {
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
.telegram-voice-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--color-border, #d8e0e7);
  border-radius: 10px;
  background: linear-gradient(180deg, rgba(249, 251, 252, 0.98) 0%, rgba(241, 246, 249, 0.98) 100%);
}
.telegram-voice-card__meta,
.telegram-voice-card__footer,
.telegram-voice-card__empty {
  display: flex;
  align-items: center;
  gap: 10px;
}
.telegram-voice-card__meta strong,
.telegram-voice-card__meta small {
  display: block;
}
.telegram-voice-card__meta small,
.telegram-voice-card__footer,
.telegram-voice-card__empty {
  font-size: 11px;
  color: var(--color-text-secondary, #667085);
}
.telegram-voice-card__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 999px;
  background: rgba(12, 74, 110, 0.08);
  color: #0c4a6e;
  flex-shrink: 0;
}
.telegram-voice-card__jump {
  margin-left: auto;
  border: none;
  background: transparent;
  color: var(--color-text-secondary, #667085);
  padding: 4px;
  border-radius: 6px;
  cursor: pointer;
}
.telegram-voice-card__jump:hover,
.telegram-voice-card__empty button:hover:not(:disabled) {
  background: var(--color-primary-subtle, #e3f2fd);
}
.telegram-voice-card__player {
  width: 100%;
}
.telegram-voice-card__empty {
  justify-content: space-between;
  flex-wrap: wrap;
}
.telegram-voice-card__empty button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid var(--color-border, #d0d5dd);
  background: var(--color-surface, #fff);
  border-radius: 999px;
  padding: 6px 10px;
  cursor: pointer;
}
.telegram-voice-card__footer {
  justify-content: space-between;
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
.telegram-timeline-row-action {
  cursor: pointer;
}
.telegram-timeline-row-action:hover {
  background: var(--color-primary-subtle, #e3f2fd);
}
</style>
