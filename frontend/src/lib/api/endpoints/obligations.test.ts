import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiClient } from '../client';
import { fetchObligationReviewItems, fetchObligations, reviewObligation } from './obligations';

const obligation = {
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

describe('obligations API endpoints', () => {
	beforeEach(() => {
		ApiClient.init('http://127.0.0.1:8080', 'local-secret');
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				if (url.includes('/review')) {
					return new Response(JSON.stringify({ ...obligation, review_state: 'user_rejected' }), {
						status: 200,
						headers: { 'Content-Type': 'application/json' }
					});
				}

				return new Response(JSON.stringify({ items: [obligation] }), {
					status: 200,
					headers: { 'Content-Type': 'application/json' }
				});
			})
		);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('requests entity-scoped obligations with the configured local API secret', async () => {
		const response = await fetchObligations({
			entityKind: 'persona',
			entityId: 'person:v1:email:alex@example.com',
			limit: 20
		});

		expect(response.items).toEqual([obligation]);
		const fetchMock = vi.mocked(fetch);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/obligations?entity_kind=persona&entity_id=person%3Av1%3Aemail%3Aalex%40example.com&limit=20'
		);
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('requests global obligation review items by review state', async () => {
		const response = await fetchObligationReviewItems({
			reviewState: 'suggested',
			limit: 30
		});

		expect(response.items).toEqual([obligation]);
		const fetchMock = vi.mocked(fetch);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/obligations?review_state=suggested&limit=30'
		);
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('submits explicit review state without creating task links locally', async () => {
		await reviewObligation('obligation with spaces', { review_state: 'user_rejected' });

		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/obligations/obligation%20with%20spaces/review'
		);
		expect(init?.method).toBe('PUT');
		expect(init?.headers).toEqual({
			'Content-Type': 'application/json',
			'X-Hermes-Secret': 'local-secret'
		});
		expect(JSON.parse(String(init?.body))).toEqual({
			review_state: 'user_rejected'
		});
	});
});
