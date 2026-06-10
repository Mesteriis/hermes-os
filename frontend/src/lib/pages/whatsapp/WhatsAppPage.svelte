<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';
	import WhatsAppSessionList from './widgets/WhatsAppSessionList.svelte';
	import WhatsAppMessageThread from './widgets/WhatsAppMessageThread.svelte';
	import WhatsAppRail from './widgets/WhatsAppRail.svelte';
	import * as whatsappService from '$lib/services/whatsapp';
	import * as formattingService from '$lib/services/formatting';
	import { openAccountDrawer } from '$lib/stores/accountWizard';
	import { whatsappProviderAccounts } from '$lib/stores/settings';
	import type {
		WhatsappWebSession,
		WhatsappWebMessage,
		WhatsappCapabilitiesResponse,
		WhatsappCapabilityStatus,
		MessageAnalyzeResponse
	} from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface WhatsappMessageForm {
		account_id: string;
		provider_chat_id: string;
		provider_message_id: string;
		chat_title: string;
		sender_id: string;
		sender_display_name: string;
		text: string;
		import_batch_id: string;
		occurred_at: string;
		delivery_state: string;
	}

	let whatsappSessions = $state<WhatsappWebSession[]>([]);
	let whatsappMessages = $state<WhatsappWebMessage[]>([]);
	let whatsappCapabilities = $state<WhatsappCapabilitiesResponse | null>(null);
	let selectedWhatsappSessionId = $state('');
	let whatsappError = $state('');
	let whatsappActionMessage = $state('');
	let isWhatsappLoading = $state(false);
	let isWhatsappActionSubmitting = $state(false);

	let whatsappMessageForm = $state<WhatsappMessageForm>({
		account_id: 'whatsapp-primary',
		provider_chat_id: 'wa-fixture-chat-1',
		provider_message_id: 'wa-fixture-msg-1',
		chat_title: 'WhatsApp Planning',
		sender_id: 'wa-fixture-user',
		sender_display_name: 'WhatsApp Fixture',
		text: 'WhatsApp fixture WhatsApp Web message for local memory and graph recall.',
		import_batch_id: 'whatsapp-web-fixture-ui',
		occurred_at: new Date().toISOString(),
		delivery_state: 'received'
	});

	const selectedWhatsappSession = $derived(
		whatsappSessions.find((session) => session.session_id === selectedWhatsappSessionId) ??
			whatsappSessions[0] ??
			null
	);
	const selectedWhatsappMessages = $derived(
		selectedWhatsappSession
			? whatsappMessages.filter((message) => message.account_id === selectedWhatsappSession.account_id)
			: whatsappMessages
	);
	const whatsappClosureCapabilities = $derived(
		whatsappCapabilities?.capabilities.filter((capability) => capability.closure_gate) ?? []
	);
	const whatsappBlockedCapabilities = $derived(
		whatsappCapabilities?.capabilities.filter((capability) => capability.status === 'blocked') ?? []
	);

	const formatDateTime = formattingService.formatDateTime;
	const whatsappMessageTime = whatsappService.whatsappMessageTime;

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
		void loadWhatsappWebWorkspace();
	});

	async function loadWhatsappWebWorkspace() {
		isWhatsappLoading = true;
		const result = await whatsappService.loadWhatsappWebWorkspace(selectedWhatsappSessionId);
		whatsappCapabilities = result.capabilities;
		whatsappSessions = result.sessions;
		whatsappMessages = result.messages;
		selectedWhatsappSessionId = result.selectedSessionId;
		whatsappError = result.error;
		isWhatsappLoading = false;
	}

	function selectWhatsappSession(session: WhatsappWebSession) {
		selectedWhatsappSessionId = session.session_id;
		whatsappMessageForm = {
			...whatsappMessageForm,
			account_id: session.account_id
		};
	}

	async function ingestWhatsappWebMessageFixture() {
		if (isWhatsappActionSubmitting) return;
		isWhatsappActionSubmitting = true;
		whatsappActionMessage = '';
		whatsappError = '';
		const result = await whatsappService.ingestWhatsappWebMessageFixture({
			account_id: whatsappMessageForm.account_id,
			provider_chat_id: whatsappMessageForm.provider_chat_id,
			provider_message_id: whatsappMessageForm.provider_message_id,
			chat_title: whatsappMessageForm.chat_title,
			sender_id: whatsappMessageForm.sender_id,
			sender_display_name: whatsappMessageForm.sender_display_name,
			text: whatsappMessageForm.text,
			import_batch_id: whatsappMessageForm.import_batch_id,
			occurred_at: whatsappMessageForm.occurred_at,
			delivery_state: whatsappMessageForm.delivery_state
		});
		if (result.error) {
			whatsappError = result.error;
		} else {
			whatsappActionMessage = result.message;
			whatsappMessageForm = {
				...whatsappMessageForm,
				provider_message_id: result.nextProviderMessageId,
				occurred_at: result.nextOccurredAt
			};
			await loadWhatsappWebWorkspace();
		}
		isWhatsappActionSubmitting = false;
	}
