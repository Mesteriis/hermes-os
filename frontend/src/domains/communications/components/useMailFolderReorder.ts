import { ref, type ComputedRef } from 'vue'
import type { MailFolder, MailFolderUpdate } from '../types/folders'
import {
  MAIL_FOLDER_REORDER_DRAG_TYPE,
  buildMailFolderReorderUpdates,
  createMailFolderReorderPayload,
  hasMailFolderReorderDragType,
  mailFolderReorderStatus,
  parseMailFolderReorderPayload
} from './mailFolderOrdering'

type UpdateFolder = (variables: { folderId: string; request: MailFolderUpdate }) => Promise<MailFolder>

export function useMailFolderReorder(
  folders: ComputedRef<MailFolder[]>,
  updateFolder: UpdateFolder
) {
  const sourceId = ref('')
  const targetId = ref('')
  const status = ref('')
  const error = ref('')
  const isReordering = ref(false)

  function canHandleDragOver(event: DragEvent): boolean {
    return Boolean(event.dataTransfer && hasMailFolderReorderDragType(event.dataTransfer.types) && !isReordering.value)
  }

  function handleDragStart(event: DragEvent, folder: MailFolder) {
    if (!event.dataTransfer) return
    sourceId.value = folder.folder_id
    status.value = ''
    error.value = ''
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData(MAIL_FOLDER_REORDER_DRAG_TYPE, createMailFolderReorderPayload(folder.folder_id))
  }

  function handleDragEnd() {
    sourceId.value = ''
    targetId.value = ''
  }

  async function handleDrop(event: DragEvent, folder: MailFolder): Promise<boolean> {
    if (!event.dataTransfer || isReordering.value) return false
    const payload = parseMailFolderReorderPayload(event.dataTransfer.getData(MAIL_FOLDER_REORDER_DRAG_TYPE))
    if (!payload) return false

    const updates = buildMailFolderReorderUpdates(folders.value, payload.folder_id, folder.folder_id)
    if (updates.length === 0) return true

    targetId.value = folder.folder_id
    status.value = ''
    error.value = ''
    isReordering.value = true
    try {
      for (const update of updates) {
        await updateFolder({ folderId: update.folderId, request: { sort_order: update.sortOrder } })
      }
      status.value = mailFolderReorderStatus(folders.value, payload.folder_id, folder.folder_id)
      return true
    } catch (caught) {
      error.value = caught instanceof Error ? caught.message : 'Folder reorder failed'
      return true
    } finally {
      isReordering.value = false
      sourceId.value = ''
      targetId.value = ''
    }
  }

  return {
    canHandleDragOver,
    error,
    handleDragEnd,
    handleDragStart,
    handleDrop,
    isReordering,
    sourceId,
    status,
    targetId
  }
}
