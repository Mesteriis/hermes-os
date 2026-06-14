<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AgentCard } from '../types/agents'

interface Props {
	agentCards: AgentCard[]
	selectedAgentIndex: number
	isAiLoading: boolean
}

interface Emits {
	(e: 'selectAgent', index: number): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
</script>

<template>
	<div class="agent-grid">
		<template v-if="isAiLoading && agentCards.length === 0">
			<div class="graph-strip-message"><span>Loading local AI agents.</span></div>
		</template>
		<template v-else-if="agentCards.length === 0">
			<div class="graph-strip-message"><span>No V3 agents returned by the backend.</span></div>
		</template>
		<template v-else>
			<button
				v-for="(agent, index) in agentCards"
				:key="agent.agentId"
				type="button"
				class="agent-card panel"
				:class="{ active: selectedAgentIndex === index }"
				@click="emit('selectAgent', index)"
			>
				<span class="round-icon" :class="agent.tone">
					<Icon :icon="agent.icon" width="22" height="22" />
				</span>
				<div>
					<strong>{{ agent.name }}</strong>
					<p>{{ agent.summary }}</p>
					<em>{{ agent.status }}</em>
				</div>
				<footer>
					<span>{{ agent.tasks }} runs</span>
					<span>{{ agent.success }} success</span>
				</footer>
			</button>
		</template>
	</div>
</template>

<style scoped>
.agent-grid {
	display: grid;
	grid-template-columns: repeat(3, minmax(0, 1fr));
	gap: 10px;
}

.agent-card {
	display: grid;
	grid-template-columns: 44px 1fr;
	gap: 12px;
	min-height: var(--hh-widget-panel);
	padding: 12px;
	text-align: left;
}

.agent-card.active {
	border-color: rgba(45, 240, 206, 0.38);
}

.agent-card footer {
	grid-column: 1 / -1;
	display: flex;
	justify-content: space-between;
	border-top: 1px solid rgba(102, 189, 180, 0.08);
	padding-top: 10px;
	font-size: 11px;
}

@media (max-width: 1360px) {
	.agent-grid {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}
}
</style>
