<script setup lang="ts">
import SavedSearchStrip from '../../../domains/communications/components/SavedSearchStrip.vue'
import type { LocalMessageState, WorkflowState } from '../types/communications'
import type { CommunicationSavedSearch } from '../types/savedSearches'

const props = defineProps<{
  accountId: string | null
  activeId: string
  currentQuery: string
  currentWorkflowState: WorkflowState | ''
  currentLocalState: LocalMessageState
  currentChannelKind: string
}>()

const emit = defineEmits<{
  select: [savedSearch: CommunicationSavedSearch]
  deleted: [savedSearch: CommunicationSavedSearch]
}>()
</script>

<template>
  <SavedSearchStrip
    :account-id="props.accountId"
    :active-id="props.activeId"
    :current-query="props.currentQuery"
    :current-workflow-state="props.currentWorkflowState"
    :current-local-state="props.currentLocalState"
    :current-channel-kind="props.currentChannelKind"
    @select="emit('select', $event)"
    @deleted="emit('deleted', $event)"
  />
</template>
