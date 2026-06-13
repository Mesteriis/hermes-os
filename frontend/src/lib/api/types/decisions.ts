export type DecisionEntityKind =
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

export type DecisionStatus = 'active' | 'superseded' | 'reversed' | 'deprecated';

export type DecisionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected';

export type Decision = {
	decision_id: string;
	title: string;
	status: DecisionStatus;
	rationale: string;
	alternatives: unknown;
	decided_by_entity_kind: DecisionEntityKind | null;
	decided_by_entity_id: string | null;
	decided_at: string | null;
	review_state: DecisionReviewState;
	confidence: number;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type DecisionListResponse = {
	items: Decision[];
};

export type DecisionReviewRequest = {
	review_state: Exclude<DecisionReviewState, 'suggested'>;
};
