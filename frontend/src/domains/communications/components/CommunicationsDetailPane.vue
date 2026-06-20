<script setup lang="ts">
import MailViewer from './MailViewer.vue'
import ThreadConversationView from './ThreadConversationView.vue'
import type {
  AiReplyResponse,
  CommunicationMessageDetailResponse,
  CommunicationMessageInsight,
  CommunicationThreadSummary,
  MessageContextTab,
  MessageExportFormat,
  ThreadMessage
} from '../types/communications'
import type { BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'

defineProps<{
  detail: CommunicationMessageDetailResponse | null
  insight: CommunicationMessageInsight | null
  activeTab: MessageContextTab
  selectedThread: CommunicationThreadSummary | null
  threadMessages: ThreadMessage[]
  isThreadLoading: boolean
  threadErrorMessage: string
  isThreadReplySending: boolean
}>()

const emit = defineEmits<{
  'update:activeTab': [tab: MessageContextTab]
  reply: []
  replyAll: []
  forwardMessage: []
  redirectMessage: [recipientsText: string]
  createTask: []
  createNote: []
  translate: []
  generateAiReply: [payload: { tone: string; language: string }]
  applyAiReply: [payload: AiReplyResponse]
  reviewSecurity: []
  reviewRecipients: []
  analyze: []
  markMessageRead: []
  markMessageUnread: []
  deleteFromProvider: []
  togglePin: []
  toggleImportant: []
  mute: []
  exportMessage: [format: MessageExportFormat]
  addLabel: [label: string]
  removeLabel: [label: string]
  snoozeMessage: [until: string]
  openCompose: []
  sendBilingualReply: [payload: BilingualReplyFlowResponse]
  openThreadMessage: [messageId: string]
  replyToThreadMessage: [message: ThreadMessage, bodyHtml: string, draftId: string]
  saveThreadReplyDraft: [message: ThreadMessage, bodyHtml: string, draftId: string]
  sendThreadReply: [message: ThreadMessage, bodyHtml: string, draftId: string]
}>()
</script>

<template>
  <main class="communications-detail-pane">
    <ThreadConversationView
      v-if="selectedThread"
      :thread="selectedThread"
      :messages="threadMessages"
      :is-loading="isThreadLoading"
      :error-message="threadErrorMessage"
      :is-sending-reply="isThreadReplySending"
      @open-message="emit('openThreadMessage', $event)"
      @reply-to-message="(message, bodyHtml, draftId) => emit('replyToThreadMessage', message, bodyHtml, draftId)"
      @save-reply-draft="(message, bodyHtml, draftId) => emit('saveThreadReplyDraft', message, bodyHtml, draftId)"
      @send-reply="(message, bodyHtml, draftId) => emit('sendThreadReply', message, bodyHtml, draftId)"
    />
    <MailViewer
      v-else
      :detail="detail"
      :insight="insight"
      :active-tab="activeTab"
      @update:active-tab="emit('update:activeTab', $event)"
      @reply="emit('reply')"
      @reply-all="emit('replyAll')"
      @forward-message="emit('forwardMessage')"
      @redirect-message="emit('redirectMessage', $event)"
      @create-task="emit('createTask')"
      @create-note="emit('createNote')"
      @translate="emit('translate')"
      @generate-ai-reply="emit('generateAiReply', $event)"
      @apply-ai-reply="emit('applyAiReply', $event)"
      @review-security="emit('reviewSecurity')"
      @review-recipients="emit('reviewRecipients')"
      @analyze="emit('analyze')"
      @mark-message-read="emit('markMessageRead')"
      @mark-message-unread="emit('markMessageUnread')"
      @delete-from-provider="emit('deleteFromProvider')"
      @toggle-pin="emit('togglePin')"
      @toggle-important="emit('toggleImportant')"
      @mute="emit('mute')"
      @export-message="emit('exportMessage', $event)"
      @add-label="emit('addLabel', $event)"
      @remove-label="emit('removeLabel', $event)"
      @snooze-message="emit('snoozeMessage', $event)"
      @open-compose="emit('openCompose')"
      @send-bilingual-reply="emit('sendBilingualReply', $event)"
    />
  </main>
</template>

<style scoped>
.communications-detail-pane {
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--hh-bg-primary, #ffffff);
  backdrop-filter: blur(var(--hh-panel-blur));
}
</style>
