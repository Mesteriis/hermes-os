<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		tasks: Array<{ title: string; assignee: string; due: string; priority: string }>;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { tasks, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="home-priorities" data-widget-hide-if-clipped-content data-widget-hidden={!isWidgetVisible('home-priorities')}>
	<WidgetEditChrome widgetId="home-priorities" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel priorities-panel" data-widget-fit-content>
		<header class="panel-title-row"><div><h2>{_("Today's Priorities")}</h2><p>{_('Focus on what matters most')}</p></div></header>
		<div class="task-stack">
			{#each tasks.slice(0, 5) as task}
				<label>
					<input type="checkbox" />
					<span><strong>{task.title}</strong><small>{task.assignee} · {task.due}</small></span>
					<em class:high={task.priority === 'High'}>{task.priority}</em>
				</label>
			{/each}
		</div>
		<button type="button" class="link-row" disabled>View all tasks <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
	</section>
</div>
