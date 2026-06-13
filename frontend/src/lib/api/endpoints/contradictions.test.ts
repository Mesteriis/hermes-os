import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiClient } from '../client';
import { fetchContradictions, reviewContradiction } from './contradictions';

const observation = {
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
	metadata: {},
	reviewed_by: null,
	reviewed_at: null,
	resolution: null,
	created_at: '2026-06-13T01:00:00Z',
	updated_at: '2026-06-13T01:00:00Z'
};

describe('contradictions API endpoints', () => {
	beforeEach(() => {
		ApiClient.init('http://127.0.0.1:8080', 'local-secret');
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string, init?: RequestInit) => {
				if (url.includes('/review')) {
					return new Response(JSON.stringify({ ...observation, review_state: 'user_confirmed' }), {
						status: 200,
						headers: { 'Content-Type': 'application/json' }
					});
				}

				return new Response(JSON.stringify({ items: [observation] }), {
					status: 200,
					headers: { 'Content-Type': 'application/json' }
				});
			})
		);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('requests open contradiction observations with the configured local API secret', async () => {
		const response = await fetchContradictions(25);

		expect(response.items).toEqual([observation]);
		const fetchMock = vi.mocked(fetch);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe('http://127.0.0.1:8080/api/v1/contradictions?limit=25');
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('submits review state and resolution to the backend review route', async () => {
		await reviewContradiction('contradiction with spaces', {
			review_state: 'user_confirmed',
			resolution: 'Confirmed by owner'
		});

		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/contradictions/contradiction%20with%20spaces/review'
		);
		expect(init?.method).toBe('PUT');
		expect(init?.headers).toEqual({
			'Content-Type': 'application/json',
			'X-Hermes-Secret': 'local-secret'
		});
		expect(JSON.parse(String(init?.body))).toEqual({
			review_state: 'user_confirmed',
			resolution: 'Confirmed by owner'
		});
	});
});
