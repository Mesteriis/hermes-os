import type { Relationship } from '../../personas/types/persona'
import type { Decision, Obligation, DecisionReviewState, ObligationReviewState } from '../../tasks/types/task'
import type { ContradictionObservation, ContradictionReviewState } from '../../knowledge/types/knowledge'

export type { Relationship, Decision, Obligation, ContradictionObservation }
export type { DecisionReviewState, ObligationReviewState, ContradictionReviewState }

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

export interface ReviewWorkspaceItemActionResult {
	itemKey: string
	error: string
}

export type ReviewItemKind = 'relationship' | 'decision' | 'obligation' | 'contradiction'
