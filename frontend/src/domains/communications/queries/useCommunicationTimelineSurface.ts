import { createCommunicationSubSurface } from './communicationChannelSurface'

export function useCommunicationTimelineSurface() {
  return createCommunicationSubSurface({
    channelId: 'communications-timeline',
    labelKey: 'Timeline',
    status: 'facade',
    businessQueryRoot: ['communications', 'timeline'] as const,
    surfacePath: 'frontend/src/domains/communications/queries/useCommunicationTimelineSurface.ts',
    capabilityNotes: [
      'Communications timeline is the provider-neutral chronological view over messages, calls, meetings and channel events.',
      'The global Timeline domain may reuse this as source evidence, but Communications keeps the provider context.'
    ],
    capabilityGroups: [
      {
        id: 'communications-timeline-workspace',
        labelKey: 'Communications timeline',
        menuLabelKey: 'Open communication timeline capabilities',
        icon: 'tabler:timeline',
        status: 'facade',
        capabilities: [
          {
            id: 'communications-timeline-events',
            labelKey: 'Communication events',
            descriptionKey: 'Chronological provider-neutral communication events with source provenance.',
            icon: 'tabler:timeline-event',
            status: 'facade',
            kind: 'projection',
            contract: 'communications.timeline.events'
          },
          {
            id: 'communications-timeline-filters',
            labelKey: 'Communication filters',
            descriptionKey: 'Filter by provider family, account, participant, evidence state and review status.',
            icon: 'tabler:filter',
            status: 'facade',
            kind: 'query',
            contract: 'communications.timeline.filters'
          },
          {
            id: 'communications-timeline-review',
            labelKey: 'Timeline review actions',
            descriptionKey: 'Promote timeline signals into review items without bypassing evidence.',
            icon: 'tabler:clipboard-check',
            status: 'facade',
            kind: 'command',
            contract: 'communications.timeline.review'
          }
        ]
      }
    ]
  })
}

