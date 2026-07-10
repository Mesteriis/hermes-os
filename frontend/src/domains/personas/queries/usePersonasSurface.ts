import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/personas/views/PersonasWorkspaceView.vue'

export function usePersonasSurface() {
  return createDomainSurface({
    surfaceId: 'personas',
    labelKey: 'Personas',
    status: 'active',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'personas-directory',
        labelKey: 'Persona directory',
        descriptionKey: 'Canonical Persona records and identity review state.',
        icon: 'tabler:user',
        status: 'active',
        kind: 'query',
        contract: 'usePersonasPageSurface.personas'
      },
      {
        id: 'personas-identity-review',
        labelKey: 'Identity review',
        descriptionKey: 'Evidence-backed merge, alias and candidate decisions for Personas.',
        icon: 'tabler:user-check',
        status: 'active',
        kind: 'review',
        contract: 'usePersonasPageSurface.identityReviewItems'
      },
      {
        id: 'personas-relationships',
        labelKey: 'Relationships',
        descriptionKey: 'Relationship candidates and confirmed links around a Persona.',
        icon: 'tabler:hierarchy-2',
        status: 'active',
        kind: 'graph',
        contract: 'usePersonasPageSurface.relationshipReviewItems'
      }
    ],
    childSurfaces: [
      {
        id: 'personas-directory',
        labelKey: 'Directory',
        status: 'active',
        surfacePath,
        capabilityIds: ['personas-directory']
      },
      {
        id: 'personas-review',
        labelKey: 'Review',
        status: 'active',
        surfacePath,
        capabilityIds: ['personas-identity-review']
      },
      {
        id: 'personas-graph',
        labelKey: 'Relationships',
        status: 'active',
        surfacePath,
        capabilityIds: ['personas-relationships']
      }
    ]
  })
}
