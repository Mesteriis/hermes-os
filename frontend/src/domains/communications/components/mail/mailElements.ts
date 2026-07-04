import type { CommunicationStatusPresentation } from '../communicationDomainElements'
import { communicationWorkflowStatusPresentation } from '../communicationDomainElements'
import type { ProviderIconKind } from '@/shared/ui'

export type MailListItemConfidence = 'high' | 'medium' | 'low'
export type MailListItemCounterKind = 'attachments' | 'messages' | 'insights' | 'calendar'
export type MailListItemDensity = 'compact' | 'comfortable' | 'cozy'
export type MailListItemMarker = 'spam' | 'phishing' | 'important' | 'blocked' | 'archived'
export type MailListSearchField = 'subject' | 'body' | 'sender' | 'all'
export type MailListSearchBuilderField = 'all' | 'from' | 'subject' | 'body'
export type MailListSearchBuilderOperator = 'contains' | 'equals'
export type MailListSearchMatchMode = 'all' | 'any'

type MailListSearchPredicate = {
  field: MailListSearchField
  operator: MailListSearchBuilderOperator
  value: string
}

export type MailListDensityToggleItem = {
  value: MailListItemDensity
  label: string
  icon: string
  iconOnly: true
}

export type MailListAccountOption = {
  id: string
  label: string
  count: number
}

export type MailListSearchBuilderToggleItem = {
  value: string
  label: string
}

export type MailListSearchBuilderClause = {
  id: string
  field: MailListSearchBuilderField
  operator: MailListSearchBuilderOperator
  value: string
}

export type MailListSearchBuilderState = {
  matchMode: MailListSearchMatchMode
  field: MailListSearchBuilderField
  operator: MailListSearchBuilderOperator
  value: string
  clauses: readonly MailListSearchBuilderClause[]
  nextClauseId: number
}

export type MailListSearchBuilderToken = {
  id: string
  value: string
}

export type MailListSearchBuilderClauseView = {
  id: string
  pending: boolean
  tokens: readonly MailListSearchBuilderToken[]
}

export type MailListItemCounter = {
  kind: MailListItemCounterKind
  value: number
}

export type MailListItemModel = {
  id: string
  accountLabel: string
  mailboxLabel: string
  providerRecordId?: string
  fromName: string
  fromAddress?: string
  recipients?: readonly string[]
  subject: string
  snippet: string
  sourceKind?: ProviderIconKind
  timestampLabel: string
  workflowState: string
  localState?: string
  deliveryState?: string
  aiCategory?: string
  importanceScore?: number
  unreadCount?: number
  attachmentCount?: number
  confidence?: MailListItemConfidence
  counters?: readonly MailListItemCounter[]
  labels?: readonly string[]
  hermesEntities?: readonly {
    kind: string
    title: string
  }[]
  evidenceKinds?: readonly string[]
  taskCandidateCount?: number
  decisionCandidateCount?: number
  documentCandidateCount?: number
  deadlineCount?: number
  riskCount?: number
  hasOpenAction?: boolean
  markers?: readonly MailListItemMarker[]
  muted?: boolean
  selected?: boolean
  timelineHint?: string
}

export type MailListItemCounterPresentation = {
  icon: string
  label: string
}

export type MailListItemMarkerPresentation = {
  icon: string
  label: string
  tone: 'default' | 'accent' | 'success' | 'warning' | 'danger' | 'info' | 'neutral'
}

export const mailListItemDensityOptions: readonly MailListItemDensity[] = ['compact', 'comfortable', 'cozy']

export const mailListAllAccountsOptionId = 'all'

export const mailListDensityToggleItems: readonly MailListDensityToggleItem[] = [
  { value: 'compact', label: 'compact', icon: 'tabler:list', iconOnly: true },
  { value: 'comfortable', label: 'comfortable', icon: 'tabler:list-details', iconOnly: true },
  { value: 'cozy', label: 'cozy', icon: 'tabler:layout-list', iconOnly: true }
]

export const mailListSearchPlaceholder = 'Search sender, subject, body or provider id'

export const mailListSearchFieldItems: readonly MailListSearchBuilderToggleItem[] = [
  { value: 'from', label: 'from' },
  { value: 'subject', label: 'subject' },
  { value: 'body', label: 'body' },
  { value: 'all', label: 'all' }
]

export const mailListSearchOperatorItems: readonly MailListSearchBuilderToggleItem[] = [
  { value: 'contains', label: 'contains' },
  { value: 'equals', label: 'equals' }
]

export const mailListSearchMatchModeItems: readonly MailListSearchBuilderToggleItem[] = [
  { value: 'all', label: 'all' },
  { value: 'any', label: 'any' }
]

