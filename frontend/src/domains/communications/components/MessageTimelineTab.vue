<script setup lang="ts">
import type { CommunicationMessageDetailResponse } from '../types/communications'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
}>()

interface TimelineEntry {
  label: string
  time: string | null
}

const entries = props.detail?.message
  ? [
      { label: 'Received', time: props.detail.message.occurred_at ?? props.detail.message.projected_at },
      { label: 'Projected', time: props.detail.message.projected_at },
      { label: 'State changed', time: props.detail.message.local_state_changed_at },
      { label: 'AI analyzed', time: props.detail.message.ai_summary_generated_at }
    ].filter(e => e.time != null)
  : []
</script>

<template>
  <div class="timeline-tab">
    <div v-if="entries.length === 0" class="no-data">No timeline data</div>
    <div v-else class="timeline-list">
      <div v-for="(entry, i) in entries" :key="i" class="timeline-entry">
        <div class="timeline-dot" />
        <div class="timeline-content">
          <span class="timeline-label">{{ entry.label }}</span>
          <span class="timeline-time">{{ entry.time }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.timeline-tab {
  padding: 0.75rem;
}

.timeline-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  position: relative;
}

.timeline-entry {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.timeline-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--hh-accent, #3b82f6);
  margin-top: 0.375rem;
  flex-shrink: 0;
}

.timeline-content {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.timeline-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
}

.timeline-time {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.no-data {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
  padding: 2rem;
  text-align: center;
}
</style>
