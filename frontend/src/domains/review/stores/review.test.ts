import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { ApiClient } from '../../../platform/api/ApiClient'
import { useReviewStore } from './review'
import type { AttentionCard, ReviewItem } from '../types/review'

beforeEach(() => {
	setActivePinia(createPinia())
	ApiClient.resetForTests()
	ApiClient.init('http://127.0.0.1:8080', 'test-secret')
})

afterEach(() => {
	vi.unstubAllGlobals()
	ApiClient.resetForTests()
})

describe('review store attention cards', () => {
	it('loads Attention Cards with the Review workspace', async () => {
		const card = attentionCard()
		const fetchMock = vi
			.fn()
			.mockResolvedValueOnce(ok({ relationships: [] }))
			.mockResolvedValueOnce(ok({ items: [] }))
			.mockResolvedValueOnce(ok({ items: [] }))
			.mockResolvedValueOnce(ok({ items: [] }))
			.mockResolvedValueOnce(ok({ items: [reviewItem()] }))
			.mockResolvedValueOnce(ok({ cards: [card] }))
		vi.stubGlobal('fetch', fetchMock)

		const store = useReviewStore()
		await store.loadAll()

		expect(store.attentionCards).toEqual([card])
		expect(store.attentionCardsCount).toBe(1)
		expect(fetchMock.mock.calls[5][0]).toBe(
			'http://127.0.0.1:8080/api/v1/review/attention-cards?status=active&limit=50'
		)
	})

	it('refreshes Attention Cards after a Review item is dismissed', async () => {
		const item = reviewItem()
		const dismissedItem: ReviewItem = { ...item, status: 'dismissed' }
		const fetchMock = vi
			.fn()
			.mockResolvedValueOnce(ok({ relationships: [] }))
			.mockResolvedValueOnce(ok({ items: [] }))
			.mockResolvedValueOnce(ok({ items: [] }))
			.mockResolvedValueOnce(ok({ items: [] }))
			.mockResolvedValueOnce(ok({ items: [item] }))
			.mockResolvedValueOnce(ok({ cards: [attentionCard()] }))
			.mockResolvedValueOnce(ok(dismissedItem))
			.mockResolvedValueOnce(ok({ cards: [] }))
		vi.stubGlobal('fetch', fetchMock)

		const store = useReviewStore()
		await store.loadAll()

		const error = await store.reviewItem({ kind: 'review_item', item, action: 'dismiss' })

		expect(error).toBe('')
		expect(store.reviewItems[0].status).toBe('dismissed')
		expect(store.attentionCards).toEqual([])
		expect(fetchMock.mock.calls[6][0]).toBe(
			'http://127.0.0.1:8080/api/v1/review/items/review-1/dismiss'
		)
		expect(fetchMock.mock.calls[7][0]).toBe(
			'http://127.0.0.1:8080/api/v1/review/attention-cards?status=active&limit=50'
		)
	})
})

function ok(body: unknown): Response {
	return new Response(JSON.stringify(body), {
		status: 200,
		headers: { 'Content-Type': 'application/json' }
	})
}

function reviewItem(): ReviewItem {
	return {
		review_item_id: 'review-1',
		item_kind: 'potential_task',
		title: 'Contract review reminder',
		summary: 'A source-backed message suggests a deadline-backed task.',
		status: 'new',
		target_domain: null,
		target_entity_kind: null,
		target_entity_id: null,
		confidence: 0.91,
		metadata: {},
		created_at: '2026-06-18T11:00:00Z',
		updated_at: '2026-06-18T11:00:00Z'
	}
}

function attentionCard(): AttentionCard {
	return {
		id: 'attention:review:contract-deadline',
		title: 'Contract review reminder',
		summary: 'A source-backed message suggests a deadline-backed task.',
		importance: 'high',
		confidence: 0.91,
		evidence_count: 1,
		related_entities: [],
		trace_id: 'trace-1',
		review_item_ids: ['review-1'],
		suggested_actions: [
			{ action_kind: 'dismiss', label: 'Dismiss', target_domain: null, target_entity_kind: null }
		],
		source_summary: 'Review item has 1 canonical observation evidence reference(s).',
		explainability: {
			why_this_matters: 'This potential task has source evidence.',
			evidence: [{ observation_id: 'obs-1', role: 'primary' }],
			confidence: {
				score: 0.91,
				rationale: '91% confidence from 1 evidence source(s).'
			},
			related_objects: [],
			suggested_actions: [
				{ action_kind: 'dismiss', label: 'Dismiss', target_domain: null, target_entity_kind: null }
			]
		}
	}
}
