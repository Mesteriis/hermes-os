import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({
	fetchRelationshipReviewItems: vi.fn(),
	fetchRelationships: vi.fn(),
	reviewRelationship: vi.fn()
}));

import { fetchRelationshipReviewItems, fetchRelationships, reviewRelationship } from '$lib/api';
import type { Relationship } from '$lib/api';
import {
	formatRelationshipEndpoint,
	formatRelationshipScore,
	loadGlobalRelationshipReviewState,
	loadRelationshipReviewState,
	reviewRelationshipItem
} from './relationships';

const relationship: Relationship = {
	relationship_id: 'relationship:v1:persona:alex:collaborates_with:persona:maria',
	source_entity_kind: 'persona',
	source_entity_id: 'person:v1:email:alex@example.com',
	target_entity_kind: 'persona',
	target_entity_id: 'person:v1:email:maria@example.com',
	relationship_type: 'collaborates_with',
	trust_score: 0.72,
	strength_score: 0.66,
	confidence: 0.88,
	review_state: 'suggested',
	valid_from: null,
	valid_to: null,
	metadata: {},
	created_at: '2026-06-13T02:00:00Z',
	updated_at: '2026-06-13T02:00:00Z'
};

describe('Relationship review service', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('loads entity-scoped relationships from the backend review route', async () => {
		vi.mocked(fetchRelationships).mockResolvedValue({ items: [relationship] });

		const result = await loadRelationshipReviewState(
			'persona',
			'person:v1:email:alex@example.com'
		);

		expect(fetchRelationships).toHaveBeenCalledWith({
			entityKind: 'persona',
			entityId: 'person:v1:email:alex@example.com',
			limit: 50
		});
		expect(result.error).toBe('');
		expect(result.relationships).toEqual([relationship]);
		expect(result.suggestedCount).toBe(1);
	});

	it('loads global suggested relationship review items', async () => {
		vi.mocked(fetchRelationshipReviewItems).mockResolvedValue({ items: [relationship] });

		const result = await loadGlobalRelationshipReviewState();

		expect(fetchRelationshipReviewItems).toHaveBeenCalledWith({
			reviewState: 'suggested',
			limit: 50
		});
		expect(result.error).toBe('');
		expect(result.relationships).toEqual([relationship]);
		expect(result.suggestedCount).toBe(1);
	});

	it('submits explicit review state without mutating relationship scores locally', async () => {
		vi.mocked(reviewRelationship).mockResolvedValue({
			...relationship,
			review_state: 'user_rejected'
		});

		const result = await reviewRelationshipItem(relationship, 'user_rejected');

		expect(reviewRelationship).toHaveBeenCalledWith(relationship.relationship_id, {
			review_state: 'user_rejected'
		});
		expect(result.error).toBe('');
	});

	it('formats endpoints and scores for compact review cards', () => {
		expect(formatRelationshipEndpoint('persona', 'person:v1:email:alex@example.com')).toBe(
			'Persona · person:v1:email:alex@example.com'
		);
		expect(formatRelationshipScore(0.724)).toBe('72%');
	});
});
