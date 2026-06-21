<script setup lang="ts">
import { ref } from 'vue'
import SavedSearchStrip from '../../../shared/communications/components/SavedSearchStrip.vue'
import type { CommunicationSavedSearch } from '../../../shared/communications/types/savedSearches'
import { useTelegramStore } from '../stores/telegram'

const props = defineProps<{
  accountId: string | null
  currentQuery: string
}>()

const store = useTelegramStore()
const activeSavedSearchId = ref('')

function selectSavedSearch(savedSearch: CommunicationSavedSearch) {
  activeSavedSearchId.value = savedSearch.saved_search_id
  store.telegramSearchQuery = savedSearch.query
}

function clearDeletedSavedSearch(savedSearch: CommunicationSavedSearch) {
  if (activeSavedSearchId.value !== savedSearch.saved_search_id) return
  activeSavedSearchId.value = ''
  store.telegramSearchQuery = ''
}
</script>

<template>
  <SavedSearchStrip
    :account-id="props.accountId"
    :active-id="activeSavedSearchId"
    :current-query="props.currentQuery"
    current-workflow-state=""
    current-local-state="active"
    current-channel-kind="telegram"
    @select="selectSavedSearch"
    @deleted="clearDeletedSavedSearch"
  />
</template>
