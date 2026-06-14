<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AgentCard } from '../types/agents'

interface Props {
	selectedAgent: AgentCard | null
}

const props = defineProps<Props>()
</script>

<template>
	<section class="panel agent-detail">
		<template v-if="selectedAgent">
			<header>
				<span class="round-icon" :class="selectedAgent.tone">
					<Icon :icon="selectedAgent.icon" width="26" height="26" />
				</span>
				<div>
					<h2>{{ selectedAgent.name }}</h2>
					<em>{{ selectedAgent.model }}</em>
				</div>
			</header>
			<div class="section-tabs">
				<button type="button" class="active">Overview</button>
				<button type="button" disabled>Run History</button>
				<button type="button" disabled>Citations</button>
				<button type="button" disabled>Settings</button>
			</div>
			<div class="agent-detail-grid">
				<p>{{ selectedAgent.summary }}. This V3 agent reads local memory projections, retrieves citations and records every run in the backend.</p>
				<div class="spark-chart"></div>
				<ul>
					<li v-for="capability in ['Ollama Runtime','pgvector Retrieval','Source Citations','Run Provenance','Review Queue']" :key="capability">
						<Icon icon="tabler:circle-check" width="16" height="16" />{{ capability }}
					</li>
				</ul>
			</div>
		</template>
		<template v-else>
			<header>
				<span class="round-icon cyan">
					<Icon icon="tabler:robot-off" width="26" height="26" />
				</span>
				<div>
					<h2>No agent selected</h2>
					<em>Backend status required</em>
				</div>
			</header>
		</template>
	</section>
</template>

<style scoped>
.agent-detail {
	margin-top: 12px;
	padding: 14px;
}

.agent-detail header {
	display: flex;
	align-items: center;
	gap: 12px;
}

.agent-detail h2 {
	color: var(--hh-color-text-bright);
	font-size: 20px;
}

.agent-detail-grid {
	display: grid;
	grid-template-columns: 1fr 300px 240px;
	gap: 22px;
	padding: 14px 8px 0;
}

.agent-detail-grid p,
.agent-detail-grid li {
	color: #c7d9d8;
	font-size: 13px;
	line-height: 1.5;
}

.agent-detail-grid ul {
	display: grid;
	gap: 12px;
	margin: 0;
	padding: 0;
	list-style: none;
}

.agent-detail-grid li {
	display: flex;
	align-items: center;
	gap: 8px;
}

.spark-chart {
	height: 150px;
	border: 1px solid rgba(111, 205, 195, 0.1);
	border-radius: var(--hh-radius-md);
	background:
		linear-gradient(160deg, transparent 42%, rgba(45, 240, 206, 0.9) 43%, transparent 44%),
		linear-gradient(rgba(45, 240, 206, 0.035) 1px, transparent 1px);
	background-size: auto, 28px 28px;
}
</style>
