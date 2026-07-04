import { useCommunicationsWorkspaceSurface } from '../../domains/communications/queries/useCommunicationsWorkspaceSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useCommunicationsViewSurface() {
  const communications = useCommunicationsWorkspaceSurface()

  return createPlannedScreenSurface({
    screenId: 'communications',
    titleKey: 'Communications',
    descriptionKey: 'Communications UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Communications logic is preserved',
    detailKey: 'Message queries, thread orchestration, outbox state and compose flows remain in the extracted surface. This screen stays empty until the new communications UI is rebuilt.',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath: 'frontend/src/domains/communications/queries/useCommunicationsWorkspaceSurface.ts',
    childSurfaces: communications.childSurfaces
  })
}
