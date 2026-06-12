import { ApiClient } from '../client';
import type {
	AiStatus,
	AiAgentListResponse,
	AiRunListResponse,
	AiAnswerRequest,
	AiAnswerResponse,
	AiModelListResponse,
	AiModelRoute,
	AiModelRouteUpdateRequest,
	AiMeetingPrepRequest,
	AiMeetingPrepResponse,
	AiPromptActivateRequest,
	AiPromptCreateRequest,
	AiPromptEvalRun,
	AiPromptListResponse,
	AiPromptTemplate,
	AiPromptTestRequest,
	AiPromptVersion,
	AiPromptVersionCreateRequest,
	AiProviderAccount,
	AiProviderCommandResponse,
	AiProviderConsentRequest,
	AiProviderCreateRequest,
	AiProviderListResponse,
	AiProviderPatchRequest,
	AiSettingsOverviewResponse
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

export async function fetchAiSettingsOverview(): Promise<AiSettingsOverviewResponse> {
	return ApiClient.instance.get<AiSettingsOverviewResponse>(
		'/api/v1/ai/settings/overview',
		'AI settings overview request failed'
	);
}

export async function fetchAiProviders(): Promise<AiProviderListResponse> {
	return ApiClient.instance.get<AiProviderListResponse>(
		'/api/v1/ai/providers',
		'AI providers request failed'
	);
}

export async function createAiProvider(request: AiProviderCreateRequest): Promise<AiProviderAccount> {
	return ApiClient.instance.post<AiProviderAccount>(
		'/api/v1/ai/providers',
		request,
		'AI provider create request failed'
	);
}

export async function patchAiProvider(
	providerId: string,
	request: AiProviderPatchRequest
): Promise<AiProviderAccount> {
	return ApiClient.instance.patch<AiProviderAccount>(
		`/api/v1/ai/providers/${encodeURIComponent(providerId)}`,
		request,
		'AI provider update request failed'
	);
}

export async function testAiProvider(providerId: string): Promise<AiProviderCommandResponse> {
	return ApiClient.instance.post<AiProviderCommandResponse>(
		`/api/v1/ai/providers/${encodeURIComponent(providerId)}/test`,
		{},
		'AI provider test request failed'
	);
}

export async function syncAiProviderModels(providerId: string): Promise<AiProviderCommandResponse> {
	return ApiClient.instance.post<AiProviderCommandResponse>(
		`/api/v1/ai/providers/${encodeURIComponent(providerId)}/sync-models`,
		{},
		'AI provider model sync request failed'
	);
}

export async function saveAiProviderConsent(
	providerId: string,
	request: AiProviderConsentRequest
): Promise<AiProviderAccount> {
	return ApiClient.instance.post<AiProviderAccount>(
		`/api/v1/ai/providers/${encodeURIComponent(providerId)}/consent`,
		request,
		'AI provider consent request failed'
	);
}

export async function fetchAiModels(): Promise<AiModelListResponse> {
	return ApiClient.instance.get<AiModelListResponse>('/api/v1/ai/models', 'AI models request failed');
}

export async function putAiModelRoute(
	slot: string,
	request: AiModelRouteUpdateRequest
): Promise<AiModelRoute> {
	return ApiClient.instance.put<AiModelRoute>(
		`/api/v1/ai/model-routes/${encodeURIComponent(slot)}`,
		request,
		'AI model route update request failed'
	);
}

export async function fetchAiPrompts(): Promise<AiPromptListResponse> {
	return ApiClient.instance.get<AiPromptListResponse>('/api/v1/ai/prompts', 'AI prompts request failed');
}

export async function createAiPrompt(request: AiPromptCreateRequest): Promise<AiPromptTemplate> {
	return ApiClient.instance.post<AiPromptTemplate>(
		'/api/v1/ai/prompts',
		request,
		'AI prompt create request failed'
	);
}

export async function createAiPromptVersion(
	promptId: string,
	request: AiPromptVersionCreateRequest
): Promise<AiPromptVersion> {
	return ApiClient.instance.post<AiPromptVersion>(
		`/api/v1/ai/prompts/${encodeURIComponent(promptId)}/versions`,
		request,
		'AI prompt version create request failed'
	);
}

export async function activateAiPromptVersion(
	promptId: string,
	request: AiPromptActivateRequest
): Promise<AiPromptTemplate> {
	return ApiClient.instance.post<AiPromptTemplate>(
		`/api/v1/ai/prompts/${encodeURIComponent(promptId)}/activate`,
		request,
		'AI prompt activation request failed'
	);
}

export async function testAiPrompt(promptId: string, request: AiPromptTestRequest): Promise<AiPromptEvalRun> {
	return ApiClient.instance.post<AiPromptEvalRun>(
		`/api/v1/ai/prompts/${encodeURIComponent(promptId)}/test`,
		request,
		'AI prompt test request failed'
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
