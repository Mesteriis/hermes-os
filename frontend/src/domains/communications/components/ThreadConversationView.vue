<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import ThreadInlineReplyComposer from './ThreadInlineReplyComposer.vue'
import ThreadAttachmentInsightPanel from './ThreadAttachmentInsightPanel.vue'
import { useTranslateThreadMutation } from '../queries/useCommunicationsQuery'
import type { CommunicationThreadSummary, ThreadMessage } from '../types/communications'
import type { ThreadTranslationResponse } from '../types/multilingual'
import { attachmentIcon } from '../stores/communications'
import { messageTime, senderEmail, senderLabel } from '../stores/communications'
import { formatAttachmentSize, scanStatusClass } from './attachmentTable'
import { previewThreadMessageBody, splitThreadMessageBody } from './threadMessageBody'
import {
  defaultExpandedThreadMessageIds,
  hasQuotedThreadMessages,
  summarizeThreadExpansion
} from './threadConversationPresentation'

const props = defineProps<{
  thread: CommunicationThreadSummary
  messages: ThreadMessage[]
  isLoading: boolean
  errorMessage: string
  isSendingReply: boolean
}>()

const emit = defineEmits<{
  openMessage: [messageId: string]
  replyToMessage: [message: ThreadMessage, bodyHtml: string, draftId: string]
  saveReplyDraft: [message: ThreadMessage, bodyHtml: string, draftId: string]
  sendReply: [message: ThreadMessage, bodyHtml: string, draftId: string]
}>()

const expandedMessageIds = ref<Set<string>>(new Set())
const activeReplyMessageId = ref('')
const activeReplyDraftId = ref('')
const inlineReplyHtml = ref('')
const showQuotedContent = ref(true)
const autoExpandedThreadId = ref('')
const threadTranslationTarget = ref('en')
const threadTranslationResult = ref<ThreadTranslationResponse | null>(null)
const threadTranslationError = ref('')
const translateThreadMutation = useTranslateThreadMutation()
const isTranslatingThread = computed(() => translateThreadMutation.isPending.value)
const canTranslateThread = computed(() => props.messages.length > 0 && !isTranslatingThread.value)
const expansionSummary = computed(() => summarizeThreadExpansion(props.messages, expandedMessageIds.value))
const expandedMessageCount = computed(() => expansionSummary.value.expandedCount)
const hasQuotedMessages = computed(() => hasQuotedThreadMessages(props.messages))
const canExpandAllMessages = computed(() => expansionSummary.value.canExpandAll)
const canCollapseAllMessages = computed(() => expansionSummary.value.canCollapseAll)

const translatedMessages = computed(() => {
  const items = threadTranslationResult.value?.items ?? []
  return new Map(items.map((item) => [item.message_id, item]))
})
const translatedThreadCount = computed(() =>
  threadTranslationResult.value?.items.filter((item) => item.translated).length ?? 0
)

watch(
  () => props.thread.thread_id,
  () => {
    expandedMessageIds.value = new Set()
    autoExpandedThreadId.value = ''
    cancelInlineReply()
    threadTranslationResult.value = null
    threadTranslationError.value = ''
  }
)

watch(
  () => props.messages,
  (messages) => {
    if (messages.length === 0) return
    if (autoExpandedThreadId.value === props.thread.thread_id) return
    expandedMessageIds.value = defaultExpandedThreadMessageIds(messages)
    autoExpandedThreadId.value = props.thread.thread_id
  }
)

function isMessageExpanded(messageId: string): boolean {
  return expandedMessageIds.value.has(messageId)
}

function toggleMessageExpanded(messageId: string): void {
  const next = new Set(expandedMessageIds.value)
  if (next.has(messageId)) {
    next.delete(messageId)
  } else {
    next.add(messageId)
  }
  expandedMessageIds.value = next
}

function expandAllMessages(): void {
  expandedMessageIds.value = new Set(props.messages.map((message) => message.message_id))
}

function collapseAllMessages(): void {
  expandedMessageIds.value = new Set()
}

function startInlineReply(message: ThreadMessage): void {
  activeReplyMessageId.value = message.message_id
  activeReplyDraftId.value = `draft-${Date.now()}`
  inlineReplyHtml.value = ''
}

function cancelInlineReply(): void {
  activeReplyMessageId.value = ''
  activeReplyDraftId.value = ''
  inlineReplyHtml.value = ''
}

