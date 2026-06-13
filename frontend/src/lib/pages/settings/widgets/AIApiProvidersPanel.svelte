<script lang="ts">
	import HermesSelect from '$lib/components/shared/HermesSelect.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { AiProviderPreset } from '$lib/api';
	import {
		aiProviderPresets,
		apiProviders,
		createAiProviderFromPreset,
		grantAiProviderConsent,
		isAiSettingsSaving,
		syncAiProviderModels,
		testAiProvider
	} from '$lib/stores/aiSettings';
	import { categoryLabel, providerPrivacyLabel, providerStatusTone } from '$lib/services/aiSettings';

	const _ = (key: string) => t($currentLocale, key);

	let apiPresetKey = $state('omniroute');
	let apiDisplayName = $state('OmniRoute');
	let apiBaseUrl = $state('https://ai.sh-inc.ru/v1');
	let apiKey = $state('');
	let apiConsent = $state(false);
	let selectedApiPreset = $derived(
		$aiProviderPresets.find(
			(preset) => preset.provider_kind === 'api' && preset.provider_key === apiPresetKey
		) ?? null
	);
	let apiPresetOptions = $derived(
		$aiProviderPresets
			.filter((preset) => preset.provider_kind === 'api')
			.map((preset) => presetSelectOption(preset))
	);

	$effect(() => {
		if (selectedApiPreset) {
			apiDisplayName = apiDisplayName || selectedApiPreset.display_name;
			apiBaseUrl = apiBaseUrl || selectedApiPreset.base_url || '';
		}
	});

	function selectApiPreset(key: string) {
		apiPresetKey = key;
		const preset = $aiProviderPresets.find(
			(item) => item.provider_kind === 'api' && item.provider_key === key
		);
		if (preset) {
			apiDisplayName = preset.display_name;
			apiBaseUrl = preset.base_url ?? '';
			apiConsent = preset.privacy !== 'remote';
		}
	}

	async function submitApiProvider() {
		if (!selectedApiPreset) return;
		await createAiProviderFromPreset(selectedApiPreset, {
			display_name: apiDisplayName,
			base_url: apiBaseUrl,
			api_key: apiKey,
			remote_context_consent: apiConsent
		});
		apiKey = '';
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
		<h3>{_('API Providers')}</h3>
		<p>{_('Remote providers are opt-in and API keys are written only to the host vault.')}</p>
	</header>
	<form class="ai-provider-form" onsubmit={(event) => { event.preventDefault(); void submitApiProvider(); }}>
		<label>
			<span>{_('Preset')}</span>
			<HermesSelect
				value={apiPresetKey}
				options={apiPresetOptions}
				placeholder={_('Select preset')}
				searchPlaceholder={_('Search presets...')}
				emptyLabel={_('No options')}
				ariaLabel={_('API preset')}
				onChange={selectApiPreset}
			/>
		</label>
		<label>
			<span>{_('Display name')}</span>
			<input value={apiDisplayName} oninput={(event) => (apiDisplayName = (event.currentTarget as HTMLInputElement).value)} />
		</label>
		<label>
			<span>{_('Base URL')}</span>
			<input value={apiBaseUrl} oninput={(event) => (apiBaseUrl = (event.currentTarget as HTMLInputElement).value)} />
		</label>
		<label>
			<span>{_('API key')}</span>
			<input type="password" value={apiKey} autocomplete="off" oninput={(event) => (apiKey = (event.currentTarget as HTMLInputElement).value)} />
		</label>
		<label class="ai-consent-toggle">
			<input type="checkbox" checked={apiConsent} onchange={(event) => (apiConsent = (event.currentTarget as HTMLInputElement).checked)} />
			<span>{_('Allow this provider to receive selected remote context')}</span>
		</label>
		<button type="submit" class="primary-button" disabled={!selectedApiPreset || !apiConsent || $isAiSettingsSaving}>{_('Connect API provider')}</button>
	</form>
	<div class="ai-provider-grid">
		{#each $apiProviders as provider}
			<article class="ai-provider-card">
				<div class="ai-provider-title">
					<span class={`integration-status ${providerStatusTone(provider)}`}>{provider.status}</span>
					<strong>{provider.display_name}</strong>
					<small>{provider.provider_key} · {provider.consent_state}</small>
				</div>
				<div class="ai-provider-actions">
					{#if provider.consent_state !== 'granted'}
						<button type="button" class="ghost-button" onclick={() => void grantAiProviderConsent(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Grant consent')}</button>
					{/if}
					<button type="button" class="ghost-button" onclick={() => void testAiProvider(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Test')}</button>
					<button type="button" class="ghost-button" onclick={() => void syncAiProviderModels(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Sync models')}</button>
				</div>
			</article>
		{/each}
	</div>
</section>
