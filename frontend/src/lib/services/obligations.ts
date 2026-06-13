import {
	fetchObligationReviewItems,
	fetchObligations,
	reviewObligation,
	type Obligation,
	type ObligationEntityKind,
	type ObligationReviewState
} from '$lib/api';

export type ObligationReviewWorkspaceState = {
	obligations: Obligation[];
	suggestedCount: number;
	error: string;
};

export async function loadObligationReviewState(
	entityKind: ObligationEntityKind,
	entityId: string
): Promise<ObligationReviewWorkspaceState> {
	try {
		const response = await fetchObligations({
			entityKind,
			entityId,
			limit: 50
		});
		const obligations = response.items;
		return {
			obligations,
			suggestedCount: obligations.filter((item) => item.review_state === 'suggested').length,
			error: ''
		};
	} catch (error) {
		return {
			obligations: [],
			suggestedCount: 0,
			error: error instanceof Error ? error.message : 'Unknown obligation review error'
		};
	}
}

export async function loadGlobalObligationReviewState(): Promise<ObligationReviewWorkspaceState> {
	try {
		const response = await fetchObligationReviewItems({
			reviewState: 'suggested',
			limit: 50
		});
		const obligations = response.items;
		return {
			obligations,
			suggestedCount: obligations.filter((item) => item.review_state === 'suggested').length,
			error: ''
		};
	} catch (error) {
		return {
			obligations: [],
			suggestedCount: 0,
			error: error instanceof Error ? error.message : 'Unknown global obligation review error'
		};
	}
}

export async function reviewObligationItem(
	obligation: Obligation,
	reviewState: Exclude<ObligationReviewState, 'suggested'>
): Promise<{ error: string }> {
	try {
		await reviewObligation(obligation.obligation_id, { review_state: reviewState });
		return { error: '' };
	} catch (error) {
		return {
			error: error instanceof Error ? error.message : 'Unknown obligation review action error'
		};
	}
}

export function formatObligationDueTime(value: string | null): string {
	if (!value) {
		return 'No due date';
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

export function formatObligationEntity(kind: ObligationEntityKind, entityId: string): string {
	return `${formatEntityKind(kind)} · ${entityId}`;
}

function formatEntityKind(kind: string): string {
	return kind
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}
