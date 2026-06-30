import { existsSync, readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentDir = dirname(fileURLToPath(import.meta.url))

const removedSearchAndFolderFiles = [
  'AttachmentSearchPanel.vue',
  'CommunicationFolderStrip.vue',
  'SavedSearchRuleGroupEditor.vue',
  'SavedSearchStrip.vue',
  'AttachmentSearchPanel.css',
  'CommunicationFolderStrip.css',
  'SavedSearchStrip.css'
]

function readComponentArtifact(relativePath: string): string {
  return readFileSync(resolve(componentDir, relativePath), 'utf8')
}

describe('legacy communications search and folder artifacts', () => {
  it('removes the search, folder, and saved-search Vue render layer', () => {
    for (const relativePath of removedSearchAndFolderFiles) {
      expect(existsSync(resolve(componentDir, relativePath))).toBe(false)
    }
  })

  it('preserves attachment search and attachment rendering contracts as TypeScript modules', () => {
    const attachmentSearchTableSource = readComponentArtifact('attachmentSearchTable.ts')
    const attachmentTableSource = readComponentArtifact('attachmentTable.ts')

    expect(attachmentSearchTableSource).toContain('attachmentSearchTableColumns')
    expect(attachmentSearchTableSource).toContain('attachmentSearchTableRowId')
    expect(attachmentTableSource).toContain('attachmentTableColumns')
    expect(attachmentTableSource).toContain('formatAttachmentSize')
    expect(attachmentTableSource).toContain('scanStatusClass')
    expect(attachmentTableSource).toContain('isPreviewableAttachment')
    expect(attachmentTableSource).toContain('isInspectableArchiveAttachment')
  })

  it('preserves drag-drop, folder ordering, and folder presentation logic in TS artifacts', () => {
    const dragDropSource = readComponentArtifact('mailDragDrop.ts')
    const folderPresentationSource = readComponentArtifact('mailFolderPresentation.ts')
    const folderReorderSource = readComponentArtifact('useCommunicationFolderReorder.ts')

    expect(dragDropSource).toContain('MAIL_MESSAGE_DRAG_TYPE')
    expect(dragDropSource).toContain('createCommunicationMessageDragPayload')
    expect(dragDropSource).toContain('parseCommunicationMessageDragPayload')
    expect(folderPresentationSource).toContain('mailFolderColorClass')
    expect(folderPresentationSource).toContain('orderCommunicationFolderDisplayRows')
    expect(folderPresentationSource).toContain('createChildFolderDraft')
    expect(folderPresentationSource).toContain('mailFolderHierarchyDeleteImpact')
    expect(folderReorderSource).toContain('useCommunicationFolderReorder')
    expect(folderReorderSource).toContain('buildCommunicationFolderReorderUpdates')
    expect(folderReorderSource).toContain('mailFolderReorderStatus')
  })

  it('preserves saved-search rule tree helpers after deleting the old builder components', () => {
    const savedSearchTreeSource = readComponentArtifact('savedSearchRuleTreePresentation.ts')

    expect(savedSearchTreeSource).toContain('savedSearchRuleGroupDepthLabel')
    expect(savedSearchTreeSource).toContain('savedSearchRuleGroupSummary')
    expect(savedSearchTreeSource).toContain("group.matchMode === 'all'")
  })
})
