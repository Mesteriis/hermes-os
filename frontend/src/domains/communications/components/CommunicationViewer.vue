<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import Tabs from '../../../shared/ui/Tabs.vue'
import MessageBodyTab from './MessageBodyTab.vue'
import MessageHeadersTab from './MessageHeadersTab.vue'
import MessageAttachmentsTab from './MessageAttachmentsTab.vue'
import MessageRelatedTab from './MessageRelatedTab.vue'
import MessageTimelineTab from './MessageTimelineTab.vue'
import type {
  AiReplyResponse,
  CommunicationMessageDetailResponse,
  CommunicationMessageInsight,
  MessageContextTab,
  MessageExportFormat
} from '../types/communications'
import type {
  CommunicationAiState,
  CommunicationAiStateTransitionRequest
} from '../types/aiState'
import {
  useMessageAiStateQuery,
  useUpdateMessageAiStateMutation
} from '../queries/useCommunicationsQuery'
import { senderLabel, senderEmail, messageTime } from '../stores/communications'
import type { BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
  insight: CommunicationMessageInsight | null
  activeTab: MessageContextTab
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
}>()

const message = computed(() => props.detail?.message ?? null)
const sender = computed(() => message.value ? senderLabel(message.value.sender) : '')
const email = computed(() => message.value ? senderEmail(message.value.sender) : '')
const time = computed(() => message.value ? messageTime(message.value.projected_at ?? message.value.occurred_at) : '')
const messageId = computed(() => message.value?.message_id ?? null)
const {
  data: aiStateRecord,
  isFetching: isAiStateFetching
} = useMessageAiStateQuery(() => messageId.value)
const updateAiStateMutation = useUpdateMessageAiStateMutation()
const currentAiState = computed(() => aiStateRecord.value?.ai_state ?? 'NEW')
const aiStateDetail = computed(() => {
  if (aiStateRecord.value?.review_reason) return aiStateRecord.value.review_reason
  if (aiStateRecord.value?.last_error) return aiStateRecord.value.last_error
  if (isAiStateFetching.value) return 'Loading AI state...'
  return 'No review note'
})
const isAiStateUpdating = computed(() => updateAiStateMutation.isPending.value)

const tabs = [
  { id: 'message' as MessageContextTab, label: 'Message' },
  { id: 'attachments' as MessageContextTab, label: 'Attachments' },
  { id: 'headers' as MessageContextTab, label: 'Headers' },
  { id: 'related' as MessageContextTab, label: 'Related' },
  { id: 'timeline' as MessageContextTab, label: 'Timeline' }
]

function setTab(tabId: string) {
  emit('update:activeTab', tabId as MessageContextTab)
}

function transitionAiState(aiState: CommunicationAiState): void {
  const id = messageId.value
  if (!id || isAiStateUpdating.value) return

  let request: CommunicationAiStateTransitionRequest = { ai_state: aiState }
  if (aiState === 'REVIEW_REQUIRED') {
    request = { ai_state: aiState, review_reason: 'Manual review requested from Mail UI' }
  }
  if (aiState === 'FAILED') {
    request = { ai_state: aiState, last_error: 'Manual failure recorded from Mail UI' }
  }
  void updateAiStateMutation.mutateAsync({ messageId: id, request })
}
</script>

