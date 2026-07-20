// Historical pre-clean-room Communications facade. Not part of the active client graph.
import { useCommunicationsWorkspaceSurface } from '../../domains/communications/queries/useCommunicationsWorkspaceSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useCommunicationsViewSurface() {
  const communications = useCommunicationsWorkspaceSurface()

  return createPlannedScreenSurface({
    screenId: 'communications',
    titleKey: 'Communications',
    descriptionKey: 'Unified evidence-first workspace for mail, messenger channels, provider commands and review pressure.',
    preservedLogicKey: 'Communications workspace is active',
    detailKey: 'Message queries, thread orchestration, outbox state and compose flows are wired through the Communications render surface.',
    status: 'active',
    ownerLayer: 'domain',
    surfacePath: 'frontend/src/domains/communications/queries/useCommunicationsWorkspaceSurface.ts',
    childSurfaces: communications.childSurfaces
  })
}
