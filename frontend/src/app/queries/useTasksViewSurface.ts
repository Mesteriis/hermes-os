import { useTasksSurface } from '../../domains/tasks/queries/useTasksSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useTasksViewSurface() {
  const tasks = useTasksSurface()

  return createPlannedScreenSurface({
    screenId: 'tasks',
    titleKey: 'Tasks',
    descriptionKey: 'Tasks UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Tasks logic is preserved',
    detailKey: 'Task queries, review orchestration and context review state remain in the extracted surface. This screen stays empty until the new tasks UI is rebuilt.',
    status: tasks.status,
    ownerLayer: 'domain',
    surfacePath: tasks.surfacePath,
    childSurfaces: tasks.childSurfaces
  })
}
