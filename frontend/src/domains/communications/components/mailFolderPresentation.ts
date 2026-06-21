import type { CommunicationFolder } from '../types/folders'

export type CommunicationFolderDisplayRow = {
  folder: CommunicationFolder
  depth: number
  leafName: string
  pathPrefix: string
  pathParts: string[]
}

export type CommunicationFolderHierarchyDeleteImpact = {
  descendantCount: number
  descendantLeafNames: string[]
}

export function mailFolderColorClass(color: string | null): string {
  switch (color?.toLowerCase()) {
    case '#10b981':
      return 'mail-folder-color--green'
    case '#f59e0b':
      return 'mail-folder-color--amber'
    case '#ef4444':
      return 'mail-folder-color--red'
    case '#8b5cf6':
      return 'mail-folder-color--violet'
    default:
      return 'mail-folder-color--blue'
  }
}

export function deriveCommunicationFolderDisplayRow(folder: CommunicationFolder): CommunicationFolderDisplayRow {
  const parts = folder.name
    .split('/')
    .map((part) => part.trim())
    .filter(Boolean)

  const normalizedParts = parts.length ? parts : [folder.name.trim()]
  const leafName = normalizedParts[normalizedParts.length - 1] || folder.name.trim()

  return {
    folder,
    depth: Math.max(0, normalizedParts.length - 1),
    leafName,
    pathPrefix: normalizedParts.slice(0, -1).join(' / '),
    pathParts: normalizedParts
  }
}

export function orderCommunicationFolderDisplayRows(folders: ReadonlyArray<CommunicationFolder>): CommunicationFolderDisplayRow[] {
  return folders
    .map((folder) => deriveCommunicationFolderDisplayRow(folder))
    .sort(compareCommunicationFolderRows)
}

export function createChildFolderDraft(folder: CommunicationFolder): {
  parentPath: string
  sortOrder: number
} {
  return {
    parentPath: folder.name,
    sortOrder: folder.sort_order
  }
}

export function mailFolderHierarchyDeleteImpact(
  folders: ReadonlyArray<CommunicationFolder>,
  folderId: string
): CommunicationFolderHierarchyDeleteImpact {
  const rows = orderCommunicationFolderDisplayRows(folders)
  const target = rows.find((row) => row.folder.folder_id === folderId)
  if (!target) {
    return {
      descendantCount: 0,
      descendantLeafNames: []
    }
  }

  const descendants = rows.filter((row) =>
    row.folder.folder_id !== folderId && isDescendantPath(target.pathParts, row.pathParts)
  )

  return {
    descendantCount: descendants.length,
    descendantLeafNames: descendants.slice(0, 3).map((row) => row.leafName)
  }
}

function compareCommunicationFolderRows(left: CommunicationFolderDisplayRow, right: CommunicationFolderDisplayRow): number {
  if (left.folder.sort_order !== right.folder.sort_order) {
    return left.folder.sort_order - right.folder.sort_order
  }

  const segmentCount = Math.min(left.pathParts.length, right.pathParts.length)
  for (let index = 0; index < segmentCount; index += 1) {
    const comparison = left.pathParts[index].localeCompare(right.pathParts[index], undefined, {
      sensitivity: 'base'
    })
    if (comparison !== 0) return comparison
  }

  if (left.pathParts.length !== right.pathParts.length) {
    return left.pathParts.length - right.pathParts.length
  }

  return left.folder.folder_id.localeCompare(right.folder.folder_id)
}

function isDescendantPath(parentPathParts: string[], candidatePathParts: string[]): boolean {
  if (candidatePathParts.length <= parentPathParts.length) return false
  return parentPathParts.every((part, index) => (
    part.localeCompare(candidatePathParts[index] ?? '', undefined, { sensitivity: 'base' }) === 0
  ))
}
