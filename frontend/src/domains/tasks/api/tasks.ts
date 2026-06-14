import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TaskCandidateListResponse,
  TaskCandidateReviewState,
  TaskRecordsResponse,
  Task,
  Decision,
  DecisionEntityKind,
  DecisionReviewRequest,
  DecisionListResponse,
  Obligation,
  ObligationEntityKind,
  ObligationReviewRequest,
  ObligationListResponse
} from '../types/task'

// --- Task Candidates ---
export async function fetchTaskCandidates(limit = 50): Promise<TaskCandidateListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<TaskCandidateListResponse>(
    `/api/v1/task-candidates?${params.toString()}`,
    'Task candidates request failed'
  )
}

export async function reviewTaskCandidate(
  taskCandidateId: string,
  reviewState: TaskCandidateReviewState
): Promise<void> {
  return ApiClient.instance.put(
    `/api/v1/task-candidates/${encodeURIComponent(taskCandidateId)}/review`,
    {
      command_id: `task-candidate-review-${crypto.randomUUID()}`,
      review_state: reviewState
    }
  )
}

// --- Tasks ---
export async function fetchTaskRecords(
  params: { status?: string; project_id?: string; source_type?: string; limit?: number } = {}
): Promise<TaskRecordsResponse> {
  const sp = new URLSearchParams()
  if (params.status) sp.set('status', params.status)
  if (params.project_id) sp.set('project_id', params.project_id)
  if (params.source_type) sp.set('source_type', params.source_type)
  if (params.limit) sp.set('limit', String(params.limit))
  return ApiClient.instance.get<TaskRecordsResponse>(
    `/api/v1/tasks?${sp.toString()}`,
    'Tasks request failed'
  )
}

export async function updateTask(taskId: string, body: Record<string, unknown>): Promise<Task> {
  return ApiClient.instance.put<Task>(
    `/api/v1/tasks/${encodeURIComponent(taskId)}`,
    body,
    'Update task failed'
  )
}

export async function setTaskStatus(taskId: string, status: string): Promise<{ status: string }> {
  return ApiClient.instance.post<{ status: string }>(
    `/api/v1/tasks/${encodeURIComponent(taskId)}/status`,
    { status },
    'Set status failed'
  )
}

// --- Decisions ---
export async function fetchDecisions(params: {
  entityKind: DecisionEntityKind
  entityId: string
  limit?: number
}): Promise<DecisionListResponse> {
  const query = new URLSearchParams({
    entity_kind: params.entityKind,
    entity_id: params.entityId,
    limit: String(Math.trunc(params.limit ?? 50))
  })
  return ApiClient.instance.get<DecisionListResponse>(
    `/api/v1/decisions?${query.toString()}`,
    'Decisions request failed'
  )
}

export async function fetchDecisionReviewItems(params: {
  reviewState: string
  limit?: number
}): Promise<DecisionListResponse> {
  const query = new URLSearchParams({
    review_state: params.reviewState,
    limit: String(Math.trunc(params.limit ?? 50))
  })
  return ApiClient.instance.get<DecisionListResponse>(
    `/api/v1/decisions?${query.toString()}`,
    'Decision review items request failed'
  )
}

export async function reviewDecision(
  decisionId: string,
  request: DecisionReviewRequest
): Promise<Decision> {
  return ApiClient.instance.put<Decision>(
    `/api/v1/decisions/${encodeURIComponent(decisionId)}/review`,
    request,
    'Decision review request failed'
  )
}

// --- Obligations ---
export async function fetchObligations(params: {
  entityKind: ObligationEntityKind
  entityId: string
  limit?: number
}): Promise<ObligationListResponse> {
  const query = new URLSearchParams({
    entity_kind: params.entityKind,
    entity_id: params.entityId,
    limit: String(Math.trunc(params.limit ?? 50))
  })
  return ApiClient.instance.get<ObligationListResponse>(
    `/api/v1/obligations?${query.toString()}`,
    'Obligations request failed'
  )
}

export async function fetchObligationReviewItems(params: {
  reviewState: string
  limit?: number
}): Promise<ObligationListResponse> {
  const query = new URLSearchParams({
    review_state: params.reviewState,
    limit: String(Math.trunc(params.limit ?? 50))
  })
  return ApiClient.instance.get<ObligationListResponse>(
    `/api/v1/obligations?${query.toString()}`,
    'Obligation review items request failed'
  )
}

export async function reviewObligation(
  obligationId: string,
  request: ObligationReviewRequest
): Promise<Obligation> {
  return ApiClient.instance.put<Obligation>(
    `/api/v1/obligations/${encodeURIComponent(obligationId)}/review`,
    request,
    'Obligation review request failed'
  )
}
