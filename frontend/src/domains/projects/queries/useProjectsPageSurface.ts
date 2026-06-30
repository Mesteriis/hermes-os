import { computed } from 'vue'
import { useProjectQuery, useProjectsQuery } from './useProjectsQuery'
import { useProjectsStore } from '../stores/projects'
import type { ProjectDetail, ProjectStats, ProjectSummary } from '../types/project'

const emptyProjectStats: ProjectStats = {
  message_count: 0,
  document_count: 0,
  people_count: 0,
  graph_connection_count: 0,
  latest_activity_at: null
}

export function useProjectsPageSurface() {
  const store = useProjectsStore()
  const projectsQuery = useProjectsQuery()
  const projectDetailQuery = useProjectQuery(store.selectedProjectId || null)

  const projectsError = computed(() => {
    const error = projectsQuery.error.value
    return error ? error.message : ''
  })

  const projectSummaries = computed<ProjectSummary[]>(() => projectsQuery.data.value ?? [])
  const selectedProjectDetail = computed<ProjectDetail | null>(() => projectDetailQuery.data.value ?? null)
  const selectedProjectRecord = computed(() => selectedProjectDetail.value?.project ?? projectSummaries.value[0]?.project ?? null)
  const selectedProjectStats = computed(() => selectedProjectDetail.value?.stats ?? projectSummaries.value[0]?.stats ?? emptyProjectStats)
  const relatedProjectSummaries = computed<ProjectSummary[]>(() => {
    const currentProjectId = selectedProjectRecord.value?.project_id
    return projectSummaries.value.filter((item) => item.project.project_id !== currentProjectId)
  })

  function selectProject(item: ProjectSummary) {
    if (item.project.project_id === store.selectedProjectId && selectedProjectDetail.value) return
    store.selectProject(item.project.project_id)
  }

  function loadProjects() {
    void projectsQuery.refetch()
  }

  function formatNumber(value: number): string {
    return new Intl.NumberFormat('en-US').format(value)
  }

  return {
    formatNumber,
    isDetailLoading: projectDetailQuery.isLoading,
    isProjectsLoading: projectsQuery.isLoading,
    loadProjects,
    projectSummaries,
    projectsError,
    relatedProjectSummaries,
    selectProject,
    selectedProjectDetail,
    selectedProjectRecord,
    selectedProjectStats,
    store
  }
}
