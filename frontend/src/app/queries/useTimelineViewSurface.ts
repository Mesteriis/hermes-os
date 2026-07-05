import { useTimelineSurface } from '../../domains/timeline/queries/useTimelineSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useTimelineViewSurface() {
  const timeline = useTimelineSurface()

  return createPlannedScreenSurface({
    screenId: 'timeline',
    titleKey: 'Timeline',
    descriptionKey: 'Timeline UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Timeline logic is preserved',
    detailKey: 'Timeline queries, message hydration and filter store state remain in the extracted surface. This screen stays empty until the new timeline UI is rebuilt.',
    status: timeline.status,
    ownerLayer: 'domain',
    surfacePath: timeline.surfacePath,
    childSurfaces: timeline.childSurfaces
  })
}
