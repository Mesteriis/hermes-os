<script lang="ts">
	import '../communications/communications.css';
	import '../communications/communications-message.css';
	import '../communications/communications-inspector.css';
	import { onMount } from 'svelte';
	import { currentLocale, t } from '$lib/i18n';
	import TelegramActionRail from './widgets/TelegramActionRail.svelte';
	import TelegramChatList from './widgets/TelegramChatList.svelte';
	import TelegramCommandHeader from './widgets/TelegramCommandHeader.svelte';
	import TelegramMessageThread from './widgets/TelegramMessageThread.svelte';
	import TelegramRail from './widgets/TelegramRail.svelte';
	import TelegramStatusMessages from './widgets/TelegramStatusMessages.svelte';
	import * as telegramService from '$lib/services/telegram';
	import * as formattingService from '$lib/services/formatting';
	import { openAccountDrawer } from '$lib/stores/accountWizard';
	import { runNewWorkflowAction } from '$lib/stores/communications';
	import type {
		TelegramChat,
		TelegramMessage,
		TelegramCapabilitiesResponse,
		TelegramRuntimeStatus,
		MessageAnalyzeResponse
	} from '$lib/api';
	import type {
		TelegramChatFilter,
		TelegramChatGroupFilter,
		TelegramAttachmentHint,
		TelegramRailTab,
		TelegramThreadTab
	} from '$lib/services/telegram';

	const _ = (key: string) => t($currentLocale, key);

	interface TelegramManualSendForm {
		text: string;
	}

	let telegramChats = $state<TelegramChat[]>([]);
	let telegramMessages = $state<TelegramMessage[]>([]);
	let telegramCapabilities = $state<TelegramCapabilitiesResponse | null>(null);
	let telegramRuntimeStatuses = $state<Record<string, TelegramRuntimeStatus>>({});
	let selectedTelegramChatId = $state('');
	let telegramError = $state('');
	let telegramActionMessage = $state('');
	let isTelegramLoading = $state(false);
	let isTelegramActionSubmitting = $state(false);
	let isTelegramHistorySyncing = $state(false);
	let telegramSearchQuery = $state('');
	let activeTelegramFilter = $state<TelegramChatFilter>('all');
	let activeTelegramGroupFilter = $state('local:all');
	let activeThreadTab = $state<TelegramThreadTab>('messages');
	let activeRailTab = $state<TelegramRailTab>('context');
	let isTelegramFiltersMenuOpen = $state(false);
	let isTelegramNewMenuOpen = $state(false);
	let isTelegramInspectorOpen = $state(false);
	const telegramAutoHistorySyncKeys = new Set<string>();

	let telegramManualSendForm = $state<TelegramManualSendForm>({
		text: ''
	});

	const selectedTelegramChat = $derived(
		telegramChats.find((chat) => chat.provider_chat_id === selectedTelegramChatId) ??
			telegramChats[0] ??
			null
	);
	const selectedTelegramMessages = $derived(
		selectedTelegramChat
			? telegramService.telegramMessagesChronological(
					telegramMessages.filter(
						(message) => message.provider_chat_id === selectedTelegramChat.provider_chat_id
					)
				)
			: telegramMessages
	);
	const telegramChatFilterCounts = $derived(
		telegramService.telegramChatFilterCounts(telegramChats, telegramMessages)
	);
	const telegramChatGroupFilters = $derived(
		telegramService.telegramChatGroupFilters(telegramChats)
	);
	const filteredTelegramChats = $derived(
		telegramService.filterTelegramChats(
			telegramService.filterTelegramChatsByGroup(telegramChats, activeTelegramGroupFilter),
			telegramMessages,
			telegramSearchQuery,
			activeTelegramFilter
		)
	);
	const selectedTelegramRuntimeStatus = $derived(
		selectedTelegramChat ? telegramRuntimeStatuses[selectedTelegramChat.account_id] ?? null : null
	);
	const isTelegramBusy = $derived(isTelegramActionSubmitting || isTelegramHistorySyncing);
	const telegramRuntimeLabel = $derived(
		selectedTelegramRuntimeStatus?.status ?? telegramCapabilities?.runtime_mode ?? _('Runtime Status')
	);

	const formatDateTime = formattingService.formatDateTime;
	const telegramMessageTime = telegramService.telegramMessageTime;
	const telegramFilterTabs: Array<{ id: TelegramChatFilter; label: string }> = [
		{ id: 'all', label: 'All Chats' },
		{ id: 'unread', label: 'Unread' },
		{ id: 'mentions', label: 'Mentions' },
		{ id: 'pinned', label: 'Pinned' },
		{ id: 'projects', label: 'Projects' },
		{ id: 'bots', label: 'Bots' },
		{ id: 'archived', label: 'Archived' }
	];

	function closeTelegramMenus() {
		isTelegramFiltersMenuOpen = false;
		isTelegramNewMenuOpen = false;
	}

	function toggleTelegramFiltersMenu() {
		isTelegramFiltersMenuOpen = !isTelegramFiltersMenuOpen;
		isTelegramNewMenuOpen = false;
	}

	function toggleTelegramNewMenu() {
		isTelegramNewMenuOpen = !isTelegramNewMenuOpen;
		isTelegramFiltersMenuOpen = false;
	}

	function selectTelegramFilter(filter: TelegramChatFilter) {
		activeTelegramFilter = filter;
		closeTelegramMenus();
	}

	function selectTelegramGroupFilter(filter: TelegramChatGroupFilter) {
		activeTelegramGroupFilter = filter.id;
		closeTelegramMenus();
	}

	function openTelegramInspector(tab: TelegramRailTab = activeRailTab) {
		activeRailTab = tab;
		isTelegramInspectorOpen = true;
	}

	function closeTelegramInspector() {
		isTelegramInspectorOpen = false;
	}

	function openTelegramNewMessage() {
		closeTelegramMenus();
		activeThreadTab = 'messages';
		if (!selectedTelegramChat) {
			telegramActionMessage = _('Select a synced Telegram chat before composing.');
			return;
		}
		telegramManualSendForm = { text: '' };
		telegramActionMessage = `${_('Manual send target')}: ${selectedTelegramChat.title}`;
	}

	async function runTelegramQuickAction(
		action: 'create_note' | 'create_task' | 'create_contact' | 'create_document'
	) {
		closeTelegramMenus();
		await runNewWorkflowAction(action);
	}

	function openTelegramAccountSetup() {
		openAccountDrawer('telegram');
	}

	function toggleTelegramInspector() {
		isTelegramInspectorOpen ? closeTelegramInspector() : openTelegramInspector();
	}

	interface Props {
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		aiAnalysisResult: MessageAnalyzeResponse | null;
		selectedCommunication: unknown;
	}

	let {
		isLayoutEditing,
		isWidgetVisible,
		aiAnalysisResult,
		selectedCommunication
	}: Props = $props();

	onMount(() => {
		void loadTelegramWorkspace();
	});

	async function loadTelegramWorkspace() {
		isTelegramLoading = true;
		const result = await telegramService.loadTelegramWorkspace(selectedTelegramChatId, '');
		telegramCapabilities = result.capabilities;
		telegramChats = result.chats;
		telegramMessages = result.messages;
		telegramRuntimeStatuses = result.runtimeStatuses;
		selectedTelegramChatId = result.selectedChatId;
		telegramError = result.error;
		isTelegramLoading = false;
		if (!telegramService.telegramChatGroupFilters(telegramChats).some((group) => group.id === activeTelegramGroupFilter)) {
			activeTelegramGroupFilter = 'local:all';
		}
		const nextSelectedChat = telegramChats.find((chat) => chat.provider_chat_id === selectedTelegramChatId) ?? null;
		void autoSyncTelegramHistory(nextSelectedChat);
	}

	function selectTelegramChat(chat: TelegramChat) {
		selectedTelegramChatId = chat.provider_chat_id;
		activeThreadTab = 'messages';
		activeRailTab = 'context';
		void loadTelegramWorkspace();
	}

	async function startTelegramRuntime() {
		if (isTelegramBusy || !selectedTelegramChat) return;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		const result = await telegramService.startTelegramRuntimeFromUi(selectedTelegramChat.account_id);
		if (result.error) {
			telegramError = result.error;
		} else if (result.status) {
			telegramRuntimeStatuses = {
				...telegramRuntimeStatuses,
				[result.status.account_id]: result.status
			};
			telegramActionMessage = result.message;
		}
		isTelegramActionSubmitting = false;
	}

	async function syncTelegramChats() {
		if (isTelegramBusy || !selectedTelegramChat) return;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		const result = await telegramService.syncTelegramChatsFromUi(selectedTelegramChat.account_id);
		if (result.error) {
			telegramError = result.error;
		} else {
			telegramActionMessage = result.message;
			await loadTelegramWorkspace();
		}
		isTelegramActionSubmitting = false;
	}

	async function syncSelectedTelegramHistory() {
		if (isTelegramBusy || !selectedTelegramChat) return;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		const result = await telegramService.syncTelegramSelectedHistoryFromUi({
			account_id: selectedTelegramChat.account_id,
			provider_chat_id: selectedTelegramChat.provider_chat_id,
			chat_kind: selectedTelegramChat.chat_kind
		});
		if (result.error) {
			telegramError = result.error;
		} else {
			selectedTelegramChatId = result.providerChatId;
			telegramActionMessage = result.message;
			await loadTelegramWorkspace();
		}
		isTelegramActionSubmitting = false;
	}

	async function autoSyncTelegramHistory(chat: TelegramChat | null) {
		if (!chat || isTelegramBusy) return;
		const syncKey = telegramHistorySyncKey(chat);
		if (telegramAutoHistorySyncKeys.has(syncKey)) return;
		if (chat.chat_kind !== 'private' && hasProjectedMessagesForTelegramChat(chat)) return;

		telegramAutoHistorySyncKeys.add(syncKey);
		isTelegramHistorySyncing = true;
		telegramError = '';
		try {
			const result = await telegramService.syncTelegramSelectedHistoryFromUi({
				account_id: chat.account_id,
				provider_chat_id: chat.provider_chat_id,
				chat_kind: chat.chat_kind
			});
			if (result.error) {
				telegramAutoHistorySyncKeys.delete(syncKey);
				telegramError = result.error;
			} else {
				telegramActionMessage = result.message;
				await loadTelegramWorkspace();
			}
		} finally {
			isTelegramHistorySyncing = false;
		}
	}

	function hasProjectedMessagesForTelegramChat(chat: TelegramChat) {
		return telegramMessages.some(
			(message) =>
				message.account_id === chat.account_id &&
				message.provider_chat_id === chat.provider_chat_id
		);
	}

	function telegramHistorySyncKey(chat: TelegramChat) {
		return `${chat.account_id}:${chat.provider_chat_id}`;
	}

	async function syncOlderTelegramHistory() {
		if (isTelegramBusy || !selectedTelegramChat) return;
		const fromMessageId = telegramService.telegramOldestTdlibMessageId(selectedTelegramMessages);
		if (fromMessageId === null) {
			await autoSyncTelegramHistory(selectedTelegramChat);
			return;
		}

		isTelegramHistorySyncing = true;
		telegramError = '';
		try {
			const result = await telegramService.syncTelegramOlderHistoryFromUi({
				account_id: selectedTelegramChat.account_id,
				provider_chat_id: selectedTelegramChat.provider_chat_id,
				from_message_id: fromMessageId
			});
			if (result.error) {
				telegramError = result.error;
			} else {
				telegramActionMessage = result.hasMore
					? result.message
					: `${result.message}; ${_('no older Telegram messages')}`;
				await loadTelegramWorkspace();
			}
		} finally {
			isTelegramHistorySyncing = false;
		}
	}

	async function sendTelegramManualMessage() {
		if (isTelegramBusy || !selectedTelegramChat) return;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		const result = await telegramService.sendTelegramManualMessageFromUi({
			account_id: selectedTelegramChat.account_id,
			provider_chat_id: selectedTelegramChat.provider_chat_id,
			text: telegramManualSendForm.text
		});
		if (result.error) {
			telegramError = result.error;
		} else {
			selectedTelegramChatId = result.providerChatId;
			telegramActionMessage = result.message;
			telegramManualSendForm = { text: result.nextText };
			await loadTelegramWorkspace();
		}
		isTelegramActionSubmitting = false;
	}

	async function downloadTelegramMedia(attachment: TelegramAttachmentHint, message?: TelegramMessage) {
		if (isTelegramBusy) return;
		if (!selectedTelegramChat) {
			telegramActionMessage = '';
			telegramError = _('Select a Telegram chat before downloading media.');
			return;
		}
		if (attachment.tdlibFileId === null) {
			telegramActionMessage = '';
			telegramError = _('Telegram attachment does not include TDLib file metadata.');
			return;
		}
		const sourceMessage =
			message ??
			selectedTelegramMessages.find((item) => item.message_id === attachment.messageId) ??
			null;
		if (!sourceMessage) {
			telegramActionMessage = '';
			telegramError = _('Telegram source message is not available for this attachment.');
			return;
		}

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		const result = await telegramService.downloadTelegramMediaFromUi({
			account_id: sourceMessage.account_id,
			provider_chat_id: sourceMessage.provider_chat_id || selectedTelegramChat.provider_chat_id,
			provider_message_id: sourceMessage.provider_message_id,
			tdlib_file_id: attachment.tdlibFileId,
			provider_attachment_id: attachment.providerAttachmentId,
			filename: attachment.fileName,
			content_type: attachment.mimeType ?? undefined,
			priority: 16
		});
		if (result.error) {
			telegramError = result.error;
		} else {
			telegramActionMessage = result.message;
			await loadTelegramWorkspace();
		}
		isTelegramActionSubmitting = false;
	}