<template>
  <div class="mail-viewer">
    <!-- Empty state -->
    <div v-if="!detail" class="viewer-empty">
      <Icon icon="tabler:mail" class="empty-icon" />
      <p>Select a message to view</p>
    </div>

    <!-- Message detail -->
    <div v-else class="viewer-content">
      <!-- Header -->
      <div class="viewer-header">
        <div class="header-actions-top">
          <Button variant="ghost" size="sm" @click="emit('togglePin')">
            <Icon icon="tabler:pin" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('toggleImportant')">
            <Icon icon="tabler:star" />
          </Button>
        <Button variant="ghost" size="sm" @click="emit('mute')">
            <Icon icon="tabler:bell-off" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('markMessageRead')">
            <Icon icon="tabler:mail-opened" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('deleteFromProvider')">
            <Icon icon="tabler:trash" />
          </Button>
          <Button variant="ghost" size="sm" @click="emit('forwardMessage')">
            <Icon icon="tabler:mail-forward" />
          </Button>
        </div>
        <h2 class="viewer-subject">{{ message?.subject }}</h2>
        <div class="viewer-sender-row">
          <div class="sender-info">
            <span class="sender-name">{{ sender }}</span>
            <span class="sender-email">{{ email }}</span>
          </div>
          <span class="viewer-time">{{ time }}</span>
        </div>
        <section class="ai-state-panel" aria-label="AI state">
          <div>
            <span class="ai-state-kicker">AI state</span>
            <strong>{{ currentAiState }}</strong>
            <p>{{ aiStateDetail }}</p>
          </div>
          <div class="ai-state-actions">
            <button
              class="ai-state-action"
              type="button"
              :disabled="isAiStateUpdating"
              @click="transitionAiState('PROCESSING')"
            >
              Process
            </button>
            <button
              class="ai-state-action"
              type="button"
              :disabled="isAiStateUpdating"
              @click="transitionAiState('REVIEW_REQUIRED')"
            >
              Review
            </button>
            <button
              class="ai-state-action"
              type="button"
              :disabled="isAiStateUpdating"
              @click="transitionAiState('PROCESSED')"
            >
              Done
            </button>
            <button
              class="ai-state-action"
              type="button"
              :disabled="isAiStateUpdating"
              @click="transitionAiState('FAILED')"
            >
              Failed
            </button>
            <button
              class="ai-state-action"
              type="button"
              :disabled="isAiStateUpdating"
              @click="transitionAiState('ARCHIVED')"
            >
              Archive
            </button>
          </div>
        </section>
      </div>

      <!-- Tabs -->
      <Tabs :tabs="tabs.map(t => ({ id: t.id, label: t.label }))" :active="activeTab" @select="setTab" />

      <!-- Tab content -->
      <div class="viewer-body">
        <MessageBodyTab
          v-if="activeTab === 'message'"
          :detail="detail"
          :insight="insight"
          @reply="emit('reply')"
          @create-task="emit('createTask')"
          @create-note="emit('createNote')"
          @translate="emit('translate')"
          @generate-ai-reply="emit('generateAiReply', $event)"
          @apply-ai-reply="emit('applyAiReply', $event)"
          @review-security="emit('reviewSecurity')"
          @review-recipients="emit('reviewRecipients')"
          @analyze="emit('analyze')"
          @send-bilingual-reply="emit('sendBilingualReply', $event)"
        />
        <MessageAttachmentsTab v-else-if="activeTab === 'attachments'" :detail="detail" />
        <MessageHeadersTab v-else-if="activeTab === 'headers'" :detail="detail" />
        <MessageRelatedTab
          v-else-if="activeTab === 'related'"
          :detail="detail"
          @mark-message-read="emit('markMessageRead')"
          @mark-message-unread="emit('markMessageUnread')"
          @delete-from-provider="emit('deleteFromProvider')"
          @toggle-pin="emit('togglePin')"
          @toggle-important="emit('toggleImportant')"
          @mute="emit('mute')"
          @reply-all="emit('replyAll')"
          @forward-message="emit('forwardMessage')"
          @redirect-message="emit('redirectMessage', $event)"
          @export-message="emit('exportMessage', $event)"
          @add-label="emit('addLabel', $event)"
          @remove-label="emit('removeLabel', $event)"
          @snooze-message="emit('snoozeMessage', $event)"
        />
        <MessageTimelineTab v-else-if="activeTab === 'timeline'" :detail="detail" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.mail-viewer {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.viewer-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--hh-text-secondary, #6b7280);
  gap: 0.75rem;
}

.empty-icon {
  width: 48px;
  height: 48px;
  opacity: 0.3;
}

.viewer-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.viewer-header {
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.header-actions-top {
  display: flex;
  gap: 0.25rem;
  justify-content: flex-end;
}

.viewer-subject {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
  margin: 0;
  line-height: 1.3;
}

.viewer-sender-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.ai-state-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-top: 0.375rem;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 86%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.ai-state-kicker {
  display: block;
  color: var(--hh-text-tertiary, #9ca3af);
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0;
}

.ai-state-panel strong {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.ai-state-panel p {
  margin: 0.125rem 0 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.ai-state-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.375rem;
}

.ai-state-action {
  min-height: 1.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-accent, #3b82f6) 10%, transparent);
  color: var(--hh-accent, #3b82f6);
  cursor: pointer;
  font: inherit;
  font-size: 0.75rem;
  padding: 0 0.5rem;
  white-space: nowrap;
}

.ai-state-action:disabled {
  cursor: progress;
  opacity: 0.6;
}

.sender-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.sender-name {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
}

.sender-email {
  font-size: 0.75rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.viewer-time {
  font-size: 0.75rem;
  color: var(--hh-text-tertiary, #9ca3af);
  white-space: nowrap;
}

.viewer-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
}
</style>
