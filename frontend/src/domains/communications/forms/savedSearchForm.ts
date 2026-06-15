import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { LocalMessageState, WorkflowState } from '../types/communications'
import type { MailSavedSearch, SavedSearchInput } from '../types/savedSearches'
import {
  composeSavedSearchQuery,
  composeSavedSearchRuleTreeQuery,
  createSavedSearchRuleCondition,
  createSavedSearchRuleGroup,
  flattenSavedSearchRuleTree,
  normalizeSavedSearchBuilderState,
  parseSavedSearchQuery,
  resolveSavedSearchEffectiveQuery,
  tokenizeSavedSearchQuery,
  validateSavedSearchRules,
  validateSavedSearchRuleTree,
  type SavedSearchBuilderState,
  type SavedSearchMatchMode,
  type SavedSearchParsedQuery,
  type SavedSearchRule,
  type SavedSearchRuleCondition,
  type SavedSearchRuleField,
  type SavedSearchRuleGroup,
  type SavedSearchRuleNode,
  type SavedSearchRuleOperator,
  type SavedSearchRuleValidation
} from './savedSearchRuleTree'
import {
  savedSearchLocalStateLabels,
  savedSearchLocalStateOptions,
  savedSearchLocalStates,
  savedSearchMatchModeLabels,
  savedSearchWorkflowOptions,
  savedSearchWorkflowStateLabels,
  savedSearchWorkflowStates
} from './savedSearchFormOptions'

export {
  savedSearchLocalStateOptions,
  savedSearchMatchModeLabels,
  savedSearchWorkflowOptions
} from './savedSearchFormOptions'
export {
  composeSavedSearchQuery,
  composeSavedSearchRuleTreeQuery,
  createSavedSearchRuleCondition,
  createSavedSearchRuleGroup,
  flattenSavedSearchRuleTree,
  normalizeSavedSearchBuilderState,
  parseSavedSearchQuery,
  resolveSavedSearchEffectiveQuery,
  tokenizeSavedSearchQuery,
  validateSavedSearchRules,
  validateSavedSearchRuleTree
} from './savedSearchRuleTree'
export type {
  SavedSearchBuilderState,
  SavedSearchMatchMode,
  SavedSearchParsedQuery,
  SavedSearchRule,
  SavedSearchRuleCondition,
  SavedSearchRuleField,
  SavedSearchRuleGroup,
  SavedSearchRuleNode,
  SavedSearchRuleOperator,
  SavedSearchRuleValidation
} from './savedSearchRuleTree'
export type SavedSearchFormValues = z.infer<typeof savedSearchFormSchema>
export type SavedSearchDeleteDialogCopy = {
  title: string
  message: string
  confirmLabel: string
}
export type SavedSearchOption<T extends string | null = string> = {
  label: string
  value: T
}
export type SavedSearchFilterChip = {
  label: string
  value: string
}
export type SavedSearchPresetOption = {
  label: string
  values: Pick<SavedSearchFormValues, 'query' | 'workflow_state' | 'local_state' | 'channel_kind' | 'is_smart_folder'>
}

export const savedSearchFormSchema = z.object({
  name: z.string().trim().min(1, 'Name is required').max(120, 'Name is too long'),
  description: z.string().trim().max(500, 'Description is too long'),
  query: z.string().trim().max(500, 'Search query is too long'),
  workflow_state: z.enum(savedSearchWorkflowStates).nullable(),
  local_state: z.enum(savedSearchLocalStates),
  channel_kind: z.string().trim().max(80, 'Channel is too long'),
  is_smart_folder: z.boolean(),
  match_mode: z.enum(['all', 'any']).default('all')
})

export const savedSearchVeeValidationSchema = toTypedSchema(savedSearchFormSchema)
export const savedSearchChannelOptions: Array<SavedSearchOption> = [
  { label: 'Email', value: 'email' },
  { label: 'Any channel', value: '' }
]
export const savedSearchMatchModeOptions: Array<SavedSearchOption<SavedSearchMatchMode>> = [
  { label: savedSearchMatchModeLabels.all, value: 'all' },
  { label: savedSearchMatchModeLabels.any, value: 'any' }
]
export const savedSearchPresetOptions: SavedSearchPresetOption[] = [
  {
    label: 'Needs action',
    values: {
      query: '',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: true
    }
  },
  {
    label: 'Waiting',
    values: {
      query: '',
      workflow_state: 'waiting',
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: true
    }
  },
  {
    label: 'Spam review',
    values: {
      query: '',
      workflow_state: 'spam',
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: true
    }
  }
]

