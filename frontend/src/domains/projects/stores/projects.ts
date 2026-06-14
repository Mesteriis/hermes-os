import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useProjectsStore = defineStore('projects-ui', () => {
  const selectedProjectId = ref<string>('')
  const projectsError = ref<string>('')
  const isProjectsLoading = ref<boolean>(false)

  function selectProject(projectId: string) {
    selectedProjectId.value = projectId
  }

  function setError(msg: string) {
    projectsError.value = msg
  }

  function clearError() {
    projectsError.value = ''
  }

  function setLoading(val: boolean) {
    isProjectsLoading.value = val
  }

  return {
    selectedProjectId,
    projectsError,
    isProjectsLoading,
    selectProject,
    setError,
    clearError,
    setLoading
  }
})

export function projectStatusLabel(status: string): string {
  return status
    .split('_')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

export function projectTimelineIcon(itemKind: string): string {
  switch (itemKind) {
    case 'message':
      return 'tabler:mail'
    case 'document':
      return 'tabler:file-text'
    default:
      return 'tabler:circle-dot'
  }
}

export function projectDocumentIcon(documentKind: string): string {
  switch (documentKind) {
    case 'pdf':
      return 'tabler:file-type-pdf'
    case 'markdown':
      return 'tabler:file-text'
    default:
      return 'tabler:file'
  }
}

export function formatProjectDate(value: string | null): string {
  if (!value) return 'Not set'
  const date = new Date(`${value}T00:00:00`)
  if (Number.isNaN(date.getTime())) return 'Invalid date'
  return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', year: 'numeric' }).format(date)
}

export function formatProjectDateTime(value: string | null): string {
  if (!value) return 'No activity'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return 'Invalid date'
  return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date)
}
