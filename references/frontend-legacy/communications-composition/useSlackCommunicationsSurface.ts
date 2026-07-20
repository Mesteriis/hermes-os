// Historical pre-clean-room provider surface. It is not part of the active client graph.
import { createCommunicationSubSurface } from './communicationChannelSurface'

export function useSlackCommunicationsSurface() {
  return createCommunicationSubSurface({
    channelId: 'slack',
    labelKey: 'Slack',
    status: 'facade',
    businessQueryRoot: ['communications', 'channels'] as const,
    runtimeQueryRoot: ['integrations', 'slack', 'runtime'] as const,
    surfacePath: 'frontend/src/domains/communications/queries/useSlackCommunicationsSurface.ts',
    capabilityNotes: [
      'Slack will enter the same Channels workspace through a provider sub-surface.',
      'The frontend contract is present while provider runtime implementation is not connected.'
    ],
    capabilityGroups: [
      {
        id: 'slack-channel-facade',
        labelKey: 'Slack channel facade',
        menuLabelKey: 'Open Slack planned capabilities',
        icon: 'tabler:brand-slack',
        status: 'facade',
        capabilities: [
          {
            id: 'slack-channels',
            labelKey: 'Channels and threads',
            descriptionKey: 'Future Slack channels, threads and mentions use the Channels workspace contract.',
            icon: 'tabler:messages',
            status: 'facade',
            kind: 'query'
          }
        ]
      }
    ]
  })
}
