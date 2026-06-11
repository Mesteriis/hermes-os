<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type {
		CommunicationMessageDetail,
		CommunicationMessageSummary,
		MailResourceSnapshot,
		MailResourceSummary
	} from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	type InspectorMode = 'context' | 'contact' | 'organization';

	interface Props {
		mode: InspectorMode;
		selectedCommunication: CommunicationMessageSummary | null;
		selectedCommunicationDetail: CommunicationMessageDetail | null;
		mailResources: MailResourceSnapshot;
		mailResourceSummary: MailResourceSummary;
		projects: unknown[];
		tasks: unknown[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		senderLabel: (sender: string) => string;
		senderEmail: (sender: string) => string;
		communicationChannelLabel: (kind: string) => string;
		messageTime: (msg: unknown) => string;
		onModeChange: (mode: InspectorMode) => void;
		onClose: () => void;
	}

	let {
		mode,
		selectedCommunication,
		selectedCommunicationDetail,
		mailResources,
		mailResourceSummary,
		projects,
		tasks,
		isLayoutEditing,
		isWidgetVisible,
		senderLabel,
		senderEmail,
		communicationChannelLabel,
		messageTime,
		onModeChange,
		onClose
	}: Props = $props();

	const message = $derived(selectedCommunicationDetail?.message ?? selectedCommunication);
	const email = $derived(message ? senderEmail(message.sender) : '');
	const domain = $derived(email.includes('@') ? email.split('@')[1] : '');
	const graphCounts = $derived([
		{ label: _('Emails'), value: message ? 1 : 0, icon: 'tabler:mail' },
		{ label: _('Tasks'), value: tasks.length, icon: 'tabler:square-check' },
		{ label: _('Documents'), value: mailResourceSummary.legalDocuments + mailResourceSummary.certificates, icon: 'tabler:files' },
		{ label: _('Contracts'), value: mailResourceSummary.legalDocuments, icon: 'tabler:file-certificate' },
		{ label: _('Purchases'), value: mailResourceSummary.invoices, icon: 'tabler:receipt' },
		{ label: _('Subscriptions'), value: mailResourceSummary.subscriptions, icon: 'tabler:repeat' },
		{ label: _('Meetings'), value: 0, icon: 'tabler:calendar-event' }
	]);
</script>

<aside class="context-inspector">
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-context-inspector" data-widget-hidden={!isWidgetVisible('communications-sender-profile')}>
		<WidgetEditChrome widgetId="communications-context-inspector" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel inspector-panel">
			<header class="inspector-header">
				<div>
					<strong>{_('Context Inspector')}</strong>
					<p>{message ? senderLabel(message.sender) : _('No entity selected')}</p>
				</div>
				<button type="button" onclick={onClose} title={_('Close')}><Icon icon="tabler:x" width="17" height="17" /></button>
			</header>
			<nav class="inspector-tabs">
				<button type="button" class:active={mode === 'context'} onclick={() => onModeChange('context')}>{_('Context')}</button>
				<button type="button" class:active={mode === 'contact'} onclick={() => onModeChange('contact')}>{_('Contact')}</button>
				<button type="button" class:active={mode === 'organization'} onclick={() => onModeChange('organization')}>{_('Organization')}</button>
			</nav>

			{#if mode === 'context'}
				<div class="inspector-entity-head">
					<span class="round-icon cyan"><Icon icon="tabler:network" width="18" height="18" /></span>
					<div>
						<h2>{message?.subject ?? _('Context')}</h2>
						<p>{message ? communicationChannelLabel(message.channel_kind) : _('No source')}</p>
					</div>
				</div>
				<div class="mail-resource-grid graph-resource-grid">
					{#each graphCounts as item}
						<span><Icon icon={item.icon} width="15" height="15" />{item.label}</span><strong>{item.value}</strong>
					{/each}
				</div>
				<button type="button" class="link-row" disabled><Icon icon="tabler:share" width="15" height="15" />{_('Open in Knowledge Graph')}</button>
			{:else if mode === 'contact'}
				<div class="inspector-entity-head">
					<img src="/assets/hermes-reference-avatar.png" alt="" />
					<div>
						<h2>{message ? senderLabel(message.sender) : _('Contact')}</h2>
						<p>{email || _('No email')}</p>
					</div>
				</div>
				<div class="detail-list inspector-detail-list">
					<p><Icon icon="tabler:mail" width="16" height="16" />{email || _('Unknown')}</p>
					<p><Icon icon="tabler:phone" width="16" height="16" />{_('Not available')}</p>
					<p><Icon icon="tabler:brand-telegram" width="16" height="16" />{_('Not available')}</p>
					<p><Icon icon="tabler:brand-whatsapp" width="16" height="16" />{_('Not available')}</p>
				</div>
				<div class="mail-resource-list">
					<strong>{_('Recent Communications')}</strong>
					{#if message}<p>{message.subject}<em>{messageTime(message)}</em></p>{:else}<p>{_('No communications')}</p>{/if}
				</div>
			{:else}
				<div class="inspector-entity-head">
					<span class="round-icon emerald"><Icon icon="tabler:building" width="18" height="18" /></span>
					<div>
						<h2>{domain || _('Organization')}</h2>
						<p>{domain || _('No domain')}</p>
					</div>
				</div>
				<div class="mail-resource-grid graph-resource-grid">
					<span>{_('Domains')}</span><strong>{domain ? 1 : 0}</strong>
					<span>{_('Employees')}</span><strong>{message ? 1 : 0}</strong>
					<span>{_('Projects')}</span><strong>{projects.length}</strong>
					<span>{_('Documents')}</span><strong>{mailResourceSummary.legalDocuments + mailResourceSummary.certificates}</strong>
					<span>{_('Contracts')}</span><strong>{mailResourceSummary.legalDocuments}</strong>
				</div>
				{#if mailResources.invoices.length || mailResources.legalDocuments.length}
					<div class="mail-resource-list">
						<strong>{_('Documents')}</strong>
						{#each mailResources.legalDocuments.slice(0, 2) as document}
							<p>{document.title}<em>{document.status}</em></p>
						{/each}
						{#each mailResources.invoices.slice(0, 1) as invoice}
							<p>{invoice.counterparty ?? invoice.invoice_id}<em>{invoice.status}</em></p>
						{/each}
					</div>
				{/if}
			{/if}
		</section>
	</div>
</aside>
