<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import {
		integrationStatusLabel,
		serviceStateLabel,
		type IntegrationService,
		type IntegrationServiceId,
		type IntegrationViewModel
	} from '$lib/services/integrations';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		integrations: IntegrationViewModel[];
		selectedIntegrationId: string | null;
		onSelectIntegration: (integrationId: string) => void;
		onOpenAccountDrawer: (target?: string) => void;
		formatDateTimeFn: (value: string | null) => string;
	}

	let {
		integrations,
		selectedIntegrationId,
		onSelectIntegration,
		onOpenAccountDrawer,
		formatDateTimeFn
	}: Props = $props();

	const tableServices: IntegrationServiceId[] = ['mail', 'calendar', 'people', 'messages'];

	let selectedIntegration = $derived(
		integrations.find((integration) => integration.integrationId === selectedIntegrationId) ??
			integrations[0] ??
			null
	);

	function serviceFor(
		integration: IntegrationViewModel,
		serviceId: IntegrationServiceId
	): IntegrationService | null {
		return integration.services.find((service) => service.id === serviceId) ?? null;
	}

	function serviceClass(service: IntegrationService | null): string {
		if (!service || service.state === 'not_applicable') return 'off';
		if (service.state === 'ready') return 'ready';
		if (service.state === 'disabled') return 'disabled';
		return 'warn';
	}

	function statusClass(integration: IntegrationViewModel): string {
		if (integration.status === 'connected') return 'ready';
		if (integration.status === 'empty') return 'muted';
		return 'warn';
	}

	function integrationActionTarget(integration: IntegrationViewModel | null): string {
		if (!integration) return 'mail';
		if (integration.providerKind === 'telegram') return 'telegram';
		if (integration.providerKind === 'whatsapp_web') return 'whatsapp';
		if (['gmail', 'icloud', 'imap'].includes(integration.providerKind)) return integration.providerKind;
		return 'mail';
	}

	function integrationUpdatedLabel(integration: IntegrationViewModel): string {
		return formatDateTimeFn(integration.updatedAt) || _(integration.updatedLabel);
	}

	function accountCountLabel(integration: IntegrationViewModel): string {
		const accountCount = integration.accounts.length;
		const calendarCount = integration.calendarAccounts.length;
		return `${accountCount} ${_('accounts')} / ${calendarCount} ${_('calendars')}`;
	}
</script>

<div class="settings-integrations-layout">
	<section class="settings-integrations-main">
		<header class="settings-workbench-header">
			<div>
				<h2>{_('Integrations')}</h2>
				<p>{_('Connected providers, service coverage and account-level actions.')}</p>
			</div>
			<button type="button" class="primary-button" onclick={() => onOpenAccountDrawer('mail')}>
				<Icon icon="tabler:plus" width="16" height="16" />{_('Add integration')}
			</button>
		</header>

		{#if integrations.length === 0}
			<div class="empty-panel fill">{_('No integrations configured.')}</div>
		{:else}
			<div class="integrations-table" aria-label={_('Integrations')}>
				<div class="integrations-table-head">
					<span>{_('Integration')}</span>
					{#each tableServices as serviceId}
						<span>{_(serviceFor(integrations[0], serviceId)?.label ?? serviceId)}</span>
					{/each}
					<span>{_('Updated')}</span>
					<span>{_('Status')}</span>
				</div>
				{#each integrations as integration}
					<button
						type="button"
						class="integrations-table-row"
						class:selected={selectedIntegration?.integrationId === integration.integrationId}
						onclick={() => onSelectIntegration(integration.integrationId)}
					>
						<span class="integration-primary-cell">
							<span class="round-icon cyan">
								<Icon icon={integration.icon} width="20" height="20" />
							</span>
							<span>
								<strong>{integration.title}</strong>
								<small>{integration.subtitle}</small>
							</span>
						</span>
						{#each tableServices as serviceId}
							{@const service = serviceFor(integration, serviceId)}
							<span class={`integration-service-state ${serviceClass(service)}`}>
								{_(serviceStateLabel(service?.state ?? 'not_applicable'))}
							</span>
						{/each}
						<span class="integration-updated">{integrationUpdatedLabel(integration)}</span>
						<span class={`integration-status ${statusClass(integration)}`}>
							{_(integrationStatusLabel(integration.status))}
						</span>
					</button>
				{/each}
			</div>
		{/if}
	</section>

	<aside class="settings-integration-inspector">
		{#if selectedIntegration}
			<header>
				<h3>{selectedIntegration.title}</h3>
				<p>{selectedIntegration.subtitle}</p>
				<small>{accountCountLabel(selectedIntegration)}</small>
			</header>

			<section class="integration-inspector-section">
				<h4>{_('Services')}</h4>
				{#each selectedIntegration.services as service}
					<div class="integration-service-line">
						<div>
							<strong>{_(service.label)}</strong>
							<small>{_(service.description)}</small>
						</div>
						<span class={`integration-service-state ${serviceClass(service)}`}>
							{_(serviceStateLabel(service.state))}
						</span>
					</div>
				{/each}
			</section>

			<section class="integration-inspector-section">
				<h4>{_('Actions')}</h4>
				<div class="integration-action-stack">
					<button
						type="button"
						class="primary-button"
						onclick={() => onOpenAccountDrawer(integrationActionTarget(selectedIntegration))}
					>
						{_('Reconnect')}
					</button>
					<button type="button" class="ghost-button" disabled>{_('Run sync now')}</button>
					<button type="button" class="ghost-button" disabled>{_('View vault binding')}</button>
					<button type="button" class="danger-button" disabled>{_('Remove integration')}</button>
				</div>
			</section>

			<section class="integration-inspector-section">
				<h4>{_('Metadata')}</h4>
				<ul class="integration-metadata-list">
					{#each Object.entries(selectedIntegration.metadata) as [label, value]}
						<li><span>{_(label)}</span><code>{value}</code></li>
					{/each}
				</ul>
			</section>
		{:else}
			<div class="empty-panel fill">{_('Select an integration.')}</div>
		{/if}
	</aside>
</div>
