<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import CommunicationsConversationList from './widgets/CommunicationsConversationList.svelte';
	import CommunicationsMessageDetail from './widgets/CommunicationsMessageDetail.svelte';
	import CommunicationsContextInspector from './widgets/CommunicationsContextInspector.svelte';
	import type {
		CommunicationMessageDetailItem,
		CommunicationMessageSummary,
		WorkflowState
	} from '$lib/api';
	import {
		aiAnalysisResult,
		askAiAboutSelectedMessage,
		attachmentIcon,
		communicationChannelLabel,
		communicationMessages,
		communicationProjects,
		communicationTasks,
		communicationsInspectorMode,
		communicationsNavigatorMode,
		communicationsError,
		expandedCommunicationContactKey,
		addLabelToSelectedMessage,
		activeMessageContextTab,
		exportSelectedMessage,
		extractNotesForSelectedMessage,
		extractTasksForSelectedMessage,
		generateReplyForSelectedMessage,
		handleWorkflowStateTransition,
		isAiAnswerSubmitting,
		isCommunicationsLoading,
		isMailActionRunning,
		isMailStateTransitioning,
		loadCommunicationMessagesFiltered,
		mailAccountOptions,
		mailActionError,
		mailActionStatus,
		mailLocalStateFilter,
		mailMessageInsight,
		mailResources,
		mailResourceSummary,
		mailSyncError,
		mailSyncStatusMessage,
		mailSyncStatuses,
		mailStateCounts,
		mailStateFilter,
		messageContentHtml,
		messageContentText,
		messageSearchQuery,
		messageTime,
		openForwardSelected,
		openNewMessage,
		openReplyToSelected,
		runNewWorkflowAction,
		runSelectedWorkflowAction,
		selectCommunication,
		selectCommunicationSection,
		selectMailLocalState,
		selectMailAccount,
		selectedCommunication,
		selectedCommunicationDetail,
		selectedConversationIndex,
		selectedMailSyncSettings,
		selectedMailAccountId,
		senderEmail,
		senderLabel,
		snoozeSelectedMessage,
		runMailFullResync,
		runMailSyncNow,
		restoreSelectedMessage,
		toggleImportantSelectedMessage,
		toggleSelectedReadState,
		trashSelectedMessage,
		toggleMuteSelectedMessage,
		togglePinSelectedMessage,
		translateSelectedMessage,
		updateSelectedMailSyncSettings,
		updateMessageSearchQuery
	} from '$lib/stores/communications';
	import { formatBytes } from '$lib/services/formatting';
	import { relatedMessagesForSelection } from '$lib/services/communications';

	const _ = (key: string) => t($currentLocale, key);

	type ProjectItem = {
		name: string;
		kind: string;
		progress: number;
		icon: string;
		tone: string;
	};

	type TaskItem = {
		title: string;
		due: string;
	};

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
	}

	let {
		isLayoutEditing,
		isWidgetVisible
	}: Props = $props();

	const formatMessageTime = (message: unknown) =>
		messageTime(message as CommunicationMessageSummary | CommunicationMessageDetailItem);
	const syncStatusForAccount = (accountId: string) => $mailSyncStatuses.find((status) => status.account_id === accountId);
	const isSyncActive = (status: { status: string } | undefined) =>
		Boolean(status && ['queued', 'running', 'recoverable_full_resync_needed'].includes(status.status));
	const syncProgressPercent = (status: { status: string; progress_mode: string; progress_percent: number | null; processed_messages: number; estimated_total_messages: number | null } | undefined) => {
		if (!status) return 0;
		if (status.progress_mode === 'determinate' && status.progress_percent != null) return status.progress_percent;
		if (status.estimated_total_messages && status.estimated_total_messages > 0) {
			return Math.min(100, Math.round((status.processed_messages / status.estimated_total_messages) * 100));
		}
		return isSyncActive(status) ? 38 : 0;
	};
	const aggregateSyncActive = () => $mailSyncStatuses.some((status) => isSyncActive(status));
	const aggregateSyncPercent = () => {
		const active = $mailSyncStatuses.filter((status) => isSyncActive(status));
		if (!active.length) return 0;
		const determinate = active.filter((status) => status.progress_mode === 'determinate' && status.progress_percent != null);
		if (!determinate.length) return 38;
		return Math.round(determinate.reduce((sum, status) => sum + (status.progress_percent ?? 0), 0) / determinate.length);
	};
	const updateBatchSize = (value: string) => {
		const parsed = Number.parseInt(value, 10);
		if (Number.isFinite(parsed)) void updateSelectedMailSyncSettings({ batch_size: parsed });
	};
	const updatePollInterval = (value: string) => {
		const parsed = Number.parseInt(value, 10);
		if (Number.isFinite(parsed)) void updateSelectedMailSyncSettings({ poll_interval_seconds: parsed });
	};
	const selectedRelatedMessages = $derived(
		relatedMessagesForSelection($communicationMessages, $selectedCommunication, 8)
	);
	const selectRelatedCommunication = (messageId: string) => {
		const index = $communicationMessages.findIndex((message) => message.message_id === messageId);
		if (index >= 0) {
			selectCommunication(index);
		}
	};

	let isAccountsMenuOpen = $state(false);
	let isFiltersMenuOpen = $state(false);
	let isNewMenuOpen = $state(false);

	const toggleAccountsMenu = () => {
		isAccountsMenuOpen = !isAccountsMenuOpen;
		isFiltersMenuOpen = false;
		isNewMenuOpen = false;
	};
	const toggleFiltersMenu = () => {
		isFiltersMenuOpen = !isFiltersMenuOpen;
		isAccountsMenuOpen = false;
		isNewMenuOpen = false;
	};
	const toggleNewMenu = () => {
		isNewMenuOpen = !isNewMenuOpen;
		isAccountsMenuOpen = false;
		isFiltersMenuOpen = false;
	};
	const closeMenus = () => {
		isAccountsMenuOpen = false;
		isFiltersMenuOpen = false;
		isNewMenuOpen = false;
	};
