import { useAgentsSurface } from '../../domains/agents/queries/useAgentsSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useAgentsViewSurface() {
  const agents = useAgentsSurface()

  return createPlannedScreenSurface({
    screenId: 'agents',
    titleKey: 'Agents',
    descriptionKey: 'Agents UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Agents logic is preserved',
    detailKey: 'AI workspace orchestration, agent cards and action workflows remain in the extracted surface and stores. This screen stays empty until the new agents UI is rebuilt.',
    status: agents.status,
    ownerLayer: 'domain',
    surfacePath: agents.surfacePath,
    childSurfaces: agents.childSurfaces
  })
}
