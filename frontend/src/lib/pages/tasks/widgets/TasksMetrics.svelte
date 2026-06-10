<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import { currentLocale, t } from '$lib/i18n';

	const _ = (key: string) => t($currentLocale, key);

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
		<article class="metric-card"><span>{_('Active Tasks')}</span><strong>{activeTasksCount}</strong><small>{_('Active records')}</small></article>
		<article class="metric-card"><span>{_('Suggested Candidates')}</span><strong>{suggestedCandidatesCount}</strong><small>{_('Ready for review')}</small></article>
		<article class="metric-card"><span>{_('Review State')}</span><strong>{tasksError ? _('Error') : _('Ready')}</strong><small>{tasksError ? _('Show message below') : _('Live API')}</small></article>
	</div>
</div>
