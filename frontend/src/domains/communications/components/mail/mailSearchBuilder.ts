import {
  mailListItemCounters,
  mailListItemMarkers,
  type MailListItemCounterKind,
  type MailListItemMarker,
  type MailListItemModel
} from './mailElements'

export type MailListSearchField =
  | 'account' | 'ai_category' | 'all' | 'attachment' | 'body' | 'decision' | 'deadline'
  | 'delivery' | 'document' | 'entity' | 'evidence' | 'from' | 'importance' | 'label'
  | 'local' | 'mailbox' | 'marker' | 'provider' | 'recipients' | 'risk' | 'subject'
  | 'task' | 'unread' | 'workflow' | 'action'

export type MailListSearchBuilderField = MailListSearchField
export type MailListSearchBuilderOperator = 'contains' | 'equals' | 'is' | 'is_not' | 'has' | 'without' | 'gte' | 'lte'
export type MailListSearchMatchMode = 'all' | 'any'

type MailListSearchPredicate = { field: MailListSearchField; operator: MailListSearchBuilderOperator; value: string }
export type MailListSearchBuilderToggleItem = { value: string; label: string }
export type MailListSearchBuilderFieldItem = {
  value: MailListSearchBuilderField
  label: string
  operators: readonly MailListSearchBuilderOperator[]
  placeholder: string
  presets?: readonly MailListSearchBuilderToggleItem[]
}
export type MailListSearchBuilderFieldGroup = { id: string; label: string; fields: readonly MailListSearchBuilderFieldItem[] }
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
export type MailListSearchBuilderToken = { id: string; value: string }
export type MailListSearchBuilderClauseView = {
  id: string
  pending: boolean
  tokens: readonly MailListSearchBuilderToken[]
}

export const mailListSearchPlaceholder = 'Search text, mail attrs or Hermes entities'

export const mailListSearchMatchModeItems: readonly MailListSearchBuilderToggleItem[] = [
  { value: 'all', label: 'all' },
  { value: 'any', label: 'any' }
]

export const mailListSearchOperatorItems: readonly MailListSearchBuilderToggleItem[] = [
  { value: 'contains', label: 'contains' },
  { value: 'equals', label: 'equals' },
  { value: 'is', label: 'is' },
  { value: 'is_not', label: 'is not' },
  { value: 'has', label: 'has' },
  { value: 'without', label: 'without' },
  { value: 'gte', label: '>=' },
  { value: 'lte', label: '<=' }
]

export const mailListSearchFieldGroups: readonly MailListSearchBuilderFieldGroup[] = [
  {
    id: 'text',
    label: 'Text',
    fields: [
      fieldItem('from', 'from', ['contains', 'equals'], 'Sender name or address'),
      fieldItem('recipients', 'to/cc', ['contains', 'equals'], 'Recipient address'),
      fieldItem('subject', 'subject', ['contains', 'equals'], 'Subject text'),
      fieldItem('body', 'body', ['contains', 'equals'], 'Message body'),
      fieldItem('all', 'all text', ['contains', 'equals'], 'Any projected text'),
      fieldItem('provider', 'provider id', ['contains', 'equals'], 'Provider record id')
    ]
  },
  {
    id: 'mail',
    label: 'Mail attrs',
    fields: [
      fieldItem('account', 'account', ['contains', 'equals'], 'Account label'),
      fieldItem('mailbox', 'mailbox', ['contains', 'equals'], 'Inbox, Spam, Archive'),
      fieldItem('workflow', 'status', ['is', 'is_not'], 'Workflow state', workflowPresets()),
      fieldItem('local', 'local state', ['is', 'is_not'], 'Local state', localStatePresets()),
      fieldItem('label', 'label', ['contains', 'equals'], 'Message label'),
      fieldItem('attachment', 'attachments', ['has', 'without'], 'Any attachment', yesNoPresets()),
      fieldItem('marker', 'marker', ['is', 'is_not'], 'Spam, phishing, important', markerPresets()),
      fieldItem('unread', 'unread', ['has', 'without'], 'Unread count', yesNoPresets()),
      fieldItem('action', 'action', ['has', 'without'], 'Open owner action', yesNoPresets()),
      fieldItem('delivery', 'delivery', ['contains', 'equals'], 'Delivery state'),
      fieldItem('importance', 'importance', ['is', 'gte', 'lte'], 'Score or band', importancePresets()),
      fieldItem('ai_category', 'AI category', ['contains', 'equals'], 'AI category')
    ]
  },
  {
    id: 'hermes',
    label: 'Hermes',
    fields: [
      fieldItem('entity', 'entity', ['contains', 'equals'], 'Entity kind or title', entityPresets()),
      fieldItem('task', 'task candidate', ['has', 'without'], 'Task candidate', yesNoPresets()),
      fieldItem('decision', 'decision', ['has', 'without'], 'Decision candidate', yesNoPresets()),
      fieldItem('document', 'document', ['has', 'without'], 'Document candidate', yesNoPresets()),
      fieldItem('deadline', 'deadline', ['has', 'without'], 'Deadline signal', yesNoPresets()),
      fieldItem('risk', 'risk', ['has', 'without'], 'Risk signal', yesNoPresets()),
      fieldItem('evidence', 'evidence', ['contains', 'equals'], 'Evidence kind')
    ]
  }
]

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

