import type { MailFolder } from '../types/folders'

export const MAIL_FOLDER_REORDER_DRAG_TYPE = 'application/x-hermes-mail-folder-reorder'
const SORT_ORDER_STEP = 1000

export type MailFolderReorderPayload = {
  kind: 'mail-folder-reorder'
  folder_id: string
}

export type MailFolderOrderUpdate = {
  folderId: string
  sortOrder: number
}

export function createMailFolderReorderPayload(folderId: string): string {
  return JSON.stringify({
    kind: 'mail-folder-reorder',
    folder_id: folderId.trim()
  } satisfies MailFolderReorderPayload)
}

export function parseMailFolderReorderPayload(value: string): MailFolderReorderPayload | null {
  if (!value.trim()) return null

  try {
    const parsed = JSON.parse(value) as Partial<MailFolderReorderPayload>
    if (parsed.kind !== 'mail-folder-reorder') return null
    if (typeof parsed.folder_id !== 'string' || !parsed.folder_id.trim()) return null
    return {
      kind: 'mail-folder-reorder',
      folder_id: parsed.folder_id.trim()
    }
  } catch {
    return null
  }
}

export function hasMailFolderReorderDragType(types: readonly string[] | DOMStringList): boolean {
  return Array.from(types).includes(MAIL_FOLDER_REORDER_DRAG_TYPE)
}

export function buildMailFolderReorderUpdates(
  folders: Pick<MailFolder, 'folder_id' | 'sort_order'>[],
  sourceFolderId: string,
  targetFolderId: string
): MailFolderOrderUpdate[] {
  const sourceId = sourceFolderId.trim()
  const targetId = targetFolderId.trim()
  if (!sourceId || !targetId || sourceId === targetId) return []

  const sourceIndex = folders.findIndex((folder) => folder.folder_id === sourceId)
  const targetIndex = folders.findIndex((folder) => folder.folder_id === targetId)
  if (sourceIndex < 0 || targetIndex < 0) return []

  const reordered = folders.slice()
  const [source] = reordered.splice(sourceIndex, 1)
  const adjustedTargetIndex = reordered.findIndex((folder) => folder.folder_id === targetId)
  if (!source || adjustedTargetIndex < 0) return []
  reordered.splice(adjustedTargetIndex, 0, source)

  if (sameFolderOrder(folders, reordered)) return []

  const previous = reordered[adjustedTargetIndex - 1] ?? null
  const next = reordered[adjustedTargetIndex + 1] ?? null
  const singleSortOrder = midpointSortOrder(previous?.sort_order ?? null, next?.sort_order ?? null)
  if (singleSortOrder !== null && singleSortOrder !== source.sort_order) {
    return [{ folderId: source.folder_id, sortOrder: singleSortOrder }]
  }

  return reordered.flatMap((folder, index) => {
    const sortOrder = (index + 1) * SORT_ORDER_STEP
    return sortOrder === folder.sort_order ? [] : [{ folderId: folder.folder_id, sortOrder }]
  })
}

export function mailFolderReorderStatus(
  folders: Pick<MailFolder, 'folder_id' | 'name'>[],
  sourceFolderId: string,
  targetFolderId: string
): string {
  const sourceName = folders.find((folder) => folder.folder_id === sourceFolderId.trim())?.name ?? 'folder'
  const targetName = folders.find((folder) => folder.folder_id === targetFolderId.trim())?.name ?? 'folder'
  return `Moved ${sourceName} before ${targetName}`
}

function sameFolderOrder(
  left: Pick<MailFolder, 'folder_id'>[],
  right: Pick<MailFolder, 'folder_id'>[]
): boolean {
  return left.length === right.length && left.every((folder, index) => folder.folder_id === right[index]?.folder_id)
}

function midpointSortOrder(previous: number | null, next: number | null): number | null {
  if (previous === null && next === null) return SORT_ORDER_STEP
  if (previous === null && next !== null) return next > 0 ? next - SORT_ORDER_STEP : null
  if (previous !== null && next === null) return previous + SORT_ORDER_STEP
  if (previous === null || next === null || next - previous <= 1) return null
  return previous + Math.floor((next - previous) / 2)
}
