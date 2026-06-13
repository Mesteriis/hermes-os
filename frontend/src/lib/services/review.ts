import {
	loadGlobalRelationshipReviewState,
	reviewRelationshipItem,
	type RelationshipReviewWorkspaceState
} from './relationships';
import {
	loadGlobalDecisionReviewState,
	reviewDecisionItem,
	type DecisionReviewWorkspaceState
} from './decisions';
import {
	loadGlobalObligationReviewState,
	reviewObligationItem,
	type ObligationReviewWorkspaceState
} from './obligations';
import {
	loadContradictionReviewState,
	reviewContradictionObservation,
	type ContradictionReviewWorkspaceState
} from './contradictions';
import type {
	ContradictionObservation,
	ContradictionReviewState,
	Decision,
	DecisionReviewState,
	Obligation,
	ObligationReviewState,
	Relationship,
	RelationshipReviewState
} from '$lib/api';

export type ReviewWorkspace = {
	relationships: RelationshipReviewWorkspaceState;
	decisions: DecisionReviewWorkspaceState;
	obligations: ObligationReviewWorkspaceState;
	contradictions: ContradictionReviewWorkspaceState;
	totalSuggestedCount: number;
	error: string;
};

export type ReviewWorkspaceItemAction =
	| {
			kind: 'relationship';
			item: Relationship;
			reviewState: Exclude<RelationshipReviewState, 'suggested' | 'system_accepted'>;
	  }
	| {
			kind: 'decision';
			item: Decision;
			reviewState: Exclude<DecisionReviewState, 'suggested'>;
	  }
	| {
			kind: 'obligation';
			item: Obligation;
			reviewState: Exclude<ObligationReviewState, 'suggested'>;
	  }
	| {
			kind: 'contradiction';
			item: ContradictionObservation;
			reviewState: Exclude<ContradictionReviewState, 'suggested'>;
	  };

export type ReviewWorkspaceItemActionResult = {
	itemKey: string;
	error: string;
};

export async function loadReviewWorkspace(): Promise<ReviewWorkspace> {
	const [relationships, decisions, obligations, contradictions] = await Promise.all([
		loadGlobalRelationshipReviewState(),
		loadGlobalDecisionReviewState(),
		loadGlobalObligationReviewState(),
		loadContradictionReviewState()
	]);

	return {
		relationships,
		decisions,
		obligations,
		contradictions,
		totalSuggestedCount:
			relationships.suggestedCount +
			decisions.suggestedCount +
			obligations.suggestedCount +
			contradictions.suggestedCount,
		error: [relationships.error, decisions.error, obligations.error, contradictions.error]
			.filter(Boolean)
			.join(' · ')
	};
}

export function reviewWorkspaceItemKey(action: ReviewWorkspaceItemAction): string {
	switch (action.kind) {
		case 'relationship':
			return `relationship:${action.item.relationship_id}`;
		case 'decision':
			return `decision:${action.item.decision_id}`;
		case 'obligation':
			return `obligation:${action.item.obligation_id}`;
		case 'contradiction':
			return `contradiction:${action.item.observation_id}`;
	}
}

export async function reviewWorkspaceItem(
	action: ReviewWorkspaceItemAction
): Promise<ReviewWorkspaceItemActionResult> {
	const itemKey = reviewWorkspaceItemKey(action);
	let result: { error: string };

	switch (action.kind) {
		case 'relationship':
			result = await reviewRelationshipItem(action.item, action.reviewState);
			break;
		case 'decision':
			result = await reviewDecisionItem(action.item, action.reviewState);
			break;
		case 'obligation':
			result = await reviewObligationItem(action.item, action.reviewState);
			break;
		case 'contradiction':
			result = await reviewContradictionObservation(action.item, action.reviewState);
			break;
	}

	return {
		itemKey,
		error: result.error
	};
}
