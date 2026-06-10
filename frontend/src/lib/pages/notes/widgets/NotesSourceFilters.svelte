<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<aside class="left-panels">
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="notes-source-filters" data-widget-hidden={!isWidgetVisible('notes-source-filters')}>
		<WidgetEditChrome widgetId="notes-source-filters" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>Sources</h2>
			{#each ['Apple Notes','Obsidian','Gmail','Anytype','Outlook'] as source}
				<label class="mini-check"><input type="checkbox" checked />{source}</label>
			{/each}
		</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="notes-tags" data-widget-hidden={!isWidgetVisible('notes-tags')}>
		<WidgetEditChrome widgetId="notes-tags" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>Tags</h2>
			{#each ['#project','#research','#meeting','#idea','#reference','#partnership'] as tag}
				<label class="mini-check"><input type="checkbox" checked />{tag}</label>
			{/each}
		</section>
	</div>
</aside>
