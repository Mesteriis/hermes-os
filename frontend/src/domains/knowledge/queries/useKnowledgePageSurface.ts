import { computed, ref, watch } from 'vue'
import { useKnowledgeStore } from '../stores/knowledge'
import { useContradictionsQuery, useGraphSummaryQuery } from './useKnowledgeQuery'
import type { GraphNode } from '../types/knowledge'

export function useKnowledgePageSurface() {
  const store = useKnowledgeStore()

  const { data: summaryData, error: summaryError, isLoading: summaryLoading } = useGraphSummaryQuery()
  const {
    data: contradictionsData,
    error: contradictionsError,
    isLoading: contradictionsLoading
  } = useContradictionsQuery(50)

  watch(summaryData, (value) => {
    if (value) {
      store.setGraphSummary(value, '')
    }
  }, { immediate: true })

  watch(summaryError, (error) => {
    if (error) {
      store.setGraphSummary(null, (error as Error)?.message || 'Unknown error')
    }
  })

  watch(contradictionsData, (value) => {
    if (value) {
      store.setContradictionObservations(value)
    }
  })

  const searchQuery = ref('')
  const searchLoading = ref(false)

  async function handleSearch() {
    if (!searchQuery.value.trim()) return
    searchLoading.value = true
    try {
      await store.runGraphSearch(searchQuery.value)
    } finally {
      searchLoading.value = false
    }
  }

  async function handleSelectSearchResult(node: GraphNode) {
    searchQuery.value = ''
    store.setGraphSearchResults([], '')
    await store.selectGraphNode(node)
  }

  const suggestedContradictionsCount = computed(() =>
    store.contradictionObservations.filter((observation) => observation.review_state === 'suggested').length
  )

  async function loadGraphNodeChoices() {
    await store.loadGraphNodeChoices()
  }

  return {
    contradictionsError,
    contradictionsLoading,
    handleSearch,
    handleSelectSearchResult,
    loadGraphNodeChoices,
    searchLoading,
    searchQuery,
    store,
    suggestedContradictionsCount,
    summaryData,
    summaryError,
    summaryLoading
  }
}
