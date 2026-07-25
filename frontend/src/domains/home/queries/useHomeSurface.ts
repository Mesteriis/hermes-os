import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/home/queries/useHomeSurface.ts'

export function useHomeSurface() {
  return createDomainSurface({
    surfaceId: 'home',
    labelKey: 'Home',
    status: 'planned',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'home-dashboard-summary',
        labelKey: 'Dashboard summary',
        descriptionKey: 'A future app projection assembled from admitted owner events.',
        icon: 'tabler:home-stats',
        status: 'planned',
        kind: 'projection',
      },
      {
        id: 'home-attention-feed',
        labelKey: 'Attention feed',
        descriptionKey: 'A future Review projection; no Communications fallback is retained.',
        icon: 'tabler:alert-circle',
        status: 'planned',
        kind: 'review',
      },
      {
        id: 'home-relationship-snapshot',
        labelKey: 'Relationship snapshot',
        descriptionKey: 'A future cross-owner app composition backed by admitted public contracts.',
        icon: 'tabler:users',
        status: 'planned',
        kind: 'projection',
      }
    ],
    childSurfaces: [
      {
        id: 'home-overview',
        labelKey: 'Overview',
        status: 'planned',
        surfacePath,
        capabilityIds: ['home-dashboard-summary']
      },
      {
        id: 'home-attention',
        labelKey: 'Attention',
        status: 'planned',
        surfacePath,
        capabilityIds: ['home-attention-feed']
      },
      {
        id: 'home-memory',
        labelKey: 'Memory',
        status: 'planned',
        surfacePath,
        capabilityIds: ['home-relationship-snapshot']
      }
    ]
  })
}
