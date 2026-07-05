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
      ],
      capabilityGroups: [
        {
          id: 'whatsapp-communications',
          labelKey: 'WhatsApp communications',
          menuLabelKey: 'Open WhatsApp communication capabilities',
          icon: 'tabler:brand-whatsapp',
          status: 'available',
          capabilities: [
            {
              id: 'whatsapp-conversations',
              labelKey: 'WhatsApp conversations',
              descriptionKey: 'Companion channel data is exposed through Communications conversations and messages.',
              icon: 'tabler:messages',
              status: 'available',
              kind: 'query',
              contract: 'whatsappBusinessQueryKeys'
            },
            {
              id: 'whatsapp-composer',
              labelKey: 'WhatsApp composer',
              descriptionKey: 'WhatsApp-specific media, template, contact and location actions are provider-scoped.',
              icon: 'tabler:edit',
              status: 'available',
              kind: 'composer',
              contract: 'whatsAppMessengerComposerPreset'
            }
          ]
        }
      ]
    }),
    queryKeys: whatsappBusinessQueryKeys
  }
}
