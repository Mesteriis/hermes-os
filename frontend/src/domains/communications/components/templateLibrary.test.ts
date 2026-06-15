import { describe, expect, it } from 'vitest'
import {
  applyTemplateRecipientMapping,
  buildTemplateRecipientPreviewRows,
  deriveTemplateLibraryCategories,
  inferRecipientVariableMapping,
  recipientPreviewSummary,
  suggestTemplateSaveName,
  templateMatchesLibraryCategory
} from './templateLibrary'

describe('template library helpers', () => {
  it('derives stable categories from template structure and diagnostics', () => {
    const template = {
      variables: ['recipient', 'project'],
      malformed_placeholders: [],
      undeclared_variables: []
    }

    expect(deriveTemplateLibraryCategories(template)).toEqual([
      'mail-merge',
      'recipient-aware'
    ])
    expect(templateMatchesLibraryCategory(template, 'mail-merge')).toBe(true)
    expect(templateMatchesLibraryCategory(template, 'static-copy')).toBe(false)
  })

  it('classifies static and broken templates into useful categories', () => {
    expect(deriveTemplateLibraryCategories({
      variables: [],
      malformed_placeholders: [],
      undeclared_variables: []
    })).toEqual(['static-copy'])

    expect(deriveTemplateLibraryCategories({
      variables: ['project'],
      malformed_placeholders: ['{{ }}'],
      undeclared_variables: ['recipient']
    })).toEqual([
      'mail-merge',
      'needs-attention'
    ])
  })

  it('infers recipient mapping and fills mapped variables from compose recipients', () => {
    const mapping = inferRecipientVariableMapping(['recipient', 'cc', 'bcc', 'project'])

    expect(mapping).toEqual({
      toVariable: 'recipient',
      ccVariable: 'cc',
      bccVariable: 'bcc'
    })

    expect(applyTemplateRecipientMapping({
      project: 'Hermes'
    }, mapping, {
      toText: 'alex@example.com, sam@example.com',
      ccText: 'ops@example.com',
      bccText: 'audit@example.com'
    })).toEqual({
      project: 'Hermes',
      recipient: 'alex@example.com, sam@example.com',
      cc: 'ops@example.com',
      bcc: 'audit@example.com'
    })
  })

  it('builds mail-merge preview rows from To recipients using the selected mapping', () => {
    expect(buildTemplateRecipientPreviewRows(
      ['recipient', 'cc', 'project'],
      {
        toVariable: 'recipient',
        ccVariable: 'cc',
        bccVariable: ''
      },
      {
        toText: 'alex@example.com, sam@example.com',
        ccText: 'ops@example.com',
        bccText: ''
      },
      {
        project: 'Hermes rollout'
      }
    )).toEqual([
      {
        row_id: 'recipient-1',
        variables: {
          recipient: 'alex@example.com',
          cc: 'ops@example.com',
          project: 'Hermes rollout'
        }
      },
      {
        row_id: 'recipient-2',
        variables: {
          recipient: 'sam@example.com',
          cc: 'ops@example.com',
          project: 'Hermes rollout'
        }
      }
    ])
  })

  it('summarizes compose recipient counts for the mapping panel', () => {
    expect(recipientPreviewSummary({
      toText: 'alex@example.com, sam@example.com',
      ccText: 'ops@example.com',
      bccText: ''
    })).toBe('2 To · 1 CC · 0 BCC')
  })

  it('suggests stable save names for new and duplicate template flows', () => {
    expect(suggestTemplateSaveName('Quarterly follow-up', '', { duplicate: false })).toBe('Quarterly follow-up')
    expect(suggestTemplateSaveName('', 'Client follow-up', { duplicate: true })).toBe('Client follow-up copy')
    expect(suggestTemplateSaveName('', 'Client follow-up', { duplicate: false })).toBe('Client follow-up')
  })
})
