import type {
	Relationship,
	RelationshipReviewState
} from '../../personas/types/persona'
import type {
	Decision,
	DecisionReviewState,
	Obligation,
	ObligationReviewState
} from '../../tasks/types/task'
import type {
	ContradictionObservation,
	ContradictionReviewState
} from '../../knowledge/types/knowledge'

export type { Relationship, Decision, Obligation, ContradictionObservation }
export type {
	DecisionReviewState,
	ObligationReviewState,
	ContradictionReviewState,
	RelationshipReviewState
}

export type ReviewWorkspaceItemKind = 'relationship' | 'decision' | 'obligation' | 'contradiction'

export interface ReviewItem {
	review_item_id: string
	item_kind: ReviewItemKind
	title: string
	summary: string
	status: ReviewItemStatus
	target_domain: string | null
	target_entity_kind: string | null
	target_entity_id: string | null
	confidence: number
	metadata: Record<string, unknown>
	created_at: string
	updated_at: string
}

export interface ReviewItemsResponse {
	items: ReviewItem[]
}

export interface ReviewItemPromotionRequest {
	target_domain: string
	target_entity_kind: string
	target_entity_id: string
}

export type ReviewItemStatus =
	| 'new'
	| 'in_review'
	| 'approved'
	| 'promoted'
	| 'dismissed'
	| 'archived'

export type ReviewItemKind =
	| 'new_person'
	| 'new_organization'
	| 'identity_candidate'
	| 'project_link_candidate'
	| 'contradiction_candidate'
	| 'potential_task'
	| 'potential_obligation'
	| 'potential_decision'
	| 'potential_relationship'
	| 'potential_project'
	| 'knowledge_candidate'

export type ReviewWorkspaceItemAction =
	| {
			kind: 'relationship'
			item: Relationship
			reviewState: string
	  }
	| {
			kind: 'decision'
			item: Decision
			reviewState: 'user_confirmed' | 'user_rejected'
	  }
	| {
			kind: 'obligation'
			item: Obligation
			reviewState: 'user_confirmed' | 'user_rejected'
	  }
	| {
			kind: 'contradiction'
			item: ContradictionObservation
			reviewState: 'user_confirmed' | 'user_rejected'
	  }
	| {
			kind: 'review_item'
			item: ReviewItem
			action: 'approve' | 'dismiss'
	  }
	| {
			kind: 'review_item_take'
			item: ReviewItem
	  }
	| {
			kind: 'review_item_archive'
			item: ReviewItem
	  }
	| {
			kind: 'review_item_promote'
			item: ReviewItem
			promotion: ReviewItemPromotionRequest
	  }

export interface ReviewWorkspaceItemActionResult {
	itemKey: string
	error: string
}
