import { useCalendarSurface } from '../../domains/calendar/queries/useCalendarSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useCalendarViewSurface() {
  const calendar = useCalendarSurface()

  return createPlannedScreenSurface({
    screenId: 'calendar',
    titleKey: 'Calendar',
    descriptionKey: 'Calendar UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Calendar logic is preserved',
    detailKey: 'Calendar queries, event creation orchestration and weekly brief state remain in the extracted surface. This screen stays empty until the new calendar UI is rebuilt.',
    status: calendar.status,
    ownerLayer: 'domain',
    surfacePath: calendar.surfacePath,
    childSurfaces: calendar.childSurfaces
  })
}
