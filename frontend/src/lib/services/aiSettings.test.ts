import { describe, expect, it } from 'vitest';
import type { AiCapabilitySlot, AiModelCatalogItem, AiProviderAccount, AiProviderPreset } from '$lib/api';
import {
	buildModelSelectGroups,
	modelFromSelectValue,
	modelRoutePayload,
	modelSelectValue,
	providerWizardDraftFromPreset
} from './aiSettings';

const provider: AiProviderAccount = {
	provider_id: 'provider:built_in:ollama',
	provider_kind: 'built_in',
	provider_key: 'ollama',
	display_name: 'Built-in Ollama',
	status: 'ready',
	consent_state: 'not_required',
	consented_at: null,
	config: {},
	capabilities: ['chat', 'embeddings'],
	created_at: '2026-06-11T00:00:00Z',
	updated_at: '2026-06-11T00:00:00Z'
};

const models: AiModelCatalogItem[] = [
	{
		model_key: 'qwen3:4b',
		provider_id: provider.provider_id,
		display_name: 'Qwen3 4B',
		category: 'chat',
		privacy: 'local',
		capabilities: ['chat', 'reasoning'],
		context_window: 32768,
		embedding_dimension: null,
		is_available: true,
		metadata: {},
		created_at: '2026-06-11T00:00:00Z',
		updated_at: '2026-06-11T00:00:00Z'
	},
	{
		model_key: 'qwen3-embedding:4b',
		provider_id: provider.provider_id,
		display_name: 'Qwen3 Embedding 4B',
		category: 'embeddings',
		privacy: 'local',
		capabilities: ['embeddings'],
		context_window: 8192,
		embedding_dimension: 2560,
		is_available: true,
		metadata: {},
		created_at: '2026-06-11T00:00:00Z',
		updated_at: '2026-06-11T00:00:00Z'
	}
];

describe('AI settings service helpers', () => {
	it('builds searchable grouped model options with local and capability metadata', () => {
		const groups = buildModelSelectGroups(models, [provider], 'embed');

		expect(groups).toEqual([
			expect.objectContaining({
				category: 'embeddings',
				label: 'Embeddings',
				options: [
					expect.objectContaining({
						value: 'provider:built_in:ollama::qwen3-embedding:4b',
						providerLabel: 'Built-in Ollama',
						privacyLabel: 'Local',
						capabilityLabel: 'Embeddings',
						disabledReason: ''
					})
				]
			})
		]);
	});

	it('marks invalid embedding route models unavailable without removing them from selectors', () => {
		const embeddingSlot: AiCapabilitySlot = {
			slot: 'embeddings',
			label: 'Embeddings',
			description: 'Embedding route',
			requires_embedding_dimension: 2560
		};
		const groups = buildModelSelectGroups(models, [provider], '', embeddingSlot);
		const chatOption = groups
			.flatMap((group) => group.options)
			.find((option) => option.model.model_key === 'qwen3:4b');

		expect(chatOption?.disabledReason).toBe('Requires 2560 dimensions');
	});

	it('builds stable route payloads from selected provider model ids', () => {
		const selectedValue = modelSelectValue(models[1]);
		const selectedModel = modelFromSelectValue(selectedValue, models);

		expect(selectedModel).toEqual(models[1]);
		expect(modelRoutePayload(models[1])).toEqual({
			provider_id: 'provider:built_in:ollama',
			model_key: 'qwen3-embedding:4b'
		});
	});

	it('creates provider wizard drafts without embedding API keys in preset config', () => {
		const preset: AiProviderPreset = {
			provider_kind: 'api',
			provider_key: 'omniroute',
			display_name: 'OmniRoute',
			privacy: 'remote',
			base_url: 'https://ai.sh-inc.ru/v1',
			command_preset: null,
			capabilities: ['chat', 'embeddings']
		};

		expect(providerWizardDraftFromPreset(preset)).toEqual({
			provider_kind: 'api',
			provider_key: 'omniroute',
			display_name: 'OmniRoute',
			base_url: 'https://ai.sh-inc.ru/v1',
			capabilities: ['chat', 'embeddings'],
			enabled: true,
			remote_context_consent: false
		});
	});
});
