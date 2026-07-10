import { useHomeSurface } from '../../domains/home/queries/useHomeSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useHomeViewSurface() {
  const home = useHomeSurface()

  return createPlannedScreenSurface({
    screenId: 'home',
    titleKey: 'Home',
    descriptionKey: 'Home UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Home logic is preserved',
    detailKey: 'Dashboard stats, message feed derivations and Persona summaries remain in the extracted surface. This screen stays empty until the new home UI is rebuilt.',
    status: home.status,
    ownerLayer: 'domain',
    surfacePath: home.surfacePath,
    childSurfaces: home.childSurfaces
  })
}
