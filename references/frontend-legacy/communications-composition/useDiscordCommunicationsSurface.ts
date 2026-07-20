// Historical pre-clean-room provider surface. It is not part of the active client graph.
import { createCommunicationSubSurface } from './communicationChannelSurface'

export function useDiscordCommunicationsSurface() {
  return createCommunicationSubSurface({
    channelId: 'discord',
    labelKey: 'Discord',
    status: 'facade',
    businessQueryRoot: ['communications', 'channels'] as const,
    runtimeQueryRoot: ['integrations', 'discord', 'runtime'] as const,
    surfacePath: 'frontend/src/domains/communications/queries/useDiscordCommunicationsSurface.ts',
    capabilityNotes: [
      'Discord servers and channels will reuse the Channels workspace shell.',
      'Provider-specific capabilities remain facade-only until backend support exists.'
    ],
    capabilityGroups: [
      {
        id: 'discord-channel-facade',
        labelKey: 'Discord channel facade',
        menuLabelKey: 'Open Discord planned capabilities',
        icon: 'tabler:brand-discord',
        status: 'facade',
        capabilities: [
          {
            id: 'discord-servers',
            labelKey: 'Servers and channels',
            descriptionKey: 'Future Discord servers, channels and threads use the same channel list and viewer contract.',
            icon: 'tabler:messages',
            status: 'facade',
            kind: 'query'
          }
        ]
      }
    ]
  })
}
