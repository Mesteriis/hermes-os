<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AiRun, AiStatus, AiCitation, OwnerPersona } from '../types/agents'
import { aiRuntimeSummary, runStatusLabel, safeCitations } from '../stores/agents'
import { formatDuration, formatDateTime } from '../stores/agents'

interface Props {
	aiRuns: AiRun[]
	aiStatus: AiStatus | null
	ownerPersona: OwnerPersona | null
	isAiLoading: boolean
}

const props = defineProps<Props>()

function formatAgentPersonaName(agentId: string): string {
	return `${agentId.trim().toLowerCase()}@sh-inc.ru`
}
</script>

<template>
	<aside class="stacked-rail">
		<section class="panel info-card">
			<h2>Runtime</h2>
			<div class="health-row">
				<span>Status</span>
				<strong>{{ aiRuntimeSummary(aiStatus, isAiLoading) }}</strong>
			</div>
			<div class="health-row">
				<span>Owner Persona</span>
				<strong>{{ ownerPersona?.display_name ?? 'not set' }}</strong>
			</div>
			<div class="health-row">
				<span>Chat</span>
				<strong>{{ aiStatus?.chat_model ?? 'unknown' }}</strong>
			</div>
			<div class="health-row">
				<span>Embedding</span>
				<strong>{{ aiStatus?.embedding_model ?? 'unknown' }}</strong>
			</div>
		</section>

		<section class="panel info-card">
			<h2>Run History</h2>
			<template v-if="aiRuns.length">
				<div v-for="run in aiRuns.slice(0, 6)" :key="run.run_id" class="deadline">
					<span>{{ formatAgentPersonaName(run.agent_id) }} · {{ runStatusLabel(run) }}</span>
					<time>{{ formatDateTime(run.started_at) }} · {{ formatDuration(run.duration_ms) }}</time>
				</div>
			</template>
			<p v-else>No AI runs persisted yet.</p>
		</section>

		<section class="panel info-card">
			<h2>Latest Citations</h2>
			<template v-if="aiRuns[0] && safeCitations(aiRuns[0].citations).length">
				<div v-for="citation in safeCitations(aiRuns[0].citations).slice(0, 3)" :key="citation.source_id + citation.source_kind" class="evidence-row">
					<strong>{{ citation.title }}</strong>
					<p>{{ citation.excerpt }}</p>
				</div>
			</template>
			<p v-else>Citations appear after an answer or briefing run.</p>
		</section>
	</aside>
</template>
