<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { BulkMessageAction, BulkMessageActionRequest } from '../types/communications'
import {
  MAIL_MESSAGE_DRAG_TYPE,
  hasCommunicationMessageDragType,
  parseCommunicationMessageDragPayload
} from './mailDragDrop'

type BulkActionCommand = Omit<BulkMessageActionRequest, 'message_ids'>

const props = defineProps<{
  selectedCount: number
  isRunning: boolean
}>()

const emit = defineEmits<{
  action: [command: BulkActionCommand]
  clear: []
}>()

const actions: { action: BulkMessageAction; label: string; icon: string }[] = [
  { action: 'mark_read', label: 'Read', icon: 'tabler:mail-opened' },
  { action: 'mark_unread', label: 'Unread', icon: 'tabler:mail' },
  { action: 'archive', label: 'Archive', icon: 'tabler:archive' },
  { action: 'trash', label: 'Trash', icon: 'tabler:trash' },
  { action: 'pin', label: 'Pin', icon: 'tabler:pin' },
  { action: 'unpin', label: 'Unpin', icon: 'tabler:pinned-off' },
  { action: 'important', label: 'Important', icon: 'tabler:flag' },
  { action: 'not_important', label: 'Normal', icon: 'tabler:flag-off' }
]

const metadataActions: { command: BulkActionCommand; label: string; icon: string }[] = [
  { command: { action: 'add_label', label: 'Follow up' }, label: 'Label', icon: 'tabler:tag' },
  { command: { action: 'remove_label', label: 'Follow up' }, label: 'Unlabel', icon: 'tabler:tag-off' }
]

function handleActionDrop(event: DragEvent, action: BulkMessageAction) {
  if (props.isRunning || !event.dataTransfer) return
  const payload = parseCommunicationMessageDragPayload(event.dataTransfer.getData(MAIL_MESSAGE_DRAG_TYPE))
  if (!payload) return
  emit('action', { action })
}

function nextBusinessMorningIso() {
  const nextMorning = new Date()
  nextMorning.setDate(nextMorning.getDate() + 1)
  nextMorning.setHours(9, 0, 0, 0)
  return nextMorning.toISOString()
}

function handleActionDragOver(event: DragEvent) {
  if (!event.dataTransfer || !hasCommunicationMessageDragType(event.dataTransfer.types)) return
  event.preventDefault()
  event.dataTransfer.dropEffect = 'move'
}
</script>

<template>
  <div class="bulk-actions-bar">
    <div class="bulk-count">{{ selectedCount }} selected</div>
    <div class="bulk-buttons">
      <button
        v-for="item in actions"
        :key="item.action"
        class="bulk-button"
        type="button"
        :disabled="isRunning"
        @dragover="handleActionDragOver"
        @drop.prevent="handleActionDrop($event, item.action)"
        @click="emit('action', { action: item.action })"
      >
        <Icon :icon="item.icon" class="bulk-icon" />
        <span>{{ item.label }}</span>
      </button>
      <button
        v-for="item in metadataActions"
        :key="`${item.command.action}-${item.command.label}`"
        class="bulk-button"
        type="button"
        :disabled="isRunning"
        @click="emit('action', item.command)"
      >
        <Icon :icon="item.icon" class="bulk-icon" />
        <span>{{ item.label }}</span>
      </button>
      <button
        class="bulk-button"
        type="button"
        :disabled="isRunning"
        @click="emit('action', { action: 'snooze', snooze_until: nextBusinessMorningIso() })"
      >
        <Icon icon="tabler:clock-pause" class="bulk-icon" />
        <span>Snooze</span>
      </button>
      <button class="bulk-button icon-only" type="button" :disabled="isRunning" title="Clear selection" @click="emit('clear')">
        <Icon icon="tabler:x" class="bulk-icon" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.bulk-actions-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 0.5rem;
  border-right: 1px solid var(--hh-border, #e5e7eb);
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 84%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.bulk-count {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  white-space: nowrap;
}

.bulk-buttons {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.bulk-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.25rem;
  min-height: 1.75rem;
  padding: 0 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 90%, transparent);
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
  cursor: pointer;
}

.bulk-button:hover:not(:disabled) {
  background: var(--hh-bg-hover, #f3f4f6);
}

.bulk-button:disabled {
  cursor: wait;
  opacity: 0.65;
}

.bulk-button.icon-only {
  width: 1.75rem;
  padding: 0;
}

.bulk-icon {
  width: 14px;
  height: 14px;
}
</style>
