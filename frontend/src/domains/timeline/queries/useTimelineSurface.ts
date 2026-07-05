import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/timeline/queries/useTimelinePageSurface.ts'

export function useTimelineSurface() {
  return createDomainSurface({
    surfaceId: 'timeline',
    labelKey: 'Timeline',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'timeline-events',
        labelKey: 'Event stream',
        descriptionKey: 'Chronological owner memory, communication and system events.',
        icon: 'tabler:timeline',
        status: 'active',
        kind: 'timeline',
        contract: 'useTimelinePageSurface.timelineItems'
      },
      {
        id: 'timeline-filters',
        labelKey: 'Filters',
        descriptionKey: 'Timeline filtering by source, entity, confidence and date.',
        icon: 'tabler:filter',
        status: 'active',
        kind: 'search',
        contract: 'useTimelinePageSurface.filterStore'
      },
      {
        id: 'timeline-hydration',
        labelKey: 'Source hydration',
        descriptionKey: 'Source-backed message and event detail hydration.',
        icon: 'tabler:database-import',
        status: 'active',
        kind: 'projection',
        contract: 'useTimelinePageSurface.messageHydrator'
      }
    ],
    childSurfaces: [
      {
        id: 'timeline-events',
        labelKey: 'Events',
        status: 'facade',
        surfacePath,
        capabilityIds: ['timeline-events']
      },
      {
        id: 'timeline-filters',
        labelKey: 'Filters',
        status: 'facade',
        surfacePath,
        capabilityIds: ['timeline-filters']
      },
      {
        id: 'timeline-sources',
        labelKey: 'Sources',
        status: 'facade',
        surfacePath,
        capabilityIds: ['timeline-hydration']
      }
    ]
  })
}

