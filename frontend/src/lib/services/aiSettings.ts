import {
	createAiProvider,
	createAiPrompt,
	createAiPromptVersion,
	fetchAiSettingsOverview,
	putAiModelRoute,
	saveAiProviderConsent,
	syncAiProviderModels,
	testAiPrompt,
	testAiProvider,
	type AiCapabilitySlot,
	type AiModelCatalogItem,
	type AiModelRoute,
	type AiModelRouteUpdateRequest,
	type AiPromptCreateRequest,
	type AiPromptEvalRun,
	type AiPromptTemplate,
	type AiPromptTestRequest,
	type AiPromptVersion,
	type AiPromptVersionCreateRequest,
	type AiProviderAccount,
	type AiProviderCommandResponse,
	type AiProviderCreateRequest,
	type AiProviderPreset,
	type AiSettingsOverviewResponse
} from '$lib/api';

export const AI_SETTINGS_SECTIONS = [
	{ id: 'overview', label: 'Overview / health', icon: 'tabler:activity-heartbeat' },
	{ id: 'built_in', label: 'Built-in Ollama', icon: 'tabler:cpu' },
	{ id: 'cli', label: 'CLI Agents', icon: 'tabler:terminal-2' },
	{ id: 'api', label: 'API Providers', icon: 'tabler:cloud-cog' },
	{ id: 'routing', label: 'Model Routing', icon: 'tabler:route' },
	{ id: 'prompts', label: 'Prompt Studio', icon: 'tabler:prompt' },
	{ id: 'runs', label: 'Runs / health', icon: 'tabler:history' }
] as const;

export type AiSettingsPanel = (typeof AI_SETTINGS_SECTIONS)[number]['id'];

export const PROMPT_ENTITY_SCOPES = [
	'person',
	'organization',
	'project',
	'document',
	'task',
	'meeting',
	'communication',
	'conversation',
	'global'
] as const;

export type ModelSelectOption = {
	value: string;
	label: string;
	model: AiModelCatalogItem;
	provider: AiProviderAccount | null;
	providerLabel: string;
	privacyLabel: string;
	capabilityLabel: string;
	disabledReason: string;
};

export type ModelSelectGroup = {
	category: string;
	label: string;
	options: ModelSelectOption[];
};

export async function loadAiSettingsWorkspace(): Promise<{
	overview: AiSettingsOverviewResponse | null;
	error: string;
}> {
	try {
		return { overview: await fetchAiSettingsOverview(), error: '' };
	} catch (error) {
		return {
			overview: null,
			error: error instanceof Error ? error.message : 'Unknown AI settings error'
		};
	}
}

export async function createProviderFromWizard(
	request: AiProviderCreateRequest
): Promise<{ provider: AiProviderAccount | null; error: string }> {
	try {
		return { provider: await createAiProvider(stripEmptyProviderFields(request)), error: '' };
	} catch (error) {
		return {
			provider: null,
			error: error instanceof Error ? error.message : 'Unknown AI provider setup error'
		};
	}
}

export async function testProvider(
	providerId: string
): Promise<{ result: AiProviderCommandResponse | null; error: string }> {
	try {
		return { result: await testAiProvider(providerId), error: '' };
	} catch (error) {
		return {
			result: null,
			error: error instanceof Error ? error.message : 'Unknown AI provider test error'
		};
	}
}

export async function syncProviderModels(
	providerId: string
): Promise<{ result: AiProviderCommandResponse | null; error: string }> {
	try {
		return { result: await syncAiProviderModels(providerId), error: '' };
	} catch (error) {
		return {
			result: null,
			error: error instanceof Error ? error.message : 'Unknown AI model sync error'
		};
	}
}

export async function grantProviderConsent(
	providerId: string,
	consented = true
): Promise<{ provider: AiProviderAccount | null; error: string }> {
	try {
		return { provider: await saveAiProviderConsent(providerId, { consented }), error: '' };
	} catch (error) {
		return {
			provider: null,
			error: error instanceof Error ? error.message : 'Unknown AI provider consent error'
		};
	}
}