</script>

<section class="telegram-page communications-page">
	<TelegramCommandHeader
		bind:searchQuery={telegramSearchQuery}
		runtimeLabel={telegramRuntimeLabel}
		filterTabs={telegramFilterTabs}
		filterCounts={telegramChatFilterCounts}
		activeFilter={activeTelegramFilter}
		isFiltersMenuOpen={isTelegramFiltersMenuOpen}
		isNewMenuOpen={isTelegramNewMenuOpen}
		{isTelegramBusy}
		{selectedTelegramChat}
		onToggleFiltersMenu={toggleTelegramFiltersMenu}
		onToggleNewMenu={toggleTelegramNewMenu}
		onSelectFilter={selectTelegramFilter}
		onSyncChats={() => { closeTelegramMenus(); void syncTelegramChats(); }}
		onAddAccount={openTelegramAccountSetup}
		onNewMessage={openTelegramNewMessage}
		onQuickAction={(action) => void runTelegramQuickAction(action)}
	/>

	<TelegramActionRail
		groupFilters={telegramChatGroupFilters}
		activeGroupFilter={activeTelegramGroupFilter}
		{isTelegramBusy}
		hasSelectedTelegramChat={Boolean(selectedTelegramChat)}
		isInspectorOpen={isTelegramInspectorOpen}
		onSyncChats={() => void syncTelegramChats()}
		onSyncHistory={() => void syncSelectedTelegramHistory()}
		onStartRuntime={() => void startTelegramRuntime()}
		onSelectGroupFilter={selectTelegramGroupFilter}
		onToggleInspector={toggleTelegramInspector}
	/>

	<TelegramStatusMessages actionMessage={telegramActionMessage} error={telegramError} />

	<div class="three-pane communications-grid telegram-grid" class:inspector-open={isTelegramInspectorOpen}>
		<TelegramChatList
			telegramChats={filteredTelegramChats}
			telegramMessages={telegramMessages}
			selectedTelegramChatId={selectedTelegramChat?.provider_chat_id ?? ''}
			{isTelegramLoading}
			{isLayoutEditing}
			{isWidgetVisible}
			onSelectChat={selectTelegramChat}
			{formatDateTime}
		/>
		<TelegramMessageThread
			selectedTelegramChat={selectedTelegramChat}
			selectedTelegramMessages={selectedTelegramMessages}
			{aiAnalysisResult}
			selectedCommunication={selectedCommunication as { message_id?: string } | null}
			{isTelegramLoading}
			isTelegramActionSubmitting={isTelegramBusy}
			{isLayoutEditing}
			{isWidgetVisible}
			{activeThreadTab}
			onActiveThreadTabChange={(tab) => (activeThreadTab = tab)}
			onRailTabChange={(tab) => openTelegramInspector(tab)}
			telegramMessageTime={telegramMessageTime}
			{loadTelegramWorkspace}
			{syncSelectedTelegramHistory}
			{syncOlderTelegramHistory}
			{sendTelegramManualMessage}
			{downloadTelegramMedia}
			{telegramManualSendForm}
			{selectedTelegramRuntimeStatus}
		/>
		{#if isTelegramInspectorOpen}
			<TelegramRail
				{selectedTelegramChat}
				{activeRailTab}
				{isLayoutEditing}
				{isWidgetVisible}
				onActiveRailTabChange={(tab) => (activeRailTab = tab)}
				onClose={closeTelegramInspector}
			/>
		{/if}
	</div>

</section>