export function mailListSearchFieldItems(): readonly MailListSearchBuilderFieldItem[] {
  return mailListSearchFieldGroups.flatMap((group) => group.fields)
}

export function mailListSearchFieldItem(field: MailListSearchBuilderField): MailListSearchBuilderFieldItem {
  return mailListSearchFieldItems().find((item) => item.value === field) ?? defaultMailListSearchFieldItem()
}

export function mailListSearchBuilderOperatorItems(state: MailListSearchBuilderState): readonly MailListSearchBuilderToggleItem[] {
  const allowed = new Set(mailListSearchFieldItem(state.field).operators)
  return mailListSearchOperatorItems.filter((item) => allowed.has(item.value as MailListSearchBuilderOperator))
}

export function mailListSearchBuilderPresetItems(state: MailListSearchBuilderState): readonly MailListSearchBuilderToggleItem[] {
  return mailListSearchFieldItem(state.field).presets ?? []
}

export function mailListSearchLocalizedToggleItems(
  items: readonly MailListSearchBuilderToggleItem[],
  translate: (key: string) => string
): readonly MailListSearchBuilderToggleItem[] {
  return items.map((item) => ({ ...item, label: translate(item.label) }))
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
  const field = value
  const fieldConfig = mailListSearchFieldItem(field)
  const operator = fieldConfig.operators.includes(state.operator)
    ? state.operator
    : firstMailListSearchOperator(fieldConfig)

  return { ...state, field, operator, value: '' }
}

export function mailListSearchBuilderSetOperator(
  state: MailListSearchBuilderState,
  value: string | readonly string[]
): MailListSearchBuilderState {
  if (!isMailListSearchBuilderOperator(value)) return state
  if (!mailListSearchFieldItem(state.field).operators.includes(value)) return state
  return { ...state, operator: value }
}

export function mailListSearchBuilderSetValue(
  state: MailListSearchBuilderState,
  value: string
): MailListSearchBuilderState {
  const normalized = normalizeMailListSearchValue(value)

  if (booleanSearchFields.has(state.field) && ['yes', 'no', 'true', 'false', '1', '0'].includes(normalized)) {
    return {
      ...state,
      operator: normalizeBooleanSearchValue(normalized) ? 'has' : 'without',
      value: ''
    }
  }

  if (state.field === 'importance' && /^\d+$/.test(normalized) && state.operator === 'is') {
    return { ...state, operator: 'gte', value }
  }

  return { ...state, value }
}

