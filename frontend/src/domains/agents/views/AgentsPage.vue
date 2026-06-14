<script setup lang="ts">
import { watch } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useAiWorkspaceQuery } from '../queries/useAgentsQuery'
import { useAgentsStore, aiModelSummary } from '../stores/agents'
import AgentsRuntimeMetrics from '../components/AgentsRuntimeMetrics.vue'
import AgentsGrid from '../components/AgentsGrid.vue'
import AgentsDetail from '../components/AgentsDetail.vue'
import AgentsWorkflows from '../components/AgentsWorkflows.vue'
import AgentsRail from '../components/AgentsRail.vue'

const store = useAgentsStore()

const { data: workspaceData, isLoading, refetch } = useAiWorkspaceQuery()

watch(workspaceData, (val) => {
	if (val) {
		store.setWorkspace(val)
		store.setLoading(false)
	}
})

watch(isLoading, (val) => {
	store.setLoading(val)
})
</script>

<template>
	<section class="agents-page">
		<div class="view-header">
			<div class="view-title-with-icon">
				<span class="hero-mark small"><Icon icon="tabler:robot" width="28" height="28" /></span>
				<div>
					<h1>AI Agents</h1>
					<p>Local AI agents, runtime and run history</p>
				</div>
			</div>
			<button type="button" class="primary-button" :disabled="store.isAiLoading" @click="refetch()">
				<Icon icon="tabler:refresh" width="16" height="16" />Refresh
			</button>
		</div>

		<AgentsRuntimeMetrics
			:ai-status="store.aiStatus"
			:ai-agents="store.aiAgents"
			:ai-runs="store.aiRuns"
			:is-ai-loading="store.isAiLoading"
		/>

		<p v-if="store.aiError" class="inline-error">{{ store.aiError }}</p>

		<div class="filter-bar">
			<button type="button" class="active">Local Agents</button>
			<button type="button" disabled>{{ aiModelSummary(store.aiStatus) }}</button>
			<button type="button" disabled>{{ store.aiStatus?.chat_model_available ? 'Chat model ready' : 'Chat model missing' }}</button>
			<button type="button" disabled>{{ store.aiStatus?.embedding_model_available ? 'Embedding ready' : 'Embedding missing' }}</button>
		</div>

		<div class="agents-layout">
			<section class="agent-main">
				<AgentsGrid
					:agent-cards="store.agentCards"
					:selected-agent-index="store.selectedAgentIndex"
					:is-ai-loading="store.isAiLoading"
					@select-agent="store.selectAgent($event)"
				/>
				<AgentsDetail :selected-agent="store.selectedAgent" />
				<AgentsWorkflows
					:ai-question="store.aiQuestion"
					:ai-meeting-topic="store.aiMeetingTopic"
					:ai-task-query="store.aiTaskQuery"
					:ai-answer-result="store.aiAnswerResult"
					:ai-meeting-prep-result="store.aiMeetingPrepResult"
					:ai-task-refresh-result="store.aiTaskRefreshResult"
					:is-ai-answer-submitting="store.isAiAnswerSubmitting"
					:is-ai-meeting-prep-submitting="store.isAiMeetingPrepSubmitting"
					:is-ai-task-refresh-submitting="store.isAiTaskRefreshSubmitting"
					@update:ai-question="store.aiQuestion = $event"
					@update:ai-meeting-topic="store.aiMeetingTopic = $event"
					@update:ai-task-query="store.aiTaskQuery = $event"
					@submit-answer="store.submitAiAnswer()"
					@submit-meeting-prep="store.prepareAiBrief()"
					@refresh-tasks="store.refreshTasksFromAi()"
				/>
			</section>

			<AgentsRail
				:ai-runs="store.aiRuns"
				:ai-status="store.aiStatus"
				:owner-persona="store.ownerPersona"
				:is-ai-loading="store.isAiLoading"
			/>
		</div>
	</section>
</template>

<style scoped>
.agent-main {
	display: grid;
	gap: 12px;
	align-content: start;
	min-width: 0;
}

.agents-layout {
	display: grid;
	grid-template-columns: minmax(760px, 1fr) 310px;
	gap: 12px;
	min-height: var(--hh-widget-workbench-large);
}
</style>
