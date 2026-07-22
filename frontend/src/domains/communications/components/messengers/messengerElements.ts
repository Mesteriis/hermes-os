import type {
  CommunicationTone,
  EntityIconKind,
  MessageDeliveryState,
  MessageDirection,
  ProviderIconKind,
  TreeSelectOption,
  UtilityTone
} from '@/shared/ui'

export type MessengerChannelKind = 'telegram' | 'whatsapp' | 'signal'
export type MessengerConversationKind = 'direct' | 'group' | 'channel'
export type MessengerListItemDensity = 'compact' | 'comfortable' | 'cozy'
export type MessengerWorkflowState = 'needs_action' | 'waiting' | 'reviewed' | 'archived' | 'muted'

export type MessengerListDensityOption = {
  value: MessengerListItemDensity
  label: string
  icon: string
}

export type MessengerListItemModel = {
  id: string
  channelKind: MessengerChannelKind
  accountId?: string
  accountLabel?: string
  conversationKind: MessengerConversationKind
  title: string
  subtitle: string
  preview: string
  timestampLabel: string
  workflowState: MessengerWorkflowState
  unreadCount?: number
  mentionCount?: number
  attachmentCount?: number
  hermesSignalCount?: number
  muted?: boolean
  pinned?: boolean
  selected?: boolean
  labels?: readonly string[]
  profile?: MessengerProfilePreview
}

export type MessengerAvatarStoryItem = {
  id: string
  title: string
  description?: string
  timestampLabel?: string
  tone?: UtilityTone
}

export type MessengerProfilePreview = {
  displayName: string
  fallback?: string
  src?: string
  statusLabel?: string
  storyItems?: readonly MessengerAvatarStoryItem[]
}

export type MessengerAttachmentModel = {
  id: string
  name: string
  meta: string
  icon: string
  tone?: UtilityTone
  downloadable?: boolean
  providerAttachmentId?: string
  providerMessageId?: string
  tdlibFileId?: number
  contentType?: string
}

export type MessengerReactionModel = {
  emoji: string
  count: number
  active?: boolean
}

export type MessengerMessageModel = {
  id: string
  author: string
  body: string
  timestamp: string
  direction: MessageDirection
  tone?: CommunicationTone
  meta?: string
  pending?: boolean
  deliveryStatus?: MessageDeliveryState
  deliveryStatusLabel?: string
  selected?: boolean
  attachments?: readonly MessengerAttachmentModel[]
  reactions?: readonly MessengerReactionModel[]
}

export type MessengerConversationModel = {
  id: string
  channelKind: MessengerChannelKind
  kind: MessengerConversationKind
  title: string
  subtitle: string
  workflowState: MessengerWorkflowState
  participantsLabel: string
  lastSeenLabel?: string
  facts: readonly { label: string; value: string | number; tone?: UtilityTone }[]
  messages: readonly MessengerMessageModel[]
  draftPreview: string
}

export type MessengerInspectorCheck = {
  id: string
  label: string
  description: string
  icon?: string
  tone?: UtilityTone
}

export type MessengerInspectorEntity = {
  id: string
  entity: EntityIconKind
  title: string
  description: string
  evidenceLabel?: string
  tone?: UtilityTone
}

export type MessengerInspectorGroup = {
  id: string
  title: string
  items: readonly MessengerInspectorEntity[]
}

export type MessengerInspectorAction = {
  id: string
  label: string
  description: string
  icon: string
  tone?: UtilityTone
  contract?: string
}

export type MessengerInspectorContext = {
  id: string
  title: string
  description: string
  icon: string
  tone?: UtilityTone
}

export type MessengerInspectorModel = {
  intelligence: {
    score: number
    maxScore: number
    label: string
    summary: string
    checks: readonly MessengerInspectorCheck[]
  }
  entityGroups: readonly MessengerInspectorGroup[]
  suggestedActions: readonly MessengerInspectorAction[]
  relatedContext: readonly MessengerInspectorContext[]
}