function continueReplyInCompose(message: ThreadMessage): void {
  emit('replyToMessage', message, inlineReplyHtml.value, activeReplyDraftId.value)
  cancelInlineReply()
}

function saveInlineReplyDraft(message: ThreadMessage): void {
  if (!activeReplyDraftId.value || !inlineReplyHtml.value.trim()) return
  emit('saveReplyDraft', message, inlineReplyHtml.value, activeReplyDraftId.value)
}

function sendInlineReply(message: ThreadMessage): void {
  emit('sendReply', message, inlineReplyHtml.value, activeReplyDraftId.value)
}

async function handleTranslateThread(): Promise<void> {
  const firstMessage = props.messages[0]
  if (!firstMessage) return

  threadTranslationError.value = ''
  try {
    threadTranslationResult.value = await translateThreadMutation.mutateAsync({
      accountId: firstMessage.account_id,
      subject: firstMessage.subject,
      targetLanguage: threadTranslationTarget.value,
      limit: Math.max(props.messages.length, 1)
    })
  } catch (e) {
    threadTranslationError.value = e instanceof Error ? e.message : 'Thread translation failed'
  }
}

function translatedTextForMessage(messageId: string): string {
  const item = translatedMessages.value.get(messageId)
  if (!item) return ''
  if (item.translated && item.text) return item.text
  return item.reason ?? 'Translation unavailable'
}

function previewBody(message: ThreadMessage): string {
  return previewThreadMessageBody(message, isMessageExpanded(message.message_id))
}

function quotedBody(message: ThreadMessage): string {
  return splitThreadMessageBody(message.body_text).quotedText
}

function primaryBody(message: ThreadMessage): string {
  const segments = splitThreadMessageBody(message.body_text)
  return isMessageExpanded(message.message_id)
    ? (segments.mainText || segments.quotedText)
    : previewBody(message)
}
</script>

