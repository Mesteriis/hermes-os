import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useNotesStore = defineStore('notes-ui', () => {
  const searchQuery = ref('')
  const activeSources = ref<string[]>([])
  const activeTags = ref<string[]>([])
  const notesError = ref('')

  function setSearchQuery(q: string) {
    searchQuery.value = q
  }

  function toggleSource(source: string) {
    const idx = activeSources.value.indexOf(source)
    if (idx >= 0) {
      activeSources.value.splice(idx, 1)
    } else {
      activeSources.value.push(source)
    }
  }

  function toggleTag(tag: string) {
    const idx = activeTags.value.indexOf(tag)
    if (idx >= 0) {
      activeTags.value.splice(idx, 1)
    } else {
      activeTags.value.push(tag)
    }
  }

  function setNotesError(err: string) {
    notesError.value = err
  }

  return {
    searchQuery,
    activeSources,
    activeTags,
    notesError,
    setSearchQuery,
    toggleSource,
    toggleTag,
    setNotesError
  }
})
