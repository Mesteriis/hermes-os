import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useReviewViewSurface() {
  return createPlannedScreenSurface({
    screenId: 'review',
    titleKey: 'Review',
    descriptionKey: 'Review UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Review logic is preserved',
    detailKey: 'Review item orchestration, promotion defaults and suggested entity review state remain in the extracted surface. This screen stays empty until the new review UI is rebuilt.',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath: 'frontend/src/domains/review/queries/useReviewPageSurface.ts'
  })
}
