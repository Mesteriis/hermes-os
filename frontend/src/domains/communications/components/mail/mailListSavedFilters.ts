import {
  mailListSearchBuilderAddClause,
  mailListSearchBuilderCanAdd,
  type MailListSearchBuilderState,
} from './mailSearchBuilder'
import type { TreeSelectOption } from '@/shared/ui'

export interface MailListSavedFilter {
  id: string
  name: string
  state: MailListSearchBuilderState
}

export function committedSearchBuilderState(
  state: MailListSearchBuilderState
): MailListSearchBuilderState {
  if (!mailListSearchBuilderCanAdd(state)) return state
  return mailListSearchBuilderAddClause(state)
}

export function cloneSearchBuilderState(
  state: MailListSearchBuilderState
): MailListSearchBuilderState {
  return {
    ...state,
    clauses: state.clauses.map((clause) => ({ ...clause })),
  }
}

export function createSavedFilter(
  filters: readonly MailListSavedFilter[],
  name: string,
  state: MailListSearchBuilderState,
  filterId: string
): { filter: MailListSavedFilter; filters: MailListSavedFilter[] } | null {
  const trimmedName = name.trim()
  if (!trimmedName) return null

  const filter = {
    id: filterId,
    name: trimmedName,
    state: cloneSearchBuilderState(state),
  }
  return { filter, filters: [...filters, filter] }
}

export function findSavedFilter(
  filters: readonly MailListSavedFilter[],
  filterId: string
): MailListSavedFilter | null {
  return filters.find((filter) => filter.id === filterId) ?? null
}

export function savedFilterTreeOptions(
  filters: readonly MailListSavedFilter[],
  t: (key: string) => string
): TreeSelectOption[] {
  if (filters.length === 0) {
    return [{
      value: 'saved-filters:empty',
      label: t('No saved filters yet'),
      icon: 'tabler:circle-dashed',
      disabled: true,
    }]
  }
  return filters.map((filter) => ({
    value: filter.id,
    label: filter.name,
    icon: 'tabler:filter-check',
  }))
}
