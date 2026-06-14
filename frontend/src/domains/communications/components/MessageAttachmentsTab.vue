<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { MailMessageDetailResponse, CommunicationAttachment } from '../types/communications'
import { attachmentIcon } from '../stores/communications'

const props = defineProps<{
  detail: MailMessageDetailResponse | null
}>()

const attachments = props.detail?.attachments ?? []

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function scanStatusColor(status: string): string {
  switch (status) {
    case 'clean': return 'var(--hh-text-success, #16a34a)'
    case 'suspicious': return '#f59e0b'
    case 'malicious': return '#ef4444'
    case 'failed': return '#ef4444'
    default: return 'var(--hh-text-tertiary, #9ca3af)'
  }
}
</script>

<template>
  <div class="attachments-tab">
    <div v-if="attachments.length === 0" class="no-data">No attachments</div>
    <div v-for="att in attachments" :key="att.attachment_id" class="attachment-bubble">
      <Icon :icon="attachmentIcon(att.content_type)" class="att-icon" />
      <div class="att-info">
        <span class="att-filename">{{ att.filename || 'Unnamed' }}</span>
        <span class="att-meta">
          {{ formatSize(att.size_bytes) }} &middot; {{ att.content_type }}
        </span>
      </div>
      <span class="att-scan" :style="{ color: scanStatusColor(att.scan_status) }">
        {{ att.scan_status }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.attachments-tab {
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.attachment-bubble {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem;
  background: var(--hh-bg-secondary, #f9fafb);
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
}

.att-icon {
  width: 20px;
  height: 20px;
  color: var(--hh-accent, #3b82f6);
  flex-shrink: 0;
}

.att-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
}

.att-filename {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.att-meta {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.att-scan {
  font-size: 0.6875rem;
  font-weight: 500;
  white-space: nowrap;
  flex-shrink: 0;
}

.no-data {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
  padding: 2rem;
  text-align: center;
}
</style>
