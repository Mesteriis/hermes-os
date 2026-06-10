<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { ProjectDetail, ProjectStats, ProjectRecord, ProjectTimelineItem, ProjectMessageSummary, ProjectDocumentSummary } from '$lib/api';

	interface Props {
		selectedProjectDetail: ProjectDetail | null;
		selectedProjectRecord: ProjectRecord;
		selectedProjectStats: ProjectStats;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;

		projectTimelineIcon: (item: ProjectTimelineItem) => string;
		projectMessageSender: (message: ProjectMessageSummary) => string;
		projectDocumentIcon: (document: ProjectDocumentSummary) => string;
		formatProjectDateTime: (date: string | null) => string;
		formatNumber: (num: number) => string;
	}

	let {
		selectedProjectDetail,
		selectedProjectRecord,
		selectedProjectStats,
		isLayoutEditing,
		isWidgetVisible,
		projectTimelineIcon,
		projectMessageSender,
		projectDocumentIcon,
		formatProjectDateTime,
		formatNumber
	}: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-summary" data-widget-hidden={!isWidgetVisible('projects-summary')}>
	<WidgetEditChrome widgetId="projects-summary" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>Project Summary</h2>
		<div class="summary-numbers">
			<article><strong>{formatNumber(selectedProjectStats.document_count)}</strong><span>Documents</span></article>
			<article><strong>{formatNumber(selectedProjectStats.message_count)}</strong><span>Messages</span></article>
			<article><strong>{formatNumber(selectedProjectStats.people_count)}</strong><span>People</span></article>
			<article><strong>{formatNumber(selectedProjectStats.graph_connection_count)}</strong><span>Graph links</span></article>
		</div>
	</section>
</div>
<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-graph-preview" data-widget-hidden={!isWidgetVisible('projects-graph-preview')}>
	<WidgetEditChrome widgetId="projects-graph-preview" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel graph-card-large">
		<h2>Knowledge Graph</h2>
		<div class="radial-graph">
			<div class="graph-center"><Icon icon="tabler:cube" width="30" height="30" /><span>{selectedProjectRecord.name}</span></div>
			<span class="graph-chip graph-chip-messages">Messages {formatNumber(selectedProjectStats.message_count)}</span>
			<span class="graph-chip graph-chip-documents">Documents {formatNumber(selectedProjectStats.document_count)}</span>
			<span class="graph-chip graph-chip-people">People {formatNumber(selectedProjectStats.people_count)}</span>
			<span class="graph-chip graph-chip-links">Links {formatNumber(selectedProjectStats.graph_connection_count)}</span>
		</div>
	</section>
</div>
<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-timeline" data-widget-hidden={!isWidgetVisible('projects-timeline')}>
	<WidgetEditChrome widgetId="projects-timeline" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>Project Timeline</h2>
		{#if selectedProjectDetail?.timeline.length}
			{#each selectedProjectDetail.timeline as item}
				<div class="timeline-mini">
					<Icon icon={projectTimelineIcon(item)} width="16" height="16" />
					<time>{formatProjectDateTime(item.occurred_at)}</time>
					<strong>{item.title}</strong>
				</div>
			{/each}
		{:else}
			<p class="muted-copy">No timeline items from local sources.</p>
		{/if}
	</section>
</div>
<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-recent-communications" data-widget-hidden={!isWidgetVisible('projects-recent-communications')}>
	<WidgetEditChrome widgetId="projects-recent-communications" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>Recent Communications</h2>
		{#if selectedProjectDetail?.recent_messages.length}
			{#each selectedProjectDetail.recent_messages as message}
				<div class="related-row">
					<span class="round-icon cyan"><Icon icon="tabler:mail" width="16" height="16" /></span>
					<strong>{projectMessageSender(message)}</strong>
					<em>{formatProjectDateTime(message.occurred_at)}</em>
				</div>
			{/each}
		{:else}
			<p class="muted-copy">No linked communications.</p>
		{/if}
	</section>
</div>
<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-top-documents" data-widget-hidden={!isWidgetVisible('projects-top-documents')}>
	<WidgetEditChrome widgetId="projects-top-documents" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>Top Documents</h2>
		{#if selectedProjectDetail?.documents.length}
			{#each selectedProjectDetail.documents as document}
				<div class="doc-mini">
					<Icon icon={projectDocumentIcon(document)} width="20" height="20" />
					<span><strong>{document.title}</strong><small>{document.document_kind} · {formatProjectDateTime(document.imported_at)}</small></span>
				</div>
			{/each}
		{:else}
			<p class="muted-copy">No linked documents.</p>
		{/if}
	</section>
</div>
<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-source-evidence" data-widget-hidden={!isWidgetVisible('projects-source-evidence')}>
	<WidgetEditChrome widgetId="projects-source-evidence" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>Source Evidence</h2>
		<div class="summary-numbers compact">
			<article><strong>{formatNumber(selectedProjectStats.message_count + selectedProjectStats.document_count)}</strong><span>Matched records</span></article>
			<article><strong>{formatProjectDateTime(selectedProjectStats.latest_activity_at)}</strong><span>Last activity</span></article>
		</div>
	</section>
</div>
<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-open-promises" data-widget-hidden={!isWidgetVisible('projects-open-promises')}>
	<WidgetEditChrome widgetId="projects-open-promises" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel info-card">
		<h2>Open Promises</h2>
		<p class="muted-copy">No task candidates connected to this project.</p>
		<button type="button" class="link-row" disabled>View all promises <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
	</section>
</div>
