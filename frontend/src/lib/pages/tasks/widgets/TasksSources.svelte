<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="tasks-deadlines-priority" data-widget-hidden={!isWidgetVisible('tasks-deadlines-priority')}>
	<WidgetEditChrome widgetId="tasks-deadlines-priority" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card"><h2>Active Task Sources</h2>{#each ['message','document'] as source}<div class="bar-row"><span>{source}</span><div><i></i></div></div>{/each}</section>
</div>
