import {
	fetchRelationshipReviewItems,
	fetchRelationships,
	reviewRelationship,
	type Relationship,
	type RelationshipEntityKind,
	type RelationshipReviewState
} from '$lib/api';

export type RelationshipReviewWorkspaceState = {
	relationships: Relationship[];
	suggestedCount: number;
	error: string;
};

export async function loadRelationshipReviewState(
	entityKind: RelationshipEntityKind,
	entityId: string
): Promise<RelationshipReviewWorkspaceState> {
	try {
		const response = await fetchRelationships({
			entityKind,
			entityId,
			limit: 50
		});
		const relationships = response.items;
		return {
			relationships,
			suggestedCount: relationships.filter((item) => item.review_state === 'suggested').length,
			error: ''
		};
	} catch (error) {
		return {
			relationships: [],
			suggestedCount: 0,
			error: error instanceof Error ? error.message : 'Unknown relationship review error'
		};
	}
}

export async function loadGlobalRelationshipReviewState(): Promise<RelationshipReviewWorkspaceState> {
	try {
		const response = await fetchRelationshipReviewItems({
			reviewState: 'suggested',
			limit: 50
		});
		const relationships = response.items;
		return {
			relationships,
			suggestedCount: relationships.filter((item) => item.review_state === 'suggested').length,
			error: ''
		};
	} catch (error) {
		return {
			relationships: [],
			suggestedCount: 0,
			error: error instanceof Error ? error.message : 'Unknown global relationship review error'
		};
	}
}

export async function reviewRelationshipItem(
	relationship: Relationship,
	reviewState: Exclude<RelationshipReviewState, 'suggested' | 'system_accepted'>
): Promise<{ error: string }> {
	try {
		await reviewRelationship(relationship.relationship_id, { review_state: reviewState });
		return { error: '' };
	} catch (error) {
		return {
			error: error instanceof Error ? error.message : 'Unknown relationship review action error'
		};
	}
}

export function formatRelationshipEndpoint(kind: RelationshipEntityKind, entityId: string): string {
	return `${formatEntityKind(kind)} · ${entityId}`;
}

export function formatRelationshipType(value: string): string {
	return value
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

export function formatRelationshipScore(value: number): string {
	return `${Math.round(value * 100)}%`;
}

function formatEntityKind(kind: string): string {
	return kind
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}
