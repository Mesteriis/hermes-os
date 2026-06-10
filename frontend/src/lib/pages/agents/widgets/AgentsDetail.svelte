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
		selectedAgent: AgentCard | null;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { selectedAgent, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-selected-agent-detail" data-widget-hidden={!isWidgetVisible('ai-selected-agent-detail')}>
	<WidgetEditChrome widgetId="ai-selected-agent-detail" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel agent-detail">
		{#if selectedAgent}
			<header><span class="round-icon {selectedAgent.tone}"><Icon icon={selectedAgent.icon} width="26" height="26" /></span><div><h2>{selectedAgent.name}</h2><em>{selectedAgent.model}</em></div></header>
			<div class="section-tabs"><button type="button" class="active">Overview</button><button type="button" disabled>Run History</button><button type="button" disabled>Citations</button><button type="button" disabled>Settings</button></div>
			<div class="agent-detail-grid"><p>{selectedAgent.summary}. This V3 agent reads local memory projections, retrieves citations and records every run in the backend.</p><div class="spark-chart"></div><ul>{#each ['Ollama Runtime','pgvector Retrieval','Source Citations','Run Provenance','Review Queue'] as capability}<li><Icon icon="tabler:circle-check" width="16" height="16" />{capability}</li>{/each}</ul></div>
		{:else}
			<header><span class="round-icon cyan"><Icon icon="tabler:robot-off" width="26" height="26" /></span><div><h2>No agent selected</h2><em>Backend status required</em></div></header>
		{/if}
	</section>
</div>
