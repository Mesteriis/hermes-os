import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
	Relationship,
	Decision,
	Obligation,
	ContradictionObservation,
	ReviewItem,
	ReviewWorkspaceItemAction
} from '../types/review'
import {
	fetchRelationships,
	reviewRelationship,
	fetchDecisionReviewItems,
	reviewDecision,
	fetchObligationReviewItems,
	reviewObligation,
	fetchContradictions,
	reviewContradiction
} from '../api/workspace'
import {
	fetchReviewItems,
	approveReviewItem,
	dismissReviewItem,
	archiveReviewItem,
	promoteReviewItem,
	takeReviewItem
} from '../api/items'

export const useReviewStore = defineStore('review', () => {
	const relationships = ref<Relationship[]>([])
	const decisions = ref<Decision[]>([])
	const obligations = ref<Obligation[]>([])
	const contradictions = ref<ContradictionObservation[]>([])
	const reviewItems = ref<ReviewItem[]>([])
	const error = ref('')
	const reviewingItemKey = ref<string | null>(null)

	const relationsSuggestedCount = computed(() =>
		relationships.value.filter((r) => r.review_state === 'suggested').length
	)

	const decisionsSuggestedCount = computed(() =>
		decisions.value.filter((d) => d.review_state === 'suggested').length
	)

	const obligationsSuggestedCount = computed(() =>
		obligations.value.filter((o) => o.review_state === 'suggested').length
	)

	const contradictionsSuggestedCount = computed(() =>
		contradictions.value.filter((c) => c.review_state === 'suggested').length
	)
	const reviewItemsCount = computed(() => reviewItems.value.filter((r) => r.status === 'new' || r.status === 'in_review').length)

	const totalSuggestedCount = computed(() =>
		relationsSuggestedCount.value +
		decisionsSuggestedCount.value +
		obligationsSuggestedCount.value +
		contradictionsSuggestedCount.value +
		reviewItemsCount.value
	)

	async function loadAll() {
		error.value = ''
		const errors: string[] = []

		try {
			const relRes = await fetchRelationships(50)
			relationships.value = relRes.relationships || []
		} catch (e) {
			errors.push(`Relationships: ${e instanceof Error ? e.message : 'Unknown error'}`)
		}

		try {
			const decRes = await fetchDecisionReviewItems({ reviewState: 'suggested', limit: 50 })
			decisions.value = decRes.items || []
		} catch (e) {
			errors.push(`Decisions: ${e instanceof Error ? e.message : 'Unknown error'}`)
		}

		try {
			const oblRes = await fetchObligationReviewItems({ reviewState: 'suggested', limit: 50 })
			obligations.value = oblRes.items || []
		} catch (e) {
			errors.push(`Obligations: ${e instanceof Error ? e.message : 'Unknown error'}`)
		}

		try {
			const conRes = await fetchContradictions(50)
			contradictions.value = conRes.items || []
		} catch (e) {
			errors.push(`Contradictions: ${e instanceof Error ? e.message : 'Unknown error'}`)
		}
		try {
			const reviewRes = await fetchReviewItems({ status: 'active', limit: 50 })
			reviewItems.value = reviewRes.items || []
		} catch (e) {
			errors.push(`Review inbox: ${e instanceof Error ? e.message : 'Unknown error'}`)
		}

		if (errors.length > 0) {
			error.value = errors.join(' · ')
		}
	}

	async function reviewItem(action: ReviewWorkspaceItemAction): Promise<string> {
		const itemKey = reviewItemKey(action)
		reviewingItemKey.value = itemKey

		try {
			switch (action.kind) {
				case 'relationship': {
					await reviewRelationship(action.item.relationship_id, action.reviewState)
					const idx = relationships.value.findIndex(
						(r: Relationship) => r.relationship_id === action.item.relationship_id
					)
					if (idx !== -1) {
						relationships.value[idx] = { ...relationships.value[idx], review_state: action.reviewState }
					}
					break
				}
				case 'decision': {
					await reviewDecision(action.item.decision_id, { review_state: action.reviewState })
					const idx = decisions.value.findIndex(
						(d: Decision) => d.decision_id === action.item.decision_id
					)
					if (idx !== -1) {
						decisions.value[idx] = { ...decisions.value[idx], review_state: action.reviewState }
					}
					break
				}
				case 'obligation': {
					await reviewObligation(action.item.obligation_id, { review_state: action.reviewState })
					const idx = obligations.value.findIndex(
						(o: Obligation) => o.obligation_id === action.item.obligation_id
					)
					if (idx !== -1) {
						obligations.value[idx] = { ...obligations.value[idx], review_state: action.reviewState }
					}
					break
				}
				case 'contradiction': {
					await reviewContradiction(action.item.observation_id, { review_state: action.reviewState })
					const idx = contradictions.value.findIndex(
						(c: ContradictionObservation) => c.observation_id === action.item.observation_id
					)
					if (idx !== -1) {
						contradictions.value[idx] = { ...contradictions.value[idx], review_state: action.reviewState }
					}
					break
				}
				case 'review_item': {
					if (action.action === 'approve') {
						const updated = await approveReviewItem(action.item.review_item_id)
						updateReviewItem(updated)
					} else {
						const updated = await dismissReviewItem(action.item.review_item_id)
						updateReviewItem(updated)
					}
					break
				}
				case 'review_item_archive': {
					const updated = await archiveReviewItem(action.item.review_item_id)
					updateReviewItem(updated)
					break
				}
				case 'review_item_take': {
					const updated = await takeReviewItem(action.item.review_item_id)
					updateReviewItem(updated)
					break
				}
				case 'review_item_promote': {
					const updated = await promoteReviewItem(action.item.review_item_id, action.promotion)
					updateReviewItem(updated)
					break
				}
			}
			return ''
		} catch (e) {
			return e instanceof Error ? e.message : 'Unknown review action error'
		} finally {
			reviewingItemKey.value = null
		}
	}

	function updateReviewItem(updated: ReviewItem) {
		const idx = reviewItems.value.findIndex((item) => item.review_item_id === updated.review_item_id)
		if (idx === -1) return
		reviewItems.value[idx] = updated
	}

	return {
		relationships,
		decisions,
		obligations,
		contradictions,
		reviewItems,
		reviewItemsCount,
		error,
		reviewingItemKey,
		relationsSuggestedCount,
		decisionsSuggestedCount,
		obligationsSuggestedCount,
		contradictionsSuggestedCount,
		totalSuggestedCount,
		loadAll,
		reviewItem
	}
})

function reviewItemKey(action: ReviewWorkspaceItemAction): string {
	switch (action.kind) {
		case 'relationship':
			return `relationship:${action.item.relationship_id}`
		case 'decision':
			return `decision:${action.item.decision_id}`
		case 'obligation':
			return `obligation:${action.item.obligation_id}`
		case 'contradiction':
			return `contradiction:${action.item.observation_id}`
		case 'review_item':
			return `review_item:${action.item.review_item_id}`
		case 'review_item_archive':
			return `review_item_archive:${action.item.review_item_id}`
		case 'review_item_take':
			return `review_item_take:${action.item.review_item_id}`
		case 'review_item_promote':
			return `review_item_promote:${action.item.review_item_id}`
	}
}
