<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AiStatus, AiAgent, AiRun } from '../types/agents'
import { aiRuntimeSummary, formatDuration } from '../stores/agents'

interface Props {
	aiStatus: AiStatus | null
	aiAgents: AiAgent[]
	aiRuns: AiRun[]
	isAiLoading: boolean
}

const props = defineProps<Props>()

function formatAgentPersonaName(agentId: string): string {
	return `${agentId.trim().toLowerCase()}@sh-inc.ru`
}
</script>

<template>
	<div class="metric-grid agent-metrics">
		<article class="metric-card">
			<span>Runtime</span>
			<strong>{{ aiRuntimeSummary(aiStatus, isAiLoading) }}</strong>
			<small>{{ aiStatus?.version ? `Ollama ${aiStatus.version}` : 'Ollama' }}</small>
		</article>
		<article class="metric-card">
			<span>Agents</span>
			<strong>{{ aiAgents.length }}</strong>
			<small>{{ aiAgents.length ? 'Registered' : 'Not loaded' }}</small>
		</article>
		<article class="metric-card">
			<span>Run History</span>
			<strong>{{ aiRuns.length }}</strong>
			<small>Persisted runs</small>
		</article>
		<article class="metric-card">
			<span>Embedding</span>
			<strong>{{ aiStatus?.embedding_dimension ?? 0 }}</strong>
			<small>{{ aiStatus?.embedding_model ?? 'No model' }}</small>
		</article>
		<article class="metric-card">
			<span>Suggested Tasks</span>
			<strong>0</strong>
			<small>Review queue</small>
		</article>
		<article class="metric-card">
			<span>Latest Duration</span>
			<strong>{{ formatDuration(aiRuns[0]?.duration_ms) }}</strong>
			<small>{{ aiRuns[0] ? formatAgentPersonaName(aiRuns[0].agent_id) : 'No runs' }}</small>
		</article>
	</div>
</template>

<style scoped>
.agent-metrics {
	grid-template-columns: repeat(6, 1fr);
	margin-bottom: 12px;
}
</style>
