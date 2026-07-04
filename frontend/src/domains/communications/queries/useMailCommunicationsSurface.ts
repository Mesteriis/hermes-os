import { createCommunicationChannelSurface } from './communicationChannelSurface'

export function useMailCommunicationsSurface() {
  return createCommunicationChannelSurface({
    channelId: 'mail',
    labelKey: 'Mail',
    status: 'active',
    businessQueryRoot: ['communications', 'mail'] as const,
    runtimeQueryRoot: ['integrations', 'mail', 'runtime'] as const,
    surfacePath: 'frontend/src/domains/communications/queries/useCommunicationsPageSurface.ts',
    capabilityNotes: [
      'Provider-neutral message list and thread orchestration live in the Communications page surface.',
      'Mail sync settings and runtime status remain under Communications-owned surfaces.'
    ]
  })
}
