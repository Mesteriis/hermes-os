import { defineStore } from 'pinia'
import { ref } from 'vue'
import type {
  Decision,
  DecisionEntityKind,
  Obligation,
  TaskCandidate
} from '../types/task'

export const useTasksStore = defineStore('tasks-ui', () => {
  const tasksError = ref<string>('')
  const contextReviewError = ref<string>('')
  const isAiTaskRefreshSubmitting = ref<boolean>(false)
  const reviewEntityKind = ref<DecisionEntityKind>('project')
  const reviewEntityId = ref<string>('')
  const reviewingContextItemId = ref<string | null>(null)
  const decisions = ref<Decision[]>([])
  const obligations = ref<Obligation[]>([])
  const isContextReviewLoading = ref<boolean>(false)

  function setError(msg: string) {
    tasksError.value = msg
  }

  function clearError() {
    tasksError.value = ''
  }

  function setReviewEntityKind(kind: DecisionEntityKind) {
    reviewEntityKind.value = kind
  }

  function setReviewEntityId(entityId: string) {
    reviewEntityId.value = entityId
  }

  function setReviewingItemId(id: string | null) {
    reviewingContextItemId.value = id
  }

  function setDecisions(items: Decision[]) {
    decisions.value = items
  }

  function setObligations(items: Obligation[]) {
    obligations.value = items
  }

  function setContextReviewLoading(val: boolean) {
    isContextReviewLoading.value = val
  }

  function setContextReviewError(msg: string) {
    contextReviewError.value = msg
  }

  return {
    tasksError,
    contextReviewError,
    isAiTaskRefreshSubmitting,
    reviewEntityKind,
    reviewEntityId,
    reviewingContextItemId,
    decisions,
    obligations,
    isContextReviewLoading,
    setError,
    clearError,
    setReviewEntityKind,
    setReviewEntityId,
    setReviewingItemId,
    setDecisions,
    setObligations,
    setContextReviewLoading,
    setContextReviewError
  }
})

// Utility functions

export function taskSourceLabel(item: TaskCandidate | { source_kind: string; source_id: string }): string {
  const kind = item.source_kind
  return `${kind.charAt(0).toUpperCase()}${kind.slice(1)} · ${item.source_id}`
}

export function taskConfidence(item: TaskCandidate): string {
  return `${Math.round(item.confidence * 100)}%`
}

export function taskCreatedTime(value: string | null): string {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return 'Unknown date'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

export function formatDecisionTime(value: string | null): string {
  if (!value) return 'No decision date'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return 'Unknown date'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

export function formatEntityKind(kind: string): string {
  return kind
    .split('_')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

export function formatDecisionEntity(kind: string | null, entityId: string | null): string {
  if (!kind || !entityId) return 'No decider'
  return `${formatEntityKind(kind)} · ${entityId}`
}

export function formatObligationDueTime(value: string | null): string {
  if (!value) return 'No due date'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return 'Unknown date'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

export function formatObligationEntity(kind: string, entityId: string): string {
  return `${formatEntityKind(kind)} · ${entityId}`
}
