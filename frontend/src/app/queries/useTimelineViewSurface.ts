import { useTimelineSurface } from '../../domains/timeline/queries/useTimelineSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useTimelineViewSurface() {
  const timeline = useTimelineSurface()

  return createPlannedScreenSurface({
    screenId: 'timeline',
    titleKey: 'Timeline',
    descriptionKey: 'Timeline projection is not admitted yet.',
    preservedLogicKey: 'No compatibility data path is retained',
    detailKey: 'This screen stays empty until the rebuildable Timeline projection is admitted outside business domains.',
    status: timeline.status,
    ownerLayer: 'domain',
    surfacePath: timeline.surfacePath,
    childSurfaces: timeline.childSurfaces
  })
}
