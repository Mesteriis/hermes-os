import { whatsappBusinessQueryKeys } from './whatsappBusinessQueries'
import { createCommunicationChannelSurface } from './communicationChannelSurface'

export function useWhatsappCommunicationsSurface() {
  return {
    ...createCommunicationChannelSurface({
      channelId: 'whatsapp',
      labelKey: 'WhatsApp',
      status: 'active',
      businessQueryRoot: ['communications', 'whatsapp'] as const,
      runtimeQueryRoot: ['integrations', 'whatsapp', 'runtime'] as const,
      surfacePath: 'frontend/src/domains/communications/queries/useWhatsappCommunicationsPanelSurface.ts',
      capabilityNotes: [
        'WhatsApp business conversations and messages are exposed through the Communications domain.',
        'Provider runtime controls stay under integration runtime surfaces.'
      ]
    }),
    queryKeys: whatsappBusinessQueryKeys
  }
}
