import { computed } from 'vue'
import {
  useReviewDecisionMutation,
  useReviewObligationMutation,
  useReviewTaskCandidateMutation,
  useTaskCandidatesQuery,
  useTaskContextReviewQuery,
  useTasksQuery,
} from './useTasksQuery'
import { useTasksStore } from '../stores/tasks'
import type {
  Decision,
  Obligation,
  TaskCandidate,
  Task,
  TaskCandidateReviewState,
} from '../types/task'

export function useTasksPageSurface() {
  const store = useTasksStore()

  const candidatesQuery = useTaskCandidatesQuery()
  const tasksQuery = useTasksQuery()
  const reviewTaskCandidateMutation = useReviewTaskCandidateMutation()
  const reviewDecisionMutation = useReviewDecisionMutation()
  const reviewObligationMutation = useReviewObligationMutation()
  const contextReviewQuery = useTaskContextReviewQuery(
    computed(() => store.reviewEntityKind),
    computed(() => store.reviewEntityId)
  )

  const taskCandidates = computed<TaskCandidate[]>(() => candidatesQuery.data.value ?? [])
  const activeTasks = computed<Task[]>(() => tasksQuery.data.value ?? [])
  const isTasksLoadingCombined = computed<boolean>(
    () => candidatesQuery.isLoading.value || tasksQuery.isLoading.value
  )
  const decisions = computed<Decision[]>(() => contextReviewQuery.data.value?.decisions ?? [])
  const obligations = computed<Obligation[]>(() => contextReviewQuery.data.value?.obligations ?? [])
  const isContextReviewLoading = computed<boolean>(
    () => contextReviewQuery.isLoading.value || contextReviewQuery.isFetching.value
  )
  const contextReviewError = computed<string>(() => {
    if (store.contextReviewError) return store.contextReviewError
    return contextReviewQuery.error.value instanceof Error ? contextReviewQuery.error.value.message : ''
  })
  const suggestedTaskCandidates = computed<TaskCandidate[]>(() =>
    taskCandidates.value.filter((item) => item.review_state === 'suggested')
  )
  const reviewStats = computed(() => ({
    totalCandidates: taskCandidates.value.length,
    suggestedCandidates: suggestedTaskCandidates.value.length,
    activeTasks: activeTasks.value.length,
    completedLikeCount:
      taskCandidates.value.length - suggestedTaskCandidates.value.length - activeTasks.value.length,
  }))
  const recentCandidateSignals = computed(() => suggestedTaskCandidates.value.slice(0, 5))
  const activeTaskSources = computed(() => ['message', 'document'] as const)

  async function reviewDecisionItem(
    decision: Decision,
    reviewState: 'user_confirmed' | 'user_rejected'
  ) {
    store.setReviewingItemId(`decision:${decision.decision_id}`)
    store.setContextReviewError('')
    try {
      await reviewDecisionMutation.mutateAsync({
        decisionId: decision.decision_id,
        request: { review_state: reviewState },
      })
      await contextReviewQuery.refetch()
    } catch (error: unknown) {
      store.setContextReviewError(
        error instanceof Error ? error.message : 'Unknown decision review error'
      )
    } finally {
      store.setReviewingItemId(null)
    }
  }

  async function reviewObligationItem(
    obligation: Obligation,
    reviewState: 'user_confirmed' | 'user_rejected'
  ) {
    store.setReviewingItemId(`obligation:${obligation.obligation_id}`)
    store.setContextReviewError('')
    try {
      await reviewObligationMutation.mutateAsync({
        obligationId: obligation.obligation_id,
        request: { review_state: reviewState },
      })
      await contextReviewQuery.refetch()
    } catch (error: unknown) {
      store.setContextReviewError(
        error instanceof Error ? error.message : 'Unknown obligation review error'
      )
    } finally {
      store.setReviewingItemId(null)
    }
  }

  async function loadTasks() {
    await Promise.all([candidatesQuery.refetch(), tasksQuery.refetch()])
  }

  async function setTaskCandidateReview(
    candidate: TaskCandidate,
    state: TaskCandidateReviewState
  ) {
    try {
      await reviewTaskCandidateMutation.mutateAsync({
        taskCandidateId: candidate.task_candidate_id,
        reviewState: state,
      })
      store.clearError()
      await loadTasks()
    } catch (error: unknown) {
      store.setError(error instanceof Error ? error.message : 'Unknown task candidate review error')
    }
  }

  async function reloadContextReview() {
    store.setContextReviewError('')
    await contextReviewQuery.refetch()
  }

  return {
    activeTaskSources,
    activeTasks,
    contextReviewError,
    decisions,
    isContextReviewLoading,
    isTasksLoadingCombined,
    loadTasks,
    obligations,
    recentCandidateSignals,
    reloadContextReview,
    reviewDecisionItem,
    reviewObligationItem,
    reviewStats,
    setTaskCandidateReview,
    store,
    suggestedTaskCandidates,
    taskCandidates,
  }
}
