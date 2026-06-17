<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import type { MessageAnalyzeResponse } from '../../../communications/types/communications'
import type { TelegramAttachmentHint, TelegramChat, TelegramMessage, TelegramOperationCapability, TelegramCapabilitiesResponse } from '../../types/telegram'
import { telegramMessageAttachmentHints } from '../../stores/telegram'
import { telegramAttachmentReadiness } from '../../stores/telegramMediaSearch'
import { telegramMessageMentionLabel, telegramMessageMentionProjection } from '../../stores/telegramMentionProjection'
import { telegramCanMarkMessageRead, telegramThreadReadProgress } from '../../stores/telegramReadProgress'
import TelegramMessageReferencePanel from './TelegramMessageReferencePanel.vue'
import TelegramMessageReactions from './TelegramMessageReactions.vue'

const { t } = useI18n()
const props = defineProps<{
  selectedTelegramChat: TelegramChat
  filteredMessages: TelegramMessage[]
  threadSearchQuery: string
  isTelegramActionSubmitting: boolean
  aiAnalysisResult: MessageAnalyzeResponse | null
  selectedCommunication: { message_id?: string } | null
  telegramMessageTime: (message: TelegramMessage) => string
  capabilities?: TelegramCapabilitiesResponse | null
}>()

const emit = defineEmits<{
  syncOlderHistory: []
  downloadMedia: [attachment: TelegramAttachmentHint, message?: TelegramMessage]
  editMessage: [message: TelegramMessage]
  deleteMessage: [message: TelegramMessage]
  restoreMessage: [message: TelegramMessage]
  replyMessage: [message: TelegramMessage]
  forwardMessage: [message: TelegramMessage]
  markReadMessage: [message: TelegramMessage]
  togglePinMessage: [message: TelegramMessage]
  addReaction: [payload: { message: TelegramMessage; emoji: string }]
  removeReaction: [payload: { message: TelegramMessage; emoji: string }]
  openSearchMessage: [message: TelegramMessage]
}>()

