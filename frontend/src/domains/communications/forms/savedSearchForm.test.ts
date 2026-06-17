import { describe, expect, it } from 'vitest'
import {
  composeSavedSearchRuleTreeQuery,
  createSavedSearchRuleCondition,
  createSavedSearchRuleGroup,
  composeSavedSearchQuery,
  flattenSavedSearchRuleTree,
  normalizeSavedSearchBuilderState,
  parseSavedSearchQuery,
  resolveSavedSearchEffectiveQuery,
  savedSearchDeleteDialogCopy,
  savedSearchChannelOptions,
  savedSearchFilterChips,
  savedSearchFormSchema,
  savedSearchFormToInput,
  savedSearchMessageCountLabel,
  savedSearchPresetOptions,
  validateSavedSearchRuleTree,
  validateSavedSearchRules
} from './savedSearchForm'

describe('saved search form', () => {
  it('normalizes form values into a saved-search input payload', () => {
    const values = savedSearchFormSchema.parse({
      name: '  Finance invoices  ',
      description: '  Quarterly invoice filter  ',
      query: '  invoice due  ',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: '  email  ',
      is_smart_folder: false
    })

    expect(savedSearchFormToInput(values, 'account-1')).toEqual({
      name: 'Finance invoices',
      description: 'Quarterly invoice filter',
      account_id: 'account-1',
      query: 'invoice due',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: false
    })
  })

  it('allows smart folders with empty query and rejects empty names', () => {
    expect(() =>
      savedSearchFormSchema.parse({
        name: ' ',
        description: '',
        query: '',
        workflow_state: null,
        local_state: 'active',
        channel_kind: '',
        is_smart_folder: true
      })
    ).toThrow()

    const values = savedSearchFormSchema.parse({
      name: 'Needs reply',
      description: '',
      query: '',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: '',
      is_smart_folder: true
    })

    expect(savedSearchFormToInput(values, null)).toMatchObject({
      name: 'Needs reply',
      account_id: null,
      query: '',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: null,
      is_smart_folder: true
    })
  })

  it('offers Telegram as a first-class saved-search channel kind', () => {
    expect(savedSearchChannelOptions).toEqual(
      expect.arrayContaining([
        { label: 'Telegram', value: 'telegram' }
      ])
    )
  })

  it('builds delete confirmation copy for saved searches and smart folders', () => {
    expect(savedSearchDeleteDialogCopy({ name: 'Finance', is_smart_folder: false })).toEqual({
      title: 'Delete saved search',
      message: 'Delete the saved search "Finance"? This does not delete messages.',
      confirmLabel: 'Delete'
    })

    expect(savedSearchDeleteDialogCopy({ name: 'Needs reply', is_smart_folder: true })).toEqual({
      title: 'Delete smart folder',
      message: 'Delete the smart folder "Needs reply"? This does not delete messages.',
      confirmLabel: 'Delete'
    })
  })

  it('formats saved-search message counts for compact strip badges', () => {
    expect(savedSearchMessageCountLabel({ message_count: 0 })).toBe('0')
    expect(savedSearchMessageCountLabel({ message_count: 42 })).toBe('42')
    expect(savedSearchMessageCountLabel({ message_count: 1200 })).toBe('1200')
  })

  it('summarizes active saved-search filters as rule chips', () => {
    const values = savedSearchFormSchema.parse({
      name: 'Escalations',
      description: '',
      query: ' renewal ',
      workflow_state: 'waiting',
      local_state: 'all',
      channel_kind: ' email ',
      is_smart_folder: true
    })

    expect(savedSearchFilterChips(values)).toEqual([
      { label: 'Text', value: 'renewal' },
      { label: 'Workflow', value: 'Waiting' },
      { label: 'Scope', value: 'All' },
      { label: 'Channel', value: 'email' },
      { label: 'Mode', value: 'Smart folder' }
    ])
  })

  it('does not throw when rendering filter chips for an unsaved invalid draft', () => {
    expect(() => savedSearchFilterChips({
      name: '',
      description: '',
      query: ' invoice ',
      workflow_state: null,
      local_state: 'active',
      channel_kind: ' email ',
      is_smart_folder: false,
      match_mode: 'all'
    }, [
      { field: 'sender', operator: ':', value: 'alex' }
    ])).not.toThrow()

    expect(savedSearchFilterChips({
      name: '',
      description: '',
      query: ' invoice ',
      workflow_state: null,
      local_state: 'active',
      channel_kind: ' email ',
      is_smart_folder: false,
      match_mode: 'all'
    }, [
      { field: 'sender', operator: ':', value: 'alex' }
    ])).toEqual([
      { label: 'Text', value: 'invoice' },
      { label: 'Sender contains', value: 'alex' },
      { label: 'Channel', value: 'email' },
      { label: 'Mode', value: 'Saved search' }
    ])
  })

  it('provides safe Mail-only presets for smart filters', () => {
    expect(savedSearchPresetOptions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          label: 'Needs action',
          values: expect.objectContaining({
            workflow_state: 'needs_action',
            local_state: 'active',
            is_smart_folder: true
          })
        }),
        expect.objectContaining({
          label: 'Spam review',
          values: expect.objectContaining({
            workflow_state: 'spam',
            local_state: 'active',
            is_smart_folder: true
          })
        })
      ])
    )
  })

  it('parses search rules from query text', () => {
    expect(parseSavedSearchQuery('subject:"invoice due" from:alex body:payment all==todo')).toMatchObject({
      plainQuery: '',
      rules: [
        { field: 'subject', operator: ':', value: 'invoice due' },
        { field: 'sender', operator: ':', value: 'alex' },
        { field: 'body', operator: ':', value: 'payment' },
        { field: 'all', operator: '=', value: 'todo' }
      ],
      matchMode: 'all'
    })
  })

  it('builds a saved-search query from plain text and rules', () => {
    expect(composeSavedSearchQuery('invoice', [
      { field: 'sender', operator: ':', value: 'alex' },
      { field: 'subject', operator: '=', value: 'Quarterly Report' }
    ])).toBe('invoice sender:alex subject="Quarterly Report"')
  })

  it('supports any-match query composition and filter chips', () => {
    const values = savedSearchFormSchema.parse({
      name: 'Flexible',
      description: '',
      query: 'invoice',
      workflow_state: null,
      local_state: 'active',
      channel_kind: '',
      is_smart_folder: false,
      match_mode: 'any'
    })

    expect(composeSavedSearchQuery('invoice', [
      { field: 'sender', operator: ':', value: 'alex' }
    ], 'any')).toBe('mode:any invoice sender:alex')

    expect(savedSearchFilterChips(values, [
      { field: 'sender', operator: ':', value: 'alex' }
    ])).toEqual([
      { label: 'Text', value: 'invoice' },
      { label: 'Sender contains', value: 'alex' },
      { label: 'Match', value: 'Any condition' },
      { label: 'Mode', value: 'Saved search' }
    ])
  })

  it('normalizes structured query tokens into builder state without duplicating rules', () => {
    expect(normalizeSavedSearchBuilderState(
      'mode:any invoice from:alex subject:"Quarterly Report"',
      [
        { field: 'sender', operator: ':', value: 'alex' },
        { field: 'body', operator: ':', value: 'payment' }
      ],
      'all'
    )).toMatchObject({
      plainQuery: 'invoice',
      rules: [
        { field: 'sender', operator: ':', value: 'alex' },
        { field: 'subject', operator: ':', value: 'Quarterly Report' },
        { field: 'body', operator: ':', value: 'payment' }
      ],
      matchMode: 'any'
    })
  })

  it('parses nested rule groups from explicit boolean expressions', () => {
    const parsed = parseSavedSearchQuery('(subject:quarterly OR body:invoice) AND sender:alex')

    expect(parsed.plainQuery).toBe('')
    expect(parsed.matchMode).toBe('all')
    expect(flattenSavedSearchRuleTree(parsed.tree)).toEqual([
      { field: 'subject', operator: ':', value: 'quarterly' },
      { field: 'body', operator: ':', value: 'invoice' },
      { field: 'sender', operator: ':', value: 'alex' }
    ])
  })

  it('composes nested rule groups into an explicit preview query', () => {
    const tree = createSavedSearchRuleGroup('all', [
      createSavedSearchRuleGroup('any', [
        createSavedSearchRuleCondition({ field: 'subject', operator: ':', value: 'quarterly' }),
        createSavedSearchRuleCondition({ field: 'body', operator: ':', value: 'invoice' })
      ]),
      createSavedSearchRuleCondition({ field: 'sender', operator: ':', value: 'alex' })
    ])

    expect(composeSavedSearchRuleTreeQuery('', tree)).toBe('((subject:quarterly OR body:invoice) AND sender:alex)')
  })

  it('resolves the effective saved-search query from normalized builder state', () => {
    expect(resolveSavedSearchEffectiveQuery(
      'mode:any invoice from:alex',
      [{ field: 'subject', operator: ':', value: 'Quarterly Report' }],
      'all'
    )).toBe('mode:any invoice sender:alex subject:"Quarterly Report"')
  })

  it('validates incomplete and duplicate builder rules before save', () => {
    expect(validateSavedSearchRules([
      { field: 'subject', operator: ':', value: '' }
    ])).toEqual({
      isValid: false,
      message: 'Complete or remove the empty rule before saving'
    })

    expect(validateSavedSearchRules([
      { field: 'sender', operator: ':', value: 'alex' },
      { field: 'sender', operator: ':', value: ' Alex ' }
    ])).toEqual({
      isValid: false,
      message: 'Remove duplicate rules before saving'
    })

    expect(validateSavedSearchRules([
      { field: 'sender', operator: ':', value: 'alex' },
      { field: 'subject', operator: ':', value: 'Quarterly Report' }
    ])).toEqual({
      isValid: true,
      message: ''
    })

    expect(validateSavedSearchRuleTree(createSavedSearchRuleGroup('all', []))).toEqual({
      isValid: false,
      message: 'Add at least one rule or group before saving'
    })
  })
})
