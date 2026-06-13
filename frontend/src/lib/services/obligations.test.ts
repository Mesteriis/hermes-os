import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({
	fetchObligationReviewItems: vi.fn(),
	fetchObligations: vi.fn(),
	reviewObligation: vi.fn()
}));

import { fetchObligationReviewItems, fetchObligations, reviewObligation } from '$lib/api';
import type { Obligation } from '$lib/api';
import {
	formatObligationDueTime,
	loadGlobalObligationReviewState,
	loadObligationReviewState,
	reviewObligationItem
} from './obligations';

const obligation: Obligation = {
	obligation_id: 'obligation:v1:persona:send-package',
	obligated_entity_kind: 'persona',
	obligated_entity_id: 'person:v1:email:alex@example.com',
	beneficiary_entity_kind: 'project',
	beneficiary_entity_id: 'project:v1:hermes',
	statement: 'Send evidence package',
	status: 'open',
	review_state: 'suggested',
	due_at: null,
	condition: null,
	risk_state: 'none',
	confidence: 0.82,
	metadata: {},
	created_at: '2026-06-12T12:00:00Z',
	updated_at: '2026-06-12T12:00:00Z'
};

describe('Obligation review service', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('loads entity-scoped obligations from the backend review route', async () => {
		vi.mocked(fetchObligations).mockResolvedValue({ items: [obligation] });

		const result = await loadObligationReviewState('persona', 'person:v1:email:alex@example.com');

		expect(fetchObligations).toHaveBeenCalledWith({
			entityKind: 'persona',
			entityId: 'person:v1:email:alex@example.com',
			limit: 50
		});
		expect(result.error).toBe('');
		expect(result.obligations).toEqual([obligation]);
		expect(result.suggestedCount).toBe(1);
	});

	it('loads global suggested obligation review items', async () => {
		vi.mocked(fetchObligationReviewItems).mockResolvedValue({ items: [obligation] });

		const result = await loadGlobalObligationReviewState();

		expect(fetchObligationReviewItems).toHaveBeenCalledWith({
			reviewState: 'suggested',
			limit: 50
		});
		expect(result.error).toBe('');
		expect(result.obligations).toEqual([obligation]);
		expect(result.suggestedCount).toBe(1);
	});

	it('submits explicit review state without creating task links locally', async () => {
		vi.mocked(reviewObligation).mockResolvedValue({
			...obligation,
			review_state: 'user_rejected'
		});

		const result = await reviewObligationItem(obligation, 'user_rejected');

		expect(reviewObligation).toHaveBeenCalledWith(obligation.obligation_id, {
			review_state: 'user_rejected'
		});
		expect(result.error).toBe('');
	});

	it('formats missing and invalid due dates for compact review cards', () => {
		expect(formatObligationDueTime('not-a-date')).toBe('Unknown date');
		expect(formatObligationDueTime(null)).toBe('No due date');
	});
});
