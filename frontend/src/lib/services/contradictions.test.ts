import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({
	fetchContradictions: vi.fn(),
	reviewContradiction: vi.fn()
}));

import { fetchContradictions, reviewContradiction } from '$lib/api';
import type { ContradictionObservation } from '$lib/api';
import {
	contradictionSeverityTone,
	formatContradictionClaim,
	loadContradictionReviewState,
	reviewContradictionObservation
} from './contradictions';

const observation: ContradictionObservation = {
	observation_id: 'contradiction:v1:memory:fact-1:communication:message-1',
	old_source_kind: 'memory',
	old_source_id: 'person-fact-1',
	new_source_kind: 'communication',
	new_source_id: 'message-1',
	affected_entities: [{ entity_kind: 'subject', entity_id: 'person:alex' }],
	conflict_type: 'direct_contradiction',
	old_claim: 'location=Berlin',
	new_claim: 'location=Madrid',
	confidence: 0.8,
	severity: 'medium',
	review_state: 'suggested',
	metadata: {
		detector: 'structured_evidence_claim',
		claim_type: 'location',
		source_kind: 'communication'
	},
	reviewed_by: null,
	reviewed_at: null,
	resolution: null,
	created_at: '2026-06-13T01:00:00Z',
	updated_at: '2026-06-13T01:00:00Z'
};

describe('Polygraph contradiction review service', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('loads open reviewable contradiction observations from the backend', async () => {
		vi.mocked(fetchContradictions).mockResolvedValue({ items: [observation] });

		const result = await loadContradictionReviewState();

		expect(fetchContradictions).toHaveBeenCalledWith(50);
		expect(result.error).toBe('');
		expect(result.observations).toEqual([observation]);
		expect(result.suggestedCount).toBe(1);
	});

	it('submits explicit review state and optional resolution without changing memory locally', async () => {
		vi.mocked(reviewContradiction).mockResolvedValue({
			...observation,
			review_state: 'user_rejected',
			reviewed_by: 'hermes-frontend',
			resolution: 'New message is outdated'
		});

		const result = await reviewContradictionObservation(
			observation,
			'user_rejected',
			'New message is outdated'
		);

		expect(reviewContradiction).toHaveBeenCalledWith(observation.observation_id, {
			review_state: 'user_rejected',
			resolution: 'New message is outdated'
		});
		expect(result.error).toBe('');
	});

	it('formats claim comparisons and severity tones for the review UI', () => {
		expect(formatContradictionClaim(observation)).toBe('location=Berlin -> location=Madrid');
		expect(contradictionSeverityTone('critical')).toBe('critical');
		expect(contradictionSeverityTone('high')).toBe('high');
		expect(contradictionSeverityTone('medium')).toBe('medium');
		expect(contradictionSeverityTone('low')).toBe('low');
	});
});
