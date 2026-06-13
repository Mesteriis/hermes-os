<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import * as personsService from '$lib/services/persons';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import type { PersonDossier } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	type FeedItem = {
		icon: string;
		title: string;
		meta: string;
		time: string;
		tag?: string;
		tone?: string;
	};

	type ProjectItem = {
		name: string;
		kind: string;
		progress: number;
		icon: string;
		tone: string;
	};

	type PersonItem = {
		name: string;
		role: string;
		company: string;
		channel?: string;
		status?: string;
	};

	interface Props {
		selectedPerson: PersonItem | null;
		personDossier: PersonDossier | null;
		isPersonDossierLoading: boolean;
		personDossierError: string;
		whatsNew: FeedItem[];
		projects: ProjectItem[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let {
		selectedPerson,
		personDossier,
		isPersonDossierLoading,
		personDossierError,
		whatsNew,
		projects,
		isLayoutEditing,
		isWidgetVisible
	}: Props = $props();

	let dossierPreview = $derived(personDossier ? personsService.dossierSectionPreview(personDossier) : []);
</script>

<section class="person-detail">
	{#if selectedPerson}
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-hero" data-widget-hidden={!isWidgetVisible('persons-hero')}>
		<WidgetEditChrome widgetId="persons-hero" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<header class="person-hero panel">
			<img src="/assets/hermes-reference-avatar.png" alt="" />
			<div><h1>{selectedPerson.name}</h1><p>{selectedPerson.role} at {selectedPerson.company}</p><small>{selectedPerson.status ?? selectedPerson.channel ?? 'Contact'}</small></div>
			<div class="chat-actions">
				<button type="button" disabled><Icon icon="tabler:mail" width="17" height="17" /></button>
				<button type="button" disabled><Icon icon="tabler:phone" width="17" height="17" /></button>
				<button type="button" disabled><Icon icon="tabler:video" width="17" height="17" /></button>
				<button type="button" disabled><Icon icon="tabler:brand-whatsapp" width="17" height="17" /></button>
			</div>
		</header>
	</div>
	<div class="section-tabs">
		<button type="button" class="active">Overview</button>
		<button type="button" disabled>Communications</button>
		<button type="button" disabled>Documents <em>24</em></button>
		<button type="button" disabled>Tasks <em>7</em></button>
		<button type="button" disabled>Projects <em>5</em></button>
		<button type="button" disabled>Notes</button>
	</div>
	<div class="person-cards">
		<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-information" data-widget-hidden={!isWidgetVisible('persons-information')}>
			<WidgetEditChrome widgetId="persons-information" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
			<section class="panel info-card">
				<h2>Person Information</h2>
				<ul class="detail-list">
					<li><Icon icon="tabler:mail" width="17" height="17" /> {selectedPerson.company} <em>Primary</em></li>
					<li><Icon icon="tabler:phone" width="17" height="17" /> +1 (555) 123-4567 <em>Mobile</em></li>
					<li><Icon icon="tabler:brand-telegram" width="17" height="17" /> @john.smith <em>Telegram</em></li>
					<li><Icon icon="tabler:map-pin" width="17" height="17" /> New York, USA <em>Local Time: 18:42</em></li>
				</ul>
			</section>
		</div>
		<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-about" data-widget-hidden={!isWidgetVisible('persons-about')}>
			<WidgetEditChrome widgetId="persons-about" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
			<section class="panel info-card">
				<h2>Persona Dossier</h2>
				{#if isPersonDossierLoading}
					<p>Loading dossier...</p>
				{:else if personDossierError}
					<p class="inline-error">{personDossierError}</p>
				{:else if personDossier}
					<p>{personDossier.summary || 'No dossier summary yet.'}</p>
					{#if dossierPreview.length}
						<div class="tag-cloud">
							{#each dossierPreview as item}
								<span>{item}</span>
							{/each}
						</div>
					{/if}
					<small>{personDossier.source_refs.length} source refs · generated {new Date(personDossier.generated_at).toLocaleString()}</small>
				{:else}
					<p>No dossier generated yet.</p>
				{/if}
			</section>
		</div>
		<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-relationship-strength" data-widget-hidden={!isWidgetVisible('persons-relationship-strength')}>
			<WidgetEditChrome widgetId="persons-relationship-strength" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
			<section class="panel info-card"><h2>Relationship Strength</h2><div class="big-score">85</div><strong>Strong</strong><p>Last interaction 2 hours ago</p></section>
		</div>
		<div class="widget-frame span-2" class:editing={isLayoutEditing} data-widget-id="persons-recent-interactions" data-widget-hidden={!isWidgetVisible('persons-recent-interactions')}>
			<WidgetEditChrome widgetId="persons-recent-interactions" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
			<section class="panel info-card span-2"><h2>Recent Interactions</h2>{#each whatsNew.slice(0, 3) as item}<div class="feed-row compact-row"><span class="round-icon {item.tone}"><Icon icon={item.icon} width="18" height="18" /></span><div><strong>{item.title}</strong><p>{item.meta}</p></div><time>{item.time}</time></div>{/each}</section>
		</div>
		<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-active-projects" data-widget-hidden={!isWidgetVisible('persons-active-projects')}>
			<WidgetEditChrome widgetId="persons-active-projects" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
			<section class="panel info-card"><h2>{_('Active Projects')}</h2>{#each projects.slice(0, 3) as project}<div class="related-row"><span class="round-icon {project.tone}"><Icon icon={project.icon} width="16" height="16" /></span><strong>{project.name}</strong><em>{project.progress}%</em></div>{/each}</section>
		</div>
	</div>
	{:else}
		<section class="panel empty-domain-state">
			<Icon icon="tabler:user" width="42" height="42" />
			<div>
				<h2>No person selected</h2>
				<p>Hermes Hub will show relationship memory here after persons are imported from local sources.</p>
			</div>
		</section>
	{/if}
</section>
