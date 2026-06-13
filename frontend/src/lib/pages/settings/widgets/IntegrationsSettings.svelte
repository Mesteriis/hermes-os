<script lang="ts">
	import '../integrations.css';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import {
		integrationGroupLabel,
		integrationStatusLabel,
		serviceStateLabel,
		type IntegrationGroupId,
		type IntegrationService,
		type IntegrationServiceId,
		type IntegrationViewModel
	} from '$lib/services/integrations';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		integrations: IntegrationViewModel[];
		selectedIntegrationId: string | null;
		onSelectIntegration: (integrationId: string) => void;
		onCloseIntegration: () => void;
		onOpenAccountDrawer: (target?: string) => void;
		onExportMailAccount: (accountId: string) => Promise<void>;
		onLogoutMailAccount: (accountId: string) => Promise<void>;
		onDeleteMailAccount: (accountId: string) => Promise<void>;
		onImportMailSettings: (rawJson: string) => Promise<void>;
		formatDateTimeFn: (value: string | null) => string;
	}

	let {
		integrations,
		selectedIntegrationId,
		onSelectIntegration,
		onCloseIntegration,
		onOpenAccountDrawer,
		onExportMailAccount,
		onLogoutMailAccount,
		onDeleteMailAccount,
		onImportMailSettings,
		formatDateTimeFn
	}: Props = $props();

	const tableServices: IntegrationServiceId[] = ['mail', 'calendar', 'people', 'messages'];

	type IntegrationTableGroup = {
		id: IntegrationGroupId;
		integrations: IntegrationViewModel[];
	};

	let selectedIntegration = $derived(
		selectedIntegrationId
			? (integrations.find((integration) => integration.integrationId === selectedIntegrationId) ?? null)
			: null
	);
	let selectedMailAccountId = $derived(mailAccountId(selectedIntegration));
	let integrationGroups = $derived.by(() => {
		const groups: IntegrationTableGroup[] = [];
		for (const integration of integrations) {
			let group = groups.find((item) => item.id === integration.group);
			if (!group) {
				group = { id: integration.group, integrations: [] };
				groups.push(group);
			}
			group.integrations.push(integration);
		}
		return groups;
	});
	let isImportPanelOpen = $state(false);
	let mailImportJson = $state('');
	let activeMailAction = $state<string | null>(null);

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
		if (integration.providerKind.startsWith('calendar:')) return 'calendar';
		if (['gmail', 'icloud', 'imap'].includes(integration.providerKind)) return integration.providerKind;
		return 'mail';
	}

	function integrationUpdatedLabel(integration: IntegrationViewModel): string {
		return formatDateTimeFn(integration.updatedAt) || _(integration.updatedLabel);
	}

	function accountCountLabel(integration: IntegrationViewModel): string {
		const accountCount = integration.accounts.length;
		const calendarCount = integration.calendarAccounts.length;
		const accountUnit = accountCount === 1 ? _('account') : _('accounts');
		const calendarUnit = calendarCount === 1 ? _('calendar') : _('calendars');
		return `${accountCount} ${accountUnit} / ${calendarCount} ${calendarUnit}`;
	}

	function mailAccountId(integration: IntegrationViewModel | null): string | null {
		const account = integration?.accounts.find((item) =>
			['gmail', 'icloud', 'imap'].includes(item.provider_kind)
		);
		return account?.account_id ?? null;
	}

	function isMailActionPending(action: string, accountId?: string | null): boolean {
		const actionKey = accountId ? `${action}:${accountId}` : action;
		return activeMailAction === actionKey;
	}

	async function runMailAction(
		action: 'export' | 'logout' | 'delete',
		accountId: string,
		handler: (accountId: string) => Promise<void>
	) {
		activeMailAction = `${action}:${accountId}`;
		try {
			await handler(accountId);
			if (action === 'delete') {
				onCloseIntegration();
			}
		} finally {
			activeMailAction = null;
		}
	}

	async function submitMailSettingsImport() {
		activeMailAction = 'import';
		try {
			await onImportMailSettings(mailImportJson);
			mailImportJson = '';
			isImportPanelOpen = false;
		} finally {
			activeMailAction = null;
		}
	}

	function confirmDeleteMailAccount(accountId: string) {
		if (
			typeof window !== 'undefined' &&
			!window.confirm(_('Delete this mail account metadata? Retained messages will remain in Hermes.'))
		) {
			return;
		}
		void runMailAction('delete', accountId, onDeleteMailAccount);
	}
</script>

