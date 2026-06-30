import { watch } from 'vue'
import { useAiWorkspaceQuery } from './useAgentsQuery'
import { useAgentsStore } from '../stores/agents'

export function useAgentsPageSurface() {
  const store = useAgentsStore()
  const workspaceQuery = useAiWorkspaceQuery()

  watch(workspaceQuery.data, (workspace) => {
    if (!workspace) return
    store.setWorkspace(workspace)
    store.setLoading(false)
  })

  watch(workspaceQuery.isLoading, (isLoading) => {
    store.setLoading(isLoading)
  })

  return {
    store,
    refetchWorkspace: workspaceQuery.refetch,
    workspaceData: workspaceQuery.data,
    isWorkspaceLoading: workspaceQuery.isLoading
  }
}
