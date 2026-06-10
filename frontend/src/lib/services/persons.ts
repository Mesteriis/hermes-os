import {
	fetchPersons,
	fetchOrganizations,
	fetchIdentityCandidates,
	reviewIdentityCandidate,
	type EnrichedPerson,
	type Organization,
	type PersonIdentityCandidate,
	type PersonIdentityReviewState
} from '$lib/api';

export async function loadOrganizations(): Promise<{
	organizations: Organization[];
	error: string;
}> {
	try {
		const response = await fetchOrganizations();
		return { organizations: response.items, error: '' };
	} catch (error) {
		return {
			organizations: [],
			error: error instanceof Error ? error.message : 'Unknown organizations error'
		};
	}
}

export async function loadPersons(): Promise<{ persons: EnrichedPerson[]; error: string }> {
	try {
		const response = await fetchPersons();
		return { persons: response.items, error: '' };
	} catch (error) {
		return {
			persons: [],
			error: error instanceof Error ? error.message : 'Unknown persons error'
		};
	}
}

export async function loadIdentityCandidates(): Promise<{
	candidates: PersonIdentityCandidate[];
	error: string;
}> {
	try {
		const response = await fetchIdentityCandidates(50);
		return { candidates: response.items, error: '' };
	} catch (error) {
		return {
			candidates: [],
			error: error instanceof Error ? error.message : 'Unknown identity candidate error'
		};
	}
}

export async function setIdentityCandidateReview(
	candidate: PersonIdentityCandidate,
	reviewState: PersonIdentityReviewState
): Promise<{ error: string }> {
	try {
		await reviewIdentityCandidate(
			candidate.identity_candidate_id,
			reviewState
		);
		return { error: '' };
	} catch (error) {
		return {
			error: error instanceof Error ? error.message : 'Unknown identity review error'
		};
	}
}

export async function splitConfirmedIdentityMerge(
	candidate: PersonIdentityCandidate,
	splitCandidate: PersonIdentityCandidate | null
): Promise<{ error: string }> {
	if (!splitCandidate) {
		return { error: '' };
	}

	const commandId = `person-identity-split-${Date.now()}-${candidate.identity_candidate_id}`;
	try {
		await reviewIdentityCandidate(
			splitCandidate.identity_candidate_id,
			'user_confirmed',
			commandId
		);
		return { error: '' };
	} catch (error) {
		return {
			error: error instanceof Error ? error.message : 'Unknown identity split review error'
		};
	}
}

export function identityConfidence(item: PersonIdentityCandidate) {
	return `${Math.round(item.confidence * 100)}%`;
}

export function splitCandidateForConfirmedMerge(
	candidate: PersonIdentityCandidate,
	identityCandidates: PersonIdentityCandidate[]
) {
	return splitCandidateForMerge(candidate, 'suggested', identityCandidates);
}

export function confirmedSplitCandidateForMerge(
	candidate: PersonIdentityCandidate,
	identityCandidates: PersonIdentityCandidate[]
) {
	return splitCandidateForMerge(candidate, 'user_confirmed', identityCandidates);
}

function splitCandidateForMerge(
	candidate: PersonIdentityCandidate,
	reviewState: PersonIdentityReviewState,
	identityCandidates: PersonIdentityCandidate[]
): PersonIdentityCandidate | null {
	if (!candidate.right_person_id) {
		return null;
	}
	const pairKey = personIdentityPairKey(candidate.left_person_id, candidate.right_person_id);
	return (
		identityCandidates.find(
			(item) =>
				item.candidate_kind === 'split_person' &&
				item.review_state === reviewState &&
				item.right_person_id !== null &&
				personIdentityPairKey(item.left_person_id, item.right_person_id) === pairKey
		) ?? null
	);
}

export function personIdentityPairKey(leftPersonId: string, rightPersonId: string) {
	return leftPersonId <= rightPersonId
		? `${leftPersonId}:${rightPersonId}`
		: `${rightPersonId}:${leftPersonId}`;
}
