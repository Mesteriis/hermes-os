<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationMessageSummary } from '../types/communications'
import { messageTime, communicationChannelIcon, senderLabel, conversationPreview } from '../stores/communications'
import { MAIL_MESSAGE_DRAG_TYPE, createCommunicationMessageDragPayload } from './mailDragDrop'

const props = defineProps<{
  message: CommunicationMessageSummary
  isSelected: boolean
  isChecked: boolean
  selectedMessageIds: string[]
}>()

const emit = defineEmits<{
  select: []
  toggleSelection: [extendRange: boolean]
  prefetch: []
}>()

const timeLabel = computed(() => messageTime(props.message.projected_at ?? props.message.occurred_at))
const channelIcon = computed(() => communicationChannelIcon(props.message.channel_kind))
const sender = computed(() => senderLabel(props.message.sender))
const preview = computed(() => conversationPreview(props.message))
const isUnread = computed(() => props.message.workflow_state === 'new')
const isImportant = computed(() => (props.message.importance_score ?? 0) >= 7)

function handleDragStart(event: DragEvent) {
  if (!props.isChecked || !event.dataTransfer) {
    event.preventDefault()
    return
  }
  event.dataTransfer.effectAllowed = 'move'
  event.dataTransfer.setData(MAIL_MESSAGE_DRAG_TYPE, createCommunicationMessageDragPayload(props.message.message_id, props.selectedMessageIds))
  event.dataTransfer.setData('text/plain', props.message.subject)
}
</script>

<template>
  <div
    class="mail-list-item-shell"
    :class="{ selected: isSelected, checked: isChecked, unread: isUnread }"
    role="option"
    :aria-selected="isChecked || isSelected"
    :draggable="isChecked"
    :title="isChecked ? 'Drag selected message to an action' : undefined"
    @mouseenter="emit('prefetch')"
    @dragstart="handleDragStart"
  >
    <button
      class="selection-toggle"
      type="button"
      :aria-pressed="isChecked"
      :title="isChecked ? 'Deselect message' : 'Select message'"
      @click.stop="emit('toggleSelection', $event.shiftKey)"
    >
      <Icon :icon="isChecked ? 'tabler:checkbox' : 'tabler:square'" class="selection-icon" />
    </button>
    <button class="mail-list-item" type="button" @focus="emit('prefetch')" @click="emit('select')">
      <div class="item-header">
        <div class="sender-row">
          <Icon :icon="channelIcon" class="channel-icon" />
          <span class="sender-name" :class="{ 'font-semibold': isUnread }">{{ sender }}</span>
          <span class="time-label">{{ timeLabel }}</span>
        </div>
        <div class="subject-row">
          <span v-if="isImportant" class="important-badge" title="Important">!</span>
          <span class="subject" :class="{ 'font-semibold': isUnread }">{{ message.subject }}</span>
        </div>
      </div>
      <div class="item-preview">{{ preview }}</div>
      <div v-if="message.attachment_count > 0" class="attachment-indicator">
        <Icon icon="tabler:paperclip" class="clip-icon" />
        <span>{{ message.attachment_count }}</span>
      </div>
    </button>
  </div>
</template>

<style scoped>
.mail-list-item-shell {
  display: flex;
  width: 100%;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: transparent;
  transition: background-color 0.1s;
}

.mail-list-item-shell:hover {
  background-color: var(--hh-bg-hover, #f3f4f6);
}

.mail-list-item-shell.selected {
  background-color: var(--hh-bg-selected, #eff6ff);
  border-left: 3px solid var(--hh-accent, #3b82f6);
}

.mail-list-item-shell.checked {
  background-color: color-mix(in srgb, var(--hh-accent, #3b82f6) 8%, transparent);
}

.mail-list-item-shell.unread {
  background-color: var(--hh-bg-unread, #fafafa);
}

.selection-toggle {
  display: flex;
  align-items: flex-start;
  justify-content: center;
  width: 2rem;
  padding: 0.75rem 0 0 0;
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--hh-text-tertiary, #9ca3af);
  flex-shrink: 0;
}

.selection-toggle:hover {
  color: var(--hh-accent, #3b82f6);
}

.selection-icon {
  width: 16px;
  height: 16px;
}

.mail-list-item {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  padding: 0.625rem 0.75rem 0.625rem 0;
  border: none;
  background: transparent;
  text-align: left;
  gap: 0.25rem;
  cursor: pointer;
}

.item-header {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.sender-row {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.channel-icon {
  width: 14px;
  height: 14px;
  color: var(--hh-text-tertiary, #9ca3af);
  flex-shrink: 0;
}

.sender-name {
  flex: 1;
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.time-label {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
  white-space: nowrap;
  flex-shrink: 0;
}

.subject-row {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.important-badge {
  color: #ef4444;
  font-weight: 700;
  font-size: 0.8125rem;
  line-height: 1;
}

.subject {
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-preview {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.3;
}

.attachment-indicator {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.clip-icon {
  width: 12px;
  height: 12px;
}
</style>