const counterPresentation: Record<MailListItemCounterKind, MailListItemCounterPresentation> = {
  attachments: {
    icon: 'tabler:paperclip',
    label: 'Attachments'
  },
  messages: {
    icon: 'tabler:messages',
    label: 'Messages'
  },
  insights: {
    icon: 'tabler:brain',
    label: 'Hermes insights'
  },
  calendar: {
    icon: 'tabler:calendar-event',
    label: 'Linked events'
  }
}

const markerPresentation: Record<MailListItemMarker, MailListItemMarkerPresentation> = {
  spam: {
    icon: 'tabler:mail-x',
    label: 'Spam',
    tone: 'warning'
  },
  phishing: {
    icon: 'tabler:shield-exclamation',
    label: 'Phishing',
    tone: 'danger'
  },
  important: {
    icon: 'tabler:star-filled',
    label: 'Important',
    tone: 'accent'
  },
  blocked: {
    icon: 'tabler:user-off',
    label: 'Blocked sender',
    tone: 'danger'
  },
  archived: {
    icon: 'tabler:archive',
    label: 'Archived',
    tone: 'neutral'
  }
}

const signalMarkers = new Set<MailListItemMarker>(['spam', 'phishing', 'blocked'])

export function mailListItemStatus(item: MailListItemModel): CommunicationStatusPresentation {
  return communicationWorkflowStatusPresentation(item.workflowState)
}

export function mailListItemStatusClass(item: MailListItemModel): string {
  return `mail-list-item__status-dot--${mailListItemStatus(item).badgeTone}`
}

export function mailListItemHasSignal(item: MailListItemModel): boolean {
  return Boolean(
    item.hasOpenAction ||
    item.unreadCount ||
    mailListItemMarkers(item).some((marker) => signalMarkers.has(marker))
  )
}

export function mailListItemAttachmentLabel(item: MailListItemModel): string {
  const attachmentCount = item.attachmentCount ?? 0
  return attachmentCount === 1 ? '1 file' : `${attachmentCount} files`
}

export function mailListItemAriaLabel(item: MailListItemModel): string {
  return `${item.fromName}: ${item.subject}`
}

export function mailListItemCounterPresentation(
  counter: MailListItemCounter
): MailListItemCounterPresentation {
  return counterPresentation[counter.kind]
}

export function mailListItemCounters(item: MailListItemModel): readonly MailListItemCounter[] {
  const counters = [...(item.counters ?? [])]

  if (item.attachmentCount && !counters.some((counter) => counter.kind === 'attachments')) {
    counters.unshift({
      kind: 'attachments',
      value: item.attachmentCount
    })
  }

  return counters.filter((counter) => counter.value > 0)
}

export function mailListItemMarkerPresentation(marker: MailListItemMarker): MailListItemMarkerPresentation {
  return markerPresentation[marker]
}

export function mailListItemMarkerClass(marker: MailListItemMarker): string {
  return `mail-marker-icon--${marker}`
}

export function mailListItemMarkers(item: MailListItemModel): readonly MailListItemMarker[] {
  return item.markers ?? []
}

export function mailListItemMarkerSummary(item: MailListItemModel): string {
  const markers = mailListItemMarkers(item)
  if (markers.length === 0) return 'No markers'
  return markers.map((marker) => mailListItemMarkerPresentation(marker).label).join(', ')
}

export function mailListItemSourceKind(item: MailListItemModel): ProviderIconKind {
  return item.sourceKind ?? 'mail'
}

export function mailListAccountOptions(items: readonly MailListItemModel[]): readonly MailListAccountOption[] {
  const counts = new Map<string, number>()

  for (const item of items) {
    counts.set(item.accountLabel, (counts.get(item.accountLabel) ?? 0) + 1)
  }

  const accountOptions = Array.from(counts.entries())
    .sort(([first], [second]) => first.localeCompare(second))
    .map(([label, count]) => ({
      id: mailListAccountOptionId(label),
      label,
      count
    }))

  return [
    {
      id: mailListAllAccountsOptionId,
      label: 'All accounts',
      count: items.length
    },
    ...accountOptions
  ]
}

export function mailListItemsForAccount(
  items: readonly MailListItemModel[],
  accountOptionId: string
): readonly MailListItemModel[] {
  if (accountOptionId === mailListAllAccountsOptionId) return items

  const accountLabel = mailListAccountLabelFromOptionId(accountOptionId)
  if (!accountLabel) return items

  return items.filter((item) => item.accountLabel === accountLabel)
}

export function createMailListSearchBuilderState(): MailListSearchBuilderState {
  return {
    matchMode: 'all',
    field: 'from',
    operator: 'contains',
    value: '',
    clauses: [],
    nextClauseId: 1
  }
}

function mailListAccountOptionId(accountLabel: string): string {
  return `account:${encodeURIComponent(accountLabel)}`
}

