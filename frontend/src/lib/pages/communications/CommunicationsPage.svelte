<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import CommunicationsConversationList from './widgets/CommunicationsConversationList.svelte';
	import CommunicationsMessageDetail from './widgets/CommunicationsMessageDetail.svelte';
	import CommunicationsContextRail from './widgets/CommunicationsContextRail.svelte';
	import type {
		CommunicationMessageDetailItem,
		CommunicationMessageSummary,
		WorkflowState
	} from '$lib/api';
	import {
		aiAnalysisResult,
		askAiAboutSelectedMessage,
		attachmentIcon,
		communicationChannelIcon,
		communicationChannelLabel,
		communicationMessages,
		communicationProjects,
		communicationTasks,
		communicationsError,
		handleWorkflowStateTransition,
		isAiAnswerSubmitting,
		isCommunicationsLoading,
		isMailStateTransitioning,
		loadCommunicationMessagesFiltered,
		mailStateCounts,
		mailStateFilter,
		messageTime,
		selectCommunication,
		selectCommunicationSection,
		selectedCommunication,
		selectedCommunicationDetail,
		selectedConversationIndex,
		senderEmail,
		senderLabel
	} from '$lib/stores/communications';
	import { formatBytes } from '$lib/services/formatting';

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
</script>

<section class="communications-page">
	<div class="view-header">
		<div>
			<h1>{_('Communications')}</h1>
			<p>{_('All local message records')}</p>
		</div>
		<div class="header-actions">
			<button type="button" class="segmented active"><Icon icon="tabler:message" width="16" height="16" /></button>
			<button type="button" class="segmented" disabled><Icon icon="tabler:layout-grid" width="16" height="16" /></button>
			<button type="button" class="primary-button">{_('New Message')}</button>
		</div>
	</div>
	<div class="filter-tabs">
			<button type="button" class:active={$mailStateFilter === ''} onclick={() => void selectCommunicationSection('unified')}>{_('Unified')} <em>{$communicationMessages.length}</em></button>
			<button type="button" class:active={$mailStateFilter === 'needs_action'} onclick={() => void selectCommunicationSection('needs_reply')}>{_('Needs Reply')} <em>{$mailStateCounts.find(c => c.state === 'needs_action')?.count ?? 0}</em></button>
			<button type="button" class:active={$mailStateFilter === 'waiting'} onclick={() => void selectCommunicationSection('waiting')}>{_('Waiting')} <em>{$mailStateCounts.find(c => c.state === 'waiting')?.count ?? 0}</em></button>
			<button type="button" class:active={$mailStateFilter === 'new'} onclick={() => void selectCommunicationSection('inbox')}>{_('Inbox')} <em>{$mailStateCounts.find(c => c.state === 'new')?.count ?? 0}</em></button>
			<button type="button" class:active={$mailStateFilter === 'done'} onclick={() => { mailStateFilter.set('done'); void loadCommunicationMessagesFiltered('done'); }}>{_('Done')} <em>{$mailStateCounts.find(c => c.state === 'done')?.count ?? 0}</em></button>
			<button type="button" class:active={$mailStateFilter === 'archived'} onclick={() => { mailStateFilter.set('archived'); void loadCommunicationMessagesFiltered('archived'); }}>{_('Archived')} <em>{$mailStateCounts.find(c => c.state === 'archived')?.count ?? 0}</em></button>
	</div>
	<div class="three-pane communications-grid">
		<CommunicationsConversationList
				communicationMessages={$communicationMessages as unknown[]}
				isCommunicationsLoading={$isCommunicationsLoading}
				communicationsError={$communicationsError}
				selectedConversationIndex={$selectedConversationIndex}
				selectedCommunication={$selectedCommunication as unknown | null}
			{isLayoutEditing}
			{isWidgetVisible}
			{selectCommunication}
			{communicationChannelIcon}
			{senderLabel}
				messageTime={formatMessageTime}
		/>
		<CommunicationsMessageDetail
				selectedCommunication={$selectedCommunication as unknown | null}
				selectedCommunicationDetail={$selectedCommunicationDetail}
				aiAnalysisResult={$aiAnalysisResult}
				isMailStateTransitioning={$isMailStateTransitioning}
				isAiAnswerSubmitting={$isAiAnswerSubmitting}
			{isLayoutEditing}
			{isWidgetVisible}
				handleWorkflowStateTransition={handleWorkflowStateTransition as unknown as (msgId: string, state: string) => void}
			{askAiAboutSelectedMessage}
				messageTime={formatMessageTime}
			{senderLabel}
			{attachmentIcon}
			{formatBytes}
		/>
		<CommunicationsContextRail
				selectedCommunication={$selectedCommunication as unknown | null}
				projects={$communicationProjects as unknown[]}
				tasks={$communicationTasks as unknown[]}
			{isLayoutEditing}
			{isWidgetVisible}
			{senderLabel}
			{senderEmail}
			{communicationChannelLabel}
				messageTime={formatMessageTime}
		/>
	</div>

</section>
