import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useDocumentsStore = defineStore('documents-ui', () => {
  const searchQuery = ref('')
  const activeFilter = ref<string>('all')
  const documentsError = ref('')
  const retryingJobId = ref<string | null>(null)

  function setSearchQuery(q: string) {
    searchQuery.value = q
  }

  function setActiveFilter(filter: string) {
    activeFilter.value = filter
  }

  function setDocumentsError(err: string) {
    documentsError.value = err
  }

  function setRetryingJobId(id: string | null) {
    retryingJobId.value = id
  }

  return {
    searchQuery,
    activeFilter,
    documentsError,
    retryingJobId,
    setSearchQuery,
    setActiveFilter,
    setDocumentsError,
    setRetryingJobId
  }
})
