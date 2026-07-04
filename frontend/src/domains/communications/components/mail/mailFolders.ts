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