</script>

<section class="whatsapp-page communications-page">
	<div class="view-header">
		<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:brand-whatsapp" width="28" height="28" /></span><div><h1>{_('WhatsApp')}</h1><p>{_('WhatsApp Web sessions and messages')}</p></div></div>
		<button type="button" class="primary-button" onclick={() => openAccountDrawer('whatsapp')}><Icon icon="tabler:plus" width="16" height="16" />Add Account</button>
		<button type="button" class="primary-button" onclick={() => void loadWhatsappWebWorkspace()} disabled={isWhatsappLoading}><Icon icon="tabler:refresh" width="16" height="16" />Refresh</button>
	</div>

	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="whatsapp-session-status" data-widget-hidden={!isWidgetVisible('whatsapp-session-status')}>
		<WidgetEditChrome widgetId="whatsapp-session-status" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<div class="metric-grid">
			<article class="metric-card"><span>Sessions</span><strong>{whatsappSessions.length}</strong><small>{selectedWhatsappSession?.link_state ?? 'not linked'}</small></article>
			<article class="metric-card"><span>Messages</span><strong>{whatsappMessages.length}</strong><small>Canonical WhatsApp Web records</small></article>
			<article class="metric-card"><span>Runtime</span><strong>{whatsappCapabilities?.runtime_mode ?? 'unknown'}</strong><small>Fixture/manual foundation</small></article>
			<article class="metric-card"><span>Blocked</span><strong>{whatsappBlockedCapabilities.length}</strong><small>Live runtime remains blocked</small></article>
		</div>
	</div>

	{#if whatsappActionMessage}
		<p class="setup-state success">{whatsappActionMessage}</p>
	{/if}
	{#if whatsappError}
		<p class="inline-error">{whatsappError}</p>
	{/if}

	<div class="three-pane communications-grid whatsapp-grid">
		<WhatsAppSessionList
			whatsappSessions={whatsappSessions as unknown[]}
			selectedWhatsappSessionId={selectedWhatsappSession?.session_id ?? ''}
			{isWhatsappLoading}
			{isLayoutEditing}
			{isWidgetVisible}
			onSelectSession={selectWhatsappSession as unknown as (session: unknown) => void}
			{formatDateTime}
		/>
		<WhatsAppMessageThread
			selectedWhatsappSession={selectedWhatsappSession as unknown | null}
			selectedWhatsappMessages={selectedWhatsappMessages as unknown[]}
			{aiAnalysisResult}
			{selectedCommunication}
			{isWhatsappLoading}
			{isWhatsappActionSubmitting}
			{isLayoutEditing}
			{isWidgetVisible}
			whatsappMessageTime={whatsappMessageTime as unknown as (msg: unknown) => string}
			{loadWhatsappWebWorkspace}
			{ingestWhatsappWebMessageFixture}
			whatsappMessageForm={whatsappMessageForm as unknown as { provider_message_id: string; sender_display_name: string; text: string; account_id: string; provider_chat_id: string; chat_title: string; sender_id: string }}
		/>
		<WhatsAppRail
			whatsappClosureCapabilities={whatsappClosureCapabilities as unknown[]}
			whatsappBlockedCapabilities={whatsappBlockedCapabilities as unknown[]}
			{whatsappCapabilities}
			whatsappProviderAccounts={$whatsappProviderAccounts as unknown[]}
			{isWhatsappActionSubmitting}
			{isLayoutEditing}
			{isWidgetVisible}
			{capabilityLabel}
			openAccountDrawer={openAccountWizard}
			{ingestWhatsappWebMessageFixture}
			whatsappMessageForm={whatsappMessageForm as unknown as { provider_message_id: string; sender_display_name: string; text: string; account_id: string; provider_chat_id: string; chat_title: string; sender_id: string }}
		/>
	</div>

</section>
