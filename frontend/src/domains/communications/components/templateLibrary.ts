import { splitComposeRecipients } from '../forms/composeValidation'
import type { EmailTemplate } from '../types/templates'

export type TemplateLibraryCategory =
  | 'mail-merge'
  | 'recipient-aware'
  | 'static-copy'
  | 'needs-attention'

export type TemplateLibraryCategoryOption = {
  value: TemplateLibraryCategory
  label: string
}

export type TemplateRecipientVariableMapping = {
  toVariable: string
  ccVariable: string
  bccVariable: string
}

export type TemplateRecipientContext = {
  toText: string
  ccText: string
  bccText: string
}

export type TemplateMailMergePreviewDraftRow = {
  row_id: string
  variables: Record<string, string>
}

const recipientVariableAliases: Record<keyof TemplateRecipientVariableMapping, string[]> = {
  toVariable: ['recipient', 'to', 'email', 'recipient_email'],
  ccVariable: ['cc', 'cc_email'],
  bccVariable: ['bcc', 'bcc_email']
}

export const templateLibraryCategoryOptions: TemplateLibraryCategoryOption[] = [
  { value: 'mail-merge', label: 'Mail merge' },
  { value: 'recipient-aware', label: 'Recipient-aware' },
  { value: 'static-copy', label: 'Static copy' },
  { value: 'needs-attention', label: 'Needs attention' }
]

export function templateLibraryCategoryLabel(category: TemplateLibraryCategory): string {
  return templateLibraryCategoryOptions.find((option) => option.value === category)?.label ?? category
}

export function filterTemplateLibraryTemplates(
  templates: EmailTemplate[],
  query: string,
  category: TemplateLibraryCategory | 'all'
): EmailTemplate[] {
  const normalizedQuery = query.trim().toLowerCase()
  const categoryFiltered = templates.filter((template) =>
    templateMatchesLibraryCategory(template, category)
  )
  if (!normalizedQuery) return categoryFiltered

  return categoryFiltered.filter((template) => {
    const inName = template.name.toLowerCase().includes(normalizedQuery)
    const inSubject = template.subject_template.toLowerCase().includes(normalizedQuery)
    const inBody = template.body_template.toLowerCase().includes(normalizedQuery)
    const inVariables = template.variables.some((variable) => variable.toLowerCase().includes(normalizedQuery))
    return inName || inSubject || inBody || inVariables
  })
}

export function orderTemplateLibraryTemplates(templates: EmailTemplate[]): EmailTemplate[] {
  return templates
    .slice()
    .sort((left, right) => {
      const updatedComparison = right.updated_at.localeCompare(left.updated_at)
      if (updatedComparison !== 0) return updatedComparison
      return left.name.localeCompare(right.name, undefined, { sensitivity: 'base' })
    })
}

export function formatTemplateUpdatedLabel(timestamp: string): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric'
  }).format(new Date(timestamp))
}

export function suggestTemplateSaveName(
  subject: string,
  selectedTemplateName = '',
  options: { duplicate: boolean }
): string {
  const normalizedSubject = subject.trim()
  const normalizedSelectedTemplateName = selectedTemplateName.trim()

  if (options.duplicate && normalizedSelectedTemplateName) {
    return `${normalizedSelectedTemplateName} copy`
  }
  if (normalizedSubject) return normalizedSubject
  if (normalizedSelectedTemplateName) return normalizedSelectedTemplateName
  return ''
}

export function deriveTemplateLibraryCategories(
  template: Pick<EmailTemplate, 'variables' | 'malformed_placeholders' | 'undeclared_variables'>
): TemplateLibraryCategory[] {
  const categories: TemplateLibraryCategory[] = []

  if (template.variables.length > 0) categories.push('mail-merge')
  if (templateHasRecipientVariables(template.variables)) categories.push('recipient-aware')
  if (template.variables.length === 0) categories.push('static-copy')
  if (template.malformed_placeholders.length > 0 || template.undeclared_variables.length > 0) {
    categories.push('needs-attention')
  }

  return categories
}

export function templateMatchesLibraryCategory(
  template: Pick<EmailTemplate, 'variables' | 'malformed_placeholders' | 'undeclared_variables'>,
  category: TemplateLibraryCategory | 'all'
): boolean {
  if (category === 'all') return true
  return deriveTemplateLibraryCategories(template).includes(category)
}

export function inferRecipientVariableMapping(
  variables: string[]
): TemplateRecipientVariableMapping {
  return {
    toVariable: firstRecipientVariableMatch(variables, 'toVariable'),
    ccVariable: firstRecipientVariableMatch(variables, 'ccVariable'),
    bccVariable: firstRecipientVariableMatch(variables, 'bccVariable')
  }
}

export function applyTemplateRecipientMapping(
  currentValues: Record<string, string>,
  mapping: TemplateRecipientVariableMapping,
  context: TemplateRecipientContext
): Record<string, string> {
  const nextValues = { ...currentValues }
  if (mapping.toVariable) nextValues[mapping.toVariable] = context.toText
  if (mapping.ccVariable) nextValues[mapping.ccVariable] = context.ccText
  if (mapping.bccVariable) nextValues[mapping.bccVariable] = context.bccText
  return nextValues
}

export function buildTemplateRecipientPreviewRows(
  templateVariables: string[],
  mapping: TemplateRecipientVariableMapping,
  context: TemplateRecipientContext,
  currentValues: Record<string, string>
): TemplateMailMergePreviewDraftRow[] {
  const toRecipients = splitComposeRecipients(context.toText)
  if (!toRecipients.length || !mapping.toVariable) return []

  const baseValues = templateVariables.reduce<Record<string, string>>((acc, variable) => {
    acc[variable] = currentValues[variable] ?? ''
    return acc
  }, {})

  return toRecipients.map((recipient, index) => ({
    row_id: `recipient-${index + 1}`,
    variables: {
      ...baseValues,
      ...(mapping.ccVariable ? { [mapping.ccVariable]: context.ccText } : {}),
      ...(mapping.bccVariable ? { [mapping.bccVariable]: context.bccText } : {}),
      [mapping.toVariable]: recipient
    }
  }))
}

export function recipientPreviewSummary(context: TemplateRecipientContext): string {
  const toCount = splitComposeRecipients(context.toText).length
  const ccCount = splitComposeRecipients(context.ccText).length
  const bccCount = splitComposeRecipients(context.bccText).length
  return `${toCount} To · ${ccCount} CC · ${bccCount} BCC`
}

export function templateHasRecipientVariables(variables: string[]): boolean {
  return variables.some((variable) => recipientVariableKind(variable) !== null)
}

function firstRecipientVariableMatch(
  variables: string[],
  key: keyof TemplateRecipientVariableMapping
): string {
  return variables.find((variable) => {
    const normalized = variable.trim().toLowerCase()
    return recipientVariableAliases[key].includes(normalized)
  }) ?? ''
}

function recipientVariableKind(
  variable: string
): keyof TemplateRecipientVariableMapping | null {
  const normalized = variable.trim().toLowerCase()
  if (recipientVariableAliases.toVariable.includes(normalized)) return 'toVariable'
  if (recipientVariableAliases.ccVariable.includes(normalized)) return 'ccVariable'
  if (recipientVariableAliases.bccVariable.includes(normalized)) return 'bccVariable'
  return null
}
