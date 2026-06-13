<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { isAiSettingsLoading, loadAiSettingsControlCenter } from '$lib/stores/aiSettings';

	const _ = (key: string) => t($currentLocale, key);
</script>

<header class="settings-workbench-header">
	<div>
		<h2>{_('AI Control Center')}</h2>
		<p>{_('Providers, model routes and prompts for Hermes capabilities.')}</p>
	</div>
	<div class="ai-settings-actions">
		<button
			type="button"
			class="ghost-button"
			onclick={() => void loadAiSettingsControlCenter()}
			disabled={$isAiSettingsLoading}
		>
			<Icon icon="tabler:refresh" width="16" height="16" />{_('Refresh')}
		</button>
	</div>
</header>
