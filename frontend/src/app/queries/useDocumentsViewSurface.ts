import { useDocumentsSurface } from '../../domains/documents/queries/useDocumentsSurface'
import { createPlannedScreenSurface } from './plannedScreenSurface'

export function useDocumentsViewSurface() {
  const documents = useDocumentsSurface()

  return createPlannedScreenSurface({
    screenId: 'documents',
    titleKey: 'Documents',
    descriptionKey: 'Documents UI removed after logic extraction. Rebuild pending new design language.',
    preservedLogicKey: 'Documents logic is preserved',
    detailKey: 'Document queries and retry orchestration remain in the extracted surface. This screen stays empty until the new documents UI is rebuilt.',
    status: documents.status,
    ownerLayer: 'domain',
    surfacePath: documents.surfacePath,
    childSurfaces: documents.childSurfaces
  })
}
