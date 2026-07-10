export type RelationshipReviewState = 'suggested' | 'system_accepted' | 'user_confirmed' | 'user_rejected'
export type DecisionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'
export type ObligationReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'
export type ContradictionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'
export type UserRelationshipReviewState = Extract<
	RelationshipReviewState,
	'user_confirmed' | 'user_rejected'
>

export interface Relationship {
	relationship_id: string
	source_entity_kind: string
	source_entity_id: string
	target_entity_kind: string
	target_entity_id: string
	relationship_type: string
	trust_score?: number | null
	review_state: RelationshipReviewState
}

export interface Decision {
	decision_id: string
	title: string
	decided_by_entity_kind?: string | null
	decided_by_entity_id?: string | null
	decided_at?: string | null
	review_state: DecisionReviewState
}

export interface Obligation {
	obligation_id: string
	statement: string
	obligated_entity_kind: string
	obligated_entity_id: string
	due_at?: string | null
	review_state: ObligationReviewState
}

export interface ContradictionObservation {
	observation_id: string
	old_claim: string
	new_claim: string
	severity: string
	created_at: string
	review_state: ContradictionReviewState
}

export interface RelationshipListResponse {
	relationships: Relationship[]
}

export interface DecisionListResponse {
	items: Decision[]
}

export interface ObligationListResponse {
	items: Obligation[]
}

export interface ContradictionListResponse {
	items: ContradictionObservation[]
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

export type AttentionImportance = 'low' | 'medium' | 'high' | 'critical'

export interface AttentionEvidenceRef {
	observation_id: string
	role: string
}

export interface AttentionRelatedEntity {
	entity_kind: string
	entity_id: string
	label?: string | null
}

export interface AttentionSuggestedAction {
	action_kind: string
	label: string
	target_domain?: string | null
	target_entity_kind?: string | null
}

export interface AttentionConfidenceExplanation {
	score: number
	rationale: string
}

export interface AttentionExplainability {
	why_this_matters: string
	evidence: AttentionEvidenceRef[]
	confidence: AttentionConfidenceExplanation
	related_objects: AttentionRelatedEntity[]
	suggested_actions: AttentionSuggestedAction[]
}

export interface AttentionCard {
	id: string
	title: string
	summary: string
	importance: AttentionImportance
	confidence: number
	evidence_count: number
	related_entities: AttentionRelatedEntity[]
	trace_id: string
	review_item_ids: string[]
	suggested_actions: AttentionSuggestedAction[]
	source_summary: string
	explainability: AttentionExplainability
}

export interface AttentionCardsResponse {
	cards: AttentionCard[]
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
	| 'new_persona'
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
			reviewState: UserRelationshipReviewState
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