<template>
  <section class="thread-conversation">
    <header class="thread-header">
      <div>
        <p class="thread-kicker">Conversation</p>
        <h2>{{ thread.subject }}</h2>
      </div>
      <div class="thread-meta">
        <span>{{ thread.message_count }} messages</span>
        <span>{{ thread.participant_count }} participants</span>
        <span>{{ expandedMessageCount }} expanded</span>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!canExpandAllMessages"
          @click="expandAllMessages"
        >
          Expand all
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!canCollapseAllMessages"
          @click="collapseAllMessages"
        >
          Collapse all
        </Button>
        <Button
          v-if="hasQuotedMessages"
          variant="ghost"
          size="sm"
          @click="showQuotedContent = !showQuotedContent"
        >
          {{ showQuotedContent ? 'Hide quoted' : 'Show quoted' }}
        </Button>
        <label class="thread-translation-target">
          <span>Translate</span>
          <select v-model="threadTranslationTarget">
            <option value="en">EN</option>
            <option value="ru">RU</option>
            <option value="es">ES</option>
          </select>
        </label>
        <Button
          variant="outline"
          size="sm"
          icon="tabler:language"
          :loading="isTranslatingThread"
          :disabled="!canTranslateThread"
          @click="handleTranslateThread"
        >
          Translate
        </Button>
      </div>
    </header>

    <section v-if="threadTranslationResult" class="thread-translation-panel">
      <div>
        <p class="thread-kicker">Thread translation review</p>
        <strong>{{ threadTranslationResult.items.length }} messages to {{ threadTranslationResult.target_language }}</strong>
      </div>
      <span>{{ translatedThreadCount }} translated</span>
    </section>
    <div v-if="threadTranslationError" class="thread-state error compact">
      <Icon icon="tabler:alert-circle" />
      <span>{{ threadTranslationError }}</span>
    </div>

    <div v-if="errorMessage" class="thread-state error">
      <Icon icon="tabler:alert-circle" />
      <span>{{ errorMessage }}</span>
    </div>
    <div v-else-if="isLoading" class="thread-state">
      <Icon icon="tabler:loader-2" class="spin-icon" />
      <span>Loading conversation...</span>
    </div>
    <div v-else-if="messages.length === 0" class="thread-state">
      <Icon icon="tabler:messages" />
      <span>No messages in this conversation</span>
    </div>
    <ol v-else class="thread-timeline">
      <li
        v-for="message in messages"
        :key="message.message_id"
        class="thread-message"
      >
        <div class="message-marker" />
        <article class="message-card">
          <header class="message-header">
            <div class="sender-block">
              <strong>{{ senderLabel(message.sender) }}</strong>
              <span>{{ senderEmail(message.sender) }}</span>
            </div>
            <div class="message-actions">
              <span class="message-time">{{ messageTime(message.projected_at ?? message.occurred_at) }}</span>
              <Button
                variant="ghost"
                size="sm"
                title="Reply"
                aria-label="Reply to message"
                @click="startInlineReply(message)"
              >
                <Icon icon="tabler:corner-up-left" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                :title="isMessageExpanded(message.message_id) ? 'Collapse' : 'Expand'"
                :aria-label="isMessageExpanded(message.message_id) ? 'Collapse message' : 'Expand message'"
                @click="toggleMessageExpanded(message.message_id)"
              >
                <Icon :icon="isMessageExpanded(message.message_id) ? 'tabler:chevron-up' : 'tabler:chevron-down'" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                title="Open message"
                aria-label="Open full message"
                @click="emit('openMessage', message.message_id)"
              >
                <Icon icon="tabler:mail-opened" />
              </Button>
            </div>
          </header>
          <p
            class="message-body"
            :class="{ collapsed: !isMessageExpanded(message.message_id) }"
          >
            {{ primaryBody(message) }}
          </p>
          <blockquote
            v-if="isMessageExpanded(message.message_id) && showQuotedContent && quotedBody(message)"
            class="message-quoted"
          >
            {{ quotedBody(message) }}
          </blockquote>
          <ul
            v-if="isMessageExpanded(message.message_id) && message.attachments.length > 0"
            class="message-attachments"
            aria-label="Thread message attachments"
          >
            <li
              v-for="attachment in message.attachments"
              :key="attachment.attachment_id"
              class="message-attachment"
            >
              <Icon :icon="attachmentIcon(attachment.content_type)" class="message-attachment-icon" />
              <div class="message-attachment-copy">
                <strong>{{ attachment.filename || 'Unnamed attachment' }}</strong>
                <span>{{ formatAttachmentSize(attachment.size_bytes) }} · {{ attachment.content_type }}</span>
              </div>
              <span class="message-attachment-scan" :class="scanStatusClass(attachment.scan_status)">
                {{ attachment.scan_status }}
              </span>
              <ThreadAttachmentInsightPanel :attachment="attachment" />
            </li>
          </ul>
          <div
            v-if="translatedTextForMessage(message.message_id)"
            class="message-translation"
          >
            <span>Translation</span>
            <p>{{ translatedTextForMessage(message.message_id) }}</p>
          </div>
          <footer class="message-footer">
            <span>{{ message.workflow_state }}</span>
            <span v-if="message.attachment_count > 0">{{ message.attachment_count }} attachments</span>
            <span v-if="message.ai_category">{{ message.ai_category }}</span>
            <button
              v-if="message.body_text.trim().length > 220"
              class="message-expand-link"
              type="button"
              @click="toggleMessageExpanded(message.message_id)"
            >
              {{ isMessageExpanded(message.message_id) ? 'Collapse' : 'Expand' }}
            </button>
          </footer>
          <ThreadInlineReplyComposer
            v-if="activeReplyMessageId === message.message_id"
            v-model:body-html="inlineReplyHtml"
            :message="message"
            :is-sending-reply="isSendingReply"
            @cancel="cancelInlineReply"
            @save-draft="saveInlineReplyDraft(message)"
            @continue-in-compose="continueReplyInCompose(message)"
            @send="sendInlineReply(message)"
          />
        </article>
      </li>
    </ol>
  </section>
</template>

