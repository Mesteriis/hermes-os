import { telegramBusinessQueryKeys } from './telegramBusinessQueries'
import { createCommunicationChannelSurface } from './communicationChannelSurface'

export function useTelegramCommunicationsSurface() {
  return {
    ...createCommunicationChannelSurface({
      channelId: 'telegram',
      labelKey: 'Telegram',
      status: 'facade',
      businessQueryRoot: ['communications', 'telegram'] as const,
      runtimeQueryRoot: ['integrations', 'telegram', 'runtime'] as const,
      surfacePath: 'frontend/src/domains/communications/queries/useTelegramCommunicationsSurface.ts',
      capabilityNotes: [
        'Telegram business query hooks already use the Communications cache root.',
        'The render-level Telegram communications panel is still intentionally absent.'
      ],
      capabilityGroups: [
        {
          id: 'telegram-communications',
          labelKey: 'Telegram communications',
          menuLabelKey: 'Open Telegram communication capabilities',
          icon: 'tabler:brand-telegram',
          status: 'facade',
          capabilities: [
            {
              id: 'telegram-conversations',
              labelKey: 'Telegram conversations',
              descriptionKey: 'Business conversations and messages stay under Communications query keys.',
              icon: 'tabler:messages',
              status: 'partial',
              kind: 'query',
              contract: 'telegramBusinessQueryKeys'
            },
            {
              id: 'telegram-composer',
              labelKey: 'Telegram composer',
              descriptionKey: 'Provider-specific composer actions are exposed by the messenger rich editor.',
              icon: 'tabler:edit',
              status: 'facade',
              kind: 'composer',
              contract: 'telegramMessengerComposerPreset'
            }
          ]
        }
      ]
    }),
    queryKeys: telegramBusinessQueryKeys
  }
}
