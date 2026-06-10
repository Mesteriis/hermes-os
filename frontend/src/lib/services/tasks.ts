import {
	fetchTaskCandidates,
	fetchTaskRecords,
	reviewTaskCandidate,
	refreshAiTaskCandidates,
	type TaskCandidate,
	type TaskCandidateReviewState,
	type Task
} from '$lib/api';

export async function loadTaskReviewState(): Promise<{
	candidates: TaskCandidate[];
	activeTasks: Task[];
	error: string;
}> {
	try {
		const [candidateResponse, taskResponse] = await Promise.all([
			fetchTaskCandidates(50),
			fetchTaskRecords({ limit: 50 })
		]);
		return {
			candidates: candidateResponse.items,
			activeTasks: taskResponse.items,
			error: ''
		};
	} catch (error) {
		return {
			candidates: [],
			activeTasks: [],
			error: error instanceof Error ? error.message : 'Unknown task candidate error'
		};
	}
}

export async function setTaskCandidateReview(
	candidate: TaskCandidate,
	reviewState: TaskCandidateReviewState
): Promise<{ error: string }> {
	try {
		await reviewTaskCandidate(candidate.task_candidate_id, reviewState);
		return { error: '' };
	} catch (error) {
		return {
			error: error instanceof Error ? error.message : 'Unknown task candidate review error'
		};
	}
}

export async function refreshTasksFromAi(
	query: string
): Promise<{ error: string }> {
	try {
		await refreshAiTaskCandidates({
			command_id: `ai-task-refresh-${crypto.randomUUID()}`,
			query
		});
		return { error: '' };
	} catch (error) {
		return {
			error: error instanceof Error ? error.message : 'Unknown AI task refresh error'
		};
	}
}

export function taskSourceLabel(item: TaskCandidate | Task) {
	return `${item.source_kind[0].toUpperCase()}${item.source_kind.slice(1)} · ${item.source_id}`;
}

export function taskConfidence(item: TaskCandidate) {
	return `${Math.round(item.confidence * 100)}%`;
}

export function taskCreatedTime(value: string | null) {
	if (!value) {
		return '';
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
