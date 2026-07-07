import type { TreeSelectOption } from '@/shared/ui'
import type { MailListItemModel } from './mailElements'

export type MailListViewId =
  | 'mail:all'
  | 'mail:inbox'
  | 'mail:new'
  | 'mail:needs-action'
  | 'mail:waiting'
  | 'mail:done'
  | 'mail:spam'
  | 'mail:archived'
  | 'mail:muted'
  | 'mail:trash'
  | 'mail:sent'
  | 'mail:drafts'
  | 'mail:other'

type MailListViewCounts = Record<MailListViewId, number>

type MailListViewOption = {
  id: MailListViewId
  label: string
  icon: string
}

export const mailListViewOptions: readonly MailListViewOption[] = [
  { id: 'mail:all', label: 'All mail', icon: 'tabler:mail' },
  { id: 'mail:inbox', label: 'Inbox', icon: 'tabler:inbox' },
  { id: 'mail:new', label: 'New', icon: 'tabler:mail-opened' },
  {
    id: 'mail:needs-action',
    label: 'Needs action',
    icon: 'tabler:alert-triangle',
  },
  { id: 'mail:waiting', label: 'Waiting', icon: 'tabler:clock-pause' },
  { id: 'mail:done', label: 'Done', icon: 'tabler:circle-check' },
  { id: 'mail:spam', label: 'Spam', icon: 'tabler:mail-x' },
  { id: 'mail:archived', label: 'Archived', icon: 'tabler:archive' },
  { id: 'mail:muted', label: 'Muted', icon: 'tabler:volume-off' },
  { id: 'mail:trash', label: 'Trash', icon: 'tabler:trash' },
  { id: 'mail:sent', label: 'Sent', icon: 'tabler:send' },
  { id: 'mail:drafts', label: 'Drafts', icon: 'tabler:file-pencil' },
  { id: 'mail:other', label: 'Other', icon: 'tabler:folder' },
]

const mailListViewIdSet = new Set<string>(
  mailListViewOptions.map((option) => option.id)
)

export function mailListTreeSelectOptions(
  items: readonly MailListItemModel[],
  savedFilterOptions: readonly TreeSelectOption[],
  translate: (label: string) => string,
  hasMoreItems = false
): TreeSelectOption[] {
  const counts = mailListViewCounts(items)

  return [
    {
      value: 'mailboxes',
      label: translate('Mailboxes'),
      icon: 'tabler:mailbox',
      children: mailListViewOptions.map((option) => ({
        value: option.id,
        label: mailListViewLabel(
          option.label,
          counts[option.id],
          translate,
          hasMoreItems
        ),
        icon: option.icon,
      })),
    },
    {
      value: 'saved-filters',
      label: translate('Saved filters'),
      icon: 'tabler:filter-star',
      children: [...savedFilterOptions],
    },
  ]
}

export function mailListItemsForView(
  items: readonly MailListItemModel[],
  viewId: string
): readonly MailListItemModel[] {
  if (!isMailListViewId(viewId) || viewId === 'mail:all') return items
  return items.filter((item) => mailListItemBelongsToView(item, viewId))
}

export function mailListItemIds(
  items: readonly MailListItemModel[]
): string[] {
  const ids: string[] = []
  for (const item of items) {
    ids.push(item.id)
  }
  return ids
}

export function isMailListViewId(value: string): value is MailListViewId {
  return mailListViewIdSet.has(value)
}

function mailListViewLabel(
  label: string,
  count: number,
  translate: (label: string) => string,
  hasMoreItems: boolean
): string {
  if (count <= 0) return translate(label)
  const suffix = hasMoreItems ? '+' : ''
  return `${translate(label)} ${count}${suffix}`
}

function mailListViewCounts(
  items: readonly MailListItemModel[]
): MailListViewCounts {
  const counts = Object.fromEntries(
    mailListViewOptions.map((option) => [option.id, 0])
  ) as MailListViewCounts

  for (const item of items) {
    for (const option of mailListViewOptions) {
      if (mailListItemBelongsToView(item, option.id)) {
        counts[option.id] += 1
      }
    }
  }

  return counts
}

function mailListItemBelongsToView(
  item: MailListItemModel,
  viewId: MailListViewId
): boolean {
  switch (viewId) {
    case 'mail:all':
      return true
    case 'mail:inbox':
      return mailListItemBelongsToInbox(item)
    case 'mail:new':
      return item.workflowState === 'new'
    case 'mail:needs-action':
      return item.workflowState === 'needs_action'
    case 'mail:waiting':
      return item.workflowState === 'waiting'
    case 'mail:done':
      return item.workflowState === 'done'
    case 'mail:spam':
      return mailListItemBelongsToSpam(item)
    case 'mail:archived':
      return mailListItemBelongsToArchived(item)
    case 'mail:muted':
      return mailListItemBelongsToMuted(item)
    case 'mail:trash':
      return item.localState === 'trash' || item.mailboxLabel === 'Trash'
    case 'mail:sent':
      return item.deliveryState === 'sent' || item.mailboxLabel === 'Sent'
    case 'mail:drafts':
      return item.mailboxLabel === 'Drafts'
    case 'mail:other':
      return mailListItemBelongsToOther(item)
  }
}

function mailListItemBelongsToInbox(item: MailListItemModel): boolean {
  return (
    (item.localState ?? 'active') === 'active' &&
    item.mailboxLabel === 'Inbox' &&
    item.deliveryState !== 'sent' &&
    !mailListItemBelongsToSpam(item) &&
    !mailListItemBelongsToArchived(item) &&
    !mailListItemBelongsToMuted(item) &&
    item.workflowState !== 'done'
  )
}

function mailListItemBelongsToSpam(item: MailListItemModel): boolean {
  return (
    item.workflowState === 'spam' || item.markers?.includes('spam') === true
  )
}

function mailListItemBelongsToArchived(item: MailListItemModel): boolean {
  return (
    item.workflowState === 'archived' ||
    item.localState === 'archived' ||
    item.markers?.includes('archived') === true
  )
}

function mailListItemBelongsToMuted(item: MailListItemModel): boolean {
  return item.workflowState === 'muted' || item.muted === true
}

function mailListItemBelongsToOther(item: MailListItemModel): boolean {
  return (
    !mailListItemBelongsToInbox(item) &&
    item.deliveryState !== 'sent' &&
    item.mailboxLabel !== 'Sent' &&
    item.mailboxLabel !== 'Drafts' &&
    item.localState !== 'trash' &&
    item.mailboxLabel !== 'Trash' &&
    !mailListItemBelongsToSpam(item) &&
    !mailListItemBelongsToArchived(item) &&
    !mailListItemBelongsToMuted(item)
  )
}