export function mailListSearchBuilderCanAdd(state: MailListSearchBuilderState): boolean {
  const value = state.value.trim()
  return Boolean(value) || state.operator === 'has' || state.operator === 'without'
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
  if (state.matchMode === 'any') tokens.unshift('mode:any')

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
    { id: 'draft-field', value: mailListSearchFieldItem(state.field).label },
    { id: 'draft-operator', value: mailListSearchBuilderOperatorLabel(state.operator) }
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

function fieldItem(
  value: MailListSearchBuilderField,
  label: string,
  operators: readonly MailListSearchBuilderOperator[],
  placeholder: string,
  presets?: readonly MailListSearchBuilderToggleItem[]
): MailListSearchBuilderFieldItem {
  return { value, label, operators, placeholder, presets }
}

function defaultMailListSearchFieldItem(): MailListSearchBuilderFieldItem {
  const firstField = mailListSearchFieldItems()[0]
  if (!firstField) {
    throw new Error('Mail search builder requires at least one field')
  }

  return firstField
}

function firstMailListSearchOperator(field: MailListSearchBuilderFieldItem): MailListSearchBuilderOperator {
  const firstOperator = field.operators[0]
  if (!firstOperator) {
    throw new Error(`Mail search field ${field.value} requires at least one operator`)
  }

  return firstOperator
}

function workflowPresets(): readonly MailListSearchBuilderToggleItem[] {
  return labelledToggleItems([
    ['new', 'new'],
    ['needs_action', 'needs action'],
    ['waiting', 'waiting'],
    ['reviewed', 'reviewed'],
    ['spam', 'spam'],
    ['archived', 'archived']
  ])
}

function localStatePresets(): readonly MailListSearchBuilderToggleItem[] {
  return toggleItems(['active', 'trash', 'all'])
}

function yesNoPresets(): readonly MailListSearchBuilderToggleItem[] {
  return toggleItems(['yes', 'no'])
}

function markerPresets(): readonly MailListSearchBuilderToggleItem[] {
  return toggleItems(['spam', 'phishing', 'important', 'blocked', 'archived'])
}

function importancePresets(): readonly MailListSearchBuilderToggleItem[] {
  return labelledToggleItems([
    ['high', 'high'],
    ['medium', 'medium'],
    ['low', 'low'],
    ['75', '>= 75']
  ])
}

function entityPresets(): readonly MailListSearchBuilderToggleItem[] {
  return labelledToggleItems([
    ['person', 'person'],
    ['organization', 'org'],
    ['project', 'project'],
    ['document', 'document'],
    ['decision', 'decision']
  ])
}

function toggleItems(values: readonly string[]): readonly MailListSearchBuilderToggleItem[] {
  return values.map((value) => ({ value, label: value }))
}

function labelledToggleItems(items: readonly (readonly [string, string])[]): readonly MailListSearchBuilderToggleItem[] {
  return items.map(([value, label]) => ({ value, label }))
}

function mailListSearchBuilderOperatorLabel(operator: MailListSearchBuilderOperator): string {
  return mailListSearchOperatorItems.find((item) => item.value === operator)?.label ?? operator
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
    if (value) predicates.push({ field: 'all', operator: 'contains', value })
  }

  return { matchMode, predicates }
}

