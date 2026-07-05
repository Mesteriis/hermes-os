import { useNotesSurface } from '../../domains/notes/queries/useNotesSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useNotesViewSurface() {
  const notes = useNotesSurface()

  return createPlannedScreenSurface({
    screenId: 'notes',
    titleKey: 'Notes',
    descriptionKey: 'Notes UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Notes logic is preserved',
    detailKey: 'Note queries, fallback seed notes and notes filter state remain in the extracted surface and stores. This screen stays empty until the new notes UI is rebuilt.',
    status: notes.status,
    ownerLayer: 'domain',
    surfacePath: notes.surfacePath,
    childSurfaces: notes.childSurfaces
  })
}
