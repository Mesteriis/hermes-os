<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as decisionsService from '$lib/services/decisions';
	import * as obligationsService from '$lib/services/obligations';
	import * as tasksService from '$lib/services/tasks';
	import type {
		Decision,
		DecisionEntityKind,
		DecisionReviewState,
		Obligation,
		ObligationReviewState,
		Task,
		TaskCandidate,
		TaskCandidateReviewState
	} from '$lib/api';
	import TasksMetrics from './widgets/TasksMetrics.svelte';
	import TasksActiveList from './widgets/TasksActiveList.svelte';
	import TasksReviewStats from './widgets/TasksReviewStats.svelte';
	import TasksContext from './widgets/TasksContext.svelte';
	import TasksSources from './widgets/TasksSources.svelte';
	import TasksDecisionObligationReview from './widgets/TasksDecisionObligationReview.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();

	let activeTasks = $state<Task[]>([]);
	let taskCandidates = $state<TaskCandidate[]>([]);
	let decisions = $state<Decision[]>([]);
	let obligations = $state<Obligation[]>([]);
	let tasksError = $state('');
	let contextReviewError = $state('');
	let isTasksLoading = $state(false);
	let isContextReviewLoading = $state(false);
	let isAiTaskRefreshSubmitting = $state(false);
	let reviewEntityKind = $state<DecisionEntityKind>('project');
	let reviewEntityId = $state('');
	let reviewingContextItemId = $state<string | null>(null);

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

	function setReviewEntityKind(entityKind: DecisionEntityKind) {
		reviewEntityKind = entityKind;
	}

	function setReviewEntityId(entityId: string) {
		reviewEntityId = entityId;
	}

	async function loadContextReview() {
		const entityId = reviewEntityId.trim();

		isContextReviewLoading = true;
		const [decisionResult, obligationResult] = entityId
			? await Promise.all([
					decisionsService.loadDecisionReviewState(reviewEntityKind, entityId),
					obligationsService.loadObligationReviewState(reviewEntityKind, entityId)
				])
			: await Promise.all([
					decisionsService.loadGlobalDecisionReviewState(),
					obligationsService.loadGlobalObligationReviewState()
				]);
		decisions = decisionResult.decisions;
		obligations = obligationResult.obligations;
		contextReviewError = [decisionResult.error, obligationResult.error].filter(Boolean).join(' · ');
		isContextReviewLoading = false;
	}

	async function reviewDecision(decision: Decision, reviewState: Exclude<DecisionReviewState, 'suggested'>) {
		reviewingContextItemId = `decision:${decision.decision_id}`;
		const result = await decisionsService.reviewDecisionItem(decision, reviewState);
		if (result.error) {
			contextReviewError = result.error;
		} else {
			await loadContextReview();
		}
		reviewingContextItemId = null;
	}

	async function reviewObligation(
		obligation: Obligation,
		reviewState: Exclude<ObligationReviewState, 'suggested'>
	) {
		reviewingContextItemId = `obligation:${obligation.obligation_id}`;
		const result = await obligationsService.reviewObligationItem(obligation, reviewState);
		if (result.error) {
			contextReviewError = result.error;
		} else {
			await loadContextReview();
		}
		reviewingContextItemId = null;
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
		loadContextReview();
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
			<TasksDecisionObligationReview
				{decisions}
				{obligations}
				entityKind={reviewEntityKind}
				entityId={reviewEntityId}
				isLoading={isContextReviewLoading}
				error={contextReviewError}
				reviewingItemId={reviewingContextItemId}
				{isLayoutEditing}
				{isWidgetVisible}
				onEntityKindChange={setReviewEntityKind}
				onEntityIdChange={setReviewEntityId}
				onReload={loadContextReview}
				onReviewDecision={reviewDecision}
				onReviewObligation={reviewObligation}
			/>
			<TasksContext suggestedCandidates={suggestedTaskCandidates} {isLayoutEditing} {isWidgetVisible} />
			<TasksSources {isLayoutEditing} {isWidgetVisible} />
		</aside>
	</div>

</section>
