<script lang="ts">
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import { currentLocale, t } from '$lib/i18n';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		taskCandidatesCount: number;
		suggestedCount: number;
		activeCount: number;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { taskCandidatesCount, suggestedCount, activeCount, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="tasks-ai-refresh-status" data-widget-hidden={!isWidgetVisible('tasks-ai-refresh-status')}>
	<WidgetEditChrome widgetId="tasks-ai-refresh-status" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel chart-panel"><h2>{_('Review Stats')}</h2><div class="donut"><strong>{taskCandidatesCount}</strong><span>{_('Suggestions')}</span></div><ul><li>{suggestedCount} {_('Suggested')}</li><li>{activeCount} {_('Active')}</li><li>{taskCandidatesCount - suggestedCount - activeCount} {_('Done')}</li></ul></section>
</div>
