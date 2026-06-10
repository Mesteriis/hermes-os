import { ApiClient } from '../client';
import type {
	PersonIdentityCandidateListResponse,
	PersonIdentityReviewState,
	PersonListResponse,
	EnrichedPerson
} from '../types';

export async function fetchIdentityCandidates(limit = 50): Promise<PersonIdentityCandidateListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<PersonIdentityCandidateListResponse>(
		`/api/v1/identity-candidates?${params.toString()}`,
		'Identity candidate request failed'
	);
}

export async function reviewIdentityCandidate(
	identityCandidateId: string,
	reviewState: PersonIdentityReviewState,
	commandId = `person-identity-review-${crypto.randomUUID()}`
) {
	return ApiClient.instance.put(
		`/api/v1/identity-candidates/${encodeURIComponent(identityCandidateId)}/review`,
		{
			command_id: commandId,
			review_state: reviewState
		}
	);
}

export async function fetchPersons(limit = 50, favoritesOnly = false): Promise<PersonListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	if (favoritesOnly) params.set('favorites_only', 'true');
	return ApiClient.instance.get<PersonListResponse>(
		`/api/v1/persons?${params.toString()}`,
		'Persons request failed'
	);
}

export async function fetchPerson(personId: string): Promise<EnrichedPerson> {
	return ApiClient.instance.get<EnrichedPerson>(
		`/api/v1/persons/${encodeURIComponent(personId)}`,
		'Person request failed'
	);
}
