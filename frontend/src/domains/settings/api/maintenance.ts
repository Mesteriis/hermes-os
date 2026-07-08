import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  MaintenanceActionRequest,
  MaintenanceActionResponse,
  MaintenanceOverview
} from '../types/maintenance'

export async function fetchMaintenanceOverview(): Promise<MaintenanceOverview> {
  return ApiClient.instance.get<MaintenanceOverview>(
    '/api/v1/maintenance/overview',
    'Maintenance overview request failed'
  )
}

export async function runMaintenanceAction(
  actionId: string,
  request: MaintenanceActionRequest
): Promise<MaintenanceActionResponse> {
  return ApiClient.instance.post<MaintenanceActionResponse>(
    `/api/v1/maintenance/actions/${encodeURIComponent(actionId)}`,
    request,
    'Maintenance action failed'
  )
}
