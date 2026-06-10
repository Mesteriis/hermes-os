import { ApiClient } from '../client';
import type {
	TaskCandidateListResponse,
	TaskCandidateReviewState,
	TaskRecordsResponse,
	Task,
	TaskContextPack,
	TaskEvidenceResponse,
	TaskRelationsResponse,
	TaskChecklist,
	TaskSubtasksResponse,
	TaskProvidersResponse,
	TaskRulesResponse,
	TaskTemplatesResponse,
	AiTaskCandidateRefreshRequest,
	AiTaskCandidateRefreshResponse
} from '../types';

export async function fetchTaskCandidates(limit = 50): Promise<TaskCandidateListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<TaskCandidateListResponse>(
		`/api/v1/task-candidates?${params.toString()}`,
		'Task candidates request failed'
	);
}

export async function refreshAiTaskCandidates(
	request: AiTaskCandidateRefreshRequest
): Promise<AiTaskCandidateRefreshResponse> {
	return ApiClient.instance.post<AiTaskCandidateRefreshResponse>(
		'/api/v1/ai/task-candidates/refresh',
		request,
		'AI task candidate refresh request failed'
	);
}

export async function reviewTaskCandidate(
	taskCandidateId: string,
	reviewState: TaskCandidateReviewState
) {
	return ApiClient.instance.put(
		`/api/v1/task-candidates/${encodeURIComponent(taskCandidateId)}/review`,
		{
			command_id: `task-candidate-review-${crypto.randomUUID()}`,
			review_state: reviewState
		}
	);
}

export async function fetchTaskRecords(
	params: { status?: string; project_id?: string; source_type?: string; limit?: number } = {}
): Promise<TaskRecordsResponse> {
	const sp = new URLSearchParams();
	if (params.status) sp.set('status', params.status);
	if (params.project_id) sp.set('project_id', params.project_id);
	if (params.source_type) sp.set('source_type', params.source_type);
	if (params.limit) sp.set('limit', String(params.limit));
	return ApiClient.instance.get<TaskRecordsResponse>(
		`/api/v1/tasks?${sp.toString()}`,
		'Tasks request failed'
	);
}

export async function fetchTask(taskId: string): Promise<Task> {
	return ApiClient.instance.get<Task>(
		`/api/v1/tasks/${encodeURIComponent(taskId)}`,
		'Task request failed'
	);
}

export async function createTask(
	body: { title: string; description?: string; source_type?: string; project_id?: string; hermes_status?: string; priority_score?: number; due_at?: string; area?: string; linked_person_id?: string }
): Promise<Task> {
	return ApiClient.instance.post<Task>('/api/v1/tasks', body, 'Create task failed');
}

export async function updateTask(taskId: string, body: Record<string, unknown>): Promise<Task> {
	return ApiClient.instance.put<Task>(
		`/api/v1/tasks/${encodeURIComponent(taskId)}`,
		body,
		'Update task failed'
	);
}

export async function setTaskStatus(taskId: string, status: string): Promise<{ status: string }> {
	return ApiClient.instance.post<{ status: string }>(
		`/api/v1/tasks/${encodeURIComponent(taskId)}/status`,
		{ status },
		'Set status failed'
	);
}

export async function archiveTask(taskId: string): Promise<{ archived: boolean }> {
	return ApiClient.instance.post<{ archived: boolean }>(
		`/api/v1/tasks/${encodeURIComponent(taskId)}/archive`,
		{},
		'Archive failed'
	);
}

export async function fetchTaskContextPack(taskId: string): Promise<TaskContextPack | null> {
	return ApiClient.instance.get<TaskContextPack | null>(
		`/api/v1/tasks/${encodeURIComponent(taskId)}/context-pack`,
		'Context pack failed'
	);
}

export async function fetchTaskEvidence(taskId: string): Promise<TaskEvidenceResponse> {
	return ApiClient.instance.get<TaskEvidenceResponse>(
		`/api/v1/tasks/${encodeURIComponent(taskId)}/evidence`,
		'Evidence failed'
	);
}

export async function fetchTaskRelations(taskId: string): Promise<TaskRelationsResponse> {
	return ApiClient.instance.get<TaskRelationsResponse>(
		`/api/v1/tasks/${encodeURIComponent(taskId)}/relations`,
		'Relations failed'
	);
}

export async function fetchTaskChecklist(taskId: string): Promise<TaskChecklist | null> {
	return ApiClient.instance.get<TaskChecklist | null>(
		`/api/v1/tasks/${encodeURIComponent(taskId)}/checklist`,
		'Checklist failed'
	);
}

export async function fetchTaskSubtasks(taskId: string): Promise<TaskSubtasksResponse> {
	return ApiClient.instance.get<TaskSubtasksResponse>(
		`/api/v1/tasks/${encodeURIComponent(taskId)}/subtasks`,
		'Subtasks failed'
	);
}

export async function analyzeTask(taskId: string): Promise<Record<string, unknown>> {
	return ApiClient.instance.post<Record<string, unknown>>(
		`/api/v1/tasks/${encodeURIComponent(taskId)}/analyze`,
		{},
		'Analyze failed'
	);
}

export async function fetchTaskProviders(): Promise<TaskProvidersResponse> {
	return ApiClient.instance.get<TaskProvidersResponse>('/api/v1/tasks/providers', 'Providers failed');
}

export async function fetchTaskRules(): Promise<TaskRulesResponse> {
	return ApiClient.instance.get<TaskRulesResponse>('/api/v1/tasks/rules', 'Rules failed');
}

export async function fetchTaskTemplates(): Promise<TaskTemplatesResponse> {
	return ApiClient.instance.get<TaskTemplatesResponse>('/api/v1/tasks/templates', 'Templates failed');
}

export async function fetchTaskWatchtower(): Promise<Record<string, unknown>> {
	return ApiClient.instance.get<Record<string, unknown>>('/api/v1/tasks/watchtower', 'Watchtower failed');
}

export async function fetchTaskDailyBrief(): Promise<Record<string, unknown>> {
	return ApiClient.instance.get<Record<string, unknown>>('/api/v1/tasks/daily-brief', 'Daily brief failed');
}

export async function searchTasks(q: string): Promise<Record<string, unknown>> {
	return ApiClient.instance.get<Record<string, unknown>>(
		`/api/v1/tasks/search?q=${encodeURIComponent(q)}`,
		'Task search failed'
	);
}
