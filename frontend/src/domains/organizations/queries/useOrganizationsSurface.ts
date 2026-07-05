import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/organizations/queries/useOrganizationsPageSurface.ts'

export function useOrganizationsSurface() {
  return createDomainSurface({
    surfaceId: 'organizations',
    labelKey: 'Organizations',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'organizations-directory',
        labelKey: 'Organization directory',
        descriptionKey: 'Canonical organizations and linked provider identities.',
        icon: 'tabler:building',
        status: 'active',
        kind: 'query',
        contract: 'useOrganizationsPageSurface.organizations'
      },
      {
        id: 'organizations-relationships',
        labelKey: 'Organization relationships',
        descriptionKey: 'People, projects and evidence linked to an organization.',
        icon: 'tabler:building-community',
        status: 'active',
        kind: 'graph',
        contract: 'useOrganizationsPageSurface.selectedOrganization'
      },
      {
        id: 'organizations-review',
        labelKey: 'Organization review',
        descriptionKey: 'Candidate organization records promoted from observed communications.',
        icon: 'tabler:clipboard-check',
        status: 'active',
        kind: 'review',
        contract: 'useOrganizationsPageSurface.reviewItems'
      }
    ],
    childSurfaces: [
      {
        id: 'organizations-directory',
        labelKey: 'Directory',
        status: 'facade',
        surfacePath,
        capabilityIds: ['organizations-directory']
      },
      {
        id: 'organizations-context',
        labelKey: 'Context',
        status: 'facade',
        surfacePath,
        capabilityIds: ['organizations-relationships']
      },
      {
        id: 'organizations-review',
        labelKey: 'Review',
        status: 'facade',
        surfacePath,
        capabilityIds: ['organizations-review']
      }
    ]
  })
}

