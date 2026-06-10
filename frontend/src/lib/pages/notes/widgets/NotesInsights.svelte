<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="notes-insights" data-widget-hidden={!isWidgetVisible('notes-insights')}>
	<WidgetEditChrome widgetId="notes-insights" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>{_('Note Insights')}</h2>
		<p>{_('AI generated summaries and connections across your notes will appear here.')}</p>
	</section>
</div>
