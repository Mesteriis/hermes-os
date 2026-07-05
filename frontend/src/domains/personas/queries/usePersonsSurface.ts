import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/personas/queries/usePersonsPageSurface.ts'

export function usePersonsSurface() {
  return createDomainSurface({
    surfaceId: 'persons',
    labelKey: 'Persons',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'persons-directory',
        labelKey: 'Person directory',
        descriptionKey: 'Canonical people records and identity review state.',
        icon: 'tabler:user',
        status: 'active',
        kind: 'query',
        contract: 'usePersonsPageSurface.people'
      },
      {
        id: 'persons-identity-review',
        labelKey: 'Identity review',
        descriptionKey: 'Evidence-backed merge, alias and candidate decisions for people.',
        icon: 'tabler:user-check',
        status: 'active',
        kind: 'review',
        contract: 'usePersonsPageSurface.identityReviewItems'
      },
      {
        id: 'persons-relationships',
        labelKey: 'Relationships',
        descriptionKey: 'Relationship candidates and confirmed links around a person.',
        icon: 'tabler:hierarchy-2',
        status: 'active',
        kind: 'graph',
        contract: 'usePersonsPageSurface.relationshipReviewItems'
      }
    ],
    childSurfaces: [
      {
        id: 'persons-directory',
        labelKey: 'Directory',
        status: 'facade',
        surfacePath,
        capabilityIds: ['persons-directory']
      },
      {
        id: 'persons-review',
        labelKey: 'Review',
        status: 'facade',
        surfacePath,
        capabilityIds: ['persons-identity-review']
      },
      {
        id: 'persons-graph',
        labelKey: 'Relationships',
        status: 'facade',
        surfacePath,
        capabilityIds: ['persons-relationships']
      }
    ]
  })
}

