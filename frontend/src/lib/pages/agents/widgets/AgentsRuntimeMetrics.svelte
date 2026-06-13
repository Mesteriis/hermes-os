<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { AiStatus, AiAgent, AiRun } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		aiStatus: AiStatus | null;
		aiAgents: AiAgent[];
		aiRuns: AiRun[];
		suggestedTaskCandidates: Array<{ title: string; [key: string]: unknown }>;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		aiRuntimeSummary: () => string;
		formatDuration: (ms: number | null) => string;
		formatAgentPersonaName: (agentId: string) => string;
	}

	let {
		aiStatus,
		aiAgents,
		aiRuns,
		suggestedTaskCandidates,
		isLayoutEditing,
		isWidgetVisible,
		aiRuntimeSummary,
		formatDuration,
		formatAgentPersonaName
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-runtime-metrics" data-widget-hidden={!isWidgetVisible('ai-runtime-metrics')}>
	<WidgetEditChrome widgetId="ai-runtime-metrics" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<div class="metric-grid agent-metrics">
		<article class="metric-card"><span>Runtime</span><strong>{aiRuntimeSummary()}</strong><small>{aiStatus?.version ? `Ollama ${aiStatus.version}` : 'Ollama'}</small></article>
		<article class="metric-card"><span>Agents</span><strong>{aiAgents.length}</strong><small>{aiAgents.length ? 'Registered' : 'Not loaded'}</small></article>
		<article class="metric-card"><span>Run History</span><strong>{aiRuns.length}</strong><small>Persisted runs</small></article>
		<article class="metric-card"><span>Embedding</span><strong>{aiStatus?.embedding_dimension ?? 0}</strong><small>{aiStatus?.embedding_model ?? 'No model'}</small></article>
		<article class="metric-card"><span>Suggested Tasks</span><strong>{suggestedTaskCandidates.length}</strong><small>Review queue</small></article>
		<article class="metric-card"><span>Latest Duration</span><strong>{formatDuration(aiRuns[0]?.duration_ms)}</strong><small>{aiRuns[0] ? formatAgentPersonaName(aiRuns[0].agent_id) : 'No runs'}</small></article>
	</div>
</div>
