import { ApiClient } from '../client';
import type {
	AiStatus,
	AiAgentListResponse,
	AiRunListResponse,
	AiAnswerRequest,
	AiAnswerResponse,
	AiMeetingPrepRequest,
	AiMeetingPrepResponse
} from '../types';

export async function fetchAiStatus(): Promise<AiStatus> {
	return ApiClient.instance.get<AiStatus>('/api/v1/ai/status', 'AI status request failed');
}

export async function fetchAiAgents(): Promise<AiAgentListResponse> {
	return ApiClient.instance.get<AiAgentListResponse>('/api/v1/ai/agents', 'AI agents request failed');
}

export async function fetchAiRuns(limit = 25): Promise<AiRunListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) });
	return ApiClient.instance.get<AiRunListResponse>(
		`/api/v1/ai/runs?${params.toString()}`,
		'AI run history request failed'
	);
}

export async function requestAiAnswer(request: AiAnswerRequest): Promise<AiAnswerResponse> {
	return ApiClient.instance.post<AiAnswerResponse>('/api/v1/ai/answers', request, 'AI answer request failed');
}

export async function requestAiMeetingPrep(request: AiMeetingPrepRequest): Promise<AiMeetingPrepResponse> {
	return ApiClient.instance.post<AiMeetingPrepResponse>(
		'/api/v1/ai/meeting-prep',
		request,
		'AI meeting prep request failed'
	);
}
