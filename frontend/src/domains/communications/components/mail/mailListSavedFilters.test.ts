import { describe, expect, it } from 'vitest'
import {
  cloneSearchBuilderState,
  committedSearchBuilderState,
  createSavedFilter,
  findSavedFilter,
  savedFilterTreeOptions,
} from './mailListSavedFilters'
import {
  createMailListSearchBuilderState,
  mailListSearchBuilderActiveFieldGroup,
  mailListSearchBuilderCanSave,
} from './mailSearchBuilder'

describe('mail list saved filters', () => {
  it('clones and commits builder state without sharing clause objects', () => {
    const state = createMailListSearchBuilderState()
    const clone = cloneSearchBuilderState(state)
    expect(clone).toEqual(state)
    expect(clone).not.toBe(state)
    expect(committedSearchBuilderState(state).clauses.length).toBeGreaterThanOrEqual(state.clauses.length)
  })

  it('creates and resolves named saved filters', () => {
    const state = createMailListSearchBuilderState()
    const result = createSavedFilter([], '  Important  ', state, 'saved-filter:1')
    expect(result?.filter.name).toBe('Important')
    expect(findSavedFilter(result?.filters ?? [], 'saved-filter:1')).toBe(result?.filter)
    expect(createSavedFilter([], '   ', state, 'saved-filter:2')).toBeNull()
    expect(savedFilterTreeOptions([], (key) => key)[0]?.disabled).toBe(true)
    expect(mailListSearchBuilderCanSave(state, 'name')).toBe(false)
    expect(mailListSearchBuilderActiveFieldGroup('missing')?.id).toBe('text')
  })
})