const savedSearchRuleFieldLabels: Record<SavedSearchRuleField, string> = {
  subject: 'Subject',
  body: 'Body',
  sender: 'Sender',
  all: 'All'
}
const savedSearchRuleOperatorLabels: Record<SavedSearchRuleOperator, string> = {
  ':': 'contains',
  '=': 'equals'
}

export function savedSearchFormDefaults(
  savedSearch?: MailSavedSearch | null,
  isSmartFolder = false
): SavedSearchFormValues {
  return {
    name: savedSearch?.name ?? '',
    description: savedSearch?.description ?? '',
    query: savedSearch?.query ?? '',
    workflow_state: savedSearch?.workflow_state ?? null,
    local_state: savedSearch?.local_state ?? 'active',
    channel_kind: savedSearch?.channel_kind ?? '',
    is_smart_folder: savedSearch?.is_smart_folder ?? isSmartFolder,
    match_mode: 'all'
  }
}

export function savedSearchFormToInput(
  values: SavedSearchFormValues,
  accountId: string | null
): SavedSearchInput {
  const parsed = savedSearchFormSchema.parse(values)
  return {
    name: parsed.name,
    description: parsed.description || null,
    account_id: accountId?.trim() || null,
    query: parsed.query,
    workflow_state: parsed.workflow_state as WorkflowState | null,
    local_state: parsed.local_state as LocalMessageState,
    channel_kind: parsed.channel_kind || null,
    is_smart_folder: parsed.is_smart_folder
  }
}

export function savedSearchFilterChips(
  values: SavedSearchFormValues,
  rules: SavedSearchRule[] = []
): SavedSearchFilterChip[] {
  const parsedResult = savedSearchFormSchema.safeParse(values)
  const parsed = parsedResult.success
    ? parsedResult.data
    : {
        ...values,
        name: values.name.trim(),
        description: values.description.trim(),
        query: values.query.trim(),
        channel_kind: values.channel_kind.trim()
      }
  const chips: SavedSearchFilterChip[] = []
  const query = parsed.query.trim()
  const channel = parsed.channel_kind.trim()

  if (query) chips.push({ label: 'Text', value: query })
  for (const rule of rules) {
    if (!rule.value.trim()) continue
    chips.push({
      label: `${savedSearchRuleFieldLabels[rule.field]} ${savedSearchRuleOperatorLabels[rule.operator]}`,
      value: rule.value
    })
  }
  if (parsed.workflow_state) {
    chips.push({ label: 'Workflow', value: savedSearchWorkflowStateLabels[parsed.workflow_state] })
  }
  if (parsed.local_state !== 'active') {
    chips.push({ label: 'Scope', value: savedSearchLocalStateLabels[parsed.local_state] })
  }
  if (parsed.match_mode === 'any') {
    chips.push({ label: 'Match', value: savedSearchMatchModeLabels.any })
  }
  if (channel) chips.push({ label: 'Channel', value: channel })
  chips.push({ label: 'Mode', value: parsed.is_smart_folder ? 'Smart folder' : 'Saved search' })

  return chips
}

export function savedSearchDeleteDialogCopy(
  savedSearch: Pick<MailSavedSearch, 'name' | 'is_smart_folder'>
): SavedSearchDeleteDialogCopy {
  const kind = savedSearch.is_smart_folder ? 'smart folder' : 'saved search'
  return {
    title: `Delete ${kind}`,
    message: `Delete the ${kind} "${savedSearch.name}"? This does not delete messages.`,
    confirmLabel: 'Delete'
  }
}

export function savedSearchMessageCountLabel(
  savedSearch: Pick<MailSavedSearch, 'message_count'>
): string {
  return String(Math.max(0, Math.trunc(savedSearch.message_count)))
}
