<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationMessageDetailResponse, ProjectItem, TaskItem } from '../types/communications'
import { senderLabel, senderEmail } from '../stores/communications'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
  projects: ProjectItem[]
  tasks: TaskItem[]
}>()

const message = props.detail?.message ?? null
const sender = message ? senderLabel(message.sender) : ''
const email = message ? senderEmail(message.sender) : ''
</script>

<template>
  <div class="context-rail">
    <div v-if="!detail" class="rail-empty">
      <Icon icon="tabler:sidebar" class="rail-icon" />
      <span>Select a message</span>
    </div>
    <div v-else class="rail-content">
      <!-- Sender profile -->
      <div class="rail-section">
        <h4 class="rail-section-title">Sender</h4>
        <div class="sender-card">
          <div class="sender-avatar">{{ sender.charAt(0).toUpperCase() }}</div>
          <div class="sender-details">
            <span class="sender-name">{{ sender }}</span>
            <span class="sender-email-text">{{ email }}</span>
          </div>
        </div>
      </div>

      <!-- Summary -->
      <div class="rail-section">
        <h4 class="rail-section-title">Summary</h4>
        <p class="rail-text">
          {{ message?.ai_summary || message?.body_text?.slice(0, 200) || 'No summary' }}
        </p>
      </div>

      <!-- Projects -->
      <div class="rail-section">
        <h4 class="rail-section-title">Related Projects</h4>
        <div v-if="projects.length === 0" class="rail-empty-small">No related projects</div>
        <div v-for="p in projects" :key="p.project_id" class="rail-item">
          <Icon icon="tabler:briefcase" class="rail-item-icon" />
          <span>{{ p.name }}</span>
        </div>
      </div>

      <!-- Tasks -->
      <div class="rail-section">
        <h4 class="rail-section-title">Related Tasks</h4>
        <div v-if="tasks.length === 0" class="rail-empty-small">No related tasks</div>
        <div v-for="t in tasks" :key="t.task_id" class="rail-item">
          <Icon icon="tabler:checkbox" class="rail-item-icon" />
          <span>{{ t.title }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.context-rail {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.rail-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.8125rem;
}

.rail-icon {
  width: 32px;
  height: 32px;
  opacity: 0.3;
}

.rail-content {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.rail-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.rail-section-title {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--hh-text-secondary, #6b7280);
  margin: 0;
}

.sender-card {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.sender-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--hh-accent, #3b82f6);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 0.875rem;
  flex-shrink: 0;
}

.sender-details {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
}

.sender-name {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.sender-email-text {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.rail-text {
  font-size: 0.75rem;
  color: var(--hh-text-primary, #1f2937);
  margin: 0;
  line-height: 1.4;
}

.rail-empty-small {
  font-size: 0.75rem;
  color: var(--hh-text-tertiary, #9ca3af);
  font-style: italic;
}

.rail-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: var(--hh-text-primary, #1f2937);
}

.rail-item-icon {
  width: 14px;
  height: 14px;
  color: var(--hh-text-tertiary, #9ca3af);
  flex-shrink: 0;
}
</style>
