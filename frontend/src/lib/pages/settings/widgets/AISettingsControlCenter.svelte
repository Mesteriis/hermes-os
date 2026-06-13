<script lang="ts">
	import { onMount } from 'svelte';
	import { currentLocale, t } from '$lib/i18n';
	import {
		aiProviders,
		isAiSettingsLoading,
		loadAiSettingsControlCenter,
		selectedAiSettingsPanel
	} from '$lib/stores/aiSettings';
	import type { AiSettingsPanel } from '$lib/services/aiSettings';
	import AIApiProvidersPanel from './AIApiProvidersPanel.svelte';
	import AIBuiltInProvidersPanel from './AIBuiltInProvidersPanel.svelte';
	import AICliProvidersPanel from './AICliProvidersPanel.svelte';
	import AIModelRoutingPanel from './AIModelRoutingPanel.svelte';
	import AIOverviewPanel from './AIOverviewPanel.svelte';
	import AIPromptStudioPanel from './AIPromptStudioPanel.svelte';
	import AIRunsHealthPanel from './AIRunsHealthPanel.svelte';
	import AISettingsHeader from './AISettingsHeader.svelte';
	import AISettingsRail from './AISettingsRail.svelte';
	import AISettingsStatus from './AISettingsStatus.svelte';
	import AISettingsTabs from './AISettingsTabs.svelte';

	const _ = (key: string) => t($currentLocale, key);

	onMount(() => {
		if ($aiProviders.length === 0 && !$isAiSettingsLoading) {
			void loadAiSettingsControlCenter();
		}
	});

	function selectPanel(panel: AiSettingsPanel) {
		selectedAiSettingsPanel.set(panel);
	}
</script>

<div class="ai-settings-layout">
	<section class="ai-settings-main">
		<AISettingsHeader />

		<div class="ai-settings-body">
			<AISettingsTabs selectedPanel={$selectedAiSettingsPanel} onSelectPanel={selectPanel} />
			<AISettingsStatus />

			{#if $isAiSettingsLoading && $aiProviders.length === 0}
				<div class="empty-panel fill">{_('Loading AI settings.')}</div>
			{:else if $selectedAiSettingsPanel === 'overview'}
				<AIOverviewPanel />
			{:else if $selectedAiSettingsPanel === 'built_in'}
				<AIBuiltInProvidersPanel />
			{:else if $selectedAiSettingsPanel === 'cli'}
				<AICliProvidersPanel />
			{:else if $selectedAiSettingsPanel === 'api'}
				<AIApiProvidersPanel />
			{:else if $selectedAiSettingsPanel === 'routing'}
				<AIModelRoutingPanel />
			{:else if $selectedAiSettingsPanel === 'prompts'}
				<AIPromptStudioPanel />
			{:else}
				<AIRunsHealthPanel />
			{/if}
		</div>
	</section>

	<AISettingsRail />
</div>
