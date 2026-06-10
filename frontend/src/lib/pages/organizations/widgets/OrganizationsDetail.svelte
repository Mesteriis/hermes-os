<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		selectedOrganization: Record<string, unknown> | null;
		orgPeople: unknown[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let { selectedOrganization, orgPeople, isLayoutEditing, isWidgetVisible }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="organizations-detail" data-widget-hidden={!isWidgetVisible('organizations-detail')}>
	<WidgetEditChrome widgetId="organizations-detail" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel org-detail-panel">
		{#if selectedOrganization}
			<header>
				<span class="round-icon blue"><Icon icon="tabler:building" width="26" height="26" /></span>
				<div><h2>{selectedOrganization.display_name as string}</h2><em>{selectedOrganization.industry as string || _('Unknown industry')}{#if selectedOrganization.country} · {selectedOrganization.country as string}{/if}</em></div>
			</header>
			<div class="org-detail-grid">
				<div class="info-card"><h3>{_('Status')}</h3><span class="status-chip {selectedOrganization.status as string}">{selectedOrganization.status as string}</span>{#if selectedOrganization.health_status}<span class="health-chip">{selectedOrganization.health_status as string}</span>{/if}{#if selectedOrganization.watchlist}<span class="health-chip important">{_('Watchlist')}</span>{/if}</div>
				{#if selectedOrganization.description}
					<div class="info-card"><h3>{_('About')}</h3><p>{selectedOrganization.description as string}</p></div>
				{/if}
				<div class="info-card"><h3>{_('Details')}</h3>
					{#if selectedOrganization.website}<div class="detail-row"><span>{_('Website')}</span><strong>{selectedOrganization.website as string}</strong></div>{/if}
					{#if selectedOrganization.legal_name}<div class="detail-row"><span>{_('Legal name')}</span><strong>{selectedOrganization.legal_name as string}</strong></div>{/if}
					{#if selectedOrganization.registration_number}<div class="detail-row"><span>{_('Registration')}</span><strong>{selectedOrganization.registration_number as string}</strong></div>{/if}
					{#if selectedOrganization.vat}<div class="detail-row"><span>{_('VAT')}</span><strong>{selectedOrganization.vat as string}</strong></div>{/if}
					<div class="detail-row"><span>{_('Interactions')}</span><strong>{selectedOrganization.interaction_count as string}</strong></div>
					<div class="detail-row"><span>{_('Priority')}</span><strong>{selectedOrganization.priority as string || _('normal')}</strong></div>
				</div>
				{#if orgPeople.length > 0}
					<div class="info-card"><h3>{_('Key People')}</h3>
						{#each orgPeople as person}
							<div class="person-mini"><span class="round-icon"><Icon icon="tabler:user" width="16" height="16" /></span><strong>{(person as Record<string, unknown>).display_name as string}</strong><small>{(person as Record<string, unknown>).email_address as string}</small></div>
						{/each}
					</div>
				{/if}
			</div>
		{:else}
			<header><span class="round-icon"><Icon icon="tabler:building-off" width="26" height="26" /></span><div><h2>{_('No company selected')}</h2><em>{_('Select a company from the list')}</em></div></header>
		{/if}
	</section>
</div>
