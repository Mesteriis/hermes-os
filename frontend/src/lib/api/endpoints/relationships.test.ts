import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiClient } from '../client';
import {
	fetchRelationshipReviewItems,
	fetchRelationships,
	reviewRelationship
} from './relationships';

const relationship = {
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

describe('relationships API endpoints', () => {
	beforeEach(() => {
		ApiClient.init('http://127.0.0.1:8080', 'local-secret');
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				if (url.includes('/review')) {
					return new Response(JSON.stringify({ ...relationship, review_state: 'user_confirmed' }), {
						status: 200,
						headers: { 'Content-Type': 'application/json' }
					});
				}

				return new Response(JSON.stringify({ items: [relationship] }), {
					status: 200,
					headers: { 'Content-Type': 'application/json' }
				});
			})
		);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('requests entity-scoped relationships with the configured local API secret', async () => {
		const response = await fetchRelationships({
			entityKind: 'persona',
			entityId: 'person:v1:email:alex@example.com',
			limit: 20
		});

		expect(response.items).toEqual([relationship]);
		const fetchMock = vi.mocked(fetch);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/relationships?entity_kind=persona&entity_id=person%3Av1%3Aemail%3Aalex%40example.com&limit=20'
		);
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('requests global relationship review items by review state', async () => {
		const response = await fetchRelationshipReviewItems({
			reviewState: 'suggested',
			limit: 30
		});

		expect(response.items).toEqual([relationship]);
		const fetchMock = vi.mocked(fetch);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/relationships?review_state=suggested&limit=30'
		);
		expect(init?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
	});

	it('submits explicit review state to the backend relationship review route', async () => {
		await reviewRelationship('relationship with spaces', { review_state: 'user_confirmed' });

		const fetchMock = vi.mocked(fetch);
		const [url, init] = fetchMock.mock.calls[0];
		expect(url).toBe(
			'http://127.0.0.1:8080/api/v1/relationships/relationship%20with%20spaces/review'
		);
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
