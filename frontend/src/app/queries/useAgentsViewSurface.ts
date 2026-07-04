import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useAgentsViewSurface() {
  return createPlannedScreenSurface({
    screenId: 'agents',
    titleKey: 'Agents',
    descriptionKey: 'Agents UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Agents logic is preserved',
    detailKey: 'AI workspace orchestration, agent cards and action workflows remain in the extracted surface and stores. This screen stays empty until the new agents UI is rebuilt.',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath: 'frontend/src/domains/agents/queries/useAgentsPageSurface.ts'
  })
}
