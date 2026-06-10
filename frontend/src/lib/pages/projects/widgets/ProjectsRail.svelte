<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { ProjectDetail, ProjectRecord, ProjectStats, ProjectSummary, ProjectPersonSummary } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		selectedProjectDetail: ProjectDetail | null;
		selectedProjectRecord: ProjectRecord;
		selectedProjectStats: ProjectStats;
		relatedProjectSummaries: ProjectSummary[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;

		projectStatusLabel: (status: string) => string;
		formatNumber: (num: number) => string;
	}

	let {
		selectedProjectDetail,
		selectedProjectRecord,
		selectedProjectStats,
		relatedProjectSummaries,
		isLayoutEditing,
		isWidgetVisible,
		projectStatusLabel,
		formatNumber
	}: Props = $props();
</script>

<aside class="stacked-rail project-side">
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-health" data-widget-hidden={!isWidgetVisible('projects-health')}>
		<WidgetEditChrome widgetId="projects-health" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>{_('Project Health')}</h2>
			<div class="health-row"><span>{_('Status')}</span><strong>{projectStatusLabel(selectedProjectRecord.status)}</strong></div>
			<div class="health-row"><span>{_('Progress')}</span><strong>{selectedProjectRecord.progress_percent}%</strong></div>
			<div class="health-row"><span>{_('Graph Links')}</span><strong>{formatNumber(selectedProjectStats.graph_connection_count)}</strong></div>
		</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-key-people" data-widget-hidden={!isWidgetVisible('projects-key-people')}>
		<WidgetEditChrome widgetId="projects-key-people" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>{_('Key People')}</h2>
			{#if selectedProjectDetail?.key_people.length}
				{#each selectedProjectDetail.key_people as person}
					<div class="person-compact">
						<img src="/assets/hermes-reference-avatar.png" alt="" />
						<span><strong>{person.display_name}</strong><small>{person.email_address}</small></span>
						<em>{formatNumber(person.interaction_count)}</em>
					</div>
				{/each}
			{:else}
				<p class="muted-copy">{_('No linked people.')}</p>
			{/if}
		</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-related-projects" data-widget-hidden={!isWidgetVisible('projects-related-projects')}>
		<WidgetEditChrome widgetId="projects-related-projects" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>{_('Related Projects')}</h2>
			{#if relatedProjectSummaries.length}
				{#each relatedProjectSummaries.slice(0, 4) as item}
					<div class="related-row">
						<span class="round-icon cyan"><Icon icon="tabler:cube" width="16" height="16" /></span>
						<strong>{item.project.name}</strong>
						<em>{item.project.progress_percent}%</em>
					</div>
				{/each}
			{:else}
				<p class="muted-copy">{_('No related project records.')}</p>
			{/if}
		</section>
	</div>
</aside>
