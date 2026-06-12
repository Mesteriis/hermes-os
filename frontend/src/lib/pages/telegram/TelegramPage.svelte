<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import TelegramChatList from './widgets/TelegramChatList.svelte';
	import TelegramMessageThread from './widgets/TelegramMessageThread.svelte';
	import TelegramRail from './widgets/TelegramRail.svelte';
	import * as telegramService from '$lib/services/telegram';
	import * as formattingService from '$lib/services/formatting';
	import { openAccountDrawer } from '$lib/stores/accountWizard';
	import { runNewWorkflowAction } from '$lib/stores/communications';
	import { telegramProviderAccounts } from '$lib/stores/settings';
	import type {
		TelegramChat,
		TelegramMessage,
		TelegramCapabilitiesResponse,
		TelegramCapabilityStatus,
		TelegramCall,
		AutomationTemplate,
		AutomationPolicy,
		TelegramSendDryRunResponse,
		TelegramRuntimeStatus,
		CallTranscript,
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

	interface AutomationTemplateForm {
		template_id: string;
		name: string;
		body_template: string;
		required_variables_text: string;
	}

	interface AutomationPolicyForm {
		policy_id: string;
		template_id: string;
		name: string;
		enabled: boolean;
		account_id: string;
		allowed_chat_ids_text: string;
		trigger_kind: string;
		max_sends_per_hour: number;
		quiet_hours_text: string;
		expires_at: string;
		conditions_text: string;
	}

	interface TelegramSendForm {
		policy_id: string;
		provider_chat_id: string;
		variables_text: string;
		source_context_text: string;
	}

	interface TelegramCallForm {
		call_id: string;
		account_id: string;
		provider_call_id: string;
		provider_chat_id: string;
		direction: 'incoming' | 'outgoing';
		call_state: 'ringing' | 'active' | 'ended' | 'missed' | 'declined' | 'failed';
		started_at: string;
		ended_at: string;
		transcription_policy_id: string;
		metadata_text: string;
	}

	interface TranscriptForm {
		transcript_id: string;
		account_id: string;
		provider_chat_id: string;
		source_audio_ref: string;
		language_code: string;
		always_on_policy: boolean;
	}

	let telegramChats = $state<TelegramChat[]>([]);
	let telegramMessages = $state<TelegramMessage[]>([]);
	let automationTemplates = $state<AutomationTemplate[]>([]);
	let automationPolicies = $state<AutomationPolicy[]>([]);
	let telegramCalls = $state<TelegramCall[]>([]);
	let telegramCapabilities = $state<TelegramCapabilitiesResponse | null>(null);
	let telegramRuntimeStatuses = $state<Record<string, TelegramRuntimeStatus>>({});
	let selectedTelegramChatId = $state('');
	let selectedTelegramCallId = $state('');
	let callTranscript = $state<CallTranscript | null>(null);
	let telegramError = $state('');
	let telegramActionMessage = $state('');
	let isTelegramLoading = $state(false);
	let isTelegramActionSubmitting = $state(false);
	let isTelegramHistorySyncing = $state(false);
	let telegramSendDryRunResult = $state<TelegramSendDryRunResponse | null>(null);
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

	let automationTemplateForm = $state<AutomationTemplateForm>({
		template_id: 'template-telegram-followup',
		name: 'Telegram Follow-up',
		body_template: 'Hi {{name}}, I will follow up about {{topic}}.',
		required_variables_text: 'name, topic'
	});

	let automationPolicyForm = $state<AutomationPolicyForm>({
		policy_id: 'policy-telegram-followup',
		template_id: 'template-telegram-followup',
		name: 'Telegram follow-up allowlist',
		enabled: true,
		account_id: 'telegram-primary',
		allowed_chat_ids_text: 'fixture-chat-1',
		trigger_kind: 'manual_dry_run',
		max_sends_per_hour: 3,
		quiet_hours_text: '{}',
		expires_at: '',
		conditions_text: '{}'
	});

	let telegramSendForm = $state<TelegramSendForm>({
		policy_id: 'policy-telegram-followup',
		provider_chat_id: 'fixture-chat-1',
		variables_text: '{ "name": "Maria", "topic": "Telegram client" }',
		source_context_text: '{ "source": "desktop_ui_fixture" }'
	});

	let telegramCallForm = $state<TelegramCallForm>({
		call_id: 'call-telegram-fixture-1',
		account_id: 'telegram-primary',
		provider_call_id: 'provider-call-telegram-fixture-1',
		provider_chat_id: 'fixture-chat-1',
		direction: 'incoming',
		call_state: 'ended',
		started_at: new Date().toISOString(),
		ended_at: '',
		transcription_policy_id: '',
		metadata_text: '{ "runtime": "fixture", "visible_recording_state": true }'
	});

	let transcriptForm = $state<TranscriptForm>({
		transcript_id: 'transcript-telegram-fixture-1',
		account_id: 'telegram-primary',
		provider_chat_id: 'fixture-chat-1',
		source_audio_ref: 'docker/data/calls/fixture-call.wav',
		language_code: 'en',
		always_on_policy: true
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
	const selectedTelegramCall = $derived(
		telegramCalls.find((call) => call.call_id === selectedTelegramCallId) ?? telegramCalls[0] ?? null
	);
	const selectedTelegramRuntimeStatus = $derived(
		selectedTelegramChat ? telegramRuntimeStatuses[selectedTelegramChat.account_id] ?? null : null
	);
	const isTelegramBusy = $derived(isTelegramActionSubmitting || isTelegramHistorySyncing);
	const telegramClosureCapabilities = $derived(
		telegramCapabilities?.capabilities.filter((capability) => capability.closure_gate) ?? []
	);
	const telegramBlockedCapabilities = $derived(
		telegramCapabilities?.capabilities.filter((capability) => capability.status === 'blocked') ?? []
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

	function capabilityLabel(capability: string) {
		return capability
			.split('_')
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function openAccountWizard(target: string) {
		openAccountDrawer(target as Parameters<typeof openAccountDrawer>[0]);
	}

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

	function telegramFilterCount(filter: TelegramChatFilter) {
		return telegramChatFilterCounts.find((item) => item.filter === filter)?.count ?? 0;
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
		const result = await telegramService.loadTelegramWorkspace(selectedTelegramChatId, selectedTelegramCallId);
		telegramCapabilities = result.capabilities;
		telegramChats = result.chats;
		telegramMessages = result.messages;
		telegramRuntimeStatuses = result.runtimeStatuses;
		automationTemplates = result.templates;
		automationPolicies = result.policies;
		telegramCalls = result.calls;
		selectedTelegramChatId = result.selectedChatId;
		selectedTelegramCallId = result.selectedCallId;
		callTranscript = result.transcript;
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
		automationPolicyForm = {
			...automationPolicyForm,
			account_id: chat.account_id,
			allowed_chat_ids_text: chat.provider_chat_id
		};
		telegramSendForm = {
			...telegramSendForm,
			provider_chat_id: chat.provider_chat_id
		};
		telegramCallForm = {
			...telegramCallForm,
			account_id: chat.account_id,
			provider_chat_id: chat.provider_chat_id
		};
		transcriptForm = {
			...transcriptForm,
			account_id: chat.account_id,
			provider_chat_id: chat.provider_chat_id
		};
		void loadTelegramWorkspace();
	}

	function selectTelegramCall(call: TelegramCall) {
		selectedTelegramCallId = call.call_id;
		telegramCallForm = {
			...telegramCallForm,
			call_id: call.call_id,
			account_id: call.account_id,
			provider_call_id: call.provider_call_id,
			provider_chat_id: call.provider_chat_id,
			direction: call.direction,
			call_state: call.call_state,
			started_at: call.started_at ?? '',
			ended_at: call.ended_at ?? '',
			transcription_policy_id: call.transcription_policy_id ?? '',
			metadata_text: JSON.stringify(call.metadata, null, 2)
		};
		transcriptForm = {
			...transcriptForm,
			account_id: call.account_id,
			provider_chat_id: call.provider_chat_id
		};
		void loadSelectedCallTranscript(call.call_id);
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

	async function loadSelectedCallTranscript(callId = selectedTelegramCallId) {
		const result = await telegramService.loadSelectedCallTranscript(callId);
		callTranscript = result.transcript;
		if (result.error) telegramError = result.error;
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

	async function saveTelegramAutomationTemplate() {
		if (isTelegramBusy) return;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		const result = await telegramService.saveTelegramAutomationTemplate({
			template_id: automationTemplateForm.template_id,
			name: automationTemplateForm.name,
			body_template: automationTemplateForm.body_template,
			required_variables_text: automationTemplateForm.required_variables_text
		});
		if (result.error) {
			telegramError = result.error;
		} else {
			telegramActionMessage = result.message;
			automationPolicyForm = {
				...automationPolicyForm,
				template_id: result.templateId
			};
			await loadTelegramWorkspace();
		}
		isTelegramActionSubmitting = false;
	}

	async function saveTelegramAutomationPolicy() {
		if (isTelegramBusy) return;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		const result = await telegramService.saveTelegramAutomationPolicy({
			policy_id: automationPolicyForm.policy_id,
			template_id: automationPolicyForm.template_id,
			name: automationPolicyForm.name,
			enabled: automationPolicyForm.enabled,
			account_id: automationPolicyForm.account_id,
			allowed_chat_ids_text: automationPolicyForm.allowed_chat_ids_text,
			trigger_kind: automationPolicyForm.trigger_kind,
			max_sends_per_hour: automationPolicyForm.max_sends_per_hour,
			quiet_hours_text: automationPolicyForm.quiet_hours_text,
			expires_at: automationPolicyForm.expires_at,
			conditions_text: automationPolicyForm.conditions_text
		});
		if (result.error) {
			telegramError = result.error;
		} else {
			telegramActionMessage = result.message;
			telegramSendForm = {
				...telegramSendForm,
				policy_id: result.policyId
			};
			await loadTelegramWorkspace();
		}
		isTelegramActionSubmitting = false;
	}

	async function runTelegramAutomationDryRun() {
		if (isTelegramBusy) return;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		telegramSendDryRunResult = null;
		const result = await telegramService.runTelegramAutomationDryRun({
			policy_id: telegramSendForm.policy_id,
			provider_chat_id: telegramSendForm.provider_chat_id,
			variables_text: telegramSendForm.variables_text,
			source_context_text: telegramSendForm.source_context_text
		});
		if (result.error) {
			telegramError = result.error;
		} else {
			telegramSendDryRunResult = result.result;
			telegramActionMessage = result.message;
			await loadTelegramWorkspace();
		}
		isTelegramActionSubmitting = false;
	}

	async function saveTelegramCallFixture() {
		if (isTelegramBusy) return;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		const result = await telegramService.saveTelegramCallFixture({
			call_id: telegramCallForm.call_id,
			account_id: telegramCallForm.account_id,
			provider_call_id: telegramCallForm.provider_call_id,
			provider_chat_id: telegramCallForm.provider_chat_id,
			direction: telegramCallForm.direction,
			call_state: telegramCallForm.call_state,
			started_at: telegramCallForm.started_at,
			ended_at: telegramCallForm.ended_at,
			transcription_policy_id: telegramCallForm.transcription_policy_id,
			metadata_text: telegramCallForm.metadata_text
		});
		if (result.error) {
			telegramError = result.error;
		} else {
			selectedTelegramCallId = result.callId;
			telegramActionMessage = result.message;
			await loadTelegramWorkspace();
		}
		isTelegramActionSubmitting = false;
	}

	async function saveCallTranscriptFixtureFromUi() {
		if (isTelegramBusy || !selectedTelegramCallId) return;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		const result = await telegramService.saveCallTranscriptFixtureFromUi({
			transcript_id: transcriptForm.transcript_id,
			account_id: transcriptForm.account_id,
			provider_chat_id: transcriptForm.provider_chat_id,
			source_audio_ref: transcriptForm.source_audio_ref,
			language_code: transcriptForm.language_code,
			always_on_policy: transcriptForm.always_on_policy,
			selectedCallId: selectedTelegramCallId
		});
		if (result.error) {
			telegramError = result.error;
		} else {
			callTranscript = result.transcript;
			telegramActionMessage = result.message;
			await loadTelegramWorkspace();
		}
		isTelegramActionSubmitting = false;
	}

</script>

<section class="telegram-page communications-page">
	<header class="communications-command-header telegram-command-header">
		<div class="command-title telegram-command-title">
			<h1>{_('Communications')} <span>/</span> {_('Telegram')}</h1>
			<p>{selectedTelegramRuntimeStatus?.status ?? telegramCapabilities?.runtime_mode ?? _('Runtime Status')}</p>
		</div>
		<label class="command-search">
			<Icon icon="tabler:search" width="18" height="18" />
			<input
				bind:value={telegramSearchQuery}
				placeholder={_('Search conversations...')}
				autocomplete="off"
			/>
		</label>
		<div class="command-menu">
			<button type="button" class:active={isTelegramFiltersMenuOpen || activeTelegramFilter !== 'all'} onclick={toggleTelegramFiltersMenu}>
				<Icon icon="tabler:filter" width="17" height="17" />{_('Filters')}
			</button>
			{#if isTelegramFiltersMenuOpen}
				<div class="command-popover filter-command-popover">
					{#each telegramFilterTabs as tab}
						<button type="button" class:active={activeTelegramFilter === tab.id} onclick={() => selectTelegramFilter(tab.id)}>
							<span>{_(tab.label)}</span><em>{telegramFilterCount(tab.id)}</em>
						</button>
					{/each}
					<button type="button" onclick={() => { closeTelegramMenus(); void syncTelegramChats(); }} disabled={isTelegramBusy || !selectedTelegramChat}>
						<span><Icon icon="tabler:refresh" width="15" height="15" />{_('Sync Chats')}</span>
					</button>
				</div>
			{/if}
		</div>
		<div class="command-menu telegram-add-account">
			<button type="button" onclick={() => openAccountDrawer('telegram')}>
				<Icon icon="tabler:user-plus" width="17" height="17" />{_('Add Account')}
			</button>
		</div>
		<div class="command-menu new-command">
			<button type="button" class="primary-button" onclick={toggleTelegramNewMenu}>{_('New')}<Icon icon="tabler:plus" width="17" height="17" /></button>
			{#if isTelegramNewMenuOpen}
				<div class="command-popover new-command-popover">
					<button type="button" onclick={openTelegramNewMessage}><Icon icon="tabler:send" width="16" height="16" />{_('New Message')}</button>
					<button type="button" onclick={() => void runTelegramQuickAction('create_note')}><Icon icon="tabler:notes" width="16" height="16" />{_('New Note')}</button>
					<button type="button" onclick={() => void runTelegramQuickAction('create_task')}><Icon icon="tabler:square-check" width="16" height="16" />{_('New Task')}</button>
					<button type="button" onclick={() => void runTelegramQuickAction('create_contact')}><Icon icon="tabler:user-plus" width="16" height="16" />{_('New Contact')}</button>
					<button type="button" onclick={() => void runTelegramQuickAction('create_document')}><Icon icon="tabler:file-plus" width="16" height="16" />{_('New Document')}</button>
				</div>
			{/if}
		</div>
	</header>

	<section class="telegram-action-rail" aria-label={_('Telegram actions')}>
		<div class="telegram-action-cluster">
			<button type="button" onclick={() => void syncTelegramChats()} disabled={isTelegramBusy || !selectedTelegramChat}>
				<Icon icon="tabler:refresh" width="16" height="16" />{_('Sync Chats')}
			</button>
			<button type="button" onclick={() => void syncSelectedTelegramHistory()} disabled={isTelegramBusy || !selectedTelegramChat}>
				<Icon icon="tabler:history" width="16" height="16" />{_('Sync History')}
			</button>
			<button type="button" onclick={() => void startTelegramRuntime()} disabled={isTelegramBusy || !selectedTelegramChat}>
				<Icon icon="tabler:player-play" width="16" height="16" />{_('Start Runtime')}
			</button>
		</div>
		<div class="telegram-group-filter-strip" aria-label={_('Chat Groups')}>
			{#each telegramChatGroupFilters as group}
				{#if group.count > 0 || group.id === 'local:all'}
					<button
						type="button"
						class:active={activeTelegramGroupFilter === group.id}
						onclick={() => selectTelegramGroupFilter(group)}
						title={group.source === 'telegram' ? _('Telegram folder') : _('Local group')}
					>
						<Icon icon={group.icon} width="15" height="15" />
						<span>{_(group.label)}</span>
						<em>{group.count}</em>
						{#if group.source === 'telegram'}<small>TG</small>{/if}
					</button>
				{/if}
			{/each}
		</div>
		<button
			type="button"
			class="telegram-inspector-toggle"
			class:active={isTelegramInspectorOpen}
			onclick={() => (isTelegramInspectorOpen ? closeTelegramInspector() : openTelegramInspector())}
		>
			<Icon icon="tabler:layout-sidebar-right" width="16" height="16" />{_('Details')}
		</button>
	</section>

	{#if telegramActionMessage}
		<p class="setup-state success">{telegramActionMessage}</p>
	{/if}
	{#if telegramError}
		<p class="inline-error">{telegramError}</p>
	{/if}

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
