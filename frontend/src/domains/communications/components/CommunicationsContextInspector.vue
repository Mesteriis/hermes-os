<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { MailMessageDetailResponse, InspectorMode } from '../types/communications'
import { senderLabel, senderEmail } from '../stores/communications'

const props = defineProps<{
  detail: MailMessageDetailResponse | null
  inspectorMode: InspectorMode
}>()

const emit = defineEmits<{
  'update:inspectorMode': [mode: InspectorMode]
}>()

const modes = [
  { id: 'context' as InspectorMode, icon: 'tabler:bulb', label: 'Context' },
  { id: 'contact' as InspectorMode, icon: 'tabler:user', label: 'Contact' },
  { id: 'organization' as InspectorMode, icon: 'tabler:building-community', label: 'Organization' }
]

const message = computed(() => props.detail?.message ?? null)
const sender = computed(() => message.value ? senderLabel(message.value.sender) : '')
const email = computed(() => message.value ? senderEmail(message.value.sender) : '')
</script>

<template>
  <div class="context-inspector">
    <div class="inspector-header">
      <Icon icon="tabler:bulb" class="inspector-icon" />
      <span class="inspector-title">Inspector</span>
    </div>

    <!-- Mode selector -->
    <div class="mode-selector">
      <Button
        v-for="(m, idx) in modes"
        :key="idx"
        variant="ghost"
        size="sm"
        :class="inspectorMode === m.id ? 'active' : ''"
        @click="emit('update:inspectorMode', m.id)"
      >
        <Icon :icon="m.icon" /> {{ m.label }}
      </Button>
    </div>

    <!-- Context panel -->
    <div v-if="!detail" class="inspector-empty">
      Select a message to inspect
    </div>
    <div v-else class="inspector-content">
      <div class="sender-profile">
        <div class="profile-avatar">{{ sender.charAt(0).toUpperCase() }}</div>
        <div class="profile-info">
          <span class="profile-name">{{ sender }}</span>
          <span class="profile-email">{{ email }}</span>
        </div>
      </div>

      <div class="inspector-section">
        <h4 class="section-title">Summary</h4>
        <p class="section-text">
          {{ message?.ai_summary || 'No AI summary available' }}
        </p>
      </div>

      <div class="inspector-section">
        <h4 class="section-title">Metadata</h4>
        <div class="meta-grid">
          <div class="meta-item">
            <span class="meta-label">Importance</span>
            <span class="meta-value">{{ message?.importance_score ?? 'N/A' }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">Category</span>
            <span class="meta-value">{{ message?.ai_category || 'N/A' }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">State</span>
            <span class="meta-value">{{ message?.workflow_state }}</span>
          </div>
        </div>
      </div>

      <div class="inspector-section">
        <h4 class="section-title">Attachments</h4>
        <p class="section-text">{{ detail?.attachments?.length ?? 0 }} files</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.context-inspector {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.inspector-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.inspector-icon {
  width: 18px;
  height: 18px;
  color: var(--hh-accent, #3b82f6);
}

.inspector-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.mode-selector {
  display: flex;
  gap: 0.25rem;
  padding: 0.375rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}
.mode-selector :deep(.active) {
  background: var(--hh-bg-selected, #eff6ff);
  color: var(--hh-accent, #3b82f6);
}

.inspector-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  font-size: 0.8125rem;
  color: var(--hh-text-secondary, #6b7280);
}

.inspector-content {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.sender-profile {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.profile-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--hh-accent, #3b82f6);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 1rem;
  flex-shrink: 0;
}

.profile-info {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
}

.profile-name {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.profile-email {
  font-size: 0.75rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.inspector-section {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.section-title {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--hh-text-secondary, #6b7280);
  margin: 0;
}

.section-text {
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  margin: 0;
  line-height: 1.4;
}

.meta-grid {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.meta-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.75rem;
}

.meta-label {
  color: var(--hh-text-secondary, #6b7280);
}

.meta-value {
  color: var(--hh-text-primary, #1f2937);
  font-weight: 500;
}
</style>
