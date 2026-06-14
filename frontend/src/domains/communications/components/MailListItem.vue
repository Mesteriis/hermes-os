<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationMessageSummary } from '../types/communications'
import { messageTime, communicationChannelIcon, senderLabel, conversationPreview } from '../stores/communications'

const props = defineProps<{
  message: CommunicationMessageSummary
  isSelected: boolean
}>()

const emit = defineEmits<{
  select: []
}>()

const timeLabel = computed(() => messageTime(props.message.projected_at ?? props.message.occurred_at))
const channelIcon = computed(() => communicationChannelIcon(props.message.channel_kind))
const sender = computed(() => senderLabel(props.message.sender))
const preview = computed(() => conversationPreview(props.message))
const isUnread = computed(() => props.message.workflow_state === 'new')
const isImportant = computed(() => (props.message.importance_score ?? 0) >= 7)
</script>

<template>
  <button
    class="mail-list-item"
    :class="{ selected: isSelected, unread: isUnread }"
    @click="emit('select')"
  >
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
</template>

<style scoped>
.mail-list-item {
  display: flex;
  flex-direction: column;
  width: 100%;
  padding: 0.625rem 0.75rem;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  transition: background-color 0.1s;
  gap: 0.25rem;
}
.mail-list-item:hover {
  background-color: var(--hh-bg-hover, #f3f4f6);
}
.mail-list-item.selected {
  background-color: var(--hh-bg-selected, #eff6ff);
  border-left: 3px solid var(--hh-accent, #3b82f6);
}
.mail-list-item.unread {
  background-color: var(--hh-bg-unread, #fafafa);
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
