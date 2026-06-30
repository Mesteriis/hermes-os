import { computed, ref } from 'vue'
import { useOrganizationsQuery } from './useOrganizationsQuery'
import type { Organization } from '../types/organization'

export function useOrganizationsPageSurface() {
  const organizationsQuery = useOrganizationsQuery()
  const selectedOrganizationId = ref('')

  const selectedOrganization = computed(() => {
    const organizations = organizationsQuery.data.value ?? []
    return organizations.find((organization: Organization) => organization.organization_id === selectedOrganizationId.value) ?? organizations[0] ?? null
  })

  const orgPeople = computed(() => [])

  function selectOrganization(organizationId: string) {
    selectedOrganizationId.value = organizationId
  }

  return {
    isOrganizationsLoading: organizationsQuery.isLoading,
    orgPeople,
    organizations: organizationsQuery.data,
    selectedOrganization,
    selectedOrganizationId,
    selectOrganization
  }
}
