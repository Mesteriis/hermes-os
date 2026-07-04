import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useTasksViewSurface() {
  return createPlannedScreenSurface({
    screenId: 'tasks',
    titleKey: 'Tasks',
    descriptionKey: 'Tasks UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Tasks logic is preserved',
    detailKey: 'Task queries, review orchestration and context review state remain in the extracted surface. This screen stays empty until the new tasks UI is rebuilt.',
    status: 'facade',
    ownerLayer: 'domain',
    surfacePath: 'frontend/src/domains/tasks/queries/useTasksPageSurface.ts'
  })
}