export type MessengerStatusPresentation = {
  label: string
  tone: UtilityTone
  icon: string
}

type MessengerTranslate = (key: string) => string

type MessengerAccountSummary = {
  id: string
  label: string
  count: number
}

type MessengerProviderSummary = {
  channelKind: MessengerChannelKind
  count: number
  accounts: MessengerAccountSummary[]
}

export const messengerListItemDensityOptions: readonly MessengerListItemDensity[] = ['compact', 'comfortable', 'cozy']

export const messengerListDensityOptions: readonly MessengerListDensityOption[] = [
  { value: 'compact', label: 'compact', icon: 'tabler:list' },
  { value: 'comfortable', label: 'comfortable', icon: 'tabler:list-details' },
  { value: 'cozy', label: 'cozy', icon: 'tabler:layout-list' }
]

const workflowStatus: Record<MessengerWorkflowState, MessengerStatusPresentation> = {
  needs_action: { label: 'Needs action', tone: 'warning', icon: 'tabler:alert-triangle' },
  waiting: { label: 'Waiting', tone: 'neutral', icon: 'tabler:clock' },
  reviewed: { label: 'Reviewed', tone: 'success', icon: 'tabler:circle-check' },
  archived: { label: 'Archived', tone: 'neutral', icon: 'tabler:archive' },
  muted: { label: 'Muted', tone: 'neutral', icon: 'tabler:bell-off' }
}

const providerOrder: readonly MessengerChannelKind[] = ['telegram', 'whatsapp', 'signal']
const providerIcons: Record<MessengerChannelKind, string> = {
  telegram: 'tabler:brand-telegram',
  whatsapp: 'tabler:brand-whatsapp',
  signal: 'tabler:shield-lock'
}

export function messengerChannelProviderIcon(channelKind: MessengerChannelKind): ProviderIconKind {
  if (channelKind === 'telegram') return 'telegram'
  if (channelKind === 'whatsapp') return 'whatsapp'
  return 'generic'
}

export function messengerChannelLabel(channelKind: MessengerChannelKind): string {
  if (channelKind === 'telegram') return 'Telegram'
  if (channelKind === 'whatsapp') return 'WhatsApp'
  return 'Signal'
}

export function messengerProviderViewId(channelKind: MessengerChannelKind | 'all'): string {
  return `messenger-provider:${channelKind}`
}

export function messengerAccountViewId(channelKind: MessengerChannelKind, accountId: string): string {
  return `messenger-account:${channelKind}:${accountId}`
}

export function messengerAccountId(item: MessengerListItemModel): string {
  return item.accountId ?? item.channelKind
}

export function messengerAccountLabel(item: MessengerListItemModel): string {
  return item.accountLabel ?? messengerChannelLabel(item.channelKind)
}

export function messengerListViewOptions(
  items: readonly MessengerListItemModel[],
  translate: MessengerTranslate
): TreeSelectOption[] {
  const options: TreeSelectOption[] = [
    {
      value: messengerProviderViewId('all'),
      label: messengerViewLabel('All dialogs', items.length, translate),
      icon: 'tabler:messages'
    }
  ]

  for (const provider of messengerProviderSummaries(items)) {
    const children: TreeSelectOption[] = [
      {
        value: messengerProviderViewId(provider.channelKind),
        label: messengerProviderAccountsLabel(provider.channelKind, provider.count, translate),
        icon: 'tabler:users'
      }
    ]

    for (const account of provider.accounts) {
      children.push({
        value: messengerAccountViewId(provider.channelKind, account.id),
        label: messengerViewLabel(account.label, account.count, translate),
        icon: 'tabler:user-circle'
      })
    }

    options.push({
      value: `messenger-provider-group:${provider.channelKind}`,
      label: messengerChannelLabel(provider.channelKind),
      icon: providerIcons[provider.channelKind],
      children
    })
  }

  options.push({
    value: 'saved-filters',
    label: translate('Saved filters'),
    icon: 'tabler:filter-star',
    children: [
      { value: 'messenger-filter:unread', label: translate('Unread'), icon: 'tabler:mail' },
      { value: 'messenger-filter:mentions', label: translate('Mentions'), icon: 'tabler:at' },
      { value: 'messenger-filter:pinned', label: translate('Pinned'), icon: 'tabler:pin' },
      { value: 'messenger-filter:muted', label: translate('Muted'), icon: 'tabler:bell-off' },
      { value: 'messenger-filter:archived', label: translate('Archived'), icon: 'tabler:archive' },
      { value: 'messenger-filter:open-actions', label: translate('Open actions'), icon: 'tabler:alert-triangle' }
    ]
  })

  return options
}

