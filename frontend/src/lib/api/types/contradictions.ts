export type ContradictionSourceKind =
	| 'communication'
	| 'document'
	| 'event'
	| 'memory'
	| 'knowledge'
	| 'decision'
	| 'obligation'
	| 'task'
	| 'relationship'
	| 'raw_record';

export type ContradictionSeverity = 'low' | 'medium' | 'high' | 'critical';

export type ContradictionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected';

export type ContradictionObservation = {
	observation_id: string;
	old_source_kind: ContradictionSourceKind;
	old_source_id: string;
	new_source_kind: ContradictionSourceKind;
	new_source_id: string;
	affected_entities: unknown;
	conflict_type: string;
	old_claim: string;
	new_claim: string;
	confidence: number;
	severity: ContradictionSeverity;
	review_state: ContradictionReviewState;
	metadata: Record<string, unknown>;
	reviewed_by: string | null;
	reviewed_at: string | null;
	resolution: string | null;
	created_at: string;
	updated_at: string;
};

export type ContradictionListResponse = {
	items: ContradictionObservation[];
};

export type ContradictionReviewRequest = {
	review_state: Exclude<ContradictionReviewState, 'suggested'>;
	resolution?: string;
};