</script>

<section class="communications-page">
	<header class="communications-command-header">
		<div class="command-title">
			<h1>{_('Communications')}</h1>
			<p>{_('Mail')}</p>
		</div>
		<label class="command-search">
			<Icon icon="tabler:search" width="18" height="18" />
			<input value={$messageSearchQuery} oninput={(event) => void updateMessageSearchQuery((event.currentTarget as HTMLInputElement).value)} placeholder={_('Search conversations...')} />
		</label>
		<div class="command-menu">
			<button type="button" class:active={$selectedMailAccountId !== '' || aggregateSyncActive()} onclick={toggleAccountsMenu}>
				<Icon icon="tabler:mail-cog" width="17" height="17" />{_('Accounts')}
			</button>
			{#if isAccountsMenuOpen}
				<div class="command-popover account-command-popover">
					<button type="button" class:active={$selectedMailAccountId === ''} onclick={() => { closeMenus(); void selectMailAccount(''); }}>
						<span>{_('All Accounts')}</span><em>{aggregateSyncActive() ? `${aggregateSyncPercent()}%` : _('Unified')}</em>
					</button>
					{#each $mailAccountOptions as account}
						{@const syncStatus = syncStatusForAccount(account.accountId)}
						<button type="button" class:active={$selectedMailAccountId === account.accountId} onclick={() => { closeMenus(); void selectMailAccount(account.accountId); }}>
							<span><Icon icon={account.providerKind === 'gmail' ? 'tabler:brand-gmail' : 'tabler:mail'} width="15" height="15" />{account.label}</span>
							<em>{isSyncActive(syncStatus) ? `${syncProgressPercent(syncStatus)}%` : account.canSend ? account.providerKind : _('Read only')}</em>
						</button>
					{/each}
					<button type="button" onclick={() => { closeMenus(); void runMailSyncNow($selectedMailAccountId || undefined); }}>
						<span><Icon icon="tabler:refresh" width="15" height="15" />{_('Check now')}</span>
					</button>
					{#if $selectedMailAccountId}
						<button type="button" onclick={() => { closeMenus(); void runMailFullResync($selectedMailAccountId); }}>
							<span><Icon icon="tabler:database-import" width="15" height="15" />{_('Full resync')}</span>
							<em>{_('Read mailbox again')}</em>
						</button>
					{/if}
				</div>
			{/if}
		</div>
		<div class="command-menu">
			<button type="button" class:active={$mailLocalStateFilter === 'trash'} onclick={toggleFiltersMenu}>
				<Icon icon="tabler:filter" width="17" height="17" />{_('Filters')}
			</button>
			{#if isFiltersMenuOpen}
				<div class="command-popover filter-command-popover">
					<button type="button" class:active={$mailLocalStateFilter === 'active'} onclick={() => { closeMenus(); void selectMailLocalState('active'); }}>{_('Active')}</button>
					<button type="button" class:active={$mailLocalStateFilter === 'trash'} onclick={() => { closeMenus(); void selectMailLocalState('trash'); }}><Icon icon="tabler:trash" width="15" height="15" />{_('Trash')}</button>
					{#if $selectedMailAccountId && $selectedMailSyncSettings}
						<div class="sync-settings-compact">
							<label><input type="checkbox" checked={$selectedMailSyncSettings.sync_enabled} onchange={(event) => void updateSelectedMailSyncSettings({ sync_enabled: (event.currentTarget as HTMLInputElement).checked })} />{_('Sync enabled')}</label>
							<label><span>{_('Batch size')}</span><input type="number" min="1" max="500" value={$selectedMailSyncSettings.batch_size} onchange={(event) => updateBatchSize((event.currentTarget as HTMLInputElement).value)} /></label>
							<label><span>{_('Interval seconds')}</span><input type="number" min="60" max="86400" value={$selectedMailSyncSettings.poll_interval_seconds} onchange={(event) => updatePollInterval((event.currentTarget as HTMLInputElement).value)} /></label>
						</div>
					{/if}
				</div>
			{/if}
		</div>
		<div class="command-menu new-command">
			<button type="button" class="primary-button" onclick={toggleNewMenu}>{_('New')}<Icon icon="tabler:plus" width="17" height="17" /></button>
			{#if isNewMenuOpen}
				<div class="command-popover new-command-popover">
					<button type="button" onclick={() => { closeMenus(); openNewMessage(); }}><Icon icon="tabler:mail-plus" width="16" height="16" />{_('New Message')}</button>
					<button type="button" onclick={() => { closeMenus(); void runNewWorkflowAction('create_note'); }}><Icon icon="tabler:notes" width="16" height="16" />{_('New Note')}</button>
					<button type="button" onclick={() => { closeMenus(); void runNewWorkflowAction('create_task'); }}><Icon icon="tabler:square-check" width="16" height="16" />{_('New Task')}</button>
					<button type="button" onclick={() => { closeMenus(); void runNewWorkflowAction('create_contact'); }}><Icon icon="tabler:user-plus" width="16" height="16" />{_('New Contact')}</button>
					<button type="button" onclick={() => { closeMenus(); void runNewWorkflowAction('create_document'); }}><Icon icon="tabler:file-plus" width="16" height="16" />{_('New Document')}</button>
				</div>
			{/if}
		</div>
	</header>
	{#if $mailSyncStatusMessage || $mailSyncError}
		<div class="mail-sync-status-line" class:error={Boolean($mailSyncError)}>{_($mailSyncError || $mailSyncStatusMessage)}</div>
	{/if}
	<nav class="conversation-state-tabs">
		<button type="button" class:active={$mailLocalStateFilter === 'active' && $mailStateFilter === ''} onclick={() => void selectCommunicationSection('unified')}>{_('Unified')} <em>{$communicationMessages.length}</em></button>
		<button type="button" class:active={$mailLocalStateFilter === 'active' && $mailStateFilter === 'needs_action'} onclick={() => void selectCommunicationSection('needs_reply')}>{_('Need Reply')} <em>{$mailStateCounts.find(c => c.state === 'needs_action')?.count ?? 0}</em></button>
		<button type="button" class:active={$mailLocalStateFilter === 'active' && $mailStateFilter === 'waiting'} onclick={() => void selectCommunicationSection('waiting')}>{_('Waiting')} <em>{$mailStateCounts.find(c => c.state === 'waiting')?.count ?? 0}</em></button>
		<button type="button" class:active={$mailLocalStateFilter === 'active' && $mailStateFilter === 'new'} onclick={() => void selectCommunicationSection('inbox')}>{_('Inbox')} <em>{$mailStateCounts.find(c => c.state === 'new')?.count ?? 0}</em></button>
		<button type="button" class:active={$mailLocalStateFilter === 'active' && $mailStateFilter === 'done'} onclick={() => { mailLocalStateFilter.set('active'); mailStateFilter.set('done'); void loadCommunicationMessagesFiltered('done'); }}>{_('Done')} <em>{$mailStateCounts.find(c => c.state === 'done')?.count ?? 0}</em></button>
		<button type="button" class:active={$mailLocalStateFilter === 'active' && $mailStateFilter === 'archived'} onclick={() => { mailLocalStateFilter.set('active'); mailStateFilter.set('archived'); void loadCommunicationMessagesFiltered('archived'); }}>{_('Archived')} <em>{$mailStateCounts.find(c => c.state === 'archived')?.count ?? 0}</em></button>
	</nav>
	<div class="three-pane communications-grid" class:inspector-open={Boolean($communicationsInspectorMode)}>
		<CommunicationsConversationList
				communicationMessages={$communicationMessages}
				isCommunicationsLoading={$isCommunicationsLoading}
				communicationsError={$communicationsError}
				selectedConversationIndex={$selectedConversationIndex}
				selectedCommunication={$selectedCommunication}
				navigatorMode={$communicationsNavigatorMode}
				expandedContactKey={$expandedCommunicationContactKey}
			{isLayoutEditing}
			{isWidgetVisible}
			{selectCommunication}
				onNavigatorModeChange={(mode) => { communicationsNavigatorMode.set(mode); }}
				onExpandedContactKeyChange={(key) => { expandedCommunicationContactKey.set(key); }}
			{senderLabel}
				messageTime={formatMessageTime}
		/>
		<CommunicationsMessageDetail
				selectedCommunication={$selectedCommunication}
				selectedCommunicationDetail={$selectedCommunicationDetail}
				aiAnalysisResult={$aiAnalysisResult}
				mailMessageInsight={$mailMessageInsight}
				isMailActionRunning={$isMailActionRunning}
				mailActionStatus={$mailActionStatus}
				mailActionError={$mailActionError}
				isMailStateTransitioning={$isMailStateTransitioning}
				isAiAnswerSubmitting={$isAiAnswerSubmitting}
			{isLayoutEditing}
			{isWidgetVisible}
				handleWorkflowStateTransition={handleWorkflowStateTransition}
			{askAiAboutSelectedMessage}
				onReply={openReplyToSelected}
				onForward={openForwardSelected}
				onPin={() => void togglePinSelectedMessage()}
				onImportant={() => void toggleImportantSelectedMessage()}
				onMute={() => void toggleMuteSelectedMessage()}
				onTrash={() => void trashSelectedMessage()}
				onRestore={() => void restoreSelectedMessage()}
				onSnooze={() => void snoozeSelectedMessage()}
				onAddLabel={(label: string) => void addLabelToSelectedMessage(label)}
				onExport={(format: 'md' | 'eml' | 'json') => void exportSelectedMessage(format)}
				onGenerateAiReply={() => void generateReplyForSelectedMessage()}
				onExtractTasks={() => void extractTasksForSelectedMessage()}
				onExtractNotes={() => void extractNotesForSelectedMessage()}
				onTranslate={() => void translateSelectedMessage('en')}
				onWorkflowAction={(action) => void runSelectedWorkflowAction(action)}
				onToggleReadState={() => void toggleSelectedReadState()}
				onOpenInspector={(mode) => { communicationsInspectorMode.set(mode); }}
				onSelectRelatedMessage={selectRelatedCommunication}
				activeTab={$activeMessageContextTab}
				onActiveTabChange={(tab) => { activeMessageContextTab.set(tab); }}
				messageTime={formatMessageTime}
				{messageContentHtml}
				{messageContentText}
				relatedMessages={selectedRelatedMessages}
			{senderLabel}
			{attachmentIcon}
			{formatBytes}
		/>
		{#if $communicationsInspectorMode}
			<CommunicationsContextInspector
				mode={$communicationsInspectorMode}
				selectedCommunication={$selectedCommunication}
				selectedCommunicationDetail={$selectedCommunicationDetail}
				mailResources={$mailResources}
				mailResourceSummary={$mailResourceSummary}
				projects={$communicationProjects as unknown[]}
				tasks={$communicationTasks as unknown[]}
				{isLayoutEditing}
				{isWidgetVisible}
				{senderLabel}
				{senderEmail}
				{communicationChannelLabel}
				messageTime={formatMessageTime}
				onModeChange={(mode) => { communicationsInspectorMode.set(mode); }}
				onClose={() => { communicationsInspectorMode.set(null); }}
			/>
		{/if}
	</div>

</section>
