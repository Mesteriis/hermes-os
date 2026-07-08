import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { fetchMaintenanceOverview, runMaintenanceAction } from '../api/maintenance'
import type { MaintenanceActionRequest } from '../types/maintenance'

export const maintenanceKeys = {
  all: ['settings', 'maintenance'] as const,
  overview: () => [...maintenanceKeys.all, 'overview'] as const
}

export function useMaintenanceOverviewQuery() {
  return useQuery({
    queryKey: maintenanceKeys.overview(),
    queryFn: fetchMaintenanceOverview
  })
}

export function useRunMaintenanceActionMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      actionId,
      request
    }: {
      actionId: string
      request: MaintenanceActionRequest
    }) => runMaintenanceAction(actionId, request),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: maintenanceKeys.overview() })
    }
  })
}
