import {
	fetchContradictions,
	reviewContradiction,
	type ContradictionObservation,
	type ContradictionReviewRequest,
	type ContradictionReviewState,
	type ContradictionSeverity
} from '$lib/api';

export type ContradictionReviewWorkspaceState = {
	observations: ContradictionObservation[];
	suggestedCount: number;
	error: string;
};

export async function loadContradictionReviewState(): Promise<ContradictionReviewWorkspaceState> {
	try {
		const response = await fetchContradictions(50);
		const observations = response.items;
		return {
			observations,
			suggestedCount: observations.filter((item) => item.review_state === 'suggested').length,
			error: ''
		};
	} catch (error) {
		return {
			observations: [],
			suggestedCount: 0,
			error: error instanceof Error ? error.message : 'Unknown contradiction review error'
		};
	}
}

export async function reviewContradictionObservation(
	observation: ContradictionObservation,
	reviewState: Exclude<ContradictionReviewState, 'suggested'>,
	resolution?: string
): Promise<{ error: string }> {
	try {
		const request: ContradictionReviewRequest = {
			review_state: reviewState
		};
		const trimmedResolution = resolution?.trim();
		if (trimmedResolution) {
			request.resolution = trimmedResolution;
		}

		await reviewContradiction(observation.observation_id, request);
		return { error: '' };
	} catch (error) {
		return {
			error: error instanceof Error ? error.message : 'Unknown contradiction review action error'
		};
	}
}

export function formatContradictionClaim(observation: ContradictionObservation): string {
	return `${observation.old_claim} -> ${observation.new_claim}`;
}

export function formatContradictionSource(kind: string, sourceId: string): string {
	return `${formatSourceKind(kind)} · ${sourceId}`;
}

export function formatContradictionTime(value: string): string {
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

export function contradictionSeverityTone(severity: ContradictionSeverity): ContradictionSeverity {
	return severity;
}

function formatSourceKind(kind: string): string {
	return kind
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}
