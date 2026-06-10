<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		activeTasksCount: number;
		suggestedCandidatesCount: number;
		tasksError: string;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { activeTasksCount, suggestedCandidatesCount, tasksError, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame inline-metrics" class:editing={isLayoutEditing} data-widget-id="tasks-metrics" data-widget-hidden={!isWidgetVisible('tasks-metrics')}>
	<WidgetEditChrome widgetId="tasks-metrics" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<div class="metric-grid inline-metrics">
		<article class="metric-card"><span>Active Tasks</span><strong>{activeTasksCount}</strong><small>Active records</small></article>
		<article class="metric-card"><span>Suggested Candidates</span><strong>{suggestedCandidatesCount}</strong><small>Ready for review</small></article>
		<article class="metric-card"><span>Review State</span><strong>{tasksError ? 'Error' : 'Ready'}</strong><small>{tasksError ? 'Show message below' : 'Live API'}</small></article>
	</div>
</div>
