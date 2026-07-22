import { useQuery } from '@tanstack/vue-query'
import { fetchOrganizations, fetchOrganization } from '../api/organizations'
import type { Organization } from '../types/organization'

export function useOrganizationsQuery() {
  return useQuery<Organization[]>({
    queryKey: ['organizations', 'list'],
    queryFn: async () => {
      const res = await fetchOrganizations(50)
      return res.items
    }
  })
}

export function useOrganizationQuery(orgId: string) {
  return useQuery<Organization>({
    queryKey: ['organizations', orgId],
    queryFn: () => fetchOrganization(orgId),
    enabled: !!orgId
  })
}
