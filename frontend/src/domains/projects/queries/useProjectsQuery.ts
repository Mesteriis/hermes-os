import { useQuery } from '@tanstack/vue-query'
import { fetchProjects, fetchProjectDetail } from '../api/projects'
import type { ProjectSummary, ProjectDetail } from '../types/project'

export function useProjectsQuery() {
  return useQuery<ProjectSummary[]>({
    queryKey: ['projects'],
    queryFn: async () => {
      const response = await fetchProjects(25)
      return response.items
    }
  })
}

export function useProjectQuery(projectId: string | null) {
  return useQuery<ProjectDetail | null>({
    queryKey: ['project', projectId],
    queryFn: async () => {
      if (!projectId) return null
      const detail = await fetchProjectDetail(projectId)
      return detail
    },
    enabled: !!projectId
  })
}