function mailListAccountLabelFromOptionId(accountOptionId: string): string | null {
  if (!accountOptionId.startsWith('account:')) return null
  return decodeURIComponent(accountOptionId.slice('account:'.length))
}

export function mailListSearchBuilderSetMatchMode(
  state: MailListSearchBuilderState,
  value: string | readonly string[]
): MailListSearchBuilderState {
  if (value !== 'all' && value !== 'any') return state

  return { ...state, matchMode: value }
}

export function mailListSearchBuilderSetField(
  state: MailListSearchBuilderState,
  value: string | readonly string[]
): MailListSearchBuilderState {
  if (!isMailListSearchBuilderField(value)) return state

  return { ...state, field: value }
}

export function mailListSearchBuilderSetOperator(
  state: MailListSearchBuilderState,
  value: string | readonly string[]
): MailListSearchBuilderState {
  if (value !== 'contains' && value !== 'equals') return state

  return { ...state, operator: value }
}

export function mailListSearchBuilderSetValue(
  state: MailListSearchBuilderState,
  value: string
): MailListSearchBuilderState {
  return { ...state, value }
}

export function mailListSearchBuilderCanAdd(state: MailListSearchBuilderState): boolean {
  return Boolean(state.value.trim())
}

export function mailListSearchBuilderCanApply(state: MailListSearchBuilderState): boolean {
  return mailListSearchBuilderEffectiveClauses(state).length > 0
}

export function mailListSearchBuilderAddClause(state: MailListSearchBuilderState): MailListSearchBuilderState {
  const clause = mailListSearchBuilderActiveClause(state)
  if (!clause) return state

  return {
    ...state,
    value: '',
    nextClauseId: state.nextClauseId + 1,
    clauses: [...state.clauses, clause]
  }
}

export function mailListSearchBuilderRemoveClause(
  state: MailListSearchBuilderState,
  clauseId: string
): MailListSearchBuilderState {
  return {
    ...state,
    clauses: state.clauses.filter((clause) => clause.id !== clauseId)
  }
}

export function mailListSearchBuilderClear(): MailListSearchBuilderState {
  return createMailListSearchBuilderState()
}

export function mailListSearchBuilderQuery(state: MailListSearchBuilderState): string {
  const clauses = mailListSearchBuilderEffectiveClauses(state)
  if (clauses.length === 0) return ''

  const tokens = clauses.map(mailListSearchBuilderClauseQuery)
  if (state.matchMode === 'any') {
    tokens.unshift('mode:any')
  }

  return tokens.join(' ')
}

export function mailListSearchBuilderClauseViews(
  state: MailListSearchBuilderState
): readonly MailListSearchBuilderClauseView[] {
  const activeClause = mailListSearchBuilderActiveClause(state)
  const clauseViews = state.clauses.map((clause) => mailListSearchBuilderClauseView(clause, false))

  if (!activeClause) return clauseViews

  return [...clauseViews, mailListSearchBuilderClauseView({ ...activeClause, id: 'pending' }, true)]
}

export function mailListSearchBuilderCommittedClauseViews(
  state: MailListSearchBuilderState
): readonly MailListSearchBuilderClauseView[] {
  return state.clauses.map((clause) => mailListSearchBuilderClauseView(clause, false))
}

export function mailListSearchBuilderDraftTokens(
  state: MailListSearchBuilderState
): readonly MailListSearchBuilderToken[] {
  return [
    { id: 'draft-field', value: state.field },
    { id: 'draft-operator', value: state.operator }
  ]
}

export function mailListItemsForSearch(
  items: readonly MailListItemModel[],
  rawQuery: string
): readonly MailListItemModel[] {
  const parsed = parseMailListSearchQuery(rawQuery)
  if (parsed.predicates.length === 0) return items

  return items.filter((item) => {
    const results = parsed.predicates.map((predicate) => mailListItemMatchesSearchPredicate(item, predicate))
    return parsed.matchMode === 'any' ? results.some(Boolean) : results.every(Boolean)
  })
}

function parseMailListSearchQuery(rawQuery: string): {
  matchMode: MailListSearchMatchMode
  predicates: MailListSearchPredicate[]
} {
  let matchMode: MailListSearchMatchMode = 'all'
  const predicates: MailListSearchPredicate[] = []

  for (const token of tokenizeMailListSearchQuery(rawQuery)) {
    const mode = parseMailListSearchMode(token)
    if (mode) {
      matchMode = mode
      continue
    }

    const fieldPredicate = parseMailListSearchFieldPredicate(token)
    if (fieldPredicate) {
      predicates.push(fieldPredicate)
      continue
    }

    const value = stripSearchQuotes(token)
    if (value) {
      predicates.push({ field: 'all', operator: 'contains', value })
    }
  }

  return { matchMode, predicates }
}

