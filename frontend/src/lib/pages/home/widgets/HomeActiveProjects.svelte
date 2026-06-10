<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		projects: Array<{ name: string; kind: string; progress: number; icon: string; tone: string }>;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onNavigateToProjects: () => void;
	}

	let { projects, isLayoutEditing, isWidgetVisible, onNavigateToProjects }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="home-active-projects" data-widget-hide-if-clipped-content data-widget-hidden={!isWidgetVisible('home-active-projects')}>
	<WidgetEditChrome widgetId="home-active-projects" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel full-band">
		<header class="panel-title-row">
			<h2>{_('Active Projects')}</h2>
			<button type="button" class="link-button" onclick={onNavigateToProjects}>View all projects</button>
		</header>
		<div class="project-card-row" data-widget-fit-content>
			{#each projects as project}
				<article class="compact-project">
					<span class="round-icon {project.tone}"><Icon icon={project.icon} width="20" height="20" /></span>
					<div><strong>{project.name}</strong><small>{project.kind}</small></div>
					<progress class="progress" max="100" value={project.progress} aria-label={`${project.name} progress`}>{project.progress}%</progress>
					<em>{project.progress}%</em>
				</article>
			{/each}
			<button type="button" class="new-tile" disabled><Icon icon="tabler:plus" width="22" height="22" />New Project</button>
		</div>
	</section>
</div>