export async function saveModelRoute(
	slot: string,
	model: AiModelCatalogItem
): Promise<{ route: AiModelRoute | null; error: string }> {
	try {
		return { route: await putAiModelRoute(slot, modelRoutePayload(model)), error: '' };
	} catch (error) {
		return {
			route: null,
			error: error instanceof Error ? error.message : 'Unknown AI route update error'
		};
	}
}

export async function createPrompt(
	request: AiPromptCreateRequest
): Promise<{ prompt: AiPromptTemplate | null; error: string }> {
	try {
		return { prompt: await createAiPrompt(request), error: '' };
	} catch (error) {
		return {
			prompt: null,
			error: error instanceof Error ? error.message : 'Unknown prompt create error'
		};
	}
}

export async function createPromptVersion(
	promptId: string,
	request: AiPromptVersionCreateRequest
): Promise<{ version: AiPromptVersion | null; error: string }> {
	try {
		return { version: await createAiPromptVersion(promptId, request), error: '' };
	} catch (error) {
		return {
			version: null,
			error: error instanceof Error ? error.message : 'Unknown prompt version create error'
		};
	}
}

export async function runPromptTest(
	promptId: string,
	request: AiPromptTestRequest
): Promise<{ run: AiPromptEvalRun | null; error: string }> {
	try {
		return { run: await testAiPrompt(promptId, request), error: '' };
	} catch (error) {
		return {
			run: null,
			error: error instanceof Error ? error.message : 'Unknown prompt test error'
		};
	}
}

export function providerWizardDraftFromPreset(preset: AiProviderPreset): AiProviderCreateRequest {
	return stripEmptyProviderFields({
		provider_kind: preset.provider_kind,
		provider_key: preset.provider_key,
		display_name: preset.display_name,
		base_url: preset.base_url ?? undefined,
		command_preset: preset.command_preset ?? undefined,
		capabilities: [...preset.capabilities],
		enabled: true,
		remote_context_consent: preset.privacy !== 'remote'
	});
}

export function stripEmptyProviderFields(request: AiProviderCreateRequest): AiProviderCreateRequest {
	const next: AiProviderCreateRequest = { ...request };
	for (const key of ['provider_id', 'base_url', 'command_preset', 'api_key'] as const) {
		if (typeof next[key] === 'string' && !next[key]?.trim()) {
			delete next[key];
		}
	}
	if (next.config && Object.keys(next.config).length === 0) {
		delete next.config;
	}
	return next;
}

export function buildModelSelectGroups(
	models: AiModelCatalogItem[],
	providers: AiProviderAccount[],
	query = '',
	slot?: AiCapabilitySlot | null
): ModelSelectGroup[] {
	const normalizedQuery = query.trim().toLowerCase();
	const providerById = new Map(providers.map((provider) => [provider.provider_id, provider]));
	const filtered = models
		.map((model) => modelSelectOption(model, providerById.get(model.provider_id) ?? null, slot))
		.filter((option) => matchesModelQuery(option, normalizedQuery))
		.sort((left, right) => modelOptionSortKey(left).localeCompare(modelOptionSortKey(right)));

	const groups = new Map<string, ModelSelectOption[]>();
	for (const option of filtered) {
		const key = option.model.category || 'other';
		groups.set(key, [...(groups.get(key) ?? []), option]);
	}

	return [...groups.entries()].map(([category, options]) => ({
		category,
		label: categoryLabel(category),
		options
	}));
}

export function modelSelectValue(model: AiModelCatalogItem): string {
	return `${model.provider_id}::${model.model_key}`;
}

