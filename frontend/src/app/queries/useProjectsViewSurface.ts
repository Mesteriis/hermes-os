import { useProjectsSurface } from '../../domains/projects/queries/useProjectsSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useProjectsViewSurface() {
  const projects = useProjectsSurface()

  return createPlannedScreenSurface({
    screenId: 'projects',
    titleKey: 'Projects',
    descriptionKey: 'Projects UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Projects logic is preserved',
    detailKey: 'Project queries, selection orchestration and project summary formatting remain in the extracted surface and stores. This screen stays empty until the new projects UI is rebuilt.',
    status: projects.status,
    ownerLayer: 'domain',
    surfacePath: projects.surfacePath,
    childSurfaces: projects.childSurfaces
  })
}
