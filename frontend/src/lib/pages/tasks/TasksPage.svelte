<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as tasksService from '$lib/services/tasks';
	import type { Task, TaskCandidate, TaskCandidateReviewState } from '$lib/api';
	import TasksMetrics from './widgets/TasksMetrics.svelte';
	import TasksActiveList from './widgets/TasksActiveList.svelte';
	import TasksReviewStats from './widgets/TasksReviewStats.svelte';
	import TasksContext from './widgets/TasksContext.svelte';
	import TasksSources from './widgets/TasksSources.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();

	let activeTasks = $state<Task[]>([]);
	let taskCandidates = $state<TaskCandidate[]>([]);
	let tasksError = $state('');
	let isTasksLoading = $state(false);
	let isAiTaskRefreshSubmitting = $state(false);

	let suggestedTaskCandidates = $derived(
		taskCandidates.filter((item) => item.review_state === 'suggested')
	);

	function taskSourceLabel(item: Task | TaskCandidate) {
		return `${item.source_kind[0].toUpperCase()}${item.source_kind.slice(1)} · ${item.source_id}`;
	}

	function taskConfidence(item: TaskCandidate) {
		return `${Math.round(item.confidence * 100)}%`;
	}

	function taskCreatedTime(value: string | null) {
		if (!value) return '';
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return 'Unknown date';
		return new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		}).format(date);
	}

	async function setTaskCandidateReview(
		candidate: TaskCandidate,
		state: TaskCandidateReviewState
	) {
		const result = await tasksService.setTaskCandidateReview(candidate, state);
		if (result.error) {
			tasksError = result.error;
		} else {
			await loadTasks();
		}
	}

	async function refreshTasksFromAi() {
		isAiTaskRefreshSubmitting = true;
		const result = await tasksService.refreshTasksFromAi('Find open task candidates from local messages and documents');
		if (result.error) {
			tasksError = result.error;
		} else {
			await loadTasks();
		}
		isAiTaskRefreshSubmitting = false;
	}

	async function loadTasks() {
		isTasksLoading = true;
		const result = await tasksService.loadTaskReviewState();
		taskCandidates = result.candidates;
		activeTasks = result.activeTasks;
		tasksError = result.error;
		isTasksLoading = false;
	}

	$effect(() => {
		loadTasks();
	});
</script>

<section class="tasks-page">
	<div class="view-header">
		<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:hexagon" width="28" height="28" /></span><div><h1>{_('Tasks')}</h1><p>{_('All your tasks from connected trackers')}</p></div></div>
		<TasksMetrics activeTasksCount={activeTasks.length} suggestedCandidatesCount={suggestedTaskCandidates.length} {tasksError} {isLayoutEditing} {isWidgetVisible} />
		<button type="button" class="primary-button" onclick={() => void refreshTasksFromAi()} disabled={isAiTaskRefreshSubmitting}><Icon icon="tabler:sparkles" width="16" height="16" />AI refresh</button>
	</div>
	{#if tasksError}<p class="inline-error">{tasksError}</p>{/if}
	<div class="tasks-layout">
		<TasksActiveList {activeTasks} {suggestedTaskCandidates} {isTasksLoading} {isLayoutEditing} {isWidgetVisible} taskSourceLabel={taskSourceLabel as (item: unknown) => string} taskConfidence={taskConfidence as (item: unknown) => string} {taskCreatedTime} setTaskCandidateReview={setTaskCandidateReview as (candidate: unknown, state: string) => Promise<void>} />
		<aside class="stacked-rail">
			<TasksReviewStats taskCandidatesCount={taskCandidates.length} suggestedCount={suggestedTaskCandidates.length} activeCount={activeTasks.length} {isLayoutEditing} {isWidgetVisible} />
			<TasksContext suggestedCandidates={suggestedTaskCandidates} {isLayoutEditing} {isWidgetVisible} />
			<TasksSources {isLayoutEditing} {isWidgetVisible} />
		</aside>
	</div>

</section>