export function messengerConversationKindLabel(kind: MessengerConversationKind): string {
  if (kind === 'direct') return 'Direct messages'
  if (kind === 'group') return 'Groups'
  return 'Channels'
}

export function messengerWorkflowStatusPresentation(state: MessengerWorkflowState): MessengerStatusPresentation {
  return workflowStatus[state]
}

export function messengerViewCounts(items: readonly MessengerListItemModel[]): {
  all: number
  direct: number
  group: number
  channel: number
} {
  const counts = { all: 0, direct: 0, group: 0, channel: 0 }

  for (const item of items) {
    counts.all += 1
    counts[item.conversationKind] += 1
  }

  return counts
}

export function messengerItemsForView(
  items: readonly MessengerListItemModel[],
  viewId: string
): readonly MessengerListItemModel[] {
  if (viewId.startsWith('messenger-provider:')) {
    const channelKind = viewId.replace('messenger-provider:', '')
    if (channelKind === 'all') return items
    if (!isMessengerChannelKind(channelKind)) return items
    return items.filter((item) => item.channelKind === channelKind)
  }

  if (viewId.startsWith('messenger-account:')) {
    const [, rawChannelKind, accountId] = viewId.split(':')
    if (!accountId || !isMessengerChannelKind(rawChannelKind)) return items
    const channelKind = rawChannelKind
    return items.filter((item) => item.channelKind === channelKind && messengerAccountId(item) === accountId)
  }

  if (viewId === 'messenger-filter:mentions') {
    return items.filter((item) => (item.mentionCount ?? 0) > 0)
  }

  if (viewId === 'messenger-filter:unread') {
    return items.filter((item) => (item.unreadCount ?? 0) > 0)
  }

  if (viewId === 'messenger-filter:pinned') {
    return items.filter((item) => item.pinned)
  }

  if (viewId === 'messenger-filter:muted') {
    return items.filter((item) => item.muted)
  }

  if (viewId === 'messenger-filter:archived') {
    return items.filter((item) => item.workflowState === 'archived')
  }

  if (viewId === 'messenger-filter:open-actions') {
    return items.filter((item) => item.workflowState === 'needs_action' || (item.hermesSignalCount ?? 0) > 0)
  }

  const kind = viewId.replace('messenger:', '')
  if (kind === 'all') return items
  if (!isMessengerConversationKind(kind)) return items
  return items.filter((item) => item.conversationKind === kind)
}

function isMessengerChannelKind(value: string | undefined): value is MessengerChannelKind {
  return value === 'telegram' || value === 'whatsapp' || value === 'signal'
}

function isMessengerConversationKind(value: string): value is MessengerConversationKind {
  return value === 'direct' || value === 'group' || value === 'channel'
}

export function messengerItemsForSearch(
  items: readonly MessengerListItemModel[],
  rawQuery: string
): readonly MessengerListItemModel[] {
  const query = rawQuery.trim().toLocaleLowerCase()
  if (!query) return items

  return items.filter((item) => {
    for (const token of messengerSearchTokens(item)) {
      if (token.toLocaleLowerCase().includes(query)) {
        return true
      }
    }

    return false
  })
}

export function messengerListItemHasSignal(item: MessengerListItemModel): boolean {
  return Boolean(item.unreadCount || item.mentionCount || item.hermesSignalCount || item.workflowState === 'needs_action')
}

