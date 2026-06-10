<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	type AgentCard = {
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
		agentCards: AgentCard[];
		selectedAgentIndex: number;
		isAiLoading: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onSelectAgent: (index: number) => void;
	}

	let {
		agentCards,
		selectedAgentIndex,
		isAiLoading,
		isLayoutEditing,
		isWidgetVisible,
		onSelectAgent
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-agent-list" data-widget-hidden={!isWidgetVisible('ai-agent-list')}>
	<WidgetEditChrome widgetId="ai-agent-list" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<div class="agent-grid">
		{#if isAiLoading && agentCards.length === 0}
			<div class="graph-strip-message"><span>Loading local AI agents.</span></div>
		{:else if agentCards.length === 0}
			<div class="graph-strip-message"><span>No V3 agents returned by the backend.</span></div>
		{:else}
			{#each agentCards as agent, index}
				<button type="button" class="agent-card panel" class:active={selectedAgentIndex === index} onclick={() => onSelectAgent(index)}>
					<span class="round-icon {agent.tone}"><Icon icon={agent.icon} width="22" height="22" /></span>
					<div><strong>{agent.name}</strong><p>{agent.summary}</p><em>{agent.status}</em></div>
					<footer><span>{agent.tasks} runs</span><span>{agent.success} success</span></footer>
				</button>
			{/each}
		{/if}
	</div>
</div>
