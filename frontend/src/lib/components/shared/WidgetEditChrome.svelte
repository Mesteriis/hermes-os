<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import { openWidgetSettingsDrawer, selectedLayoutWidgetId } from '$lib/stores/layoutEditor';
	import './widgetEditChrome.css';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		widgetId: string;
		isLayoutEditing: boolean;
		isSelected: boolean;
		onConfigure: (widgetId: string) => void;
	}

	let { widgetId, isLayoutEditing, isSelected, onConfigure }: Props = $props();
</script>

{#if isLayoutEditing}
	<div class="widget-edit-chrome">
		<button
			type="button"
			class="widget-config-button"
			title={_('Configure widget')}
			aria-label={_('Configure widget')}
			aria-expanded={isSelected || $selectedLayoutWidgetId === widgetId}
			onclick={() => { openWidgetSettingsDrawer(widgetId); onConfigure(widgetId); }}
		>
			<Icon icon="tabler:adjustments-horizontal" width="15" height="15" />
		</button>
	</div>
{/if}