<style scoped>
.thread-conversation {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  background: var(--hh-bg-primary, #ffffff);
}

.thread-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.thread-kicker {
  margin: 0 0 0.25rem;
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0;
  color: var(--hh-text-tertiary, #9ca3af);
}

.thread-header h2 {
  margin: 0;
  font-size: 1.125rem;
  line-height: 1.3;
  color: var(--hh-text-primary, #1f2937);
}

.thread-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
  justify-content: flex-end;
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
}

.thread-meta :deep(button) {
  flex: 0 0 auto;
}

.thread-translation-target {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
}

.thread-translation-target select {
  min-height: 1.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  color: var(--hh-text-primary, #1f2937);
  font: inherit;
}

.thread-translation-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin: 0.75rem 1rem 0;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 84%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.8125rem;
}

.thread-translation-panel strong {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.875rem;
}

.thread-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  flex: 1;
  color: var(--hh-text-secondary, #6b7280);
}

.thread-state.error {
  color: var(--hh-text-error, #ef4444);
}

.thread-state.compact {
  flex: 0 0 auto;
  justify-content: flex-start;
  padding: 0.5rem 1rem 0;
}

.spin-icon {
  animation: spin 1s linear infinite;
}

.thread-timeline {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  margin: 0;
  padding: 1rem;
  list-style: none;
}

.thread-message {
  position: relative;
  display: grid;
  grid-template-columns: 0.75rem minmax(0, 1fr);
  gap: 0.75rem;
  padding-bottom: 1rem;
}

.thread-message::before {
  content: "";
  position: absolute;
  left: 0.3125rem;
  top: 0.75rem;
  bottom: -0.25rem;
  width: 1px;
  background: var(--hh-border, #e5e7eb);
}

.thread-message:last-child::before {
  display: none;
}

.message-marker {
  width: 0.625rem;
  height: 0.625rem;
  border-radius: 50%;
  margin-top: 0.625rem;
  background: var(--hh-accent, #3b82f6);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--hh-accent, #3b82f6) 16%, transparent);
  z-index: 1;
}

.message-card {
  min-width: 0;
  padding: 0.875rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
  box-shadow: var(--hh-shadow-sm, 0 1px 2px rgb(0 0 0 / 0.08));
}

.message-header {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.sender-block {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.sender-block strong {
  font-size: 0.875rem;
  color: var(--hh-text-primary, #1f2937);
}

.sender-block span,
.message-time,
.message-footer {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
}

.message-actions {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  flex: 0 0 auto;
}

.message-body {
  margin: 0;
  white-space: pre-wrap;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.875rem;
  line-height: 1.5;
}

.message-body.collapsed {
  color: var(--hh-text-secondary, #6b7280);
}

.message-translation {
  display: grid;
  gap: 0.375rem;
  margin-top: 0.75rem;
  padding: 0.75rem;
  border: 1px solid color-mix(in srgb, var(--hh-accent, #3b82f6) 22%, var(--hh-border, #e5e7eb));
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-accent, #3b82f6) 7%, transparent);
}

.message-translation span {
  color: var(--hh-text-tertiary, #9ca3af);
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0;
}

.message-translation p {
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.875rem;
  line-height: 1.5;
  white-space: pre-wrap;
}

.message-quoted {
  margin: 0.75rem 0 0;
  padding: 0.75rem 0.875rem;
  border-left: 3px solid color-mix(in srgb, var(--hh-accent, #3b82f6) 36%, var(--hh-border, #e5e7eb));
  background: color-mix(in srgb, var(--hh-bg-secondary, #f3f4f6) 72%, transparent);
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.8125rem;
  line-height: 1.5;
  white-space: pre-wrap;
}

.message-attachments {
  display: grid;
  gap: 0.5rem;
  margin: 0.75rem 0 0;
  padding: 0;
  list-style: none;
}

.message-attachment {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 92%, transparent);
}

.message-attachment-icon {
  width: 0.9375rem;
  height: 0.9375rem;
  flex: 0 0 auto;
  color: var(--hh-text-secondary, #6b7280);
}

.message-attachment-copy {
  display: grid;
  gap: 0.125rem;
  min-width: 0;
  flex: 1 1 auto;
}

.message-attachment-copy strong,
.message-attachment-copy span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.message-attachment-copy strong {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.message-attachment-copy span {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.message-attachment-scan {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 5.5rem;
  min-height: 1.5rem;
  padding: 0 0.5rem;
  border-radius: 999px;
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0;
}

.message-attachment-scan.att-scan--clean {
  color: #047857;
  background: color-mix(in srgb, #10b981 14%, transparent);
}

.message-attachment-scan.att-scan--suspicious,
.message-attachment-scan.att-scan--unknown {
  color: #b45309;
  background: color-mix(in srgb, #f59e0b 16%, transparent);
}

.message-attachment-scan.att-scan--danger {
  color: #b91c1c;
  background: color-mix(in srgb, #ef4444 16%, transparent);
}

.message-footer {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  margin-top: 0.75rem;
}

.message-expand-link {
  border: 0;
  padding: 0;
  background: transparent;
  color: var(--hh-accent, #3b82f6);
  cursor: pointer;
  font: inherit;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
