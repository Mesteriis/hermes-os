<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import TelegramChatList from './widgets/TelegramChatList.svelte';
	import TelegramMessageThread from './widgets/TelegramMessageThread.svelte';
	import TelegramRail from './widgets/TelegramRail.svelte';
	import * as telegramService from '$lib/services/telegram';
	import * as formattingService from '$lib/services/formatting';
	import { openAccountDrawer } from '$lib/stores/accountWizard';
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
		CallTranscript,
		MessageAnalyzeResponse
	} from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface TelegramMessageForm {
		account_id: string;
		provider_chat_id: string;
		provider_message_id: string;
		chat_kind: 'private' | 'group' | 'channel' | 'bot';
		chat_title: string;
		sender_id: string;
		sender_display_name: string;
		text: string;
		import_batch_id: string;
		occurred_at: string;
		delivery_state: 'received' | 'sent' | 'send_dry_run' | 'send_blocked';
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
	let selectedTelegramChatId = $state('');
	let selectedTelegramCallId = $state('');
	let callTranscript = $state<CallTranscript | null>(null);
	let telegramError = $state('');
	let telegramActionMessage = $state('');
	let isTelegramLoading = $state(false);
	let isTelegramActionSubmitting = $state(false);
	let telegramSendDryRunResult = $state<TelegramSendDryRunResponse | null>(null);

	let telegramMessageForm = $state<TelegramMessageForm>({
		account_id: 'telegram-primary',
		provider_chat_id: 'fixture-chat-1',
		provider_message_id: 'fixture-msg-1',
		chat_kind: 'private',
		chat_title: 'Telegram Planning',
		sender_id: 'telegram-fixture-user',
		sender_display_name: 'Telegram Fixture',
		text: 'Telegram fixture Telegram message for policy and graph smoke coverage.',
		import_batch_id: 'telegram-fixture-ui',
		occurred_at: new Date().toISOString(),
		delivery_state: 'received'
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
			? telegramMessages.filter(
					(message) => message.provider_chat_id === selectedTelegramChat.provider_chat_id
				)
			: telegramMessages
	);
	const selectedTelegramCall = $derived(
		telegramCalls.find((call) => call.call_id === selectedTelegramCallId) ?? telegramCalls[0] ?? null
	);
	const telegramClosureCapabilities = $derived(
		telegramCapabilities?.capabilities.filter((capability) => capability.closure_gate) ?? []
	);
	const telegramBlockedCapabilities = $derived(
		telegramCapabilities?.capabilities.filter((capability) => capability.status === 'blocked') ?? []
	);

	const formatDateTime = formattingService.formatDateTime;
	const telegramMessageTime = telegramService.telegramMessageTime;

	function capabilityLabel(capability: string) {
		return capability
			.split('_')
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function openAccountWizard(target: string) {
		openAccountDrawer(target as Parameters<typeof openAccountDrawer>[0]);
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
		automationTemplates = result.templates;
		automationPolicies = result.policies;
		telegramCalls = result.calls;
		selectedTelegramChatId = result.selectedChatId;
		selectedTelegramCallId = result.selectedCallId;
		callTranscript = result.transcript;
		telegramError = result.error;
		isTelegramLoading = false;
	}

	function selectTelegramChat(chat: TelegramChat) {
		selectedTelegramChatId = chat.provider_chat_id;
		telegramMessageForm = {
			...telegramMessageForm,
			account_id: chat.account_id,
			provider_chat_id: chat.provider_chat_id,
			chat_kind: telegramChatKindValue(chat.chat_kind),
			chat_title: chat.title
		};
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

	async function loadSelectedCallTranscript(callId = selectedTelegramCallId) {
		const result = await telegramService.loadSelectedCallTranscript(callId);
		callTranscript = result.transcript;
		if (result.error) telegramError = result.error;
	}

	async function ingestTelegramMessageFixture() {
		if (isTelegramActionSubmitting) return;
		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		const result = await telegramService.ingestTelegramMessageFixture({
			account_id: telegramMessageForm.account_id,
			provider_chat_id: telegramMessageForm.provider_chat_id,
			provider_message_id: telegramMessageForm.provider_message_id,
			chat_kind: telegramMessageForm.chat_kind,
			chat_title: telegramMessageForm.chat_title,
			sender_id: telegramMessageForm.sender_id,
			sender_display_name: telegramMessageForm.sender_display_name,
			text: telegramMessageForm.text,
			import_batch_id: telegramMessageForm.import_batch_id,
			occurred_at: telegramMessageForm.occurred_at,
			delivery_state: telegramMessageForm.delivery_state
		});
		if (result.error) {
			telegramError = result.error;
		} else {
			selectedTelegramChatId = result.providerChatId;
			telegramActionMessage = result.message;
			telegramMessageForm = {
				...telegramMessageForm,
				provider_message_id: result.nextProviderMessageId,
				occurred_at: result.nextOccurredAt
			};
			await loadTelegramWorkspace();
		}
		isTelegramActionSubmitting = false;
	}

	async function saveTelegramAutomationTemplate() {
		if (isTelegramActionSubmitting) return;
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
		if (isTelegramActionSubmitting) return;
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
		if (isTelegramActionSubmitting) return;
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
		if (isTelegramActionSubmitting) return;
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
		if (isTelegramActionSubmitting || !selectedTelegramCallId) return;
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

	function telegramChatKindValue(value: string): 'private' | 'group' | 'channel' | 'bot' {
		if (value === 'group' || value === 'channel' || value === 'bot') return value;
		return 'private';
	}
</script>

<section class="telegram-page communications-page">
	<div class="view-header">
		<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:brand-telegram" width="28" height="28" /></span><div><h1>{_('Telegram')}</h1><p>{_('Telegram messages, chats, policies and calls')}</p></div></div>
		<button type="button" class="primary-button" onclick={() => openAccountDrawer('telegram')}><Icon icon="tabler:plus" width="16" height="16" />{_('Add Account')}</button>
		<button type="button" class="primary-button" onclick={() => void loadTelegramWorkspace()} disabled={isTelegramLoading}><Icon icon="tabler:refresh" width="16" height="16" />{_('Refresh')}</button>
	</div>

	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="telegram-account-status" data-widget-hidden={!isWidgetVisible('telegram-account-status')}>
		<WidgetEditChrome widgetId="telegram-account-status" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<div class="metric-grid">
			<article class="metric-card"><span>{_('Chats')}</span><strong>{telegramChats.length}</strong><small>{selectedTelegramChat?.sync_state ?? _('not synced')}</small></article>
			<article class="metric-card"><span>{_('Messages')}</span><strong>{telegramMessages.length}</strong><small>{_('Projected channel records')}</small></article>
			<article class="metric-card"><span>{_('Templates')}</span><strong>{automationTemplates.length}</strong><small>{_('UI-approved only')}</small></article>
			<article class="metric-card"><span>{_('Policies')}</span><strong>{automationPolicies.length}</strong><small>{automationPolicies.filter((policy) => policy.enabled).length} {_('enabled')}</small></article>
			<article class="metric-card"><span>{_('Calls')}</span><strong>{telegramCalls.length}</strong><small>{selectedTelegramCall?.call_state ?? _('no history')}</small></article>
			<article class="metric-card"><span>{_('Transcript')}</span><strong>{callTranscript?.transcript_status ?? _('none')}</strong><small>{callTranscript?.stt_provider ?? _('fixture STT')}</small></article>
		</div>
	</div>

	{#if telegramActionMessage}
		<p class="setup-state success">{telegramActionMessage}</p>
	{/if}
	{#if telegramError}
		<p class="inline-error">{telegramError}</p>
	{/if}

	<div class="three-pane communications-grid telegram-grid">
		<TelegramChatList
			telegramChats={telegramChats as unknown[]}
			selectedTelegramChatId={selectedTelegramChat?.provider_chat_id ?? ''}
			{isTelegramLoading}
			{isLayoutEditing}
			{isWidgetVisible}
			onSelectChat={selectTelegramChat as unknown as (chat: unknown) => void}
			{formatDateTime}
		/>
		<TelegramMessageThread
			selectedTelegramChat={selectedTelegramChat as unknown | null}
			selectedTelegramMessages={selectedTelegramMessages as unknown[]}
			{aiAnalysisResult}
			{selectedCommunication}
			{isTelegramLoading}
			{isTelegramActionSubmitting}
			{isLayoutEditing}
			{isWidgetVisible}
			telegramMessageTime={telegramMessageTime as unknown as (msg: unknown) => string}
			{loadTelegramWorkspace}
			{ingestTelegramMessageFixture}
			telegramMessageForm={telegramMessageForm as unknown as { provider_message_id: string; sender_display_name: string; text: string }}
		/>
		<TelegramRail
			telegramClosureCapabilities={telegramClosureCapabilities as unknown[]}
			telegramBlockedCapabilities={telegramBlockedCapabilities as unknown[]}
			{telegramCapabilities}
			automationTemplates={automationTemplates as unknown[]}
			telegramCalls={telegramCalls as unknown[]}
			selectedTelegramCall={selectedTelegramCall as unknown | null}
			{selectedTelegramCallId}
			{callTranscript}
			telegramSendDryRunResult={telegramSendDryRunResult as unknown | null}
			telegramProviderAccounts={$telegramProviderAccounts as unknown[]}
			{isTelegramActionSubmitting}
			{isLayoutEditing}
			{isWidgetVisible}
			{capabilityLabel}
			openAccountDrawer={openAccountWizard}
			selectTelegramCall={selectTelegramCall as unknown as (call: unknown) => void}
			{saveTelegramAutomationTemplate}
			{saveTelegramAutomationPolicy}
			{runTelegramAutomationDryRun}
			{saveTelegramCallFixture}
			{saveCallTranscriptFixtureFromUi}
			automationTemplateForm={automationTemplateForm as unknown as { template_id: string; name: string; body_template: string; required_variables_text: string }}
			automationPolicyForm={automationPolicyForm as unknown as { policy_id: string; template_id: string; name: string; account_id: string; allowed_chat_ids_text: string; trigger_kind: string; max_sends_per_hour: number; quiet_hours_text: string; conditions_text: string; enabled: boolean }}
			telegramSendForm={telegramSendForm as unknown as { policy_id: string; provider_chat_id: string; variables_text: string; source_context_text: string }}
			telegramCallForm={telegramCallForm as unknown as { call_id: string; provider_call_id: string; account_id: string; provider_chat_id: string; direction: string; call_state: string; metadata_text: string }}
			transcriptForm={transcriptForm as unknown as { transcript_id: string; source_audio_ref: string; language_code: string; always_on_policy: boolean }}
		/>
	</div>

</section>
