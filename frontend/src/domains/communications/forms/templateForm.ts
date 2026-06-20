import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { CommunicationTemplate, RichTemplateUpsertRequest } from '../types/templates'

export type TemplateFormValues = z.infer<typeof templateFormSchema>
export type TemplateComposeContent = {
  templateId?: string
  subject: string
  body: string
  bodyHtml: string | null
}
export type TemplateContentDiagnostics = {
  variables: string[]
  malformedPlaceholders: string[]
}
export type StoredTemplateDiagnosticMessage = {
  kind: 'error' | 'warning'
  label: string
  values: string[]
}
export type StoredTemplateDiagnosticSource = Pick<
  CommunicationTemplate,
  'malformed_placeholders' | 'undeclared_variables' | 'unused_variables'
>
export type TemplateVariableDefaultsContext = {
  toText: string
  ccText: string
  bccText: string
  subject: string
  body: string
}
export type TemplateMailMergePreviewRowInput = {
  row_id: string
  variables: Record<string, string>
}

const templateVariableNamePattern = /^[A-Za-z0-9_.-]+$/

export const templateFormSchema = z.object({
  name: z.string().trim().min(1, 'Template name is required').max(120, 'Template name is too long')
})

export const templateVeeValidationSchema = toTypedSchema(templateFormSchema)

export function templateFormDefaults(): TemplateFormValues {
  return {
    name: ''
  }
}

export function extractTemplateVariables(...sources: Array<string | null | undefined>): string[] {
  return templateContentDiagnostics(...sources).variables
}

export function templateContentDiagnostics(
  ...sources: Array<string | null | undefined>
): TemplateContentDiagnostics {
  const variables: string[] = []
  const malformedPlaceholders: string[] = []
  const seenVariables = new Set<string>()
  const seenMalformed = new Set<string>()

  for (const source of sources) {
    if (!source) continue
    inspectTemplateSource(source, variables, seenVariables, malformedPlaceholders, seenMalformed)
  }

  return { variables, malformedPlaceholders }
}

function inspectTemplateSource(
  source: string,
  variables: string[],
  seenVariables: Set<string>,
  malformedPlaceholders: string[],
  seenMalformed: Set<string>
): void {
  let rest = source
  while (true) {
    const start = rest.indexOf('{{')
    if (start === -1) return

    const afterOpen = rest.slice(start + 2)
    const end = afterOpen.indexOf('}}')
    if (end === -1) {
      addUnique(malformedPlaceholders, seenMalformed, rest.slice(start))
      return
    }

    const rawPlaceholder = rest.slice(start, start + 2 + end + 2)
    const variable = afterOpen.slice(0, end).trim()
    if (!variable || !templateVariableNamePattern.test(variable)) {
      addUnique(malformedPlaceholders, seenMalformed, rawPlaceholder)
    } else {
      addUnique(variables, seenVariables, variable)
    }
    rest = afterOpen.slice(end + 2)
  }
}

function addUnique(target: string[], seen: Set<string>, value: string): void {
  if (seen.has(value)) return
  seen.add(value)
  target.push(value)
}

export function missingTemplateVariables(
  variables: string[],
  values: Record<string, string>
): string[] {
  return variables.filter((variable) => !(values[variable] ?? '').trim())
}

export function templateMergeErrorMessage(missingVariables: string[]): string {
  if (!missingVariables.length) return ''
  return `Fill template variables: ${missingVariables.join(', ')}`
}

export function templateDiagnosticsErrorMessage(diagnostics: TemplateContentDiagnostics): string {
  if (!diagnostics.malformedPlaceholders.length) return ''
  return `Fix malformed template placeholders: ${diagnostics.malformedPlaceholders.join(', ')}`
}

export function storedTemplateDiagnosticMessages(
  template: StoredTemplateDiagnosticSource | null | undefined
): StoredTemplateDiagnosticMessage[] {
  if (!template) return []

  const messages: StoredTemplateDiagnosticMessage[] = []
  if (template.malformed_placeholders.length) {
    messages.push({
      kind: 'error',
      label: 'Fix malformed placeholders',
      values: template.malformed_placeholders
    })
  }
  if (template.undeclared_variables.length) {
    messages.push({
      kind: 'error',
      label: 'Declare missing variables',
      values: template.undeclared_variables
    })
  }
  if (template.unused_variables.length) {
    messages.push({
      kind: 'warning',
      label: 'Unused variables',
      values: template.unused_variables
    })
  }
  return messages
}

