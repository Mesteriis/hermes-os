import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({
	fetchDecisionReviewItems: vi.fn(),
	fetchDecisions: vi.fn(),
	reviewDecision: vi.fn()
}));

import { fetchDecisionReviewItems, fetchDecisions, reviewDecision } from '$lib/api';
import type { Decision } from '$lib/api';
import {
	formatDecisionTime,
	loadDecisionReviewState,
	loadGlobalDecisionReviewState,
	reviewDecisionItem
} from './decisions';

const decision: Decision = {
	decision_id: 'decision:v1:project:local-first',
	title: 'Keep Hermes local-first',
	status: 'active',
	rationale: 'Private memory must remain under owner control.',
	alternatives: ['cloud-first memory', 'provider-owned sync'],
	decided_by_entity_kind: 'persona',
	decided_by_entity_id: 'person:v1:self',
	decided_at: '2026-06-12T11:00:00Z',
	review_state: 'suggested',
	confidence: 0.84,
	metadata: {},
	created_at: '2026-06-12T11:00:00Z',
	updated_at: '2026-06-12T11:00:00Z'
};

describe('Decision review service', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('loads entity-scoped decisions from the backend review route', async () => {
		vi.mocked(fetchDecisions).mockResolvedValue({ items: [decision] });

		const result = await loadDecisionReviewState('project', 'project:v1:hermes');

		expect(fetchDecisions).toHaveBeenCalledWith({
			entityKind: 'project',
			entityId: 'project:v1:hermes',
			limit: 50
		});
		expect(result.error).toBe('');
		expect(result.decisions).toEqual([decision]);
		expect(result.suggestedCount).toBe(1);
	});

	it('loads global suggested decision review items', async () => {
		vi.mocked(fetchDecisionReviewItems).mockResolvedValue({ items: [decision] });

		const result = await loadGlobalDecisionReviewState();

		expect(fetchDecisionReviewItems).toHaveBeenCalledWith({
			reviewState: 'suggested',
			limit: 50
		});
		expect(result.error).toBe('');
		expect(result.decisions).toEqual([decision]);
		expect(result.suggestedCount).toBe(1);
	});

	it('submits explicit review state without creating work locally', async () => {
		vi.mocked(reviewDecision).mockResolvedValue({
			...decision,
			review_state: 'user_confirmed'
		});

		const result = await reviewDecisionItem(decision, 'user_confirmed');

		expect(reviewDecision).toHaveBeenCalledWith(decision.decision_id, {
			review_state: 'user_confirmed'
		});
		expect(result.error).toBe('');
	});

	it('formats decision timestamps for compact review cards', () => {
		expect(formatDecisionTime('not-a-date')).toBe('Unknown date');
		expect(formatDecisionTime(null)).toBe('No decision date');
	});
});
