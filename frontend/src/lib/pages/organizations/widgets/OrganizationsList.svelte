<script lang="ts">
	import { currentLocale, t } from '$lib/i18n';
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		organizations: Array<{ organization_id: string; display_name: string; industry?: string | null; country?: string | null; status?: string | null; watchlist?: boolean | null }>;
		selectedOrganizationId: string;
		isOrganizationsLoading: boolean;
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		onSelectOrg: (id: string) => void;
	}

	let { organizations, selectedOrganizationId, isOrganizationsLoading, isLayoutEditing, isWidgetVisible, onSelectOrg }: Props = $props();
</script>

<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="organizations-list" data-widget-hidden={!isWidgetVisible('organizations-list')}>
	<WidgetEditChrome widgetId="organizations-list" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
	<section class="panel org-list-panel">
		<header class="panel-title-row"><h2>{_('All Companies')} ({organizations.length})</h2></header>
		{#if isOrganizationsLoading && organizations.length === 0}
			<div class="graph-strip-message"><span>{_('Loading companies.')}</span></div>
		{:else if organizations.length === 0}
			<div class="graph-strip-message"><span>{_('No companies yet.')}</span></div>
		{:else}
			{#each organizations as org}
				<button type="button" class="org-row" class:active={selectedOrganizationId === org.organization_id} onclick={() => onSelectOrg(org.organization_id)}>
					<span class="round-icon blue"><Icon icon="tabler:building" width="20" height="20" /></span>
					<div>
						<strong>{org.display_name}</strong>
						<p>{org.industry || _('Unknown industry')}{#if org.country} · {org.country}{/if}</p>
					</div>
					<small>{org.status}{#if org.watchlist} · ⚠ {_('watchlist')}{/if}</small>
				</button>
			{/each}
		{/if}
	</section>
</div>
