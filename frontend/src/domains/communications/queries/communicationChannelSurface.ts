export type CommunicationChannelSurfaceStatus = 'active' | 'facade'
export type CommunicationSubSurfaceStatus = CommunicationChannelSurfaceStatus
export type CommunicationCapabilitySurfaceStatus = 'available' | 'partial' | 'facade' | 'blocked'
export type CommunicationSubSurfaceId =
  | 'calls'
  | 'meetings'
  | 'mail'
  | 'telegram'
  | 'whatsapp'
  | 'communications-timeline'
  | 'zulip'
  | 'slack'
  | 'discord'
  | 'mattermost'

export type CommunicationCapabilityKind =
  | 'common'
  | 'query'
  | 'command'
  | 'projection'
  | 'runtime'
  | 'composer'
  | 'inspector'

export type CommunicationSurfaceCapability = {
  id: string
  labelKey: string
  descriptionKey: string
  icon: string
  status: CommunicationCapabilitySurfaceStatus
  kind: CommunicationCapabilityKind
  contract?: string
  disabled?: boolean
}

export type CommunicationSurfaceCapabilityGroup = {
  id: string
  labelKey: string
  menuLabelKey: string
  icon: string
  status: CommunicationCapabilitySurfaceStatus
  capabilities: readonly CommunicationSurfaceCapability[]
}

export type CommunicationSubSurface = {
  channelId: CommunicationSubSurfaceId
  labelKey: string
  status: CommunicationSubSurfaceStatus
  businessQueryRoot: readonly ['communications', string]
  runtimeQueryRoot?: readonly ['integrations', string, 'runtime']
  surfacePath?: string
  capabilityNotes: readonly string[]
  capabilityGroups: readonly CommunicationSurfaceCapabilityGroup[]
}

export type CommunicationChannelSurface = CommunicationSubSurface

export type CommunicationSurfaceChild = {
  id: CommunicationSubSurfaceId
  labelKey: string
  status: CommunicationSubSurfaceStatus
  surfacePath?: string
}

export type CommunicationSurface = {
  surfaceId: 'communications'
  commonCapabilities: readonly CommunicationSurfaceCapability[]
  subSurfaces: readonly CommunicationSubSurface[]
  childSurfaces: readonly CommunicationSurfaceChild[]
}

export function createCommunicationSurface<T extends CommunicationSurface>(surface: T): T {
  return surface
}

export function createCommunicationSubSurface<T extends CommunicationSubSurface>(surface: T): T {
  return surface
}

export function createCommunicationChannelSurface<T extends CommunicationChannelSurface>(surface: T): T {
  return createCommunicationSubSurface(surface)
}

export function communicationSurfaceChild(surface: CommunicationSubSurface): CommunicationSurfaceChild {
  return {
    id: surface.channelId,
    labelKey: surface.labelKey,
    status: surface.status,
    surfacePath: surface.surfacePath
  }
}

export function communicationSubSurfaceCapabilities(
  surface: CommunicationSubSurface,
  kind?: CommunicationCapabilityKind
): CommunicationSurfaceCapability[] {
  return surface.capabilityGroups.flatMap((group) => (
    kind
      ? group.capabilities.filter((capability) => capability.kind === kind)
      : [...group.capabilities]
  ))
}
