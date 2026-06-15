import { describe, expect, it } from 'vitest'
import { createSavedSearchRuleCondition, createSavedSearchRuleGroup } from '../forms/savedSearchForm'
import {
  savedSearchRuleGroupDepthLabel,
  savedSearchRuleGroupSummary
} from './savedSearchRuleTreePresentation'

describe('saved search rule tree presentation', () => {
  it('labels root and nested groups with stable depth cues', () => {
    expect(savedSearchRuleGroupDepthLabel(0)).toBe('Root group')
    expect(savedSearchRuleGroupDepthLabel(1)).toBe('Group 2')
    expect(savedSearchRuleGroupDepthLabel(2)).toBe('Group 3')
  })

  it('summarizes group structure for the rules builder header', () => {
    expect(savedSearchRuleGroupSummary(createSavedSearchRuleGroup('all', []))).toBe(
      'All conditions · Empty'
    )

    expect(savedSearchRuleGroupSummary(createSavedSearchRuleGroup('any', [
      createSavedSearchRuleCondition({ field: 'subject', operator: ':', value: 'quarterly' }),
      createSavedSearchRuleGroup('all', [
        createSavedSearchRuleCondition({ field: 'sender', operator: ':', value: 'alex' })
      ])
    ]))).toBe('Any condition · 1 rule · 1 nested group')
  })
})
