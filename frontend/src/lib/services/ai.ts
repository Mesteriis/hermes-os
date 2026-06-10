import {
	fetchAiAgents,
	fetchAiRuns,
	fetchAiStatus,
	requestAiAnswer,
	requestAiMeetingPrep,
	refreshAiTaskCandidates,
	type AiAgent,
	type AiAnswerResponse,
	type AiCitation,
	type AiMeetingPrepResponse,
	type AiRun,
	type AiStatus,
	type AiTaskCandidateRefreshResponse
} from '$lib/api';

export async function loadAiWorkspace(): Promise<{
	agents: AiAgent[];
	runs: AiRun[];
	status: AiStatus | null;
	error: string;
}> {
	try {
		const [agentResponse, runResponse] = await Promise.all([
			fetchAiAgents(),
			fetchAiRuns(25)
		]);
		const agents = agentResponse.items;
		const runs = runResponse.items;
		let aiStatus: AiStatus | null = null;
		let error = '';
		try {
			aiStatus = await fetchAiStatus();
		} catch (statusError) {
			error = statusError instanceof Error ? statusError.message : 'Unknown AI status error';
		}
		return { agents, runs, status: aiStatus, error };
	} catch (error) {
		return {
			agents: [],
			runs: [],
			status: null,
			error: error instanceof Error ? error.message : 'Unknown AI runtime error'
		};
	}
}

export async function submitAiAnswer(query: string, agentId: string): Promise<{
	result: AiAnswerResponse | null;
	error: string;
}> {
	try {
		const result = await requestAiAnswer({
			command_id: `ai-answer-${crypto.randomUUID()}`,
			query,
			agent_id: agentId || 'MNEMOSYNE'
		});
		return { result, error: '' };
	} catch (error) {
		return {
			result: null,
			error: error instanceof Error ? error.message : 'Unknown AI answer error'
		};
	}
}

export async function prepareAiBrief(
	topic: string,
	projectId: string | undefined
): Promise<{
	result: AiMeetingPrepResponse | null;
	error: string;
}> {
	try {
		const result = await requestAiMeetingPrep({
			command_id: `ai-meeting-prep-${crypto.randomUUID()}`,
			topic,
			project_id: projectId
		});
		return { result, error: '' };
	} catch (error) {
		return {
			result: null,
			error: error instanceof Error ? error.message : 'Unknown AI meeting prep error'
		};
	}
}

export async function refreshTasksFromAi(
	query: string,
	_agentId: string
): Promise<{
	result: AiTaskCandidateRefreshResponse | null;
	error: string;
}> {
	try {
		const result = await refreshAiTaskCandidates({
			command_id: `ai-task-refresh-${crypto.randomUUID()}`,
			query
		});
		return { result, error: '' };
	} catch (error) {
		return {
			result: null,
			error: error instanceof Error ? error.message : 'Unknown AI task refresh error'
		};
	}
}

export async function loadAiRunsOnly(): Promise<{
	runs: AiRun[];
	error: string;
}> {
	try {
		const response = await fetchAiRuns(25);
		return { runs: response.items, error: '' };
	} catch (error) {
		return {
			runs: [],
			error: error instanceof Error ? error.message : 'Unknown AI run history error'
		};
	}
}

export function agentCardView(agent: AiAgent, aiRuns: AiRun[]) {
	const visual = agentVisual(agent.agent_id);
	const runs = aiRuns.filter((run) => run.agent_id === agent.agent_id);
	const completed = runs.filter((run) => run.status === 'completed').length;
	const success = runs.length > 0 ? Math.round((completed / runs.length) * 100) : 0;

	return {
		agentId: agent.agent_id,
		name: agent.display_name,
		summary: agent.role,
		icon: visual.icon,
		tasks: runs.length,
		success: success,
		status: agent.status,
		tone: visual.tone,
		model: agent.default_model
	};
}

export function agentVisual(agentId: string): { icon: string; tone: string } {
	switch (agentId) {
		case 'HESTIA':
			return { icon: 'tabler:calendar-stats', tone: 'mint' };
		case 'HERMES':
			return { icon: 'tabler:route', tone: 'blue' };
		case 'MNEMOSYNE':
			return { icon: 'tabler:database-search', tone: 'purple' };
		case 'ATHENA':
			return { icon: 'tabler:target-arrow', tone: 'amber' };
		default:
			return { icon: 'tabler:sparkles', tone: 'cyan' };
	}
}

export function runStatusLabel(run: AiRun) {
	if (run.status === 'completed') {
		return 'Completed';
	}
	if (run.status === 'failed') {
		return 'Failed';
	}
	return 'Requested';
}

export function aiRuntimeSummary(aiStatus: AiStatus | null, isAiLoading: boolean) {
	if (!aiStatus) {
		return isAiLoading ? 'Loading' : 'Unknown';
	}
	return aiStatus.status === 'ok' ? 'Ready' : 'Unavailable';
}

export function aiModelSummary(aiStatus: AiStatus | null) {
	if (!aiStatus) {
		return 'No status';
	}
	return `${aiStatus.chat_model} / ${aiStatus.embedding_model}`;
}

export function safeCitations(value: unknown): AiCitation[] {
	if (!Array.isArray(value)) {
		return [];
	}
	return value.filter(isAiCitation);
}

export function isAiCitation(value: unknown): value is AiCitation {
	return (
		typeof value === 'object' &&
		value !== null &&
		typeof (value as { source_kind?: unknown }).source_kind === 'string' &&
		typeof (value as { source_id?: unknown }).source_id === 'string' &&
		typeof (value as { title?: unknown }).title === 'string' &&
		typeof (value as { excerpt?: unknown }).excerpt === 'string'
	);
}
