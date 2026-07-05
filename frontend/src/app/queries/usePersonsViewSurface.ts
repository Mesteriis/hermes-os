import { usePersonsSurface } from '../../domains/personas/queries/usePersonsSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function usePersonsViewSurface() {
  const persons = usePersonsSurface()

  return createPlannedScreenSurface({
    screenId: 'persons',
    titleKey: 'Persons',
    descriptionKey: 'Persons UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Persons logic is preserved',
    detailKey: 'Person queries, identity review orchestration and relationship review state remain in the extracted surface. This screen stays empty until the new persons UI is rebuilt.',
    status: persons.status,
    ownerLayer: 'domain',
    surfacePath: persons.surfacePath,
    childSurfaces: persons.childSurfaces
  })
}
