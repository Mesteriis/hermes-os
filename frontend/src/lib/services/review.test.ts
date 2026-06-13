import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/services/relationships', () => ({
	loadGlobalRelationshipReviewState: vi.fn(),
	reviewRelationshipItem: vi.fn()
}));

vi.mock('$lib/services/decisions', () => ({
	loadGlobalDecisionReviewState: vi.fn(),
	reviewDecisionItem: vi.fn()
}));

vi.mock('$lib/services/obligations', () => ({
	loadGlobalObligationReviewState: vi.fn(),
	reviewObligationItem: vi.fn()
}));

vi.mock('$lib/services/contradictions', () => ({
	loadContradictionReviewState: vi.fn(),
	reviewContradictionObservation: vi.fn()
}));

import { loadGlobalRelationshipReviewState, reviewRelationshipItem } from '$lib/services/relationships';
import { loadGlobalDecisionReviewState, reviewDecisionItem } from '$lib/services/decisions';
import { loadGlobalObligationReviewState, reviewObligationItem } from '$lib/services/obligations';
import {
	loadContradictionReviewState,
	reviewContradictionObservation
} from '$lib/services/contradictions';
import { loadReviewWorkspace, reviewWorkspaceItem } from './review';

describe('cross-domain review workspace service', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('aggregates suggested review queues across domains and engines', async () => {
		vi.mocked(loadGlobalRelationshipReviewState).mockResolvedValue({
			relationships: [{ relationship_id: 'relationship-1', review_state: 'suggested' }],
			suggestedCount: 1,
			error: ''
		} as never);
		vi.mocked(loadGlobalDecisionReviewState).mockResolvedValue({
			decisions: [{ decision_id: 'decision-1', review_state: 'suggested' }],
			suggestedCount: 1,
			error: ''
		} as never);
		vi.mocked(loadGlobalObligationReviewState).mockResolvedValue({
			obligations: [{ obligation_id: 'obligation-1', review_state: 'suggested' }],
			suggestedCount: 1,
			error: ''
		} as never);
		vi.mocked(loadContradictionReviewState).mockResolvedValue({
			observations: [{ observation_id: 'contradiction-1', review_state: 'suggested' }],
			suggestedCount: 1,
			error: ''
		} as never);

		const result = await loadReviewWorkspace();

		expect(loadGlobalRelationshipReviewState).toHaveBeenCalledTimes(1);
		expect(loadGlobalDecisionReviewState).toHaveBeenCalledTimes(1);
		expect(loadGlobalObligationReviewState).toHaveBeenCalledTimes(1);
		expect(loadContradictionReviewState).toHaveBeenCalledTimes(1);
		expect(result.totalSuggestedCount).toBe(4);
		expect(result.error).toBe('');
	});

	it('keeps partial review queues visible when one source fails', async () => {
		vi.mocked(loadGlobalRelationshipReviewState).mockResolvedValue({
			relationships: [],
			suggestedCount: 0,
			error: 'Relationship review unavailable'
		} as never);
		vi.mocked(loadGlobalDecisionReviewState).mockResolvedValue({
			decisions: [{ decision_id: 'decision-1', review_state: 'suggested' }],
			suggestedCount: 1,
			error: ''
		} as never);
		vi.mocked(loadGlobalObligationReviewState).mockResolvedValue({
			obligations: [],
			suggestedCount: 0,
			error: ''
		} as never);
		vi.mocked(loadContradictionReviewState).mockResolvedValue({
			observations: [],
			suggestedCount: 0,
			error: ''
		} as never);

		const result = await loadReviewWorkspace();

		expect(result.totalSuggestedCount).toBe(1);
		expect(result.error).toBe('Relationship review unavailable');
	});

	it('routes relationship review actions through the cross-domain policy', async () => {
		const relationship = {
			relationship_id: 'relationship-1',
			review_state: 'suggested'
		};
		vi.mocked(reviewRelationshipItem).mockResolvedValue({ error: '' });

		const result = await reviewWorkspaceItem({
			kind: 'relationship',
			item: relationship,
			reviewState: 'user_confirmed'
		} as never);

		expect(reviewRelationshipItem).toHaveBeenCalledWith(relationship, 'user_confirmed');
		expect(result.itemKey).toBe('relationship:relationship-1');
		expect(result.error).toBe('');
	});

	it('routes decision, obligation and contradiction review actions through one service entrypoint', async () => {
		const decision = { decision_id: 'decision-1', review_state: 'suggested' };
		const obligation = { obligation_id: 'obligation-1', review_state: 'suggested' };
		const observation = { observation_id: 'contradiction-1', review_state: 'suggested' };
		vi.mocked(reviewDecisionItem).mockResolvedValue({ error: '' });
		vi.mocked(reviewObligationItem).mockResolvedValue({ error: '' });
		vi.mocked(reviewContradictionObservation).mockResolvedValue({ error: '' });

		await reviewWorkspaceItem({
			kind: 'decision',
			item: decision,
			reviewState: 'user_rejected'
		} as never);
		await reviewWorkspaceItem({
			kind: 'obligation',
			item: obligation,
			reviewState: 'user_confirmed'
		} as never);
		await reviewWorkspaceItem({
			kind: 'contradiction',
			item: observation,
			reviewState: 'user_rejected'
		} as never);

		expect(reviewDecisionItem).toHaveBeenCalledWith(decision, 'user_rejected');
		expect(reviewObligationItem).toHaveBeenCalledWith(obligation, 'user_confirmed');
		expect(reviewContradictionObservation).toHaveBeenCalledWith(observation, 'user_rejected');
	});
});
