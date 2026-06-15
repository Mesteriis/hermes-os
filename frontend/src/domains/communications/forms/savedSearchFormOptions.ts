import type { LocalMessageState, WorkflowState } from '../types/communications'

export const savedSearchWorkflowStates = [
  'new',
  'reviewed',
  'needs_action',
  'waiting',
  'done',
  'archived',
  'muted',
  'spam'
] as const

export const savedSearchLocalStates = ['active', 'trash', 'all'] as const

export const savedSearchWorkflowStateLabels: Record<(typeof savedSearchWorkflowStates)[number], string> = {
  new: 'New',
  reviewed: 'Reviewed',
  needs_action: 'Needs action',
  waiting: 'Waiting',
  done: 'Done',
  archived: 'Archived',
  muted: 'Muted',
  spam: 'Spam'
}

export const savedSearchLocalStateLabels: Record<(typeof savedSearchLocalStates)[number], string> = {
  active: 'Active',
  trash: 'Trash',
  all: 'All'
}

export const savedSearchMatchModeLabels: Record<'all' | 'any', string> = {
  all: 'All conditions',
  any: 'Any condition'
}

export const savedSearchWorkflowOptions: Array<{ label: string; value: WorkflowState | null }> = [
  { label: 'Any', value: null },
  ...savedSearchWorkflowStates.map((value) => ({ label: savedSearchWorkflowStateLabels[value], value }))
]

export const savedSearchLocalStateOptions: Array<{ label: string; value: LocalMessageState }> =
  savedSearchLocalStates.map((value) => ({
    label: savedSearchLocalStateLabels[value],
    value
  }))
