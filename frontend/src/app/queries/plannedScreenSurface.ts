export type PlannedScreenSurfaceStatus = 'active' | 'facade'

export type PlannedChildSurface = {
  id: string
  labelKey: string
  status: PlannedScreenSurfaceStatus
  surfacePath?: string
}

export type PlannedScreenSurface = {
  screenId: string
  titleKey: string
  descriptionKey: string
  preservedLogicKey: string
  detailKey: string
  status: PlannedScreenSurfaceStatus
  ownerLayer: 'app' | 'domain'
  surfacePath?: string
  childSurfaces?: readonly PlannedChildSurface[]
}

export function createPlannedScreenSurface<T extends PlannedScreenSurface>(surface: T): T {
  return surface
}