export function defaultTemplateVariableValue(
  variable: string,
  context: TemplateVariableDefaultsContext
): string {
  const normalized = variable.trim().toLowerCase()
  if (normalized === 'to' || normalized === 'recipient') return context.toText
  if (normalized === 'cc') return context.ccText
  if (normalized === 'bcc') return context.bccText
  if (normalized === 'subject') return context.subject
  if (normalized === 'body' || normalized === 'message') return context.body
  if (normalized === 'date' || normalized === 'current_date') {
    return new Intl.DateTimeFormat('en-US', { dateStyle: 'medium' }).format(new Date())
  }
  return ''
}

export function resolveTemplateVariableValues(
  template: Pick<CommunicationTemplate, 'variables'> | null | undefined,
  existingValues: Record<string, string>,
  context: TemplateVariableDefaultsContext,
  options: { preserveExisting: boolean }
): Record<string, string> {
  if (!template) return {}

  const resolved: Record<string, string> = {}
  for (const variable of template.variables) {
    const existingValue = existingValues[variable] ?? ''
    resolved[variable] = options.preserveExisting && existingValue.trim()
      ? existingValue
      : defaultTemplateVariableValue(variable, context)
  }
  return resolved
}

export function parseTemplateMailMergePreviewRows(
  rawValue: string
): TemplateMailMergePreviewRowInput[] {
  const trimmed = rawValue.trim()
  if (!trimmed) return []

  const parsed = JSON.parse(trimmed)
  if (!Array.isArray(parsed)) {
    throw new Error('Mail merge preview expects a JSON array of row objects')
  }

  return parsed.map((item, index) => normalizeTemplateMailMergePreviewRow(item, index))
}

export function stringifyTemplateMailMergePreviewRows(
  rows: TemplateMailMergePreviewRowInput[]
): string {
  return JSON.stringify(rows, null, 2)
}

export function templateFormToInput(
  values: TemplateFormValues,
  content: TemplateComposeContent
): RichTemplateUpsertRequest {
  const parsed = templateFormSchema.parse(values)
  const bodyTemplate = content.bodyHtml ?? content.body
  const diagnostics = templateContentDiagnostics(content.subject, bodyTemplate)
  return {
    ...(content.templateId?.trim() ? { template_id: content.templateId.trim() } : {}),
    name: parsed.name,
    subject_template: content.subject,
    body_template: bodyTemplate,
    variables: diagnostics.variables,
    language: null
  }
}

function normalizeTemplateMailMergePreviewRow(
  value: unknown,
  index: number
): TemplateMailMergePreviewRowInput {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`Mail merge preview row ${index + 1} must be an object`)
  }

  const record = value as Record<string, unknown>
  const rowId = typeof record.row_id === 'string' && record.row_id.trim()
    ? record.row_id.trim()
    : `row-${index + 1}`

  const variablesSource = 'variables' in record ? record.variables : record
  if (!variablesSource || typeof variablesSource !== 'object' || Array.isArray(variablesSource)) {
    throw new Error(`Mail merge preview row ${index + 1} must provide an object of variables`)
  }

  const variables: Record<string, string> = {}
  for (const [key, rawVariableValue] of Object.entries(variablesSource as Record<string, unknown>)) {
    if (key === 'row_id') continue
    if (rawVariableValue === null || rawVariableValue === undefined) {
      variables[key] = ''
      continue
    }
    if (typeof rawVariableValue === 'string' || typeof rawVariableValue === 'number' || typeof rawVariableValue === 'boolean') {
      variables[key] = String(rawVariableValue)
      continue
    }
    throw new Error(`Mail merge preview row ${index + 1} variable "${key}" must be a string, number, boolean, null, or omitted`)
  }

  return {
    row_id: rowId,
    variables
  }
}
