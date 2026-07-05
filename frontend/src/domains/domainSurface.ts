export type DomainSurfaceStatus = 'active' | 'facade'

export type DomainSurfaceOwnerLayer = 'domain' | 'app'

export type DomainCapabilityKind =
  | 'automation'
  | 'command'
  | 'evidence'
  | 'graph'
  | 'inspector'
  | 'projection'
  | 'query'
  | 'review'
  | 'search'
  | 'settings'
  | 'timeline'
  | 'workspace'

export type DomainSurfaceCapability = {
  id: string
  labelKey: string
  descriptionKey: string
  icon: string
  status: DomainSurfaceStatus
  kind: DomainCapabilityKind
  contract?: string
}

export type DomainSubSurface = {
  id: string
  labelKey: string
  status: DomainSurfaceStatus
  surfacePath?: string
  capabilityIds?: readonly string[]
}

export type DomainSurface = {
  surfaceId: string
  labelKey: string
  status: DomainSurfaceStatus
  ownerLayer: DomainSurfaceOwnerLayer
  surfacePath?: string
  capabilities: readonly DomainSurfaceCapability[]
  childSurfaces: readonly DomainSubSurface[]
}

export function createDomainSurface<T extends DomainSurface>(surface: T): T {
  return surface
}

