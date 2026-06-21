import { ref, type ComputedRef } from 'vue'
import type { CommunicationFolder, CommunicationFolderUpdate } from '../types/folders'
import {
  MAIL_FOLDER_REORDER_DRAG_TYPE,
  buildCommunicationFolderReorderUpdates,
  createCommunicationFolderReorderPayload,
  hasCommunicationFolderReorderDragType,
  mailFolderReorderStatus,
  parseCommunicationFolderReorderPayload
} from './mailFolderOrdering'

type UpdateFolder = (variables: { folderId: string; request: CommunicationFolderUpdate }) => Promise<CommunicationFolder>

export function useCommunicationFolderReorder(
  folders: ComputedRef<CommunicationFolder[]>,
  updateFolder: UpdateFolder
) {
  const sourceId = ref('')
  const targetId = ref('')
  const status = ref('')
  const error = ref('')
  const isReordering = ref(false)

  function canHandleDragOver(event: DragEvent): boolean {
    return Boolean(event.dataTransfer && hasCommunicationFolderReorderDragType(event.dataTransfer.types) && !isReordering.value)
  }

  function handleDragStart(event: DragEvent, folder: CommunicationFolder) {
    if (!event.dataTransfer) return
    sourceId.value = folder.folder_id
    status.value = ''
    error.value = ''
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData(MAIL_FOLDER_REORDER_DRAG_TYPE, createCommunicationFolderReorderPayload(folder.folder_id))
  }

  function handleDragEnd() {
    sourceId.value = ''
    targetId.value = ''
  }

  async function handleDrop(event: DragEvent, folder: CommunicationFolder): Promise<boolean> {
    if (!event.dataTransfer || isReordering.value) return false
    const payload = parseCommunicationFolderReorderPayload(event.dataTransfer.getData(MAIL_FOLDER_REORDER_DRAG_TYPE))
    if (!payload) return false

    const updates = buildCommunicationFolderReorderUpdates(folders.value, payload.folder_id, folder.folder_id)
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
