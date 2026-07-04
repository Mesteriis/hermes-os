import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useHomeViewSurface() {
  return createPlannedScreenSurface({
    screenId: 'home',
    titleKey: 'Home',
    descriptionKey: 'Home UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Home logic is preserved',
    detailKey: 'Dashboard stats, message feed derivations and people-talked summaries remain in the extracted surface. This screen stays empty until the new home UI is rebuilt.',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath: 'frontend/src/domains/home/queries/useHomePageSurface.ts'
  })
}