function tokenizeMailListSearchQuery(rawQuery: string): string[] {
  return rawQuery
    .match(/(?:[^\s"']+(?:!=|>=|<=|==|=|:)(?:"[^"]*"|'[^']*'|[^\s]+))|(?:"[^"]*"|'[^']*'|[^\s]+)/g)
    ?.map((token) => token.trim())
    .filter(Boolean) ?? []
}

function parseMailListSearchMode(token: string): MailListSearchMatchMode | null {
  const [, value] = /^mode:(all|any)$/i.exec(token) ?? []
  return value ? (value.toLowerCase() as MailListSearchMatchMode) : null
}

function parseMailListSearchFieldPredicate(token: string): MailListSearchPredicate | null {
  const match = /^([a-z_]+)(!=|>=|<=|==|=|:)(.+)$/i.exec(token)
  if (!match) return null

  const [, rawField, rawOperator, rawValue] = match
  if (!rawField || !rawOperator || !rawValue) return null

  const field = parseMailListSearchField(rawField)
  const value = stripSearchQuotes(rawValue)
  if (!field || !value) return null

  return {
    field,
    operator: parseMailListSearchOperator(field, rawOperator, value),
    value
  }
}

function parseMailListSearchField(field: string): MailListSearchField | null {
  const normalized = field.toLowerCase()
  if (normalized === 'sender') return 'from'
  if (normalized === 'to' || normalized === 'cc') return 'recipients'
  return mailListSearchFieldItems().some((item) => item.value === normalized)
    ? normalized as MailListSearchField
    : null
}

function parseMailListSearchOperator(
  field: MailListSearchField,
  operator: string,
  value: string
): MailListSearchBuilderOperator {
  if (operator === '!=') return 'is_not'
  if (operator === '>=') return 'gte'
  if (operator === '<=') return 'lte'
  if (operator === '=') return 'equals'
  if (operator === '==') return 'equals'
  if (operator === ':' && booleanSearchFields.has(field)) {
    return normalizeBooleanSearchValue(value) ? 'has' : 'without'
  }
  return 'contains'
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
  if (booleanSearchFields.has(predicate.field)) return mailListItemMatchesBooleanPredicate(item, predicate)
  if (predicate.field === 'importance') return mailListItemMatchesImportancePredicate(item, predicate)

  const targets = mailListSearchTargets(item, predicate.field)
  const needle = normalizeMailListSearchValue(predicate.value)
  if (!needle) return true

  const matched = predicate.operator === 'equals' || predicate.operator === 'is'
    ? targets.some((target) => target === needle)
    : targets.join(' ').includes(needle)

  return predicate.operator === 'is_not' ? !matched : matched
}

const booleanSearchFields = new Set<MailListSearchField>([
  'action',
  'attachment',
  'decision',
  'deadline',
  'document',
  'risk',
  'task',
  'unread'
])

function mailListItemMatchesBooleanPredicate(
  item: MailListItemModel,
  predicate: MailListSearchPredicate
): boolean {
  const present = mailListBooleanValue(item, predicate.field)
  const expectedPresent = predicate.operator === 'without'
    ? false
    : normalizeBooleanSearchValue(predicate.value)

  return present === expectedPresent
}

function mailListBooleanValue(item: MailListItemModel, field: MailListSearchField): boolean {
  switch (field) {
    case 'action':
      return Boolean(item.hasOpenAction)
    case 'attachment':
      return Boolean(item.attachmentCount && item.attachmentCount > 0)
    case 'decision':
      return Boolean(item.decisionCandidateCount && item.decisionCandidateCount > 0)
    case 'deadline':
      return Boolean(item.deadlineCount && item.deadlineCount > 0)
    case 'document':
      return Boolean(item.documentCandidateCount && item.documentCandidateCount > 0)
    case 'risk':
      return Boolean(item.riskCount && item.riskCount > 0)
    case 'task':
      return Boolean(item.taskCandidateCount && item.taskCandidateCount > 0)
    case 'unread':
      return Boolean(item.unreadCount && item.unreadCount > 0)
    default:
      return false
  }
}

function normalizeBooleanSearchValue(value: string): boolean {
  return !['0', 'false', 'no', 'none', 'without'].includes(normalizeMailListSearchValue(value))
}

function mailListItemMatchesImportancePredicate(
  item: MailListItemModel,
  predicate: MailListSearchPredicate
): boolean {
  const score = item.importanceScore ?? 0
  const band = normalizeImportanceBand(score)
  const value = normalizeMailListSearchValue(predicate.value)
  const numeric = Number(value)

  if (predicate.operator === 'gte' && Number.isFinite(numeric)) return score >= numeric
  if (predicate.operator === 'lte' && Number.isFinite(numeric)) return score <= numeric
  if (predicate.operator === 'is_not') return band !== value && String(score) !== value

  return band === value || String(score) === value
}

function normalizeImportanceBand(score: number): string {
  if (score >= 75) return 'high'
  if (score >= 40) return 'medium'
  return 'low'
}

function mailListSearchTargets(item: MailListItemModel, field: MailListSearchField): string[] {
  switch (field) {
    case 'account':
      return normalizeMailListSearchValues([item.accountLabel])
    case 'ai_category':
      return normalizeMailListSearchValues([item.aiCategory])
    case 'all':
      return normalizeMailListSearchValues([
        item.subject,
        item.fromName,
        item.fromAddress,
        item.snippet,
        item.id,
        item.providerRecordId,
        item.aiCategory,
        ...mailListEntityTargets(item),
        ...(item.labels ?? [])
      ])
    case 'body':
      return normalizeMailListSearchValues([item.snippet])
    case 'delivery':
      return normalizeMailListSearchValues([item.deliveryState])
    case 'entity':
      return normalizeMailListSearchValues(mailListEntityTargets(item))
    case 'evidence':
      return normalizeMailListSearchValues(item.evidenceKinds ?? [])
    case 'from':
      return normalizeMailListSearchValues([item.fromName, item.fromAddress])
    case 'label':
      return normalizeMailListSearchValues(item.labels ?? [])
    case 'local':
      return normalizeMailListSearchValues([item.localState ?? 'active'])
    case 'mailbox':
      return normalizeMailListSearchValues([item.mailboxLabel])
    case 'marker':
      return normalizeMailListSearchValues(mailListItemMarkers(item))
    case 'provider':
      return normalizeMailListSearchValues([item.providerRecordId, item.id])
    case 'recipients':
      return normalizeMailListSearchValues(item.recipients ?? [])
    case 'subject':
      return normalizeMailListSearchValues([item.subject])
    case 'workflow':
      return normalizeMailListSearchValues([item.workflowState])
    default:
      return normalizeMailListSearchValues([
        ...mailListEntityTargets(item),
        ...mailListCounterTargets(item)
      ])
  }
}

function mailListEntityTargets(item: MailListItemModel): string[] {
  return (item.hermesEntities ?? []).flatMap((entity) => [entity.kind, entity.title])
}

function mailListCounterTargets(item: MailListItemModel): string[] {
  return mailListItemCounters(item).flatMap((counter) => [
    counter.kind,
    `${counter.kind}:${counter.value}`
  ])
}

function normalizeMailListSearchValues(values: ReadonlyArray<string | MailListItemMarker | MailListItemCounterKind | undefined>): string[] {
  return values
    .filter((value): value is string => typeof value === 'string' && value.trim().length > 0)
    .map(normalizeMailListSearchValue)
    .filter(Boolean)
}

function normalizeMailListSearchValue(value: string): string {
  return value.trim().toLowerCase()
}

function isMailListSearchBuilderField(value: string | readonly string[]): value is MailListSearchBuilderField {
  return typeof value === 'string' && mailListSearchFieldItems().some((item) => item.value === value)
}

function isMailListSearchBuilderOperator(value: string | readonly string[]): value is MailListSearchBuilderOperator {
  return typeof value === 'string' && mailListSearchOperatorItems.some((item) => item.value === value)
}

function mailListSearchBuilderActiveClause(
  state: MailListSearchBuilderState
): MailListSearchBuilderClause | null {
  const value = state.value.trim()
  if (!value && state.operator !== 'has' && state.operator !== 'without') return null

  return {
    id: `clause-${state.nextClauseId}`,
    field: state.field,
    operator: state.operator,
    value: value || 'yes'
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
      { id: `${clause.id}-field`, value: mailListSearchFieldItem(clause.field).label },
      { id: `${clause.id}-operator`, value: mailListSearchBuilderOperatorLabel(clause.operator) },
      { id: `${clause.id}-value`, value: mailListSearchBuilderDisplayValue(clause) }
    ]
  }
}

function mailListSearchBuilderDisplayValue(clause: MailListSearchBuilderClause): string {
  if (clause.operator === 'has' || clause.operator === 'without') return 'any'
  return clause.value
}

function mailListSearchBuilderClauseQuery(clause: MailListSearchBuilderClause): string {
  const value = quoteMailListSearchValue(clause.value)
  switch (clause.operator) {
    case 'contains':
      return `${clause.field}:${value}`
    case 'equals':
    case 'is':
      return `${clause.field}=${value}`
    case 'is_not':
      return `${clause.field}!=${value}`
    case 'gte':
      return `${clause.field}>=${value}`
    case 'lte':
      return `${clause.field}<=${value}`
    case 'has':
      return `${clause.field}:yes`
    case 'without':
      return `${clause.field}:no`
  }
}

function quoteMailListSearchValue(value: string): string {
  const trimmed = value.trim()
  if (!/\s/.test(trimmed)) return trimmed
  if (!trimmed.includes('"')) return `"${trimmed}"`
  if (!trimmed.includes("'")) return `'${trimmed}'`
  return `"${trimmed.replaceAll('"', '')}"`
}
