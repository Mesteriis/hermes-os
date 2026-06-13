<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import {
		builtInProviders,
		isAiSettingsSaving,
		syncAiProviderModels,
		testAiProvider
	} from '$lib/stores/aiSettings';
	import { providerPrivacyLabel, providerStatusTone } from '$lib/services/aiSettings';

	const _ = (key: string) => t($currentLocale, key);
</script>

<section class="ai-panel-section">
	<header>
		<h3>{_('Built-in Ollama')}</h3>
		<p>{_('Local runtime is managed by Hermes; model downloads require explicit confirmation.')}</p>
	</header>
	<div class="ai-provider-grid">
		{#each $builtInProviders as provider}
			<article class="ai-provider-card">
				<div class="ai-provider-title">
					<span class={`integration-status ${providerStatusTone(provider)}`}>{provider.status}</span>
					<strong>{provider.display_name}</strong>
					<small>{providerPrivacyLabel(provider)} · {provider.provider_id}</small>
				</div>
				<div class="ai-provider-actions">
					<button type="button" class="ghost-button" onclick={() => void testAiProvider(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Test')}</button>
					<button type="button" class="ghost-button" onclick={() => void syncAiProviderModels(provider.provider_id)} disabled={$isAiSettingsSaving}>{_('Sync models')}</button>
				</div>
			</article>
		{:else}
			<div class="empty-panel fill">{_('Built-in Ollama provider will appear after migration repair runs.')}</div>
		{/each}
	</div>
</section>
