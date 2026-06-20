<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { CommunicationMessageDetailResponse, MessageExportFormat } from '../types/communications'
import {
  mailMessageLabelsFromMetadata,
  mailMessageSnoozeUntilFromMetadata
} from '../helpers/mailPageModels'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
}>()

const emit = defineEmits<{
  togglePin: []
  toggleImportant: []
  mute: []
  replyAll: []
  forwardMessage: []
  redirectMessage: [recipientsText: string]
  exportMessage: [format: MessageExportFormat]
  addLabel: [label: string]
  removeLabel: [label: string]
  markMessageRead: []
  markMessageUnread: []
  deleteFromProvider: []
  snoozeMessage: [until: string]
}>()

const redirectRecipientsText = ref('')
const exportFormats: { format: MessageExportFormat; label: string }[] = [
  { format: 'md', label: 'Markdown' },
  { format: 'eml', label: 'EML' },
  { format: 'json', label: 'JSON' }
]

const quickLabels = ['Follow up', 'Finance', 'Legal']
const labels = computed(() =>
  props.detail ? mailMessageLabelsFromMetadata(props.detail.message.message_metadata) : []
)
const snoozeUntil = computed(() =>
  props.detail ? mailMessageSnoozeUntilFromMetadata(props.detail.message.message_metadata) : null
)

function snoozePreset(days: number): string {
  const date = new Date()
  date.setDate(date.getDate() + days)
  date.setHours(9, 0, 0, 0)
  return date.toISOString()
}
</script>

<template>
  <div class="related-tab">
    <div v-if="!detail" class="no-data">No message selected</div>
    <div v-else class="related-actions">
      <div class="actions-group">
        <h4 class="group-title">Read / Delete</h4>
        <Button variant="outline" size="sm" @click="emit('markMessageRead')">
          <Icon icon="tabler:mail-opened" /> Mark as read
        </Button>
        <Button variant="outline" size="sm" @click="emit('markMessageUnread')">
          <Icon icon="tabler:mail" /> Mark as unread
        </Button>
        <Button variant="outline" size="sm" @click="emit('deleteFromProvider')">
          <Icon icon="tabler:trash" /> Delete in provider
        </Button>
      </div>
      <div class="actions-group">
        <h4 class="group-title">Message Actions</h4>
        <Button variant="outline" size="sm" @click="emit('togglePin')">
          <Icon icon="tabler:pin" /> Pin
        </Button>
        <Button variant="outline" size="sm" @click="emit('toggleImportant')">
          <Icon icon="tabler:star" /> Important
        </Button>
        <Button variant="outline" size="sm" @click="emit('mute')">
          <Icon icon="tabler:bell-off" /> Mute
        </Button>
        <Button variant="outline" size="sm" @click="emit('replyAll')">
          <Icon icon="tabler:reply-all" /> Reply All
        </Button>
        <Button variant="outline" size="sm" @click="emit('forwardMessage')">
          <Icon icon="tabler:mail-forward" /> Forward
        </Button>
        <div class="export-format-group" aria-label="Export message">
          <Button
            v-for="item in exportFormats"
            :key="item.format"
            variant="outline"
            size="sm"
            @click="emit('exportMessage', item.format)"
          >
            <Icon icon="tabler:download" /> {{ item.label }}
          </Button>
        </div>
      </div>
      <div class="actions-group">
        <h4 class="group-title">Redirect</h4>
        <div class="redirect-row">
          <input
            v-model="redirectRecipientsText"
            class="redirect-input"
            type="text"
            placeholder="recipient@example.com, team@example.com"
          />
          <Button
            variant="outline"
            size="sm"
            :disabled="!redirectRecipientsText.trim()"
            @click="emit('redirectMessage', redirectRecipientsText)"
          >
            <Icon icon="tabler:send" /> Redirect
          </Button>
        </div>
      </div>
      <div class="actions-group">
        <h4 class="group-title">Labels</h4>
        <div v-if="labels.length" class="label-chip-row">
          <button
            v-for="label in labels"
            :key="label"
            class="label-chip"
            type="button"
            @click="emit('removeLabel', label)"
          >
            {{ label }}
            <Icon icon="tabler:x" />
          </button>
        </div>
        <div class="label-chip-row">
          <button
            v-for="label in quickLabels"
            :key="label"
            class="label-chip add"
            type="button"
            :disabled="labels.includes(label)"
            @click="emit('addLabel', label)"
          >
            <Icon icon="tabler:tag-plus" />
            {{ label }}
          </button>
        </div>
      </div>
      <div class="actions-group">
        <h4 class="group-title">Snooze</h4>
        <p v-if="snoozeUntil" class="snooze-status">
          Snoozed until {{ new Date(snoozeUntil).toLocaleString() }}
        </p>
        <div class="export-format-group" aria-label="Snooze message">
          <Button variant="outline" size="sm" @click="emit('snoozeMessage', snoozePreset(1))">
            <Icon icon="tabler:clock" /> Tomorrow
          </Button>
          <Button variant="outline" size="sm" @click="emit('snoozeMessage', snoozePreset(7))">
            <Icon icon="tabler:calendar-time" /> Next week
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.related-tab {
  padding: 0.75rem;
}

.related-actions {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.actions-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.export-format-group {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(6.25rem, 1fr));
  gap: 0.375rem;
}

.redirect-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.375rem;
}

.redirect-input {
  min-width: 0;
  min-height: 1.875rem;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 84%, transparent);
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
}

.label-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.label-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  min-height: 1.875rem;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  color: var(--hh-text-primary, #1f2937);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 84%, transparent);
  font-size: 0.75rem;
}

.label-chip.add {
  color: var(--hh-accent, #2563eb);
}

.label-chip:disabled {
  cursor: default;
  opacity: 0.45;
}

.label-chip svg {
  width: 14px;
  height: 14px;
}

.snooze-status {
  margin: 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.group-title {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--hh-text-secondary, #6b7280);
  margin: 0;
}

.no-data {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
  padding: 2rem;
  text-align: center;
}
</style>
