import { describe, expect, it } from 'vitest'
import {
  defaultTemplateVariableValue,
  extractTemplateVariables,
  missingTemplateVariables,
  parseTemplateMailMergePreviewRows,
  resolveTemplateVariableValues,
  storedTemplateDiagnosticMessages,
  stringifyTemplateMailMergePreviewRows,
  templateContentDiagnostics,
  templateDiagnosticsErrorMessage,
  templateFormSchema,
  templateMergeErrorMessage,
  templateFormToInput
} from './templateForm'

describe('template form', () => {
  it('extracts unique template variables in first-seen order', () => {
    expect(extractTemplateVariables(
      'Hello {{ recipient }}',
      '<p>{{body}}</p><p>{{ recipient }}</p><p>{{project.name}}</p>'
    )).toEqual(['recipient', 'body', 'project.name'])
  })

  it('normalizes compose content into a rich template save payload', () => {
    const values = templateFormSchema.parse({ name: '  Investor follow-up  ' })

    expect(templateFormToInput(values, {
      subject: 'Hello {{ recipient }}',
      body: 'Fallback {{ignored}}',
      bodyHtml: '<p>Next step for {{ project }}</p>'
    })).toEqual({
      name: 'Investor follow-up',
      subject_template: 'Hello {{ recipient }}',
      body_template: '<p>Next step for {{ project }}</p>',
      variables: ['recipient', 'project'],
      language: null
    })
  })

  it('preserves an existing template id when updating a template', () => {
    const values = templateFormSchema.parse({ name: 'Intro' })

    expect(templateFormToInput(values, {
      templateId: 'tpl-1',
      subject: 'Updated {{ name }}',
      body: 'Updated body',
      bodyHtml: null
    })).toMatchObject({
      template_id: 'tpl-1',
      name: 'Intro',
      variables: ['name']
    })
  })

  it('rejects empty template names before save', () => {
    expect(() => templateFormSchema.parse({ name: ' ' })).toThrow()
  })

  it('reports missing merge variables with stable copy', () => {
    const missing = missingTemplateVariables(['recipient', 'project', 'date'], {
      recipient: 'Alex',
      project: ' ',
      date: 'Jun 15, 2026'
    })

    expect(missing).toEqual(['project'])
    expect(templateMergeErrorMessage(missing)).toBe('Fill template variables: project')
  })

  it('reports malformed save placeholders before creating a rich template', () => {
    const diagnostics = templateContentDiagnostics(
      'Hello {{ recipient',
      '<p>{{ }} {{ project }} {{ first name }}</p>'
    )

    expect(diagnostics.variables).toEqual(['project'])
    expect(diagnostics.malformedPlaceholders).toEqual(['{{ recipient', '{{ }}', '{{ first name }}'])
    expect(templateDiagnosticsErrorMessage(diagnostics)).toBe(
      'Fix malformed template placeholders: {{ recipient, {{ }}, {{ first name }}'
    )
  })

  it('builds stored template diagnostic messages from backend metadata', () => {
    const messages = storedTemplateDiagnosticMessages({
      malformed_placeholders: ['{{ }}'],
      undeclared_variables: ['project'],
      unused_variables: ['legacy']
    })

    expect(messages).toEqual([
      {
        kind: 'error',
        label: 'Fix malformed placeholders',
        values: ['{{ }}']
      },
      {
        kind: 'error',
        label: 'Declare missing variables',
        values: ['project']
      },
      {
        kind: 'warning',
        label: 'Unused variables',
        values: ['legacy']
      }
    ])
  })

  it('derives stable default values for common template variables', () => {
    expect(defaultTemplateVariableValue('recipient', {
      toText: 'alex@example.com',
      ccText: 'team@example.com',
      bccText: 'audit@example.com',
      subject: 'Quarterly review',
      body: 'Body copy'
    })).toBe('alex@example.com')

    expect(defaultTemplateVariableValue('subject', {
      toText: '',
      ccText: '',
      bccText: '',
      subject: 'Quarterly review',
      body: 'Body copy'
    })).toBe('Quarterly review')
  })

  it('preserves existing variable values for the same template when requested', () => {
    expect(resolveTemplateVariableValues({
      variables: ['recipient', 'project', 'body']
    }, {
      recipient: 'alex@example.com',
      project: 'Hermes',
      body: ''
    }, {
      toText: 'default@example.com',
      ccText: '',
      bccText: '',
      subject: 'Quarterly review',
      body: 'Default body'
    }, {
      preserveExisting: true
    })).toEqual({
      recipient: 'alex@example.com',
      project: 'Hermes',
      body: 'Default body'
    })
  })

  it('parses JSON mail-merge preview rows with stable row ids', () => {
    expect(parseTemplateMailMergePreviewRows(`[
      { "row_id": "row-a", "variables": { "recipient": "alex@example.com", "project": "Hermes" } },
      { "recipient": "sam@example.com", "count": 2, "active": true }
    ]`)).toEqual([
      {
        row_id: 'row-a',
        variables: { recipient: 'alex@example.com', project: 'Hermes' }
      },
      {
        row_id: 'row-2',
        variables: { recipient: 'sam@example.com', count: '2', active: 'true' }
      }
    ])
  })

  it('stringifies preview rows and rejects invalid preview payloads', () => {
    expect(stringifyTemplateMailMergePreviewRows([
      { row_id: 'row-a', variables: { recipient: 'alex@example.com' } }
    ])).toContain('"row_id": "row-a"')

    expect(() => parseTemplateMailMergePreviewRows('{ "recipient": "alex@example.com" }')).toThrow(
      'Mail merge preview expects a JSON array of row objects'
    )
  })
})
