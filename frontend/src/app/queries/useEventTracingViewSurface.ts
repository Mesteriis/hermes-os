import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useEventTracingViewSurface() {
  return createPlannedScreenSurface({
    screenId: 'event-tracing',
    titleKey: 'Event Tracing',
    descriptionKey: 'Event tracing UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Event tracing logic is preserved',
    detailKey: 'Platform event trace queries, trace lookup keys and event envelope types remain in extracted TypeScript artifacts. This screen stays empty until the new event tracing UI is rebuilt.',
    status: 'facade',
    ownerLayer: 'app'
  })
}
