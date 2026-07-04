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
      ]
    }),
    queryKeys: telegramBusinessQueryKeys
  }
}
