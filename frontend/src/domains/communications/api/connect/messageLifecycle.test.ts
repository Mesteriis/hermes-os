import { describe, expect, it } from 'vitest'
import { normalizeWorkflowAction } from './messageLifecycle'

describe('workflow action response mapping', () => {
  it('accepts only the canonical workflow action kinds', () => {
    expect(normalizeWorkflowAction('create_task')).toBe('create_task')
    expect(() => normalizeWorkflowAction('future_action')).toThrow(
      'Unsupported workflow action: future_action'
    )
  })
})