export function modelFromSelectValue(
	value: string,
	models: AiModelCatalogItem[]
): AiModelCatalogItem | null {
	const [providerId, ...modelParts] = value.split('::');
	const modelKey = modelParts.join('::');
	if (!providerId || !modelKey) {
		return null;
	}
	return models.find((model) => model.provider_id === providerId && model.model_key === modelKey) ?? null;
}

export function modelRoutePayload(model: AiModelCatalogItem): AiModelRouteUpdateRequest {
	return {
		provider_id: model.provider_id,
		model_key: model.model_key
	};
}

export function routeForSlot(routes: AiModelRoute[], slot: string): AiModelRoute | null {
	return routes.find((route) => route.capability_slot === slot) ?? null;
}

export function modelForRoute(
	models: AiModelCatalogItem[],
	route: AiModelRoute | null
): AiModelCatalogItem | null {
	if (!route) {
		return null;
	}
	return (
		models.find(
			(model) => model.provider_id === route.provider_id && model.model_key === route.model_key
		) ?? null
	);
}

export function providerStatusTone(provider: AiProviderAccount): string {
	if (provider.status === 'ready') return 'ready';
	if (provider.status === 'disabled') return 'disabled';
	if (provider.status === 'needs_setup') return 'warn';
	return 'muted';
}

export function providerPrivacyLabel(provider: AiProviderAccount | AiProviderPreset): string {
	if ('privacy' in provider) {
		return privacyLabel(provider.privacy);
	}
	if (provider.provider_kind === 'built_in') return 'Local';
	if (provider.provider_kind === 'cli') return 'Local CLI';
	return 'Remote';
}

export function capabilitySlotLabel(slot: string): string {
	return categoryLabel(slot);
}

export function categoryLabel(value: string): string {
	return value
		.split('_')
		.flatMap((part) => part.split('-'))
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

function modelSelectOption(
	model: AiModelCatalogItem,
	provider: AiProviderAccount | null,
	slot?: AiCapabilitySlot | null
): ModelSelectOption {
	const disabledReason = unavailableReason(model, provider, slot);
	return {
		value: modelSelectValue(model),
		label: `${model.display_name} · ${model.model_key}`,
		model,
		provider,
		providerLabel: provider?.display_name ?? model.provider_id,
		privacyLabel: privacyLabel(model.privacy),
		capabilityLabel: model.capabilities.map(categoryLabel).join(', '),
		disabledReason
	};
}

function unavailableReason(
	model: AiModelCatalogItem,
	provider: AiProviderAccount | null,
	slot?: AiCapabilitySlot | null
): string {
	if (!provider) return 'Provider missing';
	if (provider.status === 'disabled') return 'Provider disabled';
	if (provider.provider_kind === 'api' && provider.consent_state !== 'granted') {
		return 'Remote consent required';
	}
	if (provider.provider_kind === 'api' && provider.status !== 'ready') {
		return 'Host-vault API key required';
	}
	if (provider.status !== 'ready') return 'Provider setup incomplete';
	if (!model.is_available) return 'Model unavailable';
	if (
		slot?.requires_embedding_dimension &&
		model.embedding_dimension !== slot.requires_embedding_dimension
	) {
		return `Requires ${slot.requires_embedding_dimension} dimensions`;
	}
	return '';
}

function matchesModelQuery(option: ModelSelectOption, query: string): boolean {
	if (!query) {
		return true;
	}
	return [
		option.label,
		option.providerLabel,
		option.model.category,
		option.model.privacy,
		...option.model.capabilities
	]
		.join(' ')
		.toLowerCase()
		.includes(query);
}

function modelOptionSortKey(option: ModelSelectOption): string {
	const availability = option.disabledReason ? '1' : '0';
	return `${option.model.category}:${availability}:${option.providerLabel}:${option.model.display_name}`;
}

function privacyLabel(value: string): string {
	if (value === 'local') return 'Local';
	if (value === 'cli') return 'Local CLI';
	if (value === 'remote') return 'Remote';
	return categoryLabel(value);
}
