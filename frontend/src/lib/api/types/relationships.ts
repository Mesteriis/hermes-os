export type RelationshipEntityKind =
	| 'persona'
	| 'organization'
	| 'project'
	| 'communication'
	| 'document'
	| 'task'
	| 'event'
	| 'decision'
	| 'obligation'
	| 'knowledge';

export type RelationshipReviewState =
	| 'suggested'
	| 'system_accepted'
	| 'user_confirmed'
	| 'user_rejected';

export type Relationship = {
	relationship_id: string;
	source_entity_kind: RelationshipEntityKind;
	source_entity_id: string;
	target_entity_kind: RelationshipEntityKind;
	target_entity_id: string;
	relationship_type: string;
	trust_score: number;
	strength_score: number;
	confidence: number;
	review_state: RelationshipReviewState;
	valid_from: string | null;
	valid_to: string | null;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type RelationshipListResponse = {
	items: Relationship[];
};

export type RelationshipReviewRequest = {
	review_state: Exclude<RelationshipReviewState, 'suggested' | 'system_accepted'>;
};
