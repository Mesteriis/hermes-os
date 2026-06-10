<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { AiAnswerResponse, AiMeetingPrepResponse, AiTaskCandidateRefreshResponse, AiCitation } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		aiQuestion: string;
		aiMeetingTopic: string;
		aiTaskQuery: string;
		aiAnswerResult: AiAnswerResponse | null;
		aiMeetingPrepResult: AiMeetingPrepResponse | null;
		aiTaskRefreshResult: AiTaskCandidateRefreshResponse | null;
		isAiAnswerSubmitting: boolean;
		isAiMeetingPrepSubmitting: boolean;
		isAiTaskRefreshSubmitting: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onSubmitAnswer: () => void;
		onSubmitMeetingPrep: () => void;
		onRefreshTasks: () => void;
		safeCitations: (citations: unknown) => AiCitation[];
	}

	let {
		aiQuestion = $bindable(''),
		aiMeetingTopic = $bindable(''),
		aiTaskQuery = $bindable(''),
		aiAnswerResult,
		aiMeetingPrepResult,
		aiTaskRefreshResult,
		isAiAnswerSubmitting,
		isAiMeetingPrepSubmitting,
		isAiTaskRefreshSubmitting,
		isLayoutEditing,
		isWidgetVisible,
		onSubmitAnswer,
		onSubmitMeetingPrep,
		onRefreshTasks,
		safeCitations
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-workflows" data-widget-hidden={!isWidgetVisible('ai-workflows')}>
	<WidgetEditChrome widgetId="ai-workflows" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<div class="ai-workflow-grid">
		<form class="ai-workflow-block" onsubmit={(event) => { event.preventDefault(); onSubmitAnswer(); }}>
			<label><span>Ask AI</span><textarea bind:value={aiQuestion} rows="4"></textarea></label>
			<button type="submit" disabled={isAiAnswerSubmitting || !aiQuestion.trim()}><Icon icon="tabler:sparkles" width="16" height="16" />Ask</button>
		</form>
		<form class="ai-workflow-block" onsubmit={(event) => { event.preventDefault(); onSubmitMeetingPrep(); }}>
			<label><span>Prepare brief</span><textarea bind:value={aiMeetingTopic} rows="4"></textarea></label>
			<button type="submit" disabled={isAiMeetingPrepSubmitting || !aiMeetingTopic.trim()}><Icon icon="tabler:calendar-stats" width="16" height="16" />Prepare</button>
		</form>
		<form class="ai-workflow-block" onsubmit={(event) => { event.preventDefault(); onRefreshTasks(); }}>
			<label><span>Task extraction</span><textarea bind:value={aiTaskQuery} rows="4"></textarea></label>
			<button type="submit" disabled={isAiTaskRefreshSubmitting || !aiTaskQuery.trim()}><Icon icon="tabler:checkbox" width="16" height="16" />Refresh candidates</button>
		</form>
	</div>
	{#if aiAnswerResult}
		<div class="ai-result-block">
			<h3>Answer</h3>
			<p>{aiAnswerResult.answer}</p>
			<div class="citation-list">
				{#each aiAnswerResult.citations as citation}
					<div class="citation-row"><strong>{citation.title}</strong><span>{citation.source_kind}:{citation.source_id}</span><p>{citation.excerpt}</p></div>
				{/each}
			</div>
		</div>
	{/if}
	{#if aiMeetingPrepResult}
		<div class="ai-result-block">
			<h3>Meeting Brief</h3>
			<p>{aiMeetingPrepResult.briefing}</p>
			<div class="citation-list">
				{#each aiMeetingPrepResult.citations as citation}
					<div class="citation-row"><strong>{citation.title}</strong><span>{citation.source_kind}:{citation.source_id}</span><p>{citation.excerpt}</p></div>
				{/each}
			</div>
		</div>
	{/if}
	{#if aiTaskRefreshResult}
		<div class="ai-result-block">
			<h3>Task Candidates</h3>
			<p>{aiTaskRefreshResult.created_count} suggested candidates refreshed. Review them in Tasks.</p>
		</div>
	{/if}
</div>