const editingMessageId = ref<string | null>(null)
const editText = ref('')
const confirmDeleteId = ref<string | null>(null)
const activeReactionPicker = ref<string | null>(null)
const activeReferencePanel = ref<string | null>(null)
function capability(operation: string): TelegramOperationCapability | undefined {
  return props.capabilities?.capabilities.find((item) => item.operation === operation)
}
function toggleReactionPicker(messageId: string) {
  activeReactionPicker.value = activeReactionPicker.value === messageId ? null : messageId
}
function toggleReferencePanel(messageId: string) {
  activeReferencePanel.value = activeReferencePanel.value === messageId ? null : messageId
}
function emitReaction(payload: { message: TelegramMessage; emoji: string }) {
  activeReactionPicker.value = null
  emit('addReaction', payload)
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
function isCapabilityVisible(operation: string): boolean {
  return capability(operation)?.status !== 'unsupported'
}
function isCapabilityAvailable(operation: string): boolean {
  return capability(operation)?.status === 'available'
}
function capabilityTitle(operation: string, fallbackLabel: string): string {
  const op = capability(operation)
  if (!op) return fallbackLabel
  return op.status === 'available' ? fallbackLabel : `${fallbackLabel}: ${op.reason}`
}
function canEdit(): boolean {
  return isCapabilityAvailable('messages.edit')
}
function canDelete(): boolean {
  return isCapabilityAvailable('messages.delete')
}
function canRestore(): boolean {
  return isCapabilityAvailable('messages.restore_visibility')
}
function canForward(): boolean {
  return isCapabilityAvailable('messages.forward')
}
function canPin(): boolean {
  const status = capability('messages.pin')?.status
  return status === 'available' || status === 'degraded'
}
function isMessagePinned(message: TelegramMessage): boolean {
  return Boolean(message.metadata?.is_pinned ?? message.metadata?.pinned)
}
function messagePinTitle(message: TelegramMessage): string {
  return capabilityTitle(
    'messages.pin',
    isMessagePinned(message) ? t('Message unpinned') : t('Message pinned')
  )
}
function startEdit(message: TelegramMessage) {
  editingMessageId.value = message.message_id
  editText.value = message.text
}
function cancelEdit() {
  editingMessageId.value = null
  editText.value = ''
}
function confirmEdit(message: TelegramMessage) {
  if (editText.value.trim() && editText.value !== message.text) {
    emit('editMessage', { ...message, text: editText.value })
    editingMessageId.value = null
  } else {
    cancelEdit()
  }
}
function startDelete(message: TelegramMessage) {
  confirmDeleteId.value = message.message_id
}
function cancelDelete() {
  confirmDeleteId.value = null
}
function confirmDelete(message: TelegramMessage) {
  emit('deleteMessage', message)
  confirmDeleteId.value = null
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

function attachmentReadiness(attachment: TelegramAttachmentHint) {
  return telegramAttachmentReadiness(attachment)
}

const providerReadProgress = computed(() =>
  telegramThreadReadProgress(props.selectedTelegramChat, props.filteredMessages)
)

function isProviderReadBoundary(message: TelegramMessage): boolean {
  return providerReadProgress.value.boundaryAfterMessageId === message.message_id
}

function canMarkReadMessage(message: TelegramMessage): boolean {
  const status = capability('messages.mark_read')?.status
  if (status !== 'available' && status !== 'degraded') return false
  return telegramCanMarkMessageRead(props.selectedTelegramChat, props.filteredMessages, message)
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
        :class="{ outbound: isOutbound(message), 'is-editing': editingMessageId === message.message_id }"
      >
        <span class="telegram-message-avatar">{{ senderInitials(message) }}</span>
        <div class="bubble telegram-bubble" :class="{ outbound: isOutbound(message), inbound: !isOutbound(message) }">
          <strong>{{ senderName(message) }}</strong>
          <div v-if="editingMessageId === message.message_id" class="telegram-edit-area">
            <textarea
              v-model="editText"
              class="telegram-edit-input"
              rows="3"
              :disabled="isTelegramActionSubmitting"
            />
            <div class="telegram-edit-actions">
              <button
                type="button"
                class="btn-small btn-primary"
                :disabled="isTelegramActionSubmitting || !editText.trim()"
                @click="confirmEdit(message)"
              >
                {{ t('Save') }}
              </button>
              <button
                type="button"
                class="btn-small btn-ghost"
                :disabled="isTelegramActionSubmitting"
                @click="cancelEdit()"
              >
                {{ t('Cancel') }}
              </button>
            </div>
          </div>
          <p v-else>{{ message.text }}</p>
          <div
            v-if="telegramMessageMentionProjection(message).count > 0"
            class="telegram-message-mentions"
            :title="telegramMessageMentionProjection(message).detected_by"
          >
            <Icon icon="tabler:at" width="13" height="13" />
            <span>{{ telegramMessageMentionLabel(message) }}</span>
          </div>
          <div v-if="confirmDeleteId === message.message_id" class="telegram-delete-confirm">
            <p>{{ t('Delete this message? A tombstone record will be created.') }}</p>
            <div class="telegram-edit-actions">
              <button
                type="button"
                class="btn-small btn-danger"
                :disabled="isTelegramActionSubmitting"
                @click="confirmDelete(message)"
              >
                {{ t('Delete') }}
              </button>
              <button
                type="button"
                class="btn-small btn-ghost"
                :disabled="isTelegramActionSubmitting"
                @click="cancelDelete()"
              >
                {{ t('Cancel') }}
              </button>
            </div>
          </div>

          <div v-if="telegramMessageAttachmentHints(message).length" class="telegram-bubble-files">
            <div
              v-for="attachment in telegramMessageAttachmentHints(message)"
              :key="attachment.messageId + attachment.fileName"
              class="telegram-file-card compact"
            >
              <span><Icon icon="tabler:file" width="18" height="18" /></span>
              <div>
                <strong>{{ attachment.fileName }}</strong>
                <small>{{ attachment.sizeBytes == null ? attachmentReadiness(attachment).label : `${formatBytes(attachment.sizeBytes)} · ${attachmentReadiness(attachment).label}` }}</small>
                <small>{{ attachmentReadiness(attachment).detail }}</small>
              </div>
              <button
                type="button"
                :disabled="isTelegramActionSubmitting || !attachmentReadiness(attachment).can_request_download"
                :title="attachmentReadiness(attachment).can_request_download ? t(attachmentReadiness(attachment).action_label) : attachmentReadiness(attachment).detail"
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
          <TelegramMessageReactions
            :message="message"
            :capabilities="capabilities"
            :isTelegramActionSubmitting="isTelegramActionSubmitting"
            :isPickerOpen="activeReactionPicker === message.message_id"
            @togglePicker="toggleReactionPicker"
            @addReaction="emitReaction"
            @removeReaction="emit('removeReaction', $event)"
          />
          <div class="telegram-message-actions" v-if="isOutbound(message) && !isTelegramActionSubmitting && editingMessageId !== message.message_id && confirmDeleteId !== message.message_id">
            <button
              type="button"
              class="btn-icon-only"
              :title="t('Message references')"
              @click.stop="toggleReferencePanel(message.message_id)"
            >
              <Icon icon="tabler:git-merge" width="14" height="14" />
            </button>
            <button
              type="button"
              class="btn-icon-only"
              :title="t('Reply')"
              @click.stop="emit('replyMessage', message)"
            >
              <Icon icon="tabler:corner-up-left" width="14" height="14" />
            </button>
            <button v-if="isCapabilityVisible('messages.forward')" type="button" class="btn-icon-only" :title="capabilityTitle('messages.forward', t('Forward'))" :disabled="!canForward()" @click.stop="emit('forwardMessage', message)">
              <Icon icon="tabler:arrow-forward-up" width="14" height="14" />
            </button>
            <button
              v-if="isCapabilityVisible('messages.mark_read')"
              type="button"
              class="btn-icon-only"
              :title="capabilityTitle('messages.mark_read', t('Mark read'))"
              :disabled="!canMarkReadMessage(message)"
              @click.stop="emit('markReadMessage', message)"
            >
              <Icon icon="tabler:mail-opened" width="14" height="14" />
            </button>
            <button
              v-if="isCapabilityVisible('messages.pin')"
              type="button"
              class="btn-icon-only"
              :title="messagePinTitle(message)"
              :disabled="!canPin()"
              @click.stop="emit('togglePinMessage', message)"
            >
              <Icon :icon="isMessagePinned(message) ? 'tabler:pinned-off' : 'tabler:pinned'" width="14" height="14" />
            </button>
            <button
              v-if="isCapabilityVisible('messages.edit')"
              type="button"
              class="btn-icon-only"
              :title="capabilityTitle('messages.edit', t('Edit message'))"
              :disabled="!canEdit()"
              @click.stop="startEdit(message)"
            >
              <Icon icon="tabler:pencil" width="14" height="14" />
            </button>
            <button
              v-if="isCapabilityVisible('messages.delete')"
              type="button"
              class="btn-icon-only btn-icon-danger"
              :title="capabilityTitle('messages.delete', t('Delete message'))"
              :disabled="!canDelete()"
              @click.stop="startDelete(message)"
            >
              <Icon icon="tabler:trash" width="14" height="14" />
            </button>
          </div>
          <button
            v-else-if="!isTelegramActionSubmitting"
            type="button"
            class="btn-icon-only telegram-reference-toggle"
            :title="t('Message references')"
            @click.stop="toggleReferencePanel(message.message_id)"
          >
            <Icon icon="tabler:git-merge" width="14" height="14" />
          </button>
          <div class="telegram-message-actions" v-if="!isOutbound(message) && !isTelegramActionSubmitting && (isCapabilityVisible('messages.restore_visibility') || isCapabilityVisible('messages.pin') || isCapabilityVisible('messages.forward'))">
            <button
              type="button"
              class="btn-icon-only"
              :title="t('Reply')"
              @click.stop="emit('replyMessage', message)"
            >
              <Icon icon="tabler:corner-up-left" width="14" height="14" />
            </button>
            <button v-if="isCapabilityVisible('messages.forward')" type="button" class="btn-icon-only" :title="capabilityTitle('messages.forward', t('Forward'))" :disabled="!canForward()" @click.stop="emit('forwardMessage', message)">
              <Icon icon="tabler:arrow-forward-up" width="14" height="14" />
            </button>
            <button
              v-if="isCapabilityVisible('messages.pin')"
              type="button"
              class="btn-icon-only"
              :title="messagePinTitle(message)"
              :disabled="!canPin()"
              @click.stop="emit('togglePinMessage', message)"
            >
              <Icon :icon="isMessagePinned(message) ? 'tabler:pinned-off' : 'tabler:pinned'" width="14" height="14" />
            </button>
            <button
              type="button"
              class="btn-icon-only"
              :title="capabilityTitle('messages.restore_visibility', t('Restore visibility'))"
              :disabled="!canRestore()"
              @click.stop="emit('restoreMessage', message)"
            >
              <Icon icon="tabler:eye" width="14" height="14" />
            </button>
          </div>
          <TelegramMessageReferencePanel
            v-if="activeReferencePanel === message.message_id"
            :messageId="message.message_id"
            :isOpen="activeReferencePanel === message.message_id"
            :currentMessage="message"
            @openMessage="emit('openSearchMessage', $event)"
          />
        </div>
        <div
          v-if="isProviderReadBoundary(message)"
          class="telegram-read-progress-divider"
          :title="providerReadProgress.lastReadProviderMessageId ?? undefined"
        >
          <span>{{ t('Read through here') }}</span>
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
.telegram-history-actions, .telegram-date-chip {
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
  flex-wrap: wrap;
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
.telegram-read-progress-divider {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 6px 0 10px;
  color: var(--color-text-secondary, #7b8190);
  font-size: 10px;
  font-weight: 600;
}
.telegram-read-progress-divider::before,
.telegram-read-progress-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: color-mix(in srgb, var(--color-border, #d7dbe4) 85%, transparent);
}
.telegram-read-progress-divider span {
  white-space: nowrap;
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
.telegram-message-mentions {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-top: 6px;
  padding: 2px 7px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--color-primary, #0066cc) 10%, transparent);
  color: var(--color-primary, #0066cc);
  font-size: 10px;
  font-weight: 600;
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
.telegram-file-card button:hover:not(:disabled) { background: var(--color-primary-subtle, #e3f2fd); }
.telegram-message-actions {
  display: flex;
  gap: 2px;
  margin-top: 4px;
  justify-content: flex-end;
}
.btn-icon-only {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: var(--color-text-secondary, #777);
  padding: 0;
}
.btn-icon-only:hover {
  background: var(--color-surface-hover, #f0f0f0);
}
.btn-icon-danger:hover {
  background: var(--color-danger-subtle, #fde8e8);
  color: var(--color-danger, #c62828);
}
.telegram-edit-area {
  margin: 4px 0;
}
.telegram-edit-input {
  width: 100%;
  border: 1px solid var(--color-border, #e0e0e0);
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 12px;
  font-family: inherit;
  resize: vertical;
  background: var(--color-bg, #fff);
  color: var(--color-text, #333);
}
.telegram-edit-actions {
  display: flex;
  gap: 4px;
  margin-top: 4px;
  justify-content: flex-end;
}
.btn-small {
  padding: 2px 8px;
  font-size: 11px;
  border-radius: 4px;
  border: 1px solid var(--color-border, #e0e0e0);
  cursor: pointer;
  background: var(--color-surface, #fff);
  color: var(--color-text, #333);
}
.btn-small:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.btn-primary { background: var(--color-primary, #0066cc); color: #fff; border-color: var(--color-primary, #0066cc); }
.btn-danger { background: var(--color-danger, #c62828); color: #fff; border-color: var(--color-danger, #c62828); }
.btn-ghost { background: transparent; border-color: transparent; }
.telegram-delete-confirm {
  margin: 4px 0;
  padding: 6px;
  background: var(--color-danger-subtle, #fde8e8);
  border-radius: 4px;
}
.telegram-delete-confirm p {
  font-size: 11px;
  margin: 0 0 4px;
  color: var(--color-danger, #c62828);
}
</style>
