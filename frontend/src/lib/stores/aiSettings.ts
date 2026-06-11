import { derived, get, writable } from 'svelte/store';
import {
	activateAiPromptVersion,
	type AiModelCatalogItem,
	type AiModelRoute,
	type AiPromptCreateRequest,
	type AiPromptEvalRun,
	type AiPromptTestRequest,
	type AiPromptVersionCreateRequest,
	type AiProviderCreateRequest,
	type AiProviderPreset,
	type AiSettingsOverviewResponse
} from '$lib/api';
import * as aiSettingsService from '$lib/services/aiSettings';
import type { AiSettingsPanel } from '$lib/services/aiSettings';

export const aiSettingsOverview = writable<AiSettingsOverviewResponse | null>(null);
export const aiSettingsError = writable('');
export const aiSettingsActionMessage = writable('');
export const isAiSettingsLoading = writable(false);
export const isAiSettingsSaving = writable(false);
export const selectedAiSettingsPanel = writable<AiSettingsPanel>('overview');
export const aiRouteSearch = writable('');
export const selectedRouteDrafts = writable<Record<string, string>>({});

export const aiProviderPresets = derived(
	aiSettingsOverview,
	($overview) => $overview?.provider_presets ?? []
);
export const aiProviders = derived(aiSettingsOverview, ($overview) => $overview?.providers ?? []);
export const aiModels = derived(aiSettingsOverview, ($overview) => $overview?.models ?? []);
export const aiRoutes = derived(aiSettingsOverview, ($overview) => $overview?.routes ?? []);
export const aiCapabilitySlots = derived(
	aiSettingsOverview,
	($overview) => $overview?.capability_slots ?? []
);
export const aiPrompts = derived(aiSettingsOverview, ($overview) => $overview?.prompts ?? []);
export const aiPromptEvalRuns = derived(aiSettingsOverview, ($overview) => $overview?.eval_runs ?? []);

export const builtInProviders = derived(aiProviders, ($providers) =>
	$providers.filter((provider) => provider.provider_kind === 'built_in')
);
export const cliProviders = derived(aiProviders, ($providers) =>
	$providers.filter((provider) => provider.provider_kind === 'cli')
);
export const apiProviders = derived(aiProviders, ($providers) =>
	$providers.filter((provider) => provider.provider_kind === 'api')
);
export const systemPrompts = derived(aiPrompts, ($prompts) =>
	$prompts.filter((prompt) => prompt.is_system)
);
export const customPrompts = derived(aiPrompts, ($prompts) =>
	$prompts.filter((prompt) => !prompt.is_system)
);

export async function loadAiSettingsControlCenter(): Promise<void> {
	isAiSettingsLoading.set(true);
	const result = await aiSettingsService.loadAiSettingsWorkspace();
	aiSettingsOverview.set(result.overview);
	aiSettingsError.set(result.error);
	if (result.overview) {
		seedRouteDrafts(result.overview.routes);
	}
	isAiSettingsLoading.set(false);
}

export async function createAiProviderFromPreset(
	preset: AiProviderPreset,
	overrides: Partial<AiProviderCreateRequest> = {}
): Promise<void> {
	isAiSettingsSaving.set(true);
	aiSettingsActionMessage.set('');
	const request = {
		...aiSettingsService.providerWizardDraftFromPreset(preset),
		...overrides
	};
	const result = await aiSettingsService.createProviderFromWizard(request);
	await finishMutation(
		result.error,
		result.provider ? `${result.provider.display_name} connected` : 'AI provider saved'
	);
}

export async function testAiProvider(providerId: string): Promise<void> {
	isAiSettingsSaving.set(true);
	const result = await aiSettingsService.testProvider(providerId);
	await finishMutation(result.error, result.result?.message ?? 'AI provider checked');
}

export async function syncAiProviderModels(providerId: string): Promise<void> {
	isAiSettingsSaving.set(true);
	const result = await aiSettingsService.syncProviderModels(providerId);
	await finishMutation(result.error, result.result?.message ?? 'AI models synced');
}

export async function grantAiProviderConsent(providerId: string): Promise<void> {
	isAiSettingsSaving.set(true);
	const result = await aiSettingsService.grantProviderConsent(providerId, true);
	await finishMutation(
		result.error,
		result.provider ? `${result.provider.display_name} remote context consent saved` : 'Consent saved'
	);
}

export function updateRouteDraft(slot: string, value: string): void {
	selectedRouteDrafts.update((drafts) => ({ ...drafts, [slot]: value }));
	aiSettingsActionMessage.set('');
}

export async function saveAiModelRoute(slot: string): Promise<void> {
	const model = aiSettingsService.modelFromSelectValue(get(selectedRouteDrafts)[slot] ?? '', get(aiModels));
	if (!model) {
		aiSettingsError.set('Select a model before saving the route');
		return;
	}
	isAiSettingsSaving.set(true);
	const result = await aiSettingsService.saveModelRoute(slot, model);
	await finishMutation(result.error, result.route ? `${slot} route saved` : 'AI route saved');
}

export async function createAiPrompt(request: AiPromptCreateRequest): Promise<void> {
	isAiSettingsSaving.set(true);
	const result = await aiSettingsService.createPrompt(request);
	await finishMutation(result.error, result.prompt ? `${result.prompt.name} created` : 'Prompt created');
}

export async function createAiPromptVersion(
	promptId: string,
	request: AiPromptVersionCreateRequest
): Promise<void> {
	isAiSettingsSaving.set(true);
	const result = await aiSettingsService.createPromptVersion(promptId, request);
	await finishMutation(
		result.error,
		result.version ? `${result.version.version_label} saved` : 'Prompt version saved'
	);
}

export async function activateAiPrompt(promptId: string, promptVersionId: string): Promise<void> {
	isAiSettingsSaving.set(true);
	try {
		const prompt = await activateAiPromptVersion(promptId, { prompt_version_id: promptVersionId });
		await finishMutation('', `${prompt.name} activated`);
	} catch (error) {
		await finishMutation(
			error instanceof Error ? error.message : 'Unknown prompt activation error',
			''
		);
	}
}

export async function testAiPrompt(
	promptId: string,
	request: AiPromptTestRequest
): Promise<AiPromptEvalRun | null> {
	isAiSettingsSaving.set(true);
	const result = await aiSettingsService.runPromptTest(promptId, request);
	await finishMutation(result.error, result.run ? 'Prompt test run saved' : '');
	return result.run;
}

export function routeModelForSlot(slot: string): AiModelCatalogItem | null {
	return aiSettingsService.modelForRoute(
		get(aiModels),
		aiSettingsService.routeForSlot(get(aiRoutes), slot)
	);
}

export function routeForSlot(slot: string): AiModelRoute | null {
	return aiSettingsService.routeForSlot(get(aiRoutes), slot);
}

async function finishMutation(error: string, actionMessage: string): Promise<void> {
	if (error) {
		aiSettingsError.set(error);
		aiSettingsActionMessage.set('');
		isAiSettingsSaving.set(false);
		return;
	}
	await loadAiSettingsControlCenter();
	aiSettingsError.set('');
	aiSettingsActionMessage.set(actionMessage);
	isAiSettingsSaving.set(false);
}

function seedRouteDrafts(routes: AiModelRoute[]): void {
	selectedRouteDrafts.set(
		routes.reduce<Record<string, string>>((drafts, route) => {
			drafts[route.capability_slot] = `${route.provider_id}::${route.model_key}`;
			return drafts;
		}, {})
	);
}

export { aiSettingsService };
