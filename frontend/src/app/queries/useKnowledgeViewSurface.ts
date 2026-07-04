import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useKnowledgeViewSurface() {
  return createPlannedScreenSurface({
    screenId: 'knowledge',
    titleKey: 'Knowledge Graph',
    descriptionKey: 'Knowledge UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Knowledge logic is preserved',
    detailKey: 'Graph summary sync, search orchestration and contradiction review state remain in the extracted surface. This screen stays empty until the new knowledge UI is rebuilt.',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath: 'frontend/src/domains/knowledge/queries/useKnowledgePageSurface.ts'
  })
}
