import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/timeline/queries/useTimelineSurface.ts'

export function useTimelineSurface() {
  return createDomainSurface({
    surfaceId: 'timeline',
    labelKey: 'Timeline',
    status: 'planned',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'timeline-events',
        labelKey: 'Event stream',
        descriptionKey: 'A future rebuildable event projection admitted outside business domains.',
        icon: 'tabler:timeline',
        status: 'planned',
        kind: 'timeline',
      },
      {
        id: 'timeline-filters',
        labelKey: 'Filters',
        descriptionKey: 'Filtering remains unavailable until the Timeline projection is admitted.',
        icon: 'tabler:filter',
        status: 'planned',
        kind: 'search',
      },
      {
        id: 'timeline-hydration',
        labelKey: 'Source hydration',
        descriptionKey: 'Hydration will use owner contracts without a Communications compatibility path.',
        icon: 'tabler:database-import',
        status: 'planned',
        kind: 'projection',
      }
    ],
    childSurfaces: [
      {
        id: 'timeline-events',
        labelKey: 'Events',
        status: 'planned',
        surfacePath,
        capabilityIds: ['timeline-events']
      },
      {
        id: 'timeline-filters',
        labelKey: 'Filters',
        status: 'planned',
        surfacePath,
        capabilityIds: ['timeline-filters']
      },
      {
        id: 'timeline-sources',
        labelKey: 'Sources',
        status: 'planned',
        surfacePath,
        capabilityIds: ['timeline-hydration']
      }
    ]
  })
}
