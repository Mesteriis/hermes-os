import { ApiClient } from '../client';
import type {
	Decision,
	DecisionEntityKind,
	DecisionListResponse,
	DecisionReviewRequest,
	DecisionReviewState
} from '../types';

export type DecisionListParams = {
	entityKind: DecisionEntityKind;
	entityId: string;
	limit?: number;
};

export type DecisionReviewListParams = {
	reviewState: DecisionReviewState;
	limit?: number;
};

export async function fetchDecisions(params: DecisionListParams): Promise<DecisionListResponse> {
	const query = new URLSearchParams({
		entity_kind: params.entityKind,
		entity_id: params.entityId,
		limit: String(Math.trunc(params.limit ?? 50))
	});
	return ApiClient.instance.get<DecisionListResponse>(
		`/api/v1/decisions?${query.toString()}`,
		'Decisions request failed'
	);
}

export async function fetchDecisionReviewItems(
	params: DecisionReviewListParams
): Promise<DecisionListResponse> {
	const query = new URLSearchParams({
		review_state: params.reviewState,
		limit: String(Math.trunc(params.limit ?? 50))
	});
	return ApiClient.instance.get<DecisionListResponse>(
		`/api/v1/decisions?${query.toString()}`,
		'Decision review items request failed'
	);
}

export async function reviewDecision(
	decisionId: string,
	request: DecisionReviewRequest
): Promise<Decision> {
	return ApiClient.instance.put<Decision>(
		`/api/v1/decisions/${encodeURIComponent(decisionId)}/review`,
		request,
		'Decision review request failed'
	);
}
