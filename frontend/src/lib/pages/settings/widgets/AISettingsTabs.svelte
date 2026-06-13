<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { AI_SETTINGS_SECTIONS, type AiSettingsPanel } from '$lib/services/aiSettings';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		selectedPanel: AiSettingsPanel;
		onSelectPanel: (panel: AiSettingsPanel) => void;
	}

	let { selectedPanel, onSelectPanel }: Props = $props();
</script>

<nav class="ai-settings-tabs" aria-label={_('AI settings sections')}>
	{#each AI_SETTINGS_SECTIONS as section}
		<button
			type="button"
			class:active={selectedPanel === section.id}
			onclick={() => onSelectPanel(section.id)}
		>
			<Icon icon={section.icon} width="16" height="16" />
			<span>{_(section.label)}</span>
		</button>
	{/each}
</nav>
