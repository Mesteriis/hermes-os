<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		personList: unknown[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { personList, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="documents-insights" data-widget-hidden={!isWidgetVisible('documents-insights')}>
	<WidgetEditChrome widgetId="documents-insights" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>Document Insights</h2>
		<p>AI analysis results will appear here when document processing is complete.</p>
	</section>
</div>
