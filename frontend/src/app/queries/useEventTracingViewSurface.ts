import { useEventTracingSurface } from '../../domains/event-tracing/queries/useEventTracingSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useEventTracingViewSurface() {
  const eventTracing = useEventTracingSurface()

  return createPlannedScreenSurface({
    screenId: 'event-tracing',
    titleKey: 'Event Tracing',
    descriptionKey: 'Event tracing UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Event tracing logic is preserved',
    detailKey: 'Platform event trace queries, trace lookup keys and event envelope types remain in extracted TypeScript artifacts. This screen stays empty until the new event tracing UI is rebuilt.',
    status: eventTracing.status,
    ownerLayer: 'domain',
    surfacePath: eventTracing.surfacePath,
    childSurfaces: eventTracing.childSurfaces
  })
}
