import type { UtilityTone } from '@/shared/ui'

export type MailFolderKind =
  | 'archive'
  | 'all'
  | 'custom'
  | 'drafts'
  | 'inbox'
  | 'outbox'
  | 'sent'
  | 'spam'
  | 'trash'

export type MailFolderModel = {
  id: string
  kind: MailFolderKind
  label: string
  count?: number
  unreadCount?: number
  selected?: boolean
  children?: readonly MailFolderModel[]
}

export type MailFolderPresentation = {
  icon: string
  tone: UtilityTone
}

export type MailFolderRow = {
  folder: MailFolderModel
  depth: number
  hasChildren: boolean
  expanded: boolean
}

type Translate = (key: string, params?: Record<string, string | number>) => string

const folderPresentation: Record<MailFolderKind, MailFolderPresentation> = {
  archive: {
    icon: 'tabler:archive',
    tone: 'neutral'
  },
  all: {
    icon: 'tabler:mailbox',
    tone: 'info'
  },
  custom: {
    icon: 'tabler:folder',
    tone: 'neutral'
  },
  drafts: {
    icon: 'tabler:file-pencil',
    tone: 'warning'
  },
  inbox: {
    icon: 'tabler:inbox',
    tone: 'accent'
  },
  outbox: {
    icon: 'tabler:send-2',
    tone: 'warning'
  },
  sent: {
    icon: 'tabler:send',
    tone: 'success'
  },
  spam: {
    icon: 'tabler:mail-x',
    tone: 'danger'
  },
  trash: {
    icon: 'tabler:trash',
    tone: 'neutral'
  }
}

export const mailStandardFolders: readonly MailFolderModel[] = [
  { id: 'inbox', kind: 'inbox', label: 'Inbox', selected: true },
  { id: 'sent', kind: 'sent', label: 'Sent' },
  { id: 'drafts', kind: 'drafts', label: 'Drafts' },
  { id: 'outbox', kind: 'outbox', label: 'Outbox' },
  { id: 'archive', kind: 'archive', label: 'Archive' },
  { id: 'spam', kind: 'spam', label: 'Spam' },
  { id: 'trash', kind: 'trash', label: 'Trash' },
  { id: 'all', kind: 'all', label: 'All mail' }
]

export function mailFolderPresentation(folder: MailFolderModel): MailFolderPresentation {
  return folderPresentation[folder.kind]
}

export function mailFolderIsActive(folder: MailFolderModel, activeFolderId?: string): boolean {
  return activeFolderId ? activeFolderId === folder.id : Boolean(folder.selected)
}

export function mailFolderDepthClass(row: MailFolderRow): string {
  return `mail-folder-list__item--depth-${Math.min(row.depth, 4)}`
}

export function mailFolderToggleAriaLabel(row: MailFolderRow, t: Translate): string {
  const action = row.expanded ? t('Collapse folder') : t('Expand folder')
  return `${action}: ${t(row.folder.label)}`
}

export function mailFolderLocalizedAriaLabel(folder: MailFolderModel, t: Translate): string {
  const parts = [t(folder.label)]
  if (folder.unreadCount) parts.push(t('{count} unread', { count: folder.unreadCount }))
  if (typeof folder.count === 'number') parts.push(t('{count} total', { count: folder.count }))
  return parts.join(', ')
}

export function mailFolderExpandableIds(folders: readonly MailFolderModel[]): readonly string[] {
  return folders.flatMap((folder) => mailFolderExpandableIdsFromFolder(folder))
}

export function mailFolderExpandedIds(
  expandedFolderIds: readonly string[],
  folderId: string,
  expanded: boolean
): readonly string[] {
  if (expanded) {
    return expandedFolderIds.includes(folderId) ? expandedFolderIds : [...expandedFolderIds, folderId]
  }
  return expandedFolderIds.filter((expandedFolderId) => expandedFolderId !== folderId)
}

export function mailFolderRows(
  folders: readonly MailFolderModel[],
  expandedFolderIds: readonly string[] = mailFolderExpandableIds(folders)
): readonly MailFolderRow[] {
  return folders.flatMap((folder) => mailFolderRowsFromFolder(folder, 1, expandedFolderIds))
}

export function mailFolderAriaLabel(folder: MailFolderModel): string {
  const unread = folder.unreadCount ? `, ${folder.unreadCount} unread` : ''
  const count = typeof folder.count === 'number' ? `, ${folder.count} total` : ''
  return `${folder.label}${unread}${count}`
}

function mailFolderExpandableIdsFromFolder(folder: MailFolderModel): readonly string[] {
  const children = folder.children ?? []
  if (!children.length) return []
  return [folder.id, ...children.flatMap((child) => mailFolderExpandableIdsFromFolder(child))]
}

function mailFolderRowsFromFolder(
  folder: MailFolderModel,
  depth: number,
  expandedFolderIds: readonly string[]
): readonly MailFolderRow[] {
  const children = folder.children ?? []
  const hasChildren = children.length > 0
  const expanded = hasChildren && expandedFolderIds.includes(folder.id)
  return [
    {
      folder,
      depth,
      hasChildren,
      expanded
    },
    ...(expanded ? children.flatMap((child) => mailFolderRowsFromFolder(child, depth + 1, expandedFolderIds)) : [])
  ]
}