function tokenizeMailListSearchQuery(rawQuery: string): string[] {
  return rawQuery
    .match(/(?:[^\s"']+(?:==|=|:)(?:"[^"]*"|'[^']*'|[^\s]+))|(?:"[^"]*"|'[^']*'|[^\s]+)/g)
    ?.map((token) => token.trim())
    .filter(Boolean) ?? []
}

function parseMailListSearchMode(token: string): MailListSearchMatchMode | null {
  const [, value] = /^mode:(all|any)$/i.exec(token) ?? []
  return value ? (value.toLowerCase() as MailListSearchMatchMode) : null
}

function parseMailListSearchFieldPredicate(token: string): MailListSearchPredicate | null {
  const match = /^([a-z]+)(==|=|:)(.+)$/i.exec(token)
  if (!match) return null

  const [, rawField, rawOperator, rawValue] = match
  if (!rawField || !rawOperator || !rawValue) return null

  const field = parseMailListSearchField(rawField)
  const value = stripSearchQuotes(rawValue)
  if (!field || !value) return null

  return {
    field,
    operator: rawOperator === ':' ? 'contains' : 'equals',
    value
  }
}

function parseMailListSearchField(field: string): MailListSearchField | null {
  switch (field.toLowerCase()) {
    case 'subject':
      return 'subject'
    case 'body':
      return 'body'
    case 'sender':
    case 'from':
      return 'sender'
    case 'all':
      return 'all'
    default:
      return null
  }
}

function stripSearchQuotes(value: string): string {
  const trimmed = value.trim()
  if (trimmed.length < 2) return trimmed

  const first = trimmed[0]
  const last = trimmed[trimmed.length - 1]
  if ((first === '"' && last === '"') || (first === "'" && last === "'")) {
    return trimmed.slice(1, -1).trim()
  }

  return trimmed
}

function mailListItemMatchesSearchPredicate(item: MailListItemModel, predicate: MailListSearchPredicate): boolean {
  const targets = mailListSearchTargets(item, predicate.field)
  const needle = normalizeMailListSearchValue(predicate.value)

  if (!needle) return true
  if (predicate.operator === 'equals') {
    return targets.some((target) => target === needle)
  }
  return targets.join(' ').includes(needle)
}

function mailListSearchTargets(item: MailListItemModel, field: MailListSearchField): string[] {
  switch (field) {
    case 'subject':
      return normalizeMailListSearchValues([item.subject])
    case 'body':
      return normalizeMailListSearchValues([item.snippet])
    case 'sender':
      return normalizeMailListSearchValues([item.fromName, item.fromAddress])
    case 'all':
      return normalizeMailListSearchValues([
        item.subject,
        item.fromName,
        item.fromAddress,
        item.snippet,
        item.id
      ])
  }
}

function normalizeMailListSearchValues(values: Array<string | undefined>): string[] {
  return values
    .filter((value): value is string => Boolean(value))
    .map(normalizeMailListSearchValue)
    .filter(Boolean)
}

function normalizeMailListSearchValue(value: string): string {
  return value.trim().toLowerCase()
}

function isMailListSearchBuilderField(value: string | readonly string[]): value is MailListSearchBuilderField {
  return value === 'all' || value === 'from' || value === 'subject' || value === 'body'
}

function mailListSearchBuilderActiveClause(
  state: MailListSearchBuilderState
): MailListSearchBuilderClause | null {
  const value = state.value.trim()
  if (!value) return null

  return {
    id: `clause-${state.nextClauseId}`,
    field: state.field,
    operator: state.operator,
    value
  }
}

function mailListSearchBuilderEffectiveClauses(
  state: MailListSearchBuilderState
): readonly MailListSearchBuilderClause[] {
  const activeClause = mailListSearchBuilderActiveClause(state)
  if (!activeClause) return state.clauses

  return [...state.clauses, activeClause]
}

function mailListSearchBuilderClauseView(
  clause: MailListSearchBuilderClause,
  pending: boolean
): MailListSearchBuilderClauseView {
  return {
    id: clause.id,
    pending,
    tokens: [
      { id: `${clause.id}-field`, value: clause.field },
      { id: `${clause.id}-operator`, value: clause.operator },
      { id: `${clause.id}-value`, value: clause.value }
    ]
  }
}

function mailListSearchBuilderClauseQuery(clause: MailListSearchBuilderClause): string {
  const field = clause.field
  const operator = clause.operator === 'contains' ? ':' : '='
  return `${field}${operator}${quoteMailListSearchValue(clause.value)}`
}

function quoteMailListSearchValue(value: string): string {
  const trimmed = value.trim()
  if (!/\s/.test(trimmed)) return trimmed
  if (!trimmed.includes('"')) return `"${trimmed}"`
  if (!trimmed.includes("'")) return `'${trimmed}'`
  return `"${trimmed.replaceAll('"', '')}"`
}
