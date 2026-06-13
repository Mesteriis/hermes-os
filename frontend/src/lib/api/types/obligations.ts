export type ObligationEntityKind =
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

export type ObligationStatus = 'open' | 'fulfilled' | 'waived' | 'disputed' | 'canceled';

export type ObligationReviewState = 'suggested' | 'user_confirmed' | 'user_rejected';

export type ObligationRiskState = 'none' | 'watch' | 'at_risk' | 'breached';

export type Obligation = {
	obligation_id: string;
	obligated_entity_kind: ObligationEntityKind;
	obligated_entity_id: string;
	beneficiary_entity_kind: ObligationEntityKind | null;
	beneficiary_entity_id: string | null;
	statement: string;
	status: ObligationStatus;
	review_state: ObligationReviewState;
	due_at: string | null;
	condition: string | null;
	risk_state: ObligationRiskState;
	confidence: number;
	metadata: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type ObligationListResponse = {
	items: Obligation[];
};

export type ObligationReviewRequest = {
	review_state: Exclude<ObligationReviewState, 'suggested'>;
};
