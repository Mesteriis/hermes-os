export type CommunicationChannelSurfaceStatus = 'active' | 'facade'

export type CommunicationChannelSurface = {
  channelId: 'mail' | 'telegram' | 'whatsapp'
  labelKey: string
  status: CommunicationChannelSurfaceStatus
  businessQueryRoot: readonly ['communications', string]
  runtimeQueryRoot?: readonly ['integrations', string, 'runtime']
  surfacePath?: string
  capabilityNotes: readonly string[]
}

export function createCommunicationChannelSurface<T extends CommunicationChannelSurface>(surface: T): T {
  return surface
}