export function messengerListItemHasSecondarySignals(item: MessengerListItemModel): boolean {
  return Boolean(item.mentionCount || item.attachmentCount || item.hermesSignalCount || item.pinned)
}

export function messengerConversationIsEmpty(conversation: MessengerConversationModel): boolean {
  return conversation.id.endsWith(':empty')
}

export function messengerConversationIsTelegramEmpty(conversation: MessengerConversationModel): boolean {
  return conversation.channelKind === 'telegram' && conversation.id === 'telegram:empty'
}

export function messengerListItemAriaLabel(item: MessengerListItemModel): string {
  const parts = [
    messengerChannelLabel(item.channelKind),
    item.title,
    messengerConversationKindLabel(item.conversationKind),
    item.timestampLabel
  ]

  if (item.unreadCount) {
    parts.push(`${item.unreadCount} unread`)
  }

  if (item.mentionCount) {
    parts.push(`${item.mentionCount} mentions`)
  }

  return parts.join(', ')
}

export function messengerListItemProfile(item: MessengerListItemModel): MessengerProfilePreview {
  return {
    displayName: item.profile?.displayName ?? item.title,
    fallback: item.profile?.fallback ?? messengerProfileFallback(item.title),
    src: item.profile?.src,
    statusLabel: item.profile?.statusLabel,
    storyItems: item.profile?.storyItems ?? []
  }
}

export function messengerMessageAuthor(message: MessengerMessageModel): string | undefined {
  return message.direction === 'system' ? undefined : message.author
}

export function messengerMessageTimestamp(message: MessengerMessageModel): string | undefined {
  return message.direction === 'system' ? undefined : message.timestamp
}

export function messengerMessageMeta(message: MessengerMessageModel): string | undefined {
  return message.direction === 'system' ? undefined : message.meta
}

function messengerSearchTokens(item: MessengerListItemModel): readonly string[] {
  const tokens = [
    item.title,
    item.subtitle,
    item.preview,
    item.channelKind,
    item.conversationKind
  ]

  for (const label of item.labels ?? []) {
    tokens.push(label)
  }

  return tokens.filter(Boolean)
}

function messengerProfileFallback(displayName: string): string {
  const words = displayName.trim().split(/\s+/).filter(Boolean)
  if (words.length >= 2) return `${words[0]?.[0] ?? ''}${words[1]?.[0] ?? ''}`.toUpperCase()
  return displayName.slice(0, 2).toUpperCase()
}

function messengerViewLabel(label: string, count: number, translate: MessengerTranslate): string {
  if (count <= 0) return translate(label)
  return `${translate(label)} ${count}`
}

function messengerProviderAccountsLabel(
  channelKind: MessengerChannelKind,
  count: number,
  translate: MessengerTranslate
): string {
  if (channelKind === 'telegram') return messengerViewLabel('All Telegram accounts', count, translate)
  if (channelKind === 'whatsapp') return messengerViewLabel('All WhatsApp accounts', count, translate)
  return messengerViewLabel('All Signal accounts', count, translate)
}

function messengerProviderSummaries(items: readonly MessengerListItemModel[]): MessengerProviderSummary[] {
  const providers = new Map<MessengerChannelKind, MessengerProviderSummary>()

  for (const item of items) {
    let provider = providers.get(item.channelKind)
    if (!provider) {
      provider = {
        channelKind: item.channelKind,
        count: 0,
        accounts: []
      }
      providers.set(item.channelKind, provider)
    }

    provider.count += 1
    const accountId = messengerAccountId(item)
    const account = provider.accounts.find((candidate) => candidate.id === accountId)
    if (account) {
      account.count += 1
      continue
    }

    provider.accounts.push({
      id: accountId,
      label: messengerAccountLabel(item),
      count: 1
    })
  }

  const summaries = [...providers.values()]
  summaries.sort((left, right) => {
    const leftIndex = providerOrder.indexOf(left.channelKind)
    const rightIndex = providerOrder.indexOf(right.channelKind)
    return leftIndex - rightIndex
  })
  return summaries
}
