<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { CommunicationMessageSummary, MailResourceSnapshot, MailResourceSummary } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		selectedCommunication: CommunicationMessageSummary | null;
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
	}

	let {
		selectedCommunication,
		mailResources,
		mailResourceSummary,
		projects,
		tasks,
		isLayoutEditing,
		isWidgetVisible,
		senderLabel,
		senderEmail,
		communicationChannelLabel,
		messageTime
	}: Props = $props();
</script>

<aside class="context-rail">
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-sender-profile" data-widget-hidden={!isWidgetVisible('communications-sender-profile')}>
		<WidgetEditChrome widgetId="communications-sender-profile" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel profile-panel">
			<div class="profile-head"><img src="/assets/hermes-reference-avatar.png" alt="" /><div><h2>{selectedCommunication ? senderLabel(selectedCommunication.sender) : _('No sender selected')}</h2><p>{selectedCommunication ? communicationChannelLabel(selectedCommunication.channel_kind) : _('No channel')}</p><small>{selectedCommunication ? senderEmail(selectedCommunication.sender) : _('No local message selected')}</small></div></div>
			<div class="quick-icons">
				<button type="button" disabled><Icon icon="tabler:mail" width="17" height="17" /></button>
				<button type="button" disabled><Icon icon="tabler:phone" width="17" height="17" /></button>
				<button type="button" disabled><Icon icon="tabler:brand-telegram" width="17" height="17" /></button>
				<button type="button" disabled><Icon icon="tabler:brand-whatsapp" width="17" height="17" /></button>
			</div>
		</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-summary" data-widget-hidden={!isWidgetVisible('communications-summary')}>
		<WidgetEditChrome widgetId="communications-summary" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card">
			<h2>{_('Summary')}</h2>
			<p>{selectedCommunication ? `${_('Stored from')} ${selectedCommunication.account_id}. ${_('Channel')} ${communicationChannelLabel(selectedCommunication.channel_kind)}. ${_('Provider record')} ${selectedCommunication.provider_record_id}.` : _('Local communication metadata will appear after messages are imported.')}</p>
			<button type="button" class="link-row" disabled>{_('View full profile')} <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
			<div class="mail-resource-inline">
				<h3>{_('Mail Resources')}</h3>
				<div class="mail-resource-grid">
					<span>{_('Subscriptions')}</span><strong>{mailResourceSummary.subscriptions}</strong>
					<span>{_('Duplicates')}</span><strong>{mailResourceSummary.duplicates}</strong>
					<span>{_('Invoices')}</span><strong>{mailResourceSummary.invoices}</strong>
					<span>{_('Legal')}</span><strong>{mailResourceSummary.legalDocuments}</strong>
					<span>{_('Certificates')}</span><strong>{mailResourceSummary.certificates}</strong>
					<span>{_('Assets')}</span><strong>{mailResourceSummary.personas + mailResourceSummary.templates}</strong>
					<span>{_('Blockers')}</span><strong>{mailResourceSummary.blockers}</strong>
				</div>
				{#if mailResources.subscriptions.length}
					<div class="mail-resource-list">
						<strong>{_('Subscriptions')}</strong>
						{#each mailResources.subscriptions.slice(0, 2) as subscription}
							<p>{subscription.sender}<em>{subscription.message_count}</em></p>
						{/each}
					</div>
				{/if}
				{#if mailResources.invoices.length || mailResources.legalDocuments.length}
					<div class="mail-resource-list">
						<strong>{_('Finance And Legal')}</strong>
						{#each mailResources.invoices.slice(0, 1) as invoice}
							<p>{invoice.counterparty ?? invoice.invoice_id}<em>{invoice.status}</em></p>
						{/each}
						{#each mailResources.legalDocuments.slice(0, 1) as document}
							<p>{document.title}<em>{document.status}</em></p>
						{/each}
					</div>
				{/if}
				{#if mailResources.blockers.length}
					<div class="mail-resource-list blocker-list">
						<strong>{_('Explicit Blockers')}</strong>
						{#each mailResources.blockers.slice(0, 1) as blocker}
							<p>{blocker.section} · {blocker.feature}</p>
						{/each}
					</div>
				{/if}
			</div>
		</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-message-metadata" data-widget-hidden={!isWidgetVisible('communications-message-metadata')}>
		<WidgetEditChrome widgetId="communications-message-metadata" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card"><h2>{_('Message Metadata')}</h2>{#if selectedCommunication}<ul class="detail-list"><li><Icon icon="tabler:users" width="17" height="17" /> {selectedCommunication.recipients.length} {_('recipients')}</li><li><Icon icon="tabler:paperclip" width="17" height="17" /> {selectedCommunication.attachment_count} {_('attachments')}</li><li><Icon icon="tabler:clock" width="17" height="17" /> {messageTime(selectedCommunication)}</li></ul>{:else}<p>{_('No message selected.')}</p>{/if}</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-related-projects" data-widget-hidden={!isWidgetVisible('communications-related-projects')}>
		<WidgetEditChrome widgetId="communications-related-projects" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card"><h2>{_('Related Projects')}</h2>{#each (projects as unknown[]).slice(0, 2) as project}<div class="related-row"><span class="round-icon {(project as Record<string, unknown>).tone as string}"><Icon icon={(project as Record<string, unknown>).icon as string} width="16" height="16" /></span><strong>{(project as Record<string, unknown>).name as string}</strong><em>{(project as Record<string, unknown>).progress as number}%</em></div>{/each}</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-active-tasks" data-widget-hidden={!isWidgetVisible('communications-active-tasks')}>
		<WidgetEditChrome widgetId="communications-active-tasks" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card"><h2>{_('Active Tasks')}</h2>{#each (tasks as unknown[]).slice(0, 3) as task}<label class="mini-check"><input type="checkbox" />{(task as Record<string, unknown>).title as string}<em>{((task as Record<string, unknown>).due as string).split(' ')[0]}</em></label>{/each}</section>
	</div>
</aside>
