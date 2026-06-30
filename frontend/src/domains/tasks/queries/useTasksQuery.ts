import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchDecisionReviewItems,
  fetchDecisions,
  fetchObligationReviewItems,
  fetchObligations,
  fetchTaskCandidates,
  fetchTaskRecords,
  reviewDecision,
  reviewObligation,
  reviewTaskCandidate,
} from '../api/tasks'
import type {
  Decision,
  DecisionEntityKind,
  DecisionReviewRequest,
  Obligation,
  ObligationReviewRequest,
  TaskCandidate,
  Task,
  TaskCandidateReviewState,
} from '../types/task'

export const taskQueryKeys = {
  candidates: ['task-candidates'] as const,
  tasks: ['tasks'] as const,
  contextReview: (entityKind: string, entityId: string) =>
    ['task-context-review', entityKind, entityId || 'global'] as const,
}

export function useTaskCandidatesQuery() {
  return useQuery<TaskCandidate[]>({
    queryKey: taskQueryKeys.candidates,
    queryFn: async () => {
      const response = await fetchTaskCandidates(50)
      return response.items
    }
  })
}

export function useTasksQuery() {
  return useQuery<Task[]>({
    queryKey: taskQueryKeys.tasks,
    queryFn: async () => {
      const response = await fetchTaskRecords({ limit: 50 })
      return response.items
    }
  })
}

export function useTaskContextReviewQuery(
  entityKind: MaybeRefOrGetter<DecisionEntityKind>,
  entityId: MaybeRefOrGetter<string>,
  limit: MaybeRefOrGetter<number> = 50
) {
  return useQuery<{ decisions: Decision[]; obligations: Obligation[] }>({
    queryKey: computed(() =>
      taskQueryKeys.contextReview(toValue(entityKind), toValue(entityId).trim())
    ),
    queryFn: async () => {
      const resolvedEntityKind = toValue(entityKind)
      const resolvedEntityId = toValue(entityId).trim()
      const resolvedLimit = toValue(limit)

      if (resolvedEntityId) {
        const [decisions, obligations] = await Promise.all([
          fetchDecisions({ entityKind: resolvedEntityKind, entityId: resolvedEntityId, limit: resolvedLimit }),
          fetchObligations({ entityKind: resolvedEntityKind, entityId: resolvedEntityId, limit: resolvedLimit }),
        ])
        return {
          decisions: decisions.items,
          obligations: obligations.items,
        }
      }

      const [decisions, obligations] = await Promise.all([
        fetchDecisionReviewItems({ reviewState: 'suggested', limit: resolvedLimit }),
        fetchObligationReviewItems({ reviewState: 'suggested', limit: resolvedLimit }),
      ])

      return {
        decisions: decisions.items,
        obligations: obligations.items,
      }
    },
  })
}

export function useReviewTaskCandidateMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ taskCandidateId, reviewState }: { taskCandidateId: string; reviewState: TaskCandidateReviewState }) =>
      reviewTaskCandidate(taskCandidateId, reviewState),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: taskQueryKeys.candidates })
      queryClient.invalidateQueries({ queryKey: taskQueryKeys.tasks })
    },
  })
}

export function useReviewDecisionMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ decisionId, request }: { decisionId: string; request: DecisionReviewRequest }) =>
      reviewDecision(decisionId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['task-context-review'] })
    },
  })
}

export function useReviewObligationMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ obligationId, request }: { obligationId: string; request: ObligationReviewRequest }) =>
      reviewObligation(obligationId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['task-context-review'] })
    },
  })
}
