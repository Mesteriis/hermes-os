<script setup lang="ts">
import { ref } from 'vue'
import SavedSearchStrip from '../../../domains/communications/components/SavedSearchStrip.vue'
import type { MailSavedSearch } from '../../../domains/communications/types/savedSearches'
import { useTelegramStore } from '../stores/telegram'

const props = defineProps<{
  accountId: string | null
  currentQuery: string
}>()

const store = useTelegramStore()
const activeSavedSearchId = ref('')

function selectSavedSearch(savedSearch: MailSavedSearch) {
  activeSavedSearchId.value = savedSearch.saved_search_id
  store.telegramSearchQuery = savedSearch.query
}

function clearDeletedSavedSearch(savedSearch: MailSavedSearch) {
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
