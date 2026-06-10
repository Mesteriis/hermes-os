<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { ProjectRecord, ProjectStats, ProjectSummary } from '$lib/api';

	interface Props {
		projectsError: string;
		isProjectsLoading: boolean;
		selectedProjectRecord: ProjectRecord | null;
		selectedProjectStats: ProjectStats;
		projectSummaries: ProjectSummary[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		isAiMeetingPrepSubmitting: boolean;

		projectStatusLabel: (status: string) => string;
		formatProjectDate: (date: string | null) => string;
		formatNumber: (num: number) => string;
		selectProject: (item: ProjectSummary) => void;
		loadProjects: () => Promise<void>;
		prepareAiBrief: (projectId: string) => Promise<void>;
	}

	let {
		projectsError,
		isProjectsLoading,
		selectedProjectRecord,
		selectedProjectStats,
		projectSummaries,
		isLayoutEditing,
		isWidgetVisible,
		isAiMeetingPrepSubmitting,
		projectStatusLabel,
		formatProjectDate,
		formatNumber,
		selectProject,
		loadProjects,
		prepareAiBrief
	}: Props = $props();
</script>

{#if projectsError && !selectedProjectRecord}
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-hero" data-widget-hidden={!isWidgetVisible('projects-hero')}>
		<WidgetEditChrome widgetId="projects-hero" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card project-empty-state">
			<Icon icon="tabler:alert-circle" width="28" height="28" />
			<h2>Projects unavailable</h2>
			<p>{projectsError}</p>
			<button type="button" onclick={() => void loadProjects()}>Retry</button>
		</section>
	</div>
{:else if !selectedProjectRecord}
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-hero" data-widget-hidden={!isWidgetVisible('projects-hero')}>
		<WidgetEditChrome widgetId="projects-hero" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card project-empty-state">
			<Icon icon="tabler:cube" width="30" height="30" />
			<h2>No projects returned</h2>
			<p>{isProjectsLoading ? 'Loading local projects...' : 'Local project records are empty.'}</p>
		</section>
	</div>
{:else}
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-hero" data-widget-hidden={!isWidgetVisible('projects-hero')}>
		<WidgetEditChrome widgetId="projects-hero" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<header class="project-hero panel">
			<div class="project-logo"><Icon icon="tabler:cube" width="48" height="48" /></div>
			<div>
				<h1>{selectedProjectRecord.name} <em>{projectStatusLabel(selectedProjectRecord.status)}</em></h1>
				<p>{selectedProjectRecord.kind}</p>
				<small>{selectedProjectRecord.description}</small>
			</div>
			<button type="button" class="primary-button" onclick={() => void prepareAiBrief(selectedProjectRecord.project_id)} disabled={isAiMeetingPrepSubmitting}><Icon icon="tabler:calendar-stats" width="16" height="16" />Prepare brief</button>
		</header>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-metadata-strip" data-widget-hidden={!isWidgetVisible('projects-metadata-strip')}>
		<WidgetEditChrome widgetId="projects-metadata-strip" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<div class="project-meta-strip panel">
			<article><span>Owner</span><strong>{selectedProjectRecord.owner_display_name}</strong></article>
			<article><span>People</span><strong>{formatNumber(selectedProjectStats.people_count)}</strong></article>
			<article><span>Start Date</span><strong>{formatProjectDate(selectedProjectRecord.start_date)}</strong></article>
			<article><span>Target Date</span><strong>{formatProjectDate(selectedProjectRecord.target_date)}</strong></article>
			<article><span>Progress</span><progress class="progress" max="100" value={selectedProjectRecord.progress_percent} aria-label={`${selectedProjectRecord.name} progress`}>{selectedProjectRecord.progress_percent}%</progress><strong>{selectedProjectRecord.progress_percent}%</strong></article>
		</div>
	</div>
	{#if projectSummaries.length > 1}
		<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-switcher" data-widget-hidden={!isWidgetVisible('projects-switcher')}>
			<WidgetEditChrome widgetId="projects-switcher" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
			<div class="project-switcher panel">
				{#each projectSummaries as item}
					<button
						type="button"
						class:active={item.project.project_id === selectedProjectRecord.project_id}
						onclick={() => selectProject(item)}
					>
						<Icon icon="tabler:cube" width="16" height="16" />
						<span>{item.project.name}</span>
						<em>{item.project.progress_percent}%</em>
					</button>
				{/each}
			</div>
		</div>
	{/if}
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-section-tabs" data-widget-hidden={!isWidgetVisible('projects-section-tabs')}>
		<WidgetEditChrome widgetId="projects-section-tabs" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<div class="section-tabs">
			<button type="button" class="active">Overview</button>
			<button type="button" disabled>Communications <em>{selectedProjectStats.message_count}</em></button>
			<button type="button" disabled>Tasks</button>
			<button type="button" disabled>Documents <em>{selectedProjectStats.document_count}</em></button>
			<button type="button" disabled>Calendar</button>
			<button type="button" disabled>Team <em>{selectedProjectStats.people_count}</em></button>
			<button type="button" disabled>Notes</button>
			<button type="button" disabled>Files</button>
			<button type="button" disabled>Settings</button>
		</div>
	</div>
	{#if projectsError}
		<p class="inline-error">{projectsError}</p>
	{/if}
{/if}