<div class="settings-integrations-layout">
	<section class="settings-integrations-main">
		<header class="settings-workbench-header">
			<div>
				<h2>{_('Integrations')}</h2>
				<p>{_('Connected providers, service coverage and account-level actions.')}</p>
			</div>
			<div class="settings-workbench-header-actions">
				<button
					type="button"
					class="ghost-button"
					onclick={() => (isImportPanelOpen = !isImportPanelOpen)}
					disabled={isImportPanelOpen && isMailActionPending('import')}
				>
					<Icon icon="tabler:upload" width="16" height="16" />{_('Import settings')}
				</button>
				<button type="button" class="primary-button" onclick={() => onOpenAccountDrawer('mail')}>
					<Icon icon="tabler:plus" width="16" height="16" />{_('Add integration')}
				</button>
			</div>
		</header>

		{#if isImportPanelOpen}
			<form
				class="mail-settings-import-panel"
				onsubmit={(event) => {
					event.preventDefault();
					void submitMailSettingsImport();
				}}
			>
				<label>
					<span>{_('Mail settings JSON')}</span>
					<textarea
						bind:value={mailImportJson}
						rows="5"
						placeholder={_('Paste sanitized exported mail account settings JSON.')}
					></textarea>
				</label>
				<div class="form-actions">
					<button
						type="submit"
						class="primary-button"
						disabled={!mailImportJson.trim() || isMailActionPending('import')}
					>
						<Icon icon="tabler:upload" width="16" height="16" />{_('Import')}
					</button>
					<button type="button" class="ghost-button" onclick={() => (isImportPanelOpen = false)}>
						{_('Cancel')}
					</button>
				</div>
			</form>
		{/if}

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
				{#each integrationGroups as group}
					<div class="integrations-table-group">
						<span>{_(integrationGroupLabel(group.id))}</span>
					</div>
					{#each group.integrations as integration}
						<button
							type="button"
							class="integrations-table-row"
							class:selected={selectedIntegration?.integrationId === integration.integrationId}
							aria-expanded={selectedIntegration?.integrationId === integration.integrationId}
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
				{/each}
			</div>
		{/if}
	</section>

	<aside
		id="settings-integration-inspector"
		class="settings-integration-inspector"
		class:open={Boolean(selectedIntegration)}
		aria-hidden={!selectedIntegration}
	>
		{#if selectedIntegration}
			<header class="integration-inspector-header">
				<div>
					<h3>{selectedIntegration.title}</h3>
					<p>{selectedIntegration.subtitle}</p>
					<small>{accountCountLabel(selectedIntegration)}</small>
				</div>
				<button
					type="button"
					class="icon-button integration-inspector-close"
					aria-label={_('Close integration details')}
					title={_('Close integration details')}
					onclick={onCloseIntegration}
				>
					<Icon icon="tabler:x" width="18" height="18" />
				</button>
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
					{#if selectedMailAccountId}
						<button
							type="button"
							class="ghost-button"
							disabled={isMailActionPending('export', selectedMailAccountId)}
							onclick={() =>
								void runMailAction('export', selectedMailAccountId, onExportMailAccount)}
						>
							<Icon icon="tabler:download" width="16" height="16" />{_('Export settings')}
						</button>
						<button
							type="button"
							class="ghost-button"
							disabled={isMailActionPending('logout', selectedMailAccountId)}
							onclick={() =>
								void runMailAction('logout', selectedMailAccountId, onLogoutMailAccount)}
						>
							<Icon icon="tabler:logout" width="16" height="16" />{_('Logout')}
						</button>
						<button
							type="button"
							class="danger-button"
							disabled={isMailActionPending('delete', selectedMailAccountId)}
							title={_('This action removes only unused account metadata. Retained evidence blocks deletion.')}
							onclick={() => confirmDeleteMailAccount(selectedMailAccountId)}
						>
							<Icon icon="tabler:trash" width="16" height="16" />{_('Delete account')}
						</button>
					{:else}
						<button type="button" class="ghost-button" disabled>{_('Export settings')}</button>
						<button type="button" class="ghost-button" disabled>{_('Logout')}</button>
						<button type="button" class="danger-button" disabled>{_('Delete account')}</button>
					{/if}
					<button type="button" class="ghost-button" disabled>{_('Run sync now')}</button>
					<button type="button" class="ghost-button" disabled>{_('View vault binding')}</button>
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
			<span class="integration-inspector-empty">{_('Select an integration.')}</span>
		{/if}
	</aside>
</div>
