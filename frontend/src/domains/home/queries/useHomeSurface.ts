import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/home/queries/useHomePageSurface.ts'

export function useHomeSurface() {
  return createDomainSurface({
    surfaceId: 'home',
    labelKey: 'Home',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'home-dashboard-summary',
        labelKey: 'Dashboard summary',
        descriptionKey: 'Owner-facing daily state, message pressure and memory activity.',
        icon: 'tabler:home-stats',
        status: 'active',
        kind: 'projection',
        contract: 'useHomePageSurface.homeStats'
      },
      {
        id: 'home-attention-feed',
        labelKey: 'Attention feed',
        descriptionKey: 'Priority signals that need review before they become durable truth.',
        icon: 'tabler:alert-circle',
        status: 'active',
        kind: 'review',
        contract: 'useHomePageSurface.recentMessages'
      },
      {
        id: 'home-relationship-snapshot',
        labelKey: 'Relationship snapshot',
        descriptionKey: 'Recent people and organizations surfaced from communications context.',
        icon: 'tabler:users',
        status: 'active',
        kind: 'projection',
        contract: 'useHomePageSurface.peopleTalked'
      }
    ],
    childSurfaces: [
      {
        id: 'home-overview',
        labelKey: 'Overview',
        status: 'facade',
        surfacePath,
        capabilityIds: ['home-dashboard-summary']
      },
      {
        id: 'home-attention',
        labelKey: 'Attention',
        status: 'facade',
        surfacePath,
        capabilityIds: ['home-attention-feed']
      },
      {
        id: 'home-memory',
        labelKey: 'Memory',
        status: 'facade',
        surfacePath,
        capabilityIds: ['home-relationship-snapshot']
      }
    ]
  })
}

