<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import * as projectsService from '$lib/services/projects';
	import ProjectsHero from './widgets/ProjectsHero.svelte';
	import ProjectsDashboard from './widgets/ProjectsDashboard.svelte';
	import ProjectsRail from './widgets/ProjectsRail.svelte';
	import './projects.css';
	import type {
		ProjectRecord,
		ProjectStats,
		ProjectDetail,
		ProjectSummary,
		ProjectTimelineItem,
		ProjectMessageSummary,
		ProjectDocumentSummary
	} from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { isLayoutEditing, isWidgetVisible }: Props = $props();

	let projectSummaries = $state<ProjectSummary[]>([]);
	let selectedProjectDetail = $state<ProjectDetail | null>(null);
	let selectedProjectId = $state('');
	let projectsError = $state('');
	let isProjectsLoading = $state(false);

	let selectedProjectRecord = $derived(
		selectedProjectDetail?.project ?? projectSummaries[0]?.project ?? null
	);
	let selectedProjectStats = $derived(
		selectedProjectDetail?.stats ?? projectSummaries[0]?.stats ?? projectsService.emptyProjectStats()
	);
	let relatedProjectSummaries = $derived(
		projectSummaries.filter((item) => item.project.project_id !== selectedProjectRecord?.project_id)
	);

	function projectStatusLabel(status: string) {
		return status.split('_').map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(' ');
	}

	function projectTimelineIcon(item: ProjectTimelineItem) {
		switch (item.item_kind) {
			case 'message': return 'tabler:mail';
			case 'document': return 'tabler:file-text';
			default: return 'tabler:circle-dot';
		}
	}

	function projectMessageSender(message: ProjectMessageSummary) {
		return message.sender || _('Unknown');
	}

	function projectDocumentIcon(document: ProjectDocumentSummary) {
		switch (document.document_kind) {
			case 'pdf': return 'tabler:file-type-pdf';
			case 'markdown': return 'tabler:file-text';
			default: return 'tabler:file';
		}
	}

	function formatProjectDate(value: string | null) {
		if (!value) return _('Not set');
		const date = new Date(`${value}T00:00:00`);
	if (Number.isNaN(date.getTime())) return _('Invalid date');
	return new Intl.DateTimeFormat($currentLocale ?? 'en', { month: 'short', day: 'numeric', year: 'numeric' }).format(date);
}

function formatProjectDateTime(value: string | null) {
	if (!value) return _('No activity');
	const date = new Date(value);
	if (Number.isNaN(date.getTime())) return _('Invalid date');
	return new Intl.DateTimeFormat($currentLocale ?? 'en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date);
	}

	function formatNumber(value: number) {
		return new Intl.NumberFormat($currentLocale ?? 'en-US').format(value);
	}

	function selectProject(item: ProjectSummary) {
		if (item.project.project_id === selectedProjectId && selectedProjectDetail) return;
		selectedProjectId = item.project.project_id;
		loadProjectDetail(item.project.project_id);
	}

	async function loadProjectDetail(projectId: string) {
		if (!projectId) { selectedProjectDetail = null; return; }
		isProjectsLoading = true;
		const result = await projectsService.loadProjectDetail(projectId);
		selectedProjectDetail = result.detail;
		projectsError = result.error;
		isProjectsLoading = false;
	}

	async function loadProjects() {
		isProjectsLoading = true;
		projectSummaries = [];
		try {
			const result = await projectsService.loadProjects(selectedProjectId, 0);
			projectSummaries = result.summaries;
			selectedProjectDetail = result.detail;
			projectsError = result.error;
			selectedProjectId = result.selectedProjectId;
		} catch (e: unknown) {
			projectsError = e instanceof Error ? e.message : _('Unknown projects error');
		}
		isProjectsLoading = false;
	}

	$effect(() => {
		loadProjects();
	});

	async function prepareAiBrief(_projectId?: string) {}
</script>

<section class="projects-page">
	<ProjectsHero
		{projectsError}
		{isProjectsLoading}
		{selectedProjectRecord}
		{selectedProjectStats}
		{projectSummaries}
		{isLayoutEditing}
		{isWidgetVisible}
		isAiMeetingPrepSubmitting={false}
		{projectStatusLabel}
		{formatProjectDate}
		{formatNumber}
		{selectProject}
		{loadProjects}
		{prepareAiBrief}
	/>

	{#if selectedProjectRecord}
		<div class="project-dashboard-grid">
			<ProjectsDashboard
				{selectedProjectDetail}
				{selectedProjectRecord}
				{selectedProjectStats}
				{isLayoutEditing}
				{isWidgetVisible}
				{projectTimelineIcon}
				{projectMessageSender}
				{projectDocumentIcon}
				{formatProjectDateTime}
				{formatNumber}
			/>
			<ProjectsRail
				{selectedProjectDetail}
				{selectedProjectRecord}
				{selectedProjectStats}
				{relatedProjectSummaries}
				{isLayoutEditing}
				{isWidgetVisible}
				{projectStatusLabel}
				{formatNumber}
			/>
		</div>
	{/if}
</section>
