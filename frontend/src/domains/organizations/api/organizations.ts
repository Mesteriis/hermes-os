import { ApiClient } from '../../../platform/api/ApiClient'
import type { Organization } from '../types/organization'

export type OrganizationListResponse = { items: Organization[] }

export async function fetchOrganizations(limit = 50): Promise<OrganizationListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<OrganizationListResponse>(
    `/api/v1/organizations?${params.toString()}`,
    'Organizations request failed'
  )
}

export async function fetchOrganization(orgId: string): Promise<Organization> {
  return ApiClient.instance.get<Organization>(
    `/api/v1/organizations/${encodeURIComponent(orgId)}`,
    'Organization request failed'
  )
}
