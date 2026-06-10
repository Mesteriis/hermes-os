<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="notes-insights" data-widget-hidden={!isWidgetVisible('notes-insights')}>
	<WidgetEditChrome widgetId="notes-insights" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>Note Insights</h2>
		<p>AI generated summaries and connections across your notes will appear here.</p>
	</section>
</div>
