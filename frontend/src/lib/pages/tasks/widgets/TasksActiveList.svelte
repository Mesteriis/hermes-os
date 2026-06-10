<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		activeTasks: unknown[];
		suggestedTaskCandidates: unknown[];
		isTasksLoading: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		taskSourceLabel: (item: unknown) => string;
		taskConfidence: (item: unknown) => string;
		taskCreatedTime: (val: string | null) => string;
		setTaskCandidateReview: (candidate: unknown, state: string) => Promise<void>;
	}

	let { activeTasks, suggestedTaskCandidates, isTasksLoading, isLayoutEditing, isWidgetVisible, taskSourceLabel, taskConfidence, taskCreatedTime, setTaskCandidateReview }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="tasks-active-list" data-widget-hidden={!isWidgetVisible('tasks-active-list')}>
	<WidgetEditChrome widgetId="tasks-active-list" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel task-table">
		<h3 class="task-group">{_('Active Tasks')} <em>{activeTasks.length}</em></h3>
		<div class="table-head task-table-head"><span>{_('Task')}</span><span>{_('Source')}</span><span>{_('Project')}</span><span>{_('Created')}</span><span>{_('Status')}</span></div>
		{#if isTasksLoading}
			<p class="inline-copy">{_('Loading task state…')}</p>
		{:else if activeTasks.length === 0}
			<p class="inline-copy">{_('No active tasks yet.')}</p>
		{:else}
			{#each activeTasks as item}
				<label class="task-row"><input type="checkbox" disabled checked />				<strong>{(item as Record<string, unknown>).title as string}</strong><span>{taskSourceLabel(item)}</span><span>{(item as Record<string, unknown>).project_id as string ?? _('Unassigned')}</span><time>{taskCreatedTime((item as Record<string, unknown>).created_at as string | null ?? null)}</time><em>{(item as Record<string, unknown>).hermes_status as string}</em></label>
			{/each}
		{/if}

		<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="tasks-candidate-review" data-widget-hidden={!isWidgetVisible('tasks-candidate-review')}>
			<WidgetEditChrome widgetId="tasks-candidate-review" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
			<h3 class="task-group">{_('Review Queue')} <em>{suggestedTaskCandidates.length}</em></h3>
			<div class="table-head task-table-head"><span>{_('Candidate')}</span><span>{_('Source')}</span><span>{_('Project')}</span><span>{_('Confidence')}</span><span>{_('Action')}</span></div>
			{#if isTasksLoading}
				<p class="inline-copy">{_('Loading task candidates…')}</p>
			{:else if suggestedTaskCandidates.length === 0}
				<p class="inline-copy">{_('No suggested candidates.')}</p>
			{:else}
				{#each suggestedTaskCandidates as candidate}
					<div class="task-row task-row-actions">
						<strong>{(candidate as Record<string, unknown>).title as string}</strong>
						<span>{taskSourceLabel(candidate)}</span>
						<span>{(candidate as Record<string, unknown>).project_id as string ?? _('Unassigned')}</span>
						<em>{taskConfidence(candidate)}</em>
						<div class="task-actions">
							<button type="button" onclick={() => void setTaskCandidateReview(candidate, 'user_confirmed')}><Icon icon="tabler:check" width="15" height="15" /> {_('Confirm')}</button>
							<button type="button" onclick={() => void setTaskCandidateReview(candidate, 'user_rejected')}><Icon icon="tabler:x" width="15" height="15" /> {_('Reject')}</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>
	</section>
</div>
