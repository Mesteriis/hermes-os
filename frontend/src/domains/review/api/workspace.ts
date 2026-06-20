import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  ContradictionListResponse,
  ContradictionObservation,
  Decision,
  DecisionListResponse,
  Obligation,
  ObligationListResponse,
  RelationshipListResponse,
} from '../types/review'

export async function fetchRelationships(limit = 50): Promise<RelationshipListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<RelationshipListResponse>(
    `/api/v1/relationships?${params.toString()}`,
    'Relationships request failed'
  )
}

export async function reviewRelationship(
  relationshipId: string,
  reviewState: string
): Promise<void> {
  await ApiClient.instance.put(
    `/api/v1/relationships/${encodeURIComponent(relationshipId)}/review`,
    { review_state: reviewState }
  )
}

export async function fetchDecisionReviewItems(params: {
  reviewState: string
  limit?: number
}): Promise<DecisionListResponse> {
  const query = new URLSearchParams({
    review_state: params.reviewState,
    limit: String(Math.trunc(params.limit ?? 50)),
  })
  return ApiClient.instance.get<DecisionListResponse>(
    `/api/v1/decisions?${query.toString()}`,
    'Decision review items request failed'
  )
}

export async function reviewDecision(
  decisionId: string,
  request: { review_state: 'user_confirmed' | 'user_rejected' }
): Promise<Decision> {
  return ApiClient.instance.put<Decision>(
    `/api/v1/decisions/${encodeURIComponent(decisionId)}/review`,
    request,
    'Decision review request failed'
  )
}

export async function fetchObligationReviewItems(params: {
  reviewState: string
  limit?: number
}): Promise<ObligationListResponse> {
  const query = new URLSearchParams({
    review_state: params.reviewState,
    limit: String(Math.trunc(params.limit ?? 50)),
  })
  return ApiClient.instance.get<ObligationListResponse>(
    `/api/v1/obligations?${query.toString()}`,
    'Obligation review items request failed'
  )
}

export async function reviewObligation(
  obligationId: string,
  request: { review_state: 'user_confirmed' | 'user_rejected' }
): Promise<Obligation> {
  return ApiClient.instance.put<Obligation>(
    `/api/v1/obligations/${encodeURIComponent(obligationId)}/review`,
    request,
    'Obligation review request failed'
  )
}

export async function fetchContradictions(limit = 50): Promise<ContradictionListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<ContradictionListResponse>(
    `/api/v1/contradictions?${params.toString()}`,
    'Contradictions request failed'
  )
}

export async function reviewContradiction(
  observationId: string,
  request: { review_state: 'user_confirmed' | 'user_rejected' }
): Promise<ContradictionObservation> {
  return ApiClient.instance.put<ContradictionObservation>(
    `/api/v1/contradictions/${encodeURIComponent(observationId)}/review`,
    request,
    'Contradiction review request failed'
  )
}
