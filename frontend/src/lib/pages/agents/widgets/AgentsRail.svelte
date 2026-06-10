<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { AiRun, AiStatus, AiCitation } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		aiRuns: AiRun[];
		aiStatus: AiStatus | null;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		aiRuntimeSummary: () => string;
		runStatusLabel: (run: AiRun) => string;
		formatDuration: (ms: number | null) => string;
		formatDateTime: (date: string) => string;
		safeCitations: (citations: unknown) => AiCitation[];
	}

	let {
		aiRuns,
		aiStatus,
		isLayoutEditing,
		isWidgetVisible,
		aiRuntimeSummary,
		runStatusLabel,
		formatDuration,
		formatDateTime,
		safeCitations
	}: Props = $props();
</script>

<aside class="stacked-rail">
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-runtime-status" data-widget-hidden={!isWidgetVisible('ai-runtime-status')}>
		<WidgetEditChrome widgetId="ai-runtime-status" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card"><h2>Runtime</h2><div class="health-row"><span>Status</span><strong>{aiRuntimeSummary()}</strong></div><div class="health-row"><span>Chat</span><strong>{aiStatus?.chat_model ?? 'unknown'}</strong></div><div class="health-row"><span>Embedding</span><strong>{aiStatus?.embedding_model ?? 'unknown'}</strong></div></section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-run-history" data-widget-hidden={!isWidgetVisible('ai-run-history')}>
		<WidgetEditChrome widgetId="ai-run-history" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card"><h2>Run History</h2>{#if aiRuns.length}{#each aiRuns.slice(0,6) as run}<div class="deadline"><span>{run.agent_id} · {runStatusLabel(run)}</span><time>{formatDateTime(run.started_at)} · {formatDuration(run.duration_ms)}</time></div>{/each}{:else}<p>No AI runs persisted yet.</p>{/if}</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-citations" data-widget-hidden={!isWidgetVisible('ai-citations')}>
		<WidgetEditChrome widgetId="ai-citations" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card"><h2>Latest Citations</h2>{#if aiRuns[0] && safeCitations(aiRuns[0].citations).length}{#each safeCitations(aiRuns[0].citations).slice(0,3) as citation}<div class="evidence-row"><strong>{citation.title}</strong><p>{citation.excerpt}</p></div>{/each}{:else}<p>Citations appear after an answer or briefing run.</p>{/if}</section>
	</div>
</aside>
