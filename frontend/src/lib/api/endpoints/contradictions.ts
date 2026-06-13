import { ApiClient } from '../client';
import type {
	ContradictionListResponse,
	ContradictionObservation,
	ContradictionReviewRequest
} from '../types';

export async function fetchContradictions(limit = 50): Promise<ContradictionListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<ContradictionListResponse>(
		`/api/v1/contradictions?${params.toString()}`,
		'Contradictions request failed'
	);
}

export async function reviewContradiction(
	observationId: string,
	request: ContradictionReviewRequest
): Promise<ContradictionObservation> {
	return ApiClient.instance.put<ContradictionObservation>(
		`/api/v1/contradictions/${encodeURIComponent(observationId)}/review`,
		request,
		'Contradiction review request failed'
	);
}
