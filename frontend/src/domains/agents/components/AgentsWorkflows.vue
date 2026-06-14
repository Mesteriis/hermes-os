<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AiAnswerResponse, AiMeetingPrepResponse, AiTaskCandidateRefreshResponse, AiCitation } from '../types/agents'
import { safeCitations } from '../stores/agents'

interface Props {
	aiQuestion: string
	aiMeetingTopic: string
	aiTaskQuery: string
	aiAnswerResult: AiAnswerResponse | null
	aiMeetingPrepResult: AiMeetingPrepResponse | null
	aiTaskRefreshResult: AiTaskCandidateRefreshResponse | null
	isAiAnswerSubmitting: boolean
	isAiMeetingPrepSubmitting: boolean
	isAiTaskRefreshSubmitting: boolean
}

interface Emits {
	(e: 'update:aiQuestion', value: string): void
	(e: 'update:aiMeetingTopic', value: string): void
	(e: 'update:aiTaskQuery', value: string): void
	(e: 'submitAnswer'): void
	(e: 'submitMeetingPrep'): void
	(e: 'refreshTasks'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
</script>

<template>
	<div class="ai-workflow-grid">
		<form class="ai-workflow-block" @submit.prevent="emit('submitAnswer')">
			<label>
				<span>Ask AI</span>
				<textarea
					:value="aiQuestion"
					@input="emit('update:aiQuestion', ($event.target as HTMLTextAreaElement).value)"
					rows="4"
				></textarea>
			</label>
			<button type="submit" :disabled="isAiAnswerSubmitting || !aiQuestion.trim()">
				<Icon icon="tabler:sparkles" width="16" height="16" />Ask
			</button>
		</form>
		<form class="ai-workflow-block" @submit.prevent="emit('submitMeetingPrep')">
			<label>
				<span>Prepare brief</span>
				<textarea
					:value="aiMeetingTopic"
					@input="emit('update:aiMeetingTopic', ($event.target as HTMLTextAreaElement).value)"
					rows="4"
				></textarea>
			</label>
			<button type="submit" :disabled="isAiMeetingPrepSubmitting || !aiMeetingTopic.trim()">
				<Icon icon="tabler:calendar-stats" width="16" height="16" />Prepare
			</button>
		</form>
		<form class="ai-workflow-block" @submit.prevent="emit('refreshTasks')">
			<label>
				<span>Task extraction</span>
				<textarea
					:value="aiTaskQuery"
					@input="emit('update:aiTaskQuery', ($event.target as HTMLTextAreaElement).value)"
					rows="4"
				></textarea>
			</label>
			<button type="submit" :disabled="isAiTaskRefreshSubmitting || !aiTaskQuery.trim()">
				<Icon icon="tabler:checkbox" width="16" height="16" />Refresh candidates
			</button>
		</form>
	</div>

	<div v-if="aiAnswerResult" class="ai-result-block">
		<h3>Answer</h3>
		<p>{{ aiAnswerResult.answer }}</p>
		<div class="citation-list">
			<div v-for="citation in aiAnswerResult.citations" :key="citation.source_id + citation.source_kind" class="citation-row">
				<strong>{{ citation.title }}</strong>
				<span>{{ citation.source_kind }}:{{ citation.source_id }}</span>
				<p>{{ citation.excerpt }}</p>
			</div>
		</div>
	</div>

	<div v-if="aiMeetingPrepResult" class="ai-result-block">
		<h3>Meeting Brief</h3>
		<p>{{ aiMeetingPrepResult.briefing }}</p>
		<div class="citation-list">
			<div v-for="citation in aiMeetingPrepResult.citations" :key="citation.source_id + citation.source_kind" class="citation-row">
				<strong>{{ citation.title }}</strong>
				<span>{{ citation.source_kind }}:{{ citation.source_id }}</span>
				<p>{{ citation.excerpt }}</p>
			</div>
		</div>
	</div>

	<div v-if="aiTaskRefreshResult" class="ai-result-block">
		<h3>Task Candidates</h3>
		<p>{{ aiTaskRefreshResult.created_count }} suggested candidates refreshed. Review them in Tasks.</p>
	</div>
</template>

<style scoped>
.ai-workflow-grid {
	display: grid;
	grid-template-columns: repeat(3, minmax(0, 1fr));
	gap: 10px;
	margin-top: 16px;
}

.ai-workflow-block {
	display: grid;
	gap: 10px;
	min-height: var(--hh-widget-panel-large);
	border: 1px solid rgba(111, 205, 195, 0.1);
	border-radius: var(--hh-radius-md);
	background: rgba(5, 22, 25, 0.54);
	padding: 12px;
}

.ai-workflow-block label {
	display: grid;
	gap: 8px;
}

.ai-workflow-block span {
	color: var(--hh-color-text-soft);
	font-size: 12px;
	font-weight: 650;
}

.ai-workflow-block textarea {
	width: 100%;
	min-height: 92px;
	max-height: 130px;
	resize: vertical;
	border: 1px solid rgba(111, 205, 195, 0.16);
	border-radius: var(--hh-radius-md);
	background: rgba(2, 9, 11, 0.7);
	color: var(--hh-color-text);
	font-size: 12px;
	line-height: 1.45;
	padding: 9px 10px;
}

.ai-workflow-block button {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	gap: 7px;
	min-height: 34px;
	border-radius: var(--hh-radius-md);
	background: var(--hh-color-accent);
	color: var(--hh-color-accent-contrast);
	font-size: 12px;
	font-weight: 760;
	border: none;
	cursor: pointer;
}

.ai-workflow-block button:disabled {
	background: rgba(111, 205, 195, 0.16);
	color: #789b98;
	cursor: not-allowed;
}

.ai-result-block {
	display: grid;
	gap: 10px;
	margin-top: 14px;
	border-top: 1px solid var(--hh-border-muted);
	padding-top: 14px;
}

.ai-result-block h3 {
	margin: 0;
	color: var(--hh-color-text-bright);
	font-size: 15px;
}

.ai-result-block > p {
	margin: 0;
	color: var(--hh-color-text-soft);
	font-size: 13px;
	line-height: 1.55;
}

.citation-list {
	display: grid;
	gap: 8px;
}

.citation-row {
	display: grid;
	gap: 4px;
	border-left: 2px solid var(--hh-border-accent);
	background: rgba(45, 240, 206, 0.045);
	padding: 8px 10px;
}

.citation-row strong,
.citation-row span,
.citation-row p {
	overflow-wrap: anywhere;
}

.citation-row strong {
	color: var(--hh-color-text);
	font-size: 12px;
}

.citation-row span {
	color: #7ea4a0;
	font-size: 10px;
}

.citation-row p {
	margin: 0;
	color: #bcd3d1;
	font-size: 12px;
	line-height: 1.45;
}
</style>
