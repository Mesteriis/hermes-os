import {
	fetchDecisionReviewItems,
	fetchDecisions,
	reviewDecision,
	type Decision,
	type DecisionEntityKind,
	type DecisionReviewState
} from '$lib/api';

export type DecisionReviewWorkspaceState = {
	decisions: Decision[];
	suggestedCount: number;
	error: string;
};

export async function loadDecisionReviewState(
	entityKind: DecisionEntityKind,
	entityId: string
): Promise<DecisionReviewWorkspaceState> {
	try {
		const response = await fetchDecisions({
			entityKind,
			entityId,
			limit: 50
		});
		const decisions = response.items;
		return {
			decisions,
			suggestedCount: decisions.filter((item) => item.review_state === 'suggested').length,
			error: ''
		};
	} catch (error) {
		return {
			decisions: [],
			suggestedCount: 0,
			error: error instanceof Error ? error.message : 'Unknown decision review error'
		};
	}
}

export async function loadGlobalDecisionReviewState(): Promise<DecisionReviewWorkspaceState> {
	try {
		const response = await fetchDecisionReviewItems({
			reviewState: 'suggested',
			limit: 50
		});
		const decisions = response.items;
		return {
			decisions,
			suggestedCount: decisions.filter((item) => item.review_state === 'suggested').length,
			error: ''
		};
	} catch (error) {
		return {
			decisions: [],
			suggestedCount: 0,
			error: error instanceof Error ? error.message : 'Unknown global decision review error'
		};
	}
}

export async function reviewDecisionItem(
	decision: Decision,
	reviewState: Exclude<DecisionReviewState, 'suggested'>
): Promise<{ error: string }> {
	try {
		await reviewDecision(decision.decision_id, { review_state: reviewState });
		return { error: '' };
	} catch (error) {
		return {
			error: error instanceof Error ? error.message : 'Unknown decision review action error'
		};
	}
}

export function formatDecisionTime(value: string | null): string {
	if (!value) {
		return 'No decision date';
	}
	const date = new Date(value);
	if (Number.isNaN(date.getTime())) {
		return 'Unknown date';
	}
	return new Intl.DateTimeFormat('en', {
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	}).format(date);
}

export function formatDecisionEntity(kind: DecisionEntityKind | null, entityId: string | null): string {
	if (!kind || !entityId) {
		return 'No decider';
	}
	return `${formatEntityKind(kind)} · ${entityId}`;
}

function formatEntityKind(kind: string): string {
	return kind
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}
