<script lang="ts">
	import HermesSelect from '$lib/components/shared/HermesSelect.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { AiProviderPreset } from '$lib/api';
	import {
		aiProviderPresets,
		cliProviders,
		createAiProviderFromPreset,
		isAiSettingsSaving,
		testAiProvider
	} from '$lib/stores/aiSettings';
	import { categoryLabel, providerPrivacyLabel, providerStatusTone } from '$lib/services/aiSettings';

	const _ = (key: string) => t($currentLocale, key);

	let cliPresetKey = $state('codex');
	let selectedCliPreset = $derived(
		$aiProviderPresets.find(
			(preset) => preset.provider_kind === 'cli' && preset.provider_key === cliPresetKey
		) ?? null
	);
	let cliPresetOptions = $derived(
		$aiProviderPresets
			.filter((preset) => preset.provider_kind === 'cli')
			.map((preset) => presetSelectOption(preset))
	);

	async function submitCliProvider() {
		if (!selectedCliPreset) return;
		await createAiProviderFromPreset(selectedCliPreset);
	}

	function presetSelectOption(preset: AiProviderPreset) {
		return {
			value: preset.provider_key,
			label: preset.display_name,
			description: preset.capabilities.map(categoryLabel).join(', '),
			meta: providerPrivacyLabel(preset)
		};
	}
</script>

<section class="ai-panel-section">
	<header>
		<h3>{_('CLI Agents')}</h3>
		<p>{_('Provider bridges use allowlisted fixed commands and do not run autonomous workflows.')}</p>
	</header>
	<form class="ai-wizard-row" onsubmit={(event) => { event.preventDefault(); void submitCliProvider(); }}>
		<label>
			<span>{_('Preset')}</span>
			<HermesSelect
				value={cliPresetKey}
				options={cliPresetOptions}
				placeholder={_('Select preset')}
				searchPlaceholder={_('Search presets...')}
				emptyLabel={_('No options')}
				ariaLabel={_('CLI preset')}
				onChange={(nextValue) => (cliPresetKey = nextValue)}
			/>
		</label>
		<button type="submit" class="primary-button" disabled={!selectedCliPreset || $isAiSettingsSaving}>{_('Add CLI bridge')}</button>
	</form>
	<div class="ai-provider-grid">
		{#each $cliProviders as provider}
			<article class="ai-provider-card">
				<div class="ai-provider-title">
					<span class={`integration-status ${providerStatusTone(provider)}`}>{provider.status}</span>
					<strong>{provider.display_name}</strong>
					<small>{String(provider.config.command_preset ?? provider.provider_key)} · fixed argv bridge</small>
				</div>
				<div class="ai-provider-actions">
					<button type="button" class="ghost-button" onclick={() => void testAiProvider(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Test')}</button>
				</div>
			</article>
		{/each}
	</div>
</section>
