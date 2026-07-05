import { createCommunicationSubSurface } from './communicationChannelSurface'

export function useMattermostCommunicationsSurface() {
  return createCommunicationSubSurface({
    channelId: 'mattermost',
    labelKey: 'Mattermost',
    status: 'facade',
    businessQueryRoot: ['communications', 'channels'] as const,
    runtimeQueryRoot: ['integrations', 'mattermost', 'runtime'] as const,
    surfacePath: 'frontend/src/domains/communications/queries/useMattermostCommunicationsSurface.ts',
    capabilityNotes: [
      'Mattermost channels will reuse the provider-neutral Channels UI contract.',
      'Runtime integration and provider commands are intentionally facade-only.'
    ],
    capabilityGroups: [
      {
        id: 'mattermost-channel-facade',
        labelKey: 'Mattermost channel facade',
        menuLabelKey: 'Open Mattermost planned capabilities',
        icon: 'tabler:message-circle-cog',
        status: 'facade',
        capabilities: [
          {
            id: 'mattermost-channels',
            labelKey: 'Teams and channels',
            descriptionKey: 'Future Mattermost teams, channels and posts use the same Channels workspace contract.',
            icon: 'tabler:messages',
            status: 'facade',
            kind: 'query'
          }
        ]
      }
    ]
  })
}
