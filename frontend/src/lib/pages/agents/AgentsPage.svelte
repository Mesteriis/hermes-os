<script lang="ts">
	import './agents.css';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as aiService from '$lib/services/ai';
	import type {
		AiStatus,
		AiAgent,
		AiRun,
		AiAnswerResponse,
		AiMeetingPrepResponse,
		AiTaskCandidateRefreshResponse,
		AiCitation,
		OwnerPersona
	} from '$lib/api';
	import AgentsRuntimeMetrics from './widgets/AgentsRuntimeMetrics.svelte';
	import AgentsGrid from './widgets/AgentsGrid.svelte';
	import AgentsDetail from './widgets/AgentsDetail.svelte';
	import AgentsWorkflows from './widgets/AgentsWorkflows.svelte';
	import AgentsRail from './widgets/AgentsRail.svelte';

	const _ = (key: string) => t($currentLocale, key);

	type AgentCard = {
		agentId: string;
		name: string;
		icon: string;
		tone: string;
		summary: string;
		status: string;
		model: string;
		tasks: number;
		success: number;
	};

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();

	let aiStatus = $state<AiStatus | null>(null);
	let aiAgents = $state<AiAgent[]>([]);
	let aiRuns = $state<AiRun[]>([]);
	let ownerPersona = $state<OwnerPersona | null>(null);
	let aiError = $state('');
	let isAiLoading = $state(false);
	let isAiAnswerSubmitting = $state(false);
	let isAiMeetingPrepSubmitting = $state(false);
	let isAiTaskRefreshSubmitting = $state(false);
	let selectedAgentIndex = $state(0);
	let aiQuestion = $state('What does the local memory say about Hermes Hub V3?');
	let aiMeetingTopic = $state('Prepare a V3 implementation review brief');
	let aiTaskQuery = $state('Find open task candidates from local messages and documents');
	let aiAnswerResult = $state<AiAnswerResponse | null>(null);
	let aiMeetingPrepResult = $state<AiMeetingPrepResponse | null>(null);
	let aiTaskRefreshResult = $state<AiTaskCandidateRefreshResponse | null>(null);

	let agentCards = $derived(aiAgents.map((agent) => aiService.agentCardView(agent, aiRuns)));
	let selectedAgent = $derived(agentCards[selectedAgentIndex] ?? agentCards[0] ?? null);

	function runStatusLabel(run: AiRun) {
		if (run.status === 'completed') return 'Completed';
		if (run.status === 'failed') return 'Failed';
		return 'Requested';
	}

	function safeCitations(value: unknown): AiCitation[] {
		if (!Array.isArray(value)) return [];
		return value.filter(isAiCitation);
	}

	function isAiCitation(value: unknown): value is AiCitation {
		return (
			typeof value === 'object' && value !== null &&
			typeof (value as { source_kind?: unknown }).source_kind === 'string' &&
			typeof (value as { source_id?: unknown }).source_id === 'string' &&
			typeof (value as { title?: unknown }).title === 'string' &&
			typeof (value as { excerpt?: unknown }).excerpt === 'string'
		);
	}

	function formatDuration(durationMs: number | null | undefined) {
		if (durationMs == null) return 'n/a';
		if (durationMs < 1000) return `${durationMs} ms`;
		return `${(durationMs / 1000).toFixed(1)} s`;
	}

	function formatDateTime(date: string) {
		const d = new Date(date);
		if (Number.isNaN(d.getTime())) return 'Invalid date';
		return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(d);
	}

	function aiRuntimeSummary() {
		if (!aiStatus) return isAiLoading ? 'Loading' : 'Unknown';
		return aiStatus.status === 'ok' ? 'Ready' : 'Unavailable';
	}

	function aiModelSummary() {
		if (!aiStatus) return 'No status';
		return `${aiStatus.chat_model} / ${aiStatus.embedding_model}`;
	}

	async function loadAiWorkspace() {
		isAiLoading = true;
		const result = await aiService.loadAiWorkspace();
		aiAgents = result.agents;
		aiRuns = result.runs;
		aiStatus = result.status;
		ownerPersona = result.ownerPersona;
		aiError = result.error;
		if (selectedAgentIndex >= aiAgents.length) selectedAgentIndex = 0;
		isAiLoading = false;
	}

	async function submitAiAnswer() {
		const query = aiQuestion.trim();
		if (!query || isAiAnswerSubmitting) return;
		isAiAnswerSubmitting = true;
		const result = await aiService.submitAiAnswer(query, selectedAgent?.agentId ?? 'MNEMOSYNE');
		aiAnswerResult = result.result;
		aiError = result.error;
		isAiAnswerSubmitting = false;
		if (!result.error) await loadAiRunsOnly();
	}

	async function prepareAiBrief() {
		const topic = aiMeetingTopic.trim();
		if (!topic || isAiMeetingPrepSubmitting) return;
		isAiMeetingPrepSubmitting = true;
		const result = await aiService.prepareAiBrief(topic, undefined);
		aiMeetingPrepResult = result.result;
		aiError = result.error;
		isAiMeetingPrepSubmitting = false;
		if (!result.error) await loadAiRunsOnly();
	}

	async function refreshTasksFromAi() {
		const query = aiTaskQuery.trim();
		if (!query || isAiTaskRefreshSubmitting) return;
		isAiTaskRefreshSubmitting = true;
		const result = await aiService.refreshTasksFromAi(query, selectedAgent?.agentId ?? 'MNEMOSYNE');
		aiTaskRefreshResult = result.result;
		aiError = result.error;
		isAiTaskRefreshSubmitting = false;
		if (!result.error) await loadAiRunsOnly();
	}

	async function loadAiRunsOnly() {
		const result = await aiService.loadAiRunsOnly();
		aiRuns = result.runs;
		if (result.error) aiError = result.error;
	}

	$effect(() => {
		loadAiWorkspace();
	});
