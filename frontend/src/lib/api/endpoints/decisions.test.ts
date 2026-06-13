import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiClient } from '../client';
import { fetchDecisionReviewItems, fetchDecisions, reviewDecision } from './decisions';

const decision = {
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

describe('decisions API endpoints', () => {
	beforeEach(() => {
		ApiClient.init('http://127.0.0.1:8080', 'local-secret');
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				if (url.includes('/review')) {
					return new Response(JSON.stringify({ ...decision, review_state: 'user_confirmed' }), {
						status: 200,
						headers: { 'Content-Type': 'application/json' }
					});
				}

				return new Response(JSON.stringify({ items: [decision] }), {
					status: 200,
					headers: { 'Content-Type': 'application/json' }
				});
			})
		);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('requests entity-scoped decisions with the configured local API secret', async () => {
		const response = await fetchDecisions({
			entityKind: 'project',
			entityId: 'project:v1:alpha beta',
			limit: 25
		});

		expect(response.items).toEqual([decision]);
		const fetchMock = vi.mocked(fetch);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/decisions?entity_kind=project&entity_id=project%3Av1%3Aalpha+beta&limit=25'
		);
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('requests global decision review items by review state', async () => {
		const response = await fetchDecisionReviewItems({
			reviewState: 'suggested',
			limit: 30
		});

		expect(response.items).toEqual([decision]);
		const fetchMock = vi.mocked(fetch);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/decisions?review_state=suggested&limit=30'
		);
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('submits explicit review state to the backend review route', async () => {
		await reviewDecision('decision with spaces', { review_state: 'user_confirmed' });

		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe('http://127.0.0.1:8080/api/v1/decisions/decision%20with%20spaces/review');
		expect(init?.method).toBe('PUT');
		expect(init?.headers).toEqual({
			'Content-Type': 'application/json',
			'X-Hermes-Secret': 'local-secret'
		});
		expect(JSON.parse(String(init?.body))).toEqual({
			review_state: 'user_confirmed'
		});
	});
});
