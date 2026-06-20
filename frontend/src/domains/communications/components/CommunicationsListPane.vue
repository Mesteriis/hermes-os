<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import CommunicationsConversationList from './CommunicationsConversationList.vue'
import MailList from './MailList.vue'
import type { CommunicationMessageSummary, CommunicationThreadSummary, NavigatorMode } from '../types/communications'

defineProps<{
  accountId: string
  messages: CommunicationMessageSummary[]
  threads: CommunicationThreadSummary[]
  selectedIndex: number
  selectedThreadId: string
  selectedMessageIds: string[]
  navigatorMode: NavigatorMode
  isFolderMode: boolean
  isLoading: boolean
  hasNextPage: boolean
  isFetchingNextPage: boolean
  hasThreadNextPage: boolean
  isFetchingThreadNextPage: boolean
  errorMessage: string
}>()

const emit = defineEmits<{
  select: [index: number]
  selectThread: [thread: CommunicationThreadSummary]
  toggleSelection: [messageId: string, extendRange: boolean]
  selectVisible: [messageIds: string[]]
  clearSelection: []
  loadMore: []
  loadMoreThreads: []
  'update:navigatorMode': [mode: NavigatorMode]
}>()

function forwardToggleSelection(messageId: string, extendRange: boolean) {
  emit('toggleSelection', messageId, extendRange)
}
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
    <div v-else-if="!isFolderMode && (navigatorMode === 'threads' || navigatorMode === 'contacts')" class="pane-content">
      <CommunicationsConversationList
        :account-id="accountId"
        :messages="messages"
        :threads="threads"
        :selected-index="selectedIndex"
        :selected-thread-id="selectedThreadId"
        :navigator-mode="navigatorMode"
        :has-thread-next-page="hasThreadNextPage"
        :is-fetching-thread-next-page="isFetchingThreadNextPage"
        @select="emit('select', $event)"
        @select-thread="emit('selectThread', $event)"
        @load-more-threads="emit('loadMoreThreads')"
        @update:navigator-mode="emit('update:navigatorMode', $event)"
      />
    </div>
    <MailList
      v-else
      :messages="messages"
      :selected-index="selectedIndex"
      :selected-message-ids="selectedMessageIds"
      :is-loading="isLoading"
      :has-next-page="hasNextPage"
      :is-fetching-next-page="isFetchingNextPage"
      @select="emit('select', $event)"
      @toggle-selection="forwardToggleSelection"
      @select-visible="emit('selectVisible', $event)"
      @clear-selection="emit('clearSelection')"
      @load-more="emit('loadMore')"
    />
  </nav>
</template>

<style scoped>
.communications-list-pane {
  flex: 1;
  min-height: 0;
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
