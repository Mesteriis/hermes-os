import { describe, expect, it } from 'vitest'
import {
  canAssignIdentityTraceToOwner,
  canAssignIdentityTraceToPersona,
  isAssigningIdentityTrace,
  isReviewingIdentityCandidate,
} from './personaOverviewPresentation'
import type { PersonaIdentity } from '../types/persona'

describe('persona overview presentation', () => {
  const trace = { persona_id: 'persona-1' } satisfies Pick<PersonaIdentity, 'persona_id'>

  it('prevents assigning a trace to its current persona', () => {
    expect(canAssignIdentityTraceToPersona(trace, 'persona-1')).toBe(false)
    expect(canAssignIdentityTraceToPersona(trace, 'persona-2')).toBe(true)
  })

  it('requires an owner and prevents assigning a trace to its current owner', () => {
    expect(canAssignIdentityTraceToOwner(trace, null)).toBe(false)
    expect(canAssignIdentityTraceToOwner(trace, 'persona-1')).toBe(false)
    expect(canAssignIdentityTraceToOwner(trace, 'persona-2')).toBe(true)
  })

  it('matches active review and assignment operations', () => {
    expect(isReviewingIdentityCandidate('candidate-1', 'candidate-1')).toBe(true)
    expect(isReviewingIdentityCandidate('candidate-1', null)).toBe(false)
    expect(isAssigningIdentityTrace('trace-1', 'trace-1')).toBe(true)
    expect(isAssigningIdentityTrace('trace-1', null)).toBe(false)
  })
})
