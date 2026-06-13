import { ApiClient } from '../client';
import type {
	Obligation,
	ObligationEntityKind,
	ObligationListResponse,
	ObligationReviewRequest,
	ObligationReviewState
} from '../types';

export type ObligationListParams = {
	entityKind: ObligationEntityKind;
	entityId: string;
	limit?: number;
};

export type ObligationReviewListParams = {
	reviewState: ObligationReviewState;
	limit?: number;
};

export async function fetchObligations(
	params: ObligationListParams
): Promise<ObligationListResponse> {
	const query = new URLSearchParams({
		entity_kind: params.entityKind,
		entity_id: params.entityId,
		limit: String(Math.trunc(params.limit ?? 50))
	});
	return ApiClient.instance.get<ObligationListResponse>(
		`/api/v1/obligations?${query.toString()}`,
		'Obligations request failed'
	);
}

export async function fetchObligationReviewItems(
	params: ObligationReviewListParams
): Promise<ObligationListResponse> {
	const query = new URLSearchParams({
		review_state: params.reviewState,
		limit: String(Math.trunc(params.limit ?? 50))
	});
	return ApiClient.instance.get<ObligationListResponse>(
		`/api/v1/obligations?${query.toString()}`,
		'Obligation review items request failed'
	);
}

export async function reviewObligation(
	obligationId: string,
	request: ObligationReviewRequest
): Promise<Obligation> {
	return ApiClient.instance.put<Obligation>(
		`/api/v1/obligations/${encodeURIComponent(obligationId)}/review`,
		request,
		'Obligation review request failed'
	);
}
