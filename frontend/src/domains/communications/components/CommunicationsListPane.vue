<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import CommunicationsConversationList from './CommunicationsConversationList.vue'
import MailList from './MailList.vue'
import type { CommunicationMessageSummary, NavigatorMode } from '../types/communications'

defineProps<{
  messages: CommunicationMessageSummary[]
  selectedIndex: number
  navigatorMode: NavigatorMode
  isLoading: boolean
  errorMessage: string
}>()

const emit = defineEmits<{
  select: [index: number]
  'update:navigatorMode': [mode: NavigatorMode]
}>()
</script>

<template>
  <nav class="communications-list-pane">
    <div v-if="errorMessage" class="pane-state error">
      <Icon icon="tabler:alert-circle" />
      <span>{{ errorMessage }}</span>
    </div>
    <div v-else-if="isLoading" class="pane-state">
      <Icon icon="tabler:loader-2" class="spin-icon" />
      <span>Loading messages...</span>
    </div>
    <div v-else-if="navigatorMode === 'threads' || navigatorMode === 'contacts'" class="pane-content">
      <CommunicationsConversationList
        :messages="messages"
        :selected-index="selectedIndex"
        :navigator-mode="navigatorMode"
        @select="emit('select', $event)"
        @update:navigator-mode="emit('update:navigatorMode', $event)"
      />
    </div>
    <MailList
      v-else
      :messages="messages"
      :selected-index="selectedIndex"
      :is-loading="isLoading"
      @select="emit('select', $event)"
    />
  </nav>
</template>

<style scoped>
.communications-list-pane {
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--hh-border, #e5e7eb);
  background: var(--hh-bg-primary, #ffffff);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.pane-content {
  flex: 1;
  min-height: 0;
}

.pane-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  height: 100%;
  padding: 2rem;
  font-size: 0.875rem;
  color: var(--hh-text-secondary, #6b7280);
  text-align: center;
}

.pane-state.error {
  color: var(--hh-text-error, #ef4444);
}

.spin-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