</script>

<section class="agents-page">
	<div class="view-header">
		<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:robot" width="28" height="28" /></span><div><h1>{_('AI Agents')}</h1><p>{_('Local AI agents, runtime and run history')}</p></div></div>
		<button type="button" class="primary-button" onclick={() => void loadAiWorkspace()} disabled={isAiLoading}><Icon icon="tabler:refresh" width="16" height="16" />Refresh</button>
	</div>

	<AgentsRuntimeMetrics
		{aiStatus}
		{aiAgents}
		{aiRuns}
		suggestedTaskCandidates={[]}
		{isLayoutEditing}
		{isWidgetVisible}
		{aiRuntimeSummary}
		{formatDuration}
		formatAgentPersonaName={aiService.aiAgentPersonaEmail}
	/>

	{#if aiError}
		<p class="inline-error">{aiError}</p>
	{/if}

	<div class="filter-bar"><button type="button" class="active">Local Agents</button><button type="button" disabled>{aiModelSummary()}</button><button type="button" disabled>{aiStatus?.chat_model_available ? 'Chat model ready' : 'Chat model missing'}</button><button type="button" disabled>{aiStatus?.embedding_model_available ? 'Embedding ready' : 'Embedding missing'}</button></div>

	<div class="agents-layout">
		<section class="agent-main">
			<AgentsGrid {agentCards} {selectedAgentIndex} {isAiLoading} {isLayoutEditing} {isWidgetVisible} onSelectAgent={(index) => (selectedAgentIndex = index)} />
			<AgentsDetail {selectedAgent} {isLayoutEditing} {isWidgetVisible} />
			<AgentsWorkflows
				bind:aiQuestion
				bind:aiMeetingTopic
				bind:aiTaskQuery
				{aiAnswerResult}
				{aiMeetingPrepResult}
				{aiTaskRefreshResult}
				{isAiAnswerSubmitting}
				{isAiMeetingPrepSubmitting}
				{isAiTaskRefreshSubmitting}
				{isLayoutEditing}
				{isWidgetVisible}
				onSubmitAnswer={submitAiAnswer}
				onSubmitMeetingPrep={prepareAiBrief}
				onRefreshTasks={refreshTasksFromAi}
				{safeCitations}
			/>
		</section>

		<AgentsRail
			{aiRuns}
			{aiStatus}
			{ownerPersona}
			{isLayoutEditing}
			{isWidgetVisible}
			{aiRuntimeSummary}
			{runStatusLabel}
			{formatDuration}
			{formatDateTime}
			{safeCitations}
			formatAgentPersonaName={aiService.aiAgentPersonaEmail}
		/>
	</div>

</section>
