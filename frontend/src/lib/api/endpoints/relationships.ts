import { ApiClient } from '../client';
import type {
	Relationship,
	RelationshipEntityKind,
	RelationshipListResponse,
	RelationshipReviewRequest,
	RelationshipReviewState
} from '../types';

export type RelationshipListParams = {
	entityKind: RelationshipEntityKind;
	entityId: string;
	limit?: number;
};

export type RelationshipReviewListParams = {
	reviewState: RelationshipReviewState;
	limit?: number;
};

export async function fetchRelationships(
	params: RelationshipListParams
): Promise<RelationshipListResponse> {
	const query = new URLSearchParams({
		entity_kind: params.entityKind,
		entity_id: params.entityId,
		limit: String(Math.trunc(params.limit ?? 50))
	});
	return ApiClient.instance.get<RelationshipListResponse>(
		`/api/v1/relationships?${query.toString()}`,
		'Relationships request failed'
	);
}

export async function fetchRelationshipReviewItems(
	params: RelationshipReviewListParams
): Promise<RelationshipListResponse> {
	const query = new URLSearchParams({
		review_state: params.reviewState,
		limit: String(Math.trunc(params.limit ?? 50))
	});
	return ApiClient.instance.get<RelationshipListResponse>(
		`/api/v1/relationships?${query.toString()}`,
		'Relationship review items request failed'
	);
}

export async function reviewRelationship(
	relationshipId: string,
	request: RelationshipReviewRequest
): Promise<Relationship> {
	return ApiClient.instance.put<Relationship>(
		`/api/v1/relationships/${encodeURIComponent(relationshipId)}/review`,
		request,
		'Relationship review request failed'
	);
}
