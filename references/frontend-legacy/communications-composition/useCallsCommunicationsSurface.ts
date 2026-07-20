// Historical pre-clean-room provider surface. It is not part of the active client graph.
import { createCommunicationSubSurface } from './communicationChannelSurface'

export function useCallsCommunicationsSurface() {
  return createCommunicationSubSurface({
    channelId: 'calls',
    labelKey: 'Calls',
    status: 'active',
    businessQueryRoot: ['communications', 'calls'] as const,
    surfacePath: 'frontend/src/domains/communications/queries/useCallsCommunicationsSurface.ts',
    capabilityNotes: [
      'Calls stay inside Communications because Zoom, Yandex Telemost and Zulip calls are communication evidence.',
      'Permanent meeting rooms, recordings, transcripts and call intelligence are exposed as one provider-neutral Calls surface.'
    ],
    capabilityGroups: [
      {
        id: 'calls-workspace',
        labelKey: 'Calls workspace',
        menuLabelKey: 'Open call workspace capabilities',
        icon: 'tabler:phone-call',
        status: 'available',
        capabilities: [
          {
            id: 'calls-list',
            labelKey: 'Calls by date',
            descriptionKey: 'Date-grouped calls, permanent meetings and provider account selection.',
            icon: 'tabler:list-details',
            status: 'available',
            kind: 'query',
            contract: 'communications.calls.list'
          },
          {
            id: 'calls-recordings',
            labelKey: 'Recordings and transcripts',
            descriptionKey: 'Call recordings, transcript state and review-ready transcript evidence.',
            icon: 'tabler:file-text',
            status: 'available',
            kind: 'projection',
            contract: 'communications.calls.recordings'
          },
          {
            id: 'calls-intelligence',
            labelKey: 'Call intelligence',
            descriptionKey: 'Meeting summaries, extracted decisions, obligations and follow-up candidates.',
            icon: 'tabler:sparkles',
            status: 'available',
            kind: 'inspector',
            contract: 'communications.calls.intelligence'
          }
        ]
      }
    ]
  })
}
