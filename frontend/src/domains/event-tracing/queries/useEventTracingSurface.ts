import { createDomainSurface } from '../../domainSurface'

export function useEventTracingSurface() {
  return createDomainSurface({
    surfaceId: 'event-tracing',
    labelKey: 'Event Traces',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath: 'frontend/src/domains/event-tracing/queries/useEventTracingSurface.ts',
    capabilities: [
      {
        id: 'event-tracing-search',
        labelKey: 'Trace search',
        descriptionKey: 'Lookup events by correlation, causation, provider source and envelope ids.',
        icon: 'tabler:route',
        status: 'facade',
        kind: 'search'
      },
      {
        id: 'event-tracing-envelope',
        labelKey: 'Envelope inspector',
        descriptionKey: 'Inspect canonical event envelope metadata without exposing private payloads.',
        icon: 'tabler:package',
        status: 'facade',
        kind: 'inspector'
      },
      {
        id: 'event-tracing-causation',
        labelKey: 'Causation chain',
        descriptionKey: 'Trace observed events through workflows, projections and promoted domain actions.',
        icon: 'tabler:git-branch',
        status: 'facade',
        kind: 'timeline'
      }
    ],
    childSurfaces: [
      {
        id: 'event-tracing-search',
        labelKey: 'Search',
        status: 'facade',
        capabilityIds: ['event-tracing-search']
      },
      {
        id: 'event-tracing-envelope',
        labelKey: 'Envelope',
        status: 'facade',
        capabilityIds: ['event-tracing-envelope']
      },
      {
        id: 'event-tracing-causation',
        labelKey: 'Causation',
        status: 'facade',
        capabilityIds: ['event-tracing-causation']
      }
    ]
  })
}

