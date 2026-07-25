import { useHomeSurface } from '../../domains/home/queries/useHomeSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useHomeViewSurface() {
  const home = useHomeSurface()

  return createPlannedScreenSurface({
    screenId: 'home',
    titleKey: 'Home',
    descriptionKey: 'Home projection is not admitted yet.',
    preservedLogicKey: 'No compatibility data path is retained',
    detailKey: 'This screen stays empty until an app-owned projection is admitted over public owner contracts and durable events.',
    status: home.status,
    ownerLayer: 'domain',
    surfacePath: home.surfacePath,
    childSurfaces: home.childSurfaces
  })
}
