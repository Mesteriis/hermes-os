import {
  mailListItemCounters,
  mailListItemMarkers,
  type MailListItemModel
} from './mailElements'
import type {
  MailListSearchBuilderField,
  MailListSearchBuilderState
} from './mailSearchBuilder'

export type MailListSearchValueSuggestion = {
  value: string
  label: string
}

const searchSuggestionLimit = 12

export function mailListSearchBuilderValueSuggestions(
  items: readonly MailListItemModel[],
  state: MailListSearchBuilderState
): readonly MailListSearchValueSuggestion[] {
  const values = mailListSearchSuggestionValues(items, state.field)
  const query = normalizeSearchSuggestionValue(state.value)
  const suggestions = uniqueSearchSuggestionValues(values)
    .filter((value) => !query || normalizeSearchSuggestionValue(value).includes(query))
    .slice(0, searchSuggestionLimit)

  return suggestions.map((value) => ({ value, label: value }))
}

function mailListSearchSuggestionValues(
  items: readonly MailListItemModel[],
  field: MailListSearchBuilderField
): readonly string[] {
  return items.flatMap((item) => mailListItemSearchSuggestionValues(item, field))
}

function mailListItemSearchSuggestionValues(
  item: MailListItemModel,
  field: MailListSearchBuilderField
): readonly string[] {
  switch (field) {
    case 'account':
      return [item.accountLabel]
    case 'ai_category':
      return [item.aiCategory ?? '']
    case 'all':
      return allMailListItemSuggestionValues(item)
    case 'body':
      return [item.snippet]
    case 'delivery':
      return [item.deliveryState ?? '']
    case 'entity':
      return mailListEntitySuggestionValues(item)
    case 'evidence':
      return item.evidenceKinds ?? []
    case 'from':
      return [item.fromName, item.fromAddress ?? '']
    case 'importance':
      return [String(item.importanceScore ?? ''), importanceBand(item.importanceScore ?? 0)]
    case 'label':
      return item.labels ?? []
    case 'local':
      return [item.localState ?? 'active']
    case 'mailbox':
      return [item.mailboxLabel]
    case 'marker':
      return mailListItemMarkers(item)
    case 'provider':
      return [item.providerRecordId ?? '', item.id]
    case 'recipients':
      return item.recipients ?? []
    case 'subject':
      return [item.subject]
    case 'workflow':
      return [item.workflowState]
    default:
      return booleanSuggestionValues()
  }
}

function allMailListItemSuggestionValues(item: MailListItemModel): readonly string[] {
  return [
    item.fromName,
    item.fromAddress ?? '',
    item.subject,
    item.snippet,
    item.providerRecordId ?? '',
    item.id,
    item.aiCategory ?? '',
    item.workflowState,
    item.mailboxLabel,
    ...mailListEntitySuggestionValues(item),
    ...(item.labels ?? []),
    ...(item.evidenceKinds ?? []),
    ...mailListCounterSuggestionValues(item)
  ]
}

function mailListEntitySuggestionValues(item: MailListItemModel): readonly string[] {
  return (item.hermesEntities ?? []).flatMap((entity) => [entity.kind, entity.title])
}

function mailListCounterSuggestionValues(item: MailListItemModel): readonly string[] {
  return mailListItemCounters(item).flatMap((counter) => [
    counter.kind,
    `${counter.kind}:${counter.value}`
  ])
}

function booleanSuggestionValues(): readonly string[] {
  return ['yes', 'no']
}

function uniqueSearchSuggestionValues(values: readonly string[]): readonly string[] {
  const seen = new Set<string>()
  const uniqueValues: string[] = []

  for (const value of values) {
    const trimmed = value.trim()
    const normalized = normalizeSearchSuggestionValue(trimmed)
    if (!trimmed || seen.has(normalized)) continue

    seen.add(normalized)
    uniqueValues.push(trimmed)
  }

  return uniqueValues
}

function normalizeSearchSuggestionValue(value: string): string {
  return value.trim().toLocaleLowerCase()
}

function importanceBand(score: number): string {
  if (score >= 75) return 'high'
  if (score >= 40) return 'medium'
  return 'low'
}
