<script lang="ts">
	import Icon from '@iconify/svelte';
	import {
		completeGmailOAuthSetup,
		dryRunTelegramSend,
		fetchApplicationSettings,
		fetchAutomationPolicies,
		fetchAutomationTemplates,
		fetchCallTranscript,
		fetchCommunicationMessage,
		fetchDrafts,
	createDraft,
	fetchMailboxHealth,
	fetchTopSenders,
	fetchThreads,
	analyzeMessage,
	searchEmails,
	type EmailDraft,
	type EnrichedPerson,
	type Organization,
	type MailboxHealth,
	type SenderStats,
	type EmailThread,
	type MessageAnalyzeResponse,
	fetchMailMessages,
	fetchMailMessage,
	transitionMessageWorkflowState,
	fetchMessageStateCounts,
	type CommunicationMessageSummaryV2,
	type MailMessageDetailResponse,
	type WorkflowState,
	type WorkflowStateCountItem,
	fetchCommunicationMessages,
		fetchDocumentProcessing,
		fetchGraphNeighborhood,
		fetchGraphNodes,
		fetchGraphSummary,
		fetchProjectDetail,
		fetchDocumentProcessingJobs,
		fetchAiAgents,
		fetchAiRuns,
		fetchAiStatus,
		fetchIdentityCandidates,
		fetchProviderAccounts,
		fetchTaskCandidates,
		fetchTaskRecords,
		fetchTelegramCapabilities,
		fetchWhatsappCapabilities,
		fetchTelegramCalls,
		fetchTelegramChats,
		fetchTelegramMessages,
		fetchWhatsappWebMessages,
		fetchWhatsappWebSessions,
		fetchProjects,
		fetchV1Status,
		collectVaultEntropy,
		createVault,
		exportVaultRecovery,
		unlockVault,
		fetchPersons,
		fetchOrganizations,
		fetchCalendarAccounts,
		fetchCalendarSources,
		fetchCalendarEvents,
		fetchCalendarWatchtower,
		fetchWeeklyBrief,
		searchCalendarEvents,
		fetchEventBrief,
		fetchEventContextPack,
		fetchEventAgenda,
		fetchEventChecklist,
		fetchMeetingNotes,
		createCalendarAccount,
		createCalendarEvent,
		postCalendarBrain,
		type CalendarAccount,
		type CalendarSource,
		type CalendarEvent,
		findFrontendLayoutSetting,
		findFrontendSidebarSetting,
		findFrontendLocaleSetting,
		findFrontendThemeSetting,
		FRONTEND_THEME_SETTING_KEY,
		ingestTelegramFixtureMessage,
		ingestWhatsappWebFixtureMessage,
		refreshAiTaskCandidates,
		reviewIdentityCandidate,
		reviewTaskCandidate,
		requestAiAnswer,
		requestAiMeetingPrep,
		retryDocumentProcessingJob,
		saveApplicationSetting,
		saveFrontendLayoutSetting,
		saveFrontendSidebarSetting,
		saveFrontendLocaleSetting,
		saveFrontendThemeSetting,
		searchGraphNodes,
		saveAutomationPolicy,
		saveAutomationTemplate,
			saveCallTranscriptFixture,
			saveTelegramCall,
			setupImapAccount,
			fetchTelegramQrLoginStatus,
			setupTelegramAccount,
			setupTelegramFixtureAccount,
			setupWhatsappWebFixtureAccount,
			startTelegramQrLogin,
			submitTelegramQrLoginPassword,
			startGmailOAuthSetup,
		type ApplicationSetting,
		type AiAgent,
		type AiAnswerResponse,
		type AiCitation,
		type AiMeetingPrepResponse,
		type AiRun,
		type AiStatus,
		type AiTaskCandidateRefreshResponse,
		type AutomationPolicy,
		type AutomationTemplate,
		type CallTranscript,
		type PersonIdentityCandidate,
		type PersonIdentityReviewState,
		type CommunicationMessageDetail,
		type CommunicationMessageDetailItem,
		type CommunicationMessageSummary,
		type GmailOAuthStartResponse,
		type GraphEdge,
		type DocumentProcessingJob,
		type DocumentProcessingRecord,
		type GraphEvidenceSummary,
		type GraphNeighborhood,
		type GraphNode,
		type GraphNodeKind,
		type GraphRelationshipType,
		type GraphSummary,
		type ProviderAccount,
		type ProjectDetail,
		type ProjectDocumentSummary,
		type ProjectMessageSummary,
		type ProjectStats,
		type ProjectSummary,
		type ProjectTimelineItem,
		type TaskCandidate,
		type TaskCandidateReviewState,
		type Task,
		type TelegramCall,
		type TelegramChat,
		type TelegramMessage,
			type TelegramProviderKind,
			type TelegramQrLoginStatusResponse,
			type TelegramSendDryRunResponse,
		type TelegramCapabilitiesResponse,
		type WhatsappCapabilitiesResponse,
		type WhatsappWebMessage,
		type WhatsappWebSession,
		type V1Status,
		type VaultEntropyEvent,
		type VaultRecoveryExportResponse
	} from '$lib/api';
	import {
		defaultLayoutSettings,
		defaultSidebarSettings,
		defaultFrontendThemeSettings,
		findPresetForView,
		parseLayoutSettings,
		parseCommunicationSidebarItemId,
		parseSidebarSettings,
		parseFrontendThemeSettings,
		communicationSectionViewId,
		communicationSections,
		LAYOUT_GRID_COLUMNS,
		LAYOUT_GRID_MAX_ROWS,
		panelBlurValues,
		panelOpacityValues,
		primaryWorkspaceNav,
		resolveSidebarRootEntries,
		resolveLayout,
		sidebarGroupIdFromRootId,
		sidebarGroupRootId,
		shellAccentClass,
		shellAccentColorOptions,
		shellBackgroundClass,
		shellBackgroundBrightnessValues,
		shellBackgroundOptions,
		shellBrightnessClass,
		shellPanelBlurClass,
		shellPanelOpacityClass,
		widgetRegistry,
		type CommunicationSection,
		type CommunicationSectionId,
		type FrontendThemeSettings,
		type LayoutSettings,
		type PanelBlur,
		type PanelOpacity,
		type PrimaryNavId,
		type ResolvedSidebarItem,
		type ResolvedLayout,
		type ResolvedWidget,
		type SidebarItemId,
		type SidebarNavGroup,
		type SidebarRootItemId,
		type SidebarSettings,
		type SidebarViewId,
		type ShellAccentColorId,
		type ShellBackgroundBrightness,
		type ShellBackgroundId,
		type ViewLayoutOverride
	} from '$lib/layout';
	import { onMount } from 'svelte';
	import { currentLocale, setLocale, t } from '$lib/i18n';
	const _ = (key: string) => t($currentLocale, key);

	type Provider = 'gmail' | 'icloud' | 'imap';
	type AccountWizardKind = 'mail' | 'calendar' | 'telegram' | 'whatsapp';
	type AccountWizardTarget = AccountWizardKind | Provider;
	type MailService = Provider | 'microsoft' | 'yahoo' | 'aol';
	type MailWizardStep = 'provider' | 'details';
	type CalendarProvider = 'local' | 'google' | 'microsoft' | 'apple' | 'caldav' | 'ics';
	type CalendarWizardStep = 'provider' | 'details';
	type TelegramSetupMode = 'fixture' | 'live';
	type TelegramAuthMethod = 'fixture' | 'phone' | 'qr' | 'bot_token';
	type TelegramWizardStep = 'account' | 'auth' | 'details';
	type AppViewId = PrimaryNavId | 'organizations' | 'settings';
	type ViewId = SidebarViewId;
	type WidgetGridDimension = 'columns' | 'rows';
	type WidgetPanelSurfaceSetting = 'panelOpacity' | 'panelBlur';

	type NavItem = {
		id: PrimaryNavId;
		label: string;
		icon: string;
		badge?: string;
		enabled: boolean;
	};

	type CommunicationNavItem = CommunicationSection & {
		badge?: string;
	};

	type StatCard = {
		label: string;
		value: string;
		delta: string;
		icon: string;
		tone?: string;
	};

	type NotificationItem = {
		id: string;
		source: 'telegram' | 'communications';
		icon: string;
		title: string;
		body: string;
		meta: string;
		time: string | null;
		messageId?: string;
		accountId?: string;
		providerChatId?: string | null;
		targetSection?: CommunicationSectionId;
		isDemo?: boolean;
	};

	type FeedItem = {
		icon: string;
		title: string;
		meta: string;
		time: string;
		tag?: string;
		tone?: string;
	};

	type ProjectItem = {
		name: string;
		kind: string;
		progress: number;
		tasks: number;
		icon: string;
		tone: string;
	};

	type TaskItem = {
		title: string;
		tracker: string;
		project: string;
		assignee: string;
		status: string;
		priority: string;
		due: string;
		group: string;
	};

	type Person = {
		name: string;
		role: string;
		company: string;
		channel?: string;
		status?: string;
	};

	type Conversation = {
		name: string;
		role: string;
		project: string;
		channel: string;
		time: string;
		unread?: string;
		preview: string;
	};

	type GraphCanvasNode = GraphNode & {
		x: number;
		y: number;
		isSelected: boolean;
		layoutClass: string;
	};

	type GraphCanvasEdge = GraphEdge & {
		x1: number;
		y1: number;
		x2: number;
		y2: number;
		label: string;
	};

	type GraphPropertyRow = {
		key: string;
		value: string;
	};

	type GraphFilterChip = {
		id: string;
		label: string;
		count: number | null;
		enabled: boolean;
	};

	const apiBaseUrl = import.meta.env.VITE_HERMES_API_BASE_URL ?? 'http://127.0.0.1:8080';
	const apiSecret = import.meta.env.VITE_HERMES_LOCAL_API_SECRET ?? 'change-me-local-api-secret';

	let currentView = $state<AppViewId>('home');
	let activeCommunicationSection = $state<CommunicationSectionId>('unified');
	let expandedSidebarGroupIds = $state<string[]>(['communications']);
	let activeSidebarRailGroupId = $state<string | null>(null);
	let isSidebarRail = $state(false);
	let isNotificationsDrawerOpen = $state(false);
	let dismissedNotificationIds = $state<string[]>([]);
	let expandedNotificationIds = $state<string[]>([]);
	let demoNotificationItems = $state<NotificationItem[]>([
		{
			id: 'demo:telegram:new-message',
			source: 'telegram',
			icon: 'tabler:brand-telegram',
			title: 'New Telegram message',
			body: 'Maria sent a note about the sidebar navigation flow. She wants the Communications submenu to feel like a workspace context, not a flat list of pages, and asked for one more pass on the rail interaction.',
			meta: 'Demo · Telegram',
			time: new Date(Date.now() - 1000 * 60 * 7).toISOString(),
			providerChatId: 'demo-telegram-chat',
			targetSection: 'telegram',
			isDemo: true
		},
		{
			id: 'demo:communications:needs-reply',
			source: 'communications',
			icon: 'tabler:mail',
			title: 'Needs reply',
			body: 'Acme Corp is waiting for a short confirmation. They need an answer before tomorrow morning so the planning thread can move into execution without another status meeting.',
			meta: 'Demo · Mail',
			time: new Date(Date.now() - 1000 * 60 * 22).toISOString(),
			targetSection: 'needs_reply',
			isDemo: true
		},
		{
			id: 'demo:communications:mention',
			source: 'communications',
			icon: 'tabler:at',
			title: 'Mentioned in thread',
			body: 'Alex mentioned you in the product planning discussion. The thread includes design feedback, notification drawer behavior, and sidebar customization rules for grouped sections.',
			meta: 'Demo · Mentions',
			time: new Date(Date.now() - 1000 * 60 * 48).toISOString(),
			targetSection: 'mentions',
			isDemo: true
		}
	]);
	let status = $state<V1Status | null>(null);
	let statusError = $state('');
	let vaultWizardStep = $state<'intro' | 'entropy' | 'biometric' | 'recovery' | 'done'>('intro');
	let vaultEntropyEvents = $state<VaultEntropyEvent[]>([]);
	let vaultEntropyBuffer: VaultEntropyEvent[] = [];
	let vaultLastEntropyEvent = $state<VaultEntropyEvent | null>(null);
	let vaultWizardError = $state('');
	let vaultWizardMessage = $state('');
	let vaultRecovery = $state<VaultRecoveryExportResponse | null>(null);
	let isVaultActionSubmitting = $state(false);
	let graphSummary = $state<GraphSummary | null>(null);
	let graphError = $state('');
	let isGraphSummaryLoading = $state(false);
	let graphNodeChoices = $state<GraphNode[]>([]);
	let graphNodeChoicesError = $state('');
	let isGraphNodeChoicesLoading = $state(false);
	let graphSearchQuery = $state('');
	let graphSearchResults = $state<GraphNode[]>([]);
	let graphSearchError = $state('');
	let isGraphSearchLoading = $state(false);
	let graphSearchSubmitted = $state(false);
	let lastSubmittedGraphSearchQuery = $state('');
	let graphNeighborhood = $state<GraphNeighborhood | null>(null);
	let graphNeighborhoodError = $state('');
	let isGraphNeighborhoodLoading = $state(false);
	let graphNodeChoicesRequestSequence = 0;
	let graphSearchRequestSequence = 0;
	let graphNeighborhoodRequestSequence = 0;
	let communicationMessages = $state<CommunicationMessageSummary[]>([]);
	let isComposeOpen = $state(false);
	let composeForm = $state({
		draft_id: '',
		account_id: '',
		to_text: '',
		cc_text: '',
		subject: '',
		body: '',
	});
	let drafts = $state<EmailDraft[]>([]);
	let mailboxHealth = $state<MailboxHealth | null>(null);
	let topSenders = $state<SenderStats[]>([]);
	let threads = $state<EmailThread[]>([]);
	let isAnalyzing = $state(false);
	let aiAnalysisResult = $state<MessageAnalyzeResponse | null>(null);

	let mailStateFilter = $state<WorkflowState | ''>('');
	let mailStateCounts = $state<WorkflowStateCountItem[]>([]);
	let isMailStateTransitioning = $state(false);
	let mailStateError = $state('');

	let selectedCommunicationDetail = $state<CommunicationMessageDetail | null>(null);
	let communicationsError = $state('');
	let isCommunicationsLoading = $state(false);
	let projectSummaries = $state<ProjectSummary[]>([]);
	let selectedProjectDetail = $state<ProjectDetail | null>(null);
	let selectedProjectId = $state('');
	let projectsError = $state('');
	let isProjectsLoading = $state(false);
	let taskCandidates = $state<TaskCandidate[]>([]);
	let activeTasks = $state<Task[]>([]);
	let documentProcessingJobs = $state<DocumentProcessingJob[]>([]);
	let selectedDocumentProcessingDetail = $state<DocumentProcessingRecord | null>(null);
	let documentProcessingDetailError = $state('');
	let isDocumentProcessingJobsLoading = $state(false);
	let retryingDocumentProcessingJobId = $state<string | null>(null);
	let documentProcessingJobsError = $state('');
	let isTasksLoading = $state(false);
	let tasksError = $state('');
	let identityCandidates = $state<PersonIdentityCandidate[]>([]);
	let identityCandidatesError = $state('');
	let isIdentityCandidatesLoading = $state(false);
	let persons = $state<EnrichedPerson[]>([]);
	let organizations = $state<Organization[]>([]);
	let organizationsError = $state('');
	let isOrganizationsLoading = $state(false);

	let calendarAccounts = $state<CalendarAccount[]>([]);
	let calendarEvents = $state<CalendarEvent[]>([]);
	let calendarSources = $state<CalendarSource[]>([]);
	let calendarWatchtower = $state<Record<string, unknown>>({});
	let isCalendarLoading = $state(false);
	let calendarError = $state('');
	let calendarViewMode = $state<'day' | 'week' | 'month' | 'agenda'>('week');
	let selectedEvent = $state<CalendarEvent | null>(null);
	let calendarSearchQuery = $state('');
	let calendarSearchResults = $state<CalendarEvent[]>([]);
	let weeklyBrief = $state<Record<string, unknown> | null>(null);
	let eventBrief = $state<Record<string, unknown> | null>(null);
	let eventContext = $state<Record<string, unknown> | null>(null);
	let eventAgenda = $state<Record<string, unknown> | null>(null);
	let showNewEventForm = $state(false);
	let newEventTitle = $state('');
	let newEventStart = $state('');
	let newEventEnd = $state('');
	let newEventType = $state('meeting');
	let selectedOrganizationId = $state('');
	let personsError = $state('');
	let isPersonsLoading = $state(false);
	let projectRequestSequence = 0;
	let selectedConversationIndex = $state(0);
	let selectedPersonIndex = $state(0);
	let selectedAgentIndex = $state(0);
	let aiStatus = $state<AiStatus | null>(null);
	let aiAgents = $state<AiAgent[]>([]);
	let aiRuns = $state<AiRun[]>([]);
	let aiError = $state('');
	let isAiLoading = $state(false);
	let isAiAnswerSubmitting = $state(false);
	let isAiTaskRefreshSubmitting = $state(false);
	let isAiMeetingPrepSubmitting = $state(false);
	let aiQuestion = $state('What does the local memory say about Hermes Hub V3?');
	let aiMeetingTopic = $state('Prepare a V3 implementation review brief');
	let aiTaskQuery = $state('Find open task candidates from local messages and documents');
	let aiAnswerResult = $state<AiAnswerResponse | null>(null);
	let aiMeetingPrepResult = $state<AiMeetingPrepResponse | null>(null);
	let aiTaskRefreshResult = $state<AiTaskCandidateRefreshResponse | null>(null);
	let selectedProvider = $state<Provider>('gmail');
	let accountWizardKind = $state<AccountWizardKind>('mail');
	let mailWizardStep = $state<MailWizardStep>('provider');
	let selectedMailService = $state<MailService>('icloud');
	let mailAddress = $state('');
	let calendarWizardStep = $state<CalendarWizardStep>('provider');
	let isAccountDrawerOpen = $state(false);
	let isSetupSubmitting = $state(false);
	let setupMessage = $state('');
	let setupError = $state('');
	let gmailPending = $state<GmailOAuthStartResponse | null>(null);
	let gmailAuthorizationCode = $state('');
	let gmailForm = $state({
		account_id: 'gmail-primary',
		display_name: 'Primary Gmail',
		external_account_id: '',
		client_id: '',
		client_secret: '',
		redirect_uri: `${apiBaseUrl.replace(/\/+$/, '')}/api/v1/email-accounts/gmail/oauth/callback`
	});
	let imapForm = $state({
		account_id: 'icloud-primary',
		display_name: 'Primary iCloud',
		external_account_id: '',
		host: 'imap.mail.me.com',
		port: 993,
		tls: true,
		mailbox: 'INBOX',
		username: '',
		password: '',
		secret_kind: 'app_password' as 'app_password' | 'password'
	});
	let calendarAccountForm = $state({
		provider: 'local' as CalendarProvider,
		account_name: 'Local Calendar',
		email: ''
	});
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
	let telegramSetupMode = $state<TelegramSetupMode>('fixture');
	let telegramAuthMethod = $state<TelegramAuthMethod>('fixture');
	let telegramWizardStep = $state<TelegramWizardStep>('account');
	let telegramQrLogin = $state<TelegramQrLoginStatusResponse | null>(null);
	let telegramQrPassword = $state('');
	let telegramAccountForm = $state({
		account_id: 'telegram-primary',
		provider_kind: 'telegram_user' as TelegramProviderKind,
		display_name: 'Primary Telegram',
		external_account_id: '@telegram_fixture',
		api_id: '',
		api_hash: '',
		bot_token: '',
		session_encryption_key: '',
		tdlib_data_path: 'docker/data/telegram/telegram-primary',
		transcription_enabled: false
	});
	let telegramMessageForm = $state({
		account_id: 'telegram-primary',
		provider_chat_id: 'fixture-chat-1',
		provider_message_id: 'fixture-msg-1',
		chat_kind: 'private' as 'private' | 'group' | 'channel' | 'bot',
		chat_title: 'Telegram Planning',
		sender_id: 'telegram-fixture-user',
		sender_display_name: 'Telegram Fixture',
		text: 'Telegram fixture Telegram message for policy and graph smoke coverage.',
		import_batch_id: 'telegram-fixture-ui',
		occurred_at: new Date().toISOString(),
		delivery_state: 'received' as 'received' | 'sent' | 'send_dry_run' | 'send_blocked'
	});
	let automationTemplateForm = $state({
		template_id: 'template-telegram-followup',
		name: 'Telegram Follow-up',
		body_template: 'Hi {{name}}, I will follow up about {{topic}}.',
		required_variables_text: 'name, topic'
	});
	let automationPolicyForm = $state({
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
	let telegramSendForm = $state({
		policy_id: 'policy-telegram-followup',
		provider_chat_id: 'fixture-chat-1',
		variables_text: '{ "name": "Maria", "topic": "Telegram client" }',
		source_context_text: '{ "source": "desktop_ui_fixture" }'
	});
	let telegramCallForm = $state({
		call_id: 'call-telegram-fixture-1',
		account_id: 'telegram-primary',
		provider_call_id: 'provider-call-telegram-fixture-1',
		provider_chat_id: 'fixture-chat-1',
		direction: 'incoming' as 'incoming' | 'outgoing',
		call_state: 'ended' as 'ringing' | 'active' | 'ended' | 'missed' | 'declined' | 'failed',
		started_at: new Date().toISOString(),
		ended_at: '',
		transcription_policy_id: '',
		metadata_text: '{ "runtime": "fixture", "visible_recording_state": true }'
	});
	let whatsappSessions = $state<WhatsappWebSession[]>([]);
	let whatsappMessages = $state<WhatsappWebMessage[]>([]);
	let whatsappCapabilities = $state<WhatsappCapabilitiesResponse | null>(null);
	let selectedWhatsappSessionId = $state('');
	let whatsappError = $state('');
	let whatsappActionMessage = $state('');
	let isWhatsappLoading = $state(false);
	let isWhatsappActionSubmitting = $state(false);
	let whatsappAccountForm = $state({
		account_id: 'whatsapp-primary',
		display_name: 'Primary WhatsApp Web',
		external_account_id: 'whatsapp-fixture-device',
		device_name: 'Hermes Desktop Fixture',
		local_state_path: 'docker/data/whatsapp/whatsapp-primary'
	});
	let whatsappMessageForm = $state({
		account_id: 'whatsapp-primary',
		provider_chat_id: 'wa-fixture-chat-1',
		provider_message_id: 'wa-fixture-msg-1',
		chat_title: 'WhatsApp Planning',
		sender_id: 'wa-fixture-user',
		sender_display_name: 'WhatsApp Fixture',
		text: 'WhatsApp fixture WhatsApp Web message for local memory and graph recall.',
		import_batch_id: 'whatsapp-web-fixture-ui',
		occurred_at: new Date().toISOString(),
		delivery_state: 'received' as 'received' | 'sent' | 'send_dry_run' | 'send_blocked'
	});
	let transcriptForm = $state({
		transcript_id: 'transcript-telegram-fixture-1',
		account_id: 'telegram-primary',
		provider_chat_id: 'fixture-chat-1',
		source_audio_ref: 'docker/data/calls/fixture-call.wav',
		language_code: 'en',
		always_on_policy: true
	});
	let applicationSettings = $state<ApplicationSetting[]>([]);
	let layoutSettings = $state<LayoutSettings>(defaultLayoutSettings());
	let sidebarSettings = $state<SidebarSettings>(defaultSidebarSettings());
	let isLayoutEditing = $state(false);
	let isWidgetDrawerOpen = $state(false);
	let selectedLayoutWidgetId = $state<string | null>(null);
	let isUserMenuOpen = $state(false);
	let layoutDraft = $state<LayoutSettings | null>(null);
	let sidebarDraft = $state<SidebarSettings | null>(null);
	let layoutError = $state('');
	let sidebarError = $state('');
	let newSidebarGroupLabel = $state('');
	let isLayoutSettingsSaving = $state(false);
	let isSidebarSettingsSaving = $state(false);
	const effectiveLayoutSettings = $derived(layoutDraft ?? layoutSettings);
	const effectiveSidebarSettings = $derived(sidebarDraft ?? sidebarSettings);
	const activeWorkspaceView = $derived(
		currentView === 'communications'
			? communicationSectionViewId(activeCommunicationSection)
			: currentView
	);
	const isCommunicationMessagesSection = $derived(
		currentView === 'communications' &&
			['unified', 'inbox', 'waiting', 'needs_reply', 'mail'].includes(activeCommunicationSection)
	);
	const activeCommunicationEmptySection = $derived(
		currentView === 'communications' &&
			['mentions', 'calls', 'meetings'].includes(activeCommunicationSection)
			? communicationSections.find((item) => item.id === activeCommunicationSection) ?? null
			: null
	);
	const activeLayout = $derived(resolveActiveLayout(activeWorkspaceView, effectiveLayoutSettings));
	const activeWidgetById = $derived.by(() => {
		const widgets = Object.values(activeLayout?.widgetsByZone ?? {}).flat();
		return new Map(widgets.map((widget) => [widget.widgetId, widget]));
	});
	const selectedLayoutWidget = $derived(
		selectedLayoutWidgetId ? activeWidgetById.get(selectedLayoutWidgetId) ?? null : null
	);
	const hiddenWidgetTitles = $derived.by(() =>
		(activeLayout?.hiddenByUser ?? []).map((widget) => widget.definition.title)
	);
	let viewportHiddenWidgetTitles = $state<string[]>([]);
	const gridClassNames = [
		...Array.from({ length: 12 }, (_, index) => `widget-cols-${index + 1}`),
		...Array.from({ length: 24 }, (_, index) => `widget-rows-${index + 1}`),
		...panelOpacityValues.map((value) => `widget-panel-opacity-${value}`),
		...panelBlurValues.map((value) => `widget-panel-blur-${value}`),
		'widget-scroll-none',
		'widget-scroll-y',
		'widget-scroll-x',
		'widget-scroll-both',
		'widget-fit-hidden'
	];
	const scrollClassByMode = {
		none: 'widget-scroll-none',
		vertical: 'widget-scroll-y',
		horizontal: 'widget-scroll-x',
		both: 'widget-scroll-both'
	};
	const renderedWidgetIdsForCurrentView = $derived.by(() => {
		if (!isWidgetDrawerOpen || typeof document === 'undefined') {
			return null;
		}

		return new Set(
			Array.from(document.querySelectorAll<HTMLElement>('.widget-frame[data-widget-id]'))
				.map((element) => element.dataset.widgetId)
				.filter((widgetId): widgetId is string => Boolean(widgetId))
		);
	});
	const addableWidgetsForCurrentView = $derived.by(() => {
		const layout = activeLayout;
		const preset = layout?.preset ?? findPresetForView(activeWorkspaceView);
		if (!preset) {
			return [];
		}

		const currentWidgetIds = new Set(preset.widgets.map((widget) => widget.widgetId));
		for (const widget of layout?.hiddenByUser ?? []) {
			currentWidgetIds.add(widget.widgetId);
		}

		const hiddenWidgetIds = new Set((layout?.hiddenByUser ?? []).map((widget) => widget.widgetId));

		return widgetRegistry
			.filter(
				(widget) =>
					widget.canAdd &&
					widget.viewScope.includes(preset.viewId) &&
					currentWidgetIds.has(widget.id) &&
					(renderedWidgetIdsForCurrentView === null ||
						renderedWidgetIdsForCurrentView.has(widget.id))
			)
			.sort((left, right) => {
				const leftHidden = hiddenWidgetIds.has(left.id);
				const rightHidden = hiddenWidgetIds.has(right.id);
				if (leftHidden !== rightHidden) {
					return leftHidden ? -1 : 1;
				}

				return left.title.localeCompare(right.title);
			});
	});

	$effect(() => {
		const sync = () => syncWidgetGridClasses(activeWidgetById);
		let pendingFrame: number | null = null;
		const scheduleSync = () => {
			if (pendingFrame !== null) {
				return;
			}

			pendingFrame = requestAnimationFrame(() => {
				pendingFrame = null;
				sync();
			});
		};

		sync();
		const workspaceRoot =
			document.querySelector<HTMLElement>('.workspace') ??
			document.querySelector<HTMLElement>('.desktop-shell') ??
			document.body;
		const widgetObserver = new MutationObserver(scheduleSync);
		widgetObserver.observe(workspaceRoot, {
			childList: true,
			subtree: true
		});

		window.addEventListener('resize', scheduleSync);

		return () => {
			window.removeEventListener('resize', scheduleSync);
			widgetObserver.disconnect();
			if (pendingFrame !== null) {
				cancelAnimationFrame(pendingFrame);
			}
		};
	});
	$effect(() => {
		if (!isLayoutEditing || (selectedLayoutWidgetId && !activeWidgetById.has(selectedLayoutWidgetId))) {
			selectedLayoutWidgetId = null;
		}
	});
	let providerAccounts = $state<ProviderAccount[]>([]);
	let settingDrafts = $state<Record<string, string>>({});
	let settingsError = $state('');
	let settingsActionMessage = $state('');
	let isSettingsLoading = $state(false);
	let savingSettingKey = $state<string | null>(null);
	let selectedSettingsSection = $state<'appearance' | 'application' | 'sidebar' | 'accounts' | 'language'>('appearance');
	let themeSettings = $state<FrontendThemeSettings>(defaultFrontendThemeSettings());
	let themeDraft = $state<FrontendThemeSettings | null>(null);
	let isThemeSettingsSaving = $state(false);
	let themeError = $state('');
	const effectiveThemeSettings = $derived(themeDraft ?? themeSettings);
	const shellThemeClass = $derived(
		`${shellBackgroundClass(effectiveThemeSettings)} ${shellBrightnessClass(effectiveThemeSettings)} ${shellAccentClass(effectiveThemeSettings)} ${shellPanelOpacityClass(effectiveThemeSettings)} ${shellPanelBlurClass(effectiveThemeSettings)}`
	);

	const primaryNav = $derived.by((): NavItem[] =>
		primaryWorkspaceNav.map((item) => ({ ...item, enabled: true }))
	);
	const sidebarRootEntries = $derived(resolveSidebarRootEntries(primaryNav, effectiveSidebarSettings));
	const sidebarHiddenNavItems = $derived.by(() =>
		effectiveSidebarSettings.hiddenItemIds
			.map((itemId) => sidebarConfigItem(itemId))
			.filter((item): item is { id: SidebarItemId; label: string; icon: string } => item !== null)
	);

	const communicationsNav = $derived.by((): CommunicationNavItem[] =>
		communicationSections.map((item) => ({ ...item, badge: communicationSectionBadge(item.id) }))
	);

	const viewCopy: Record<ViewId, { title: string; subtitle: string; search: string; icon: string }> = {
		home: {
			title: 'Good evening, Alex',
			subtitle: "Here's what's happening in your world today.",
			search: 'Search anything...',
			icon: 'tabler:home'
		},
		communications: {
			title: 'Communications',
			subtitle: 'All your conversations. All channels. One place.',
			search: 'Search in communications...',
			icon: 'tabler:messages'
		},
		timeline: {
			title: 'Timeline',
			subtitle: 'Chronological activity across messages, tasks, documents and meetings.',
			search: 'Search timeline...',
			icon: 'tabler:timeline-event'
		},
		persons: {
			title: 'Persons',
			subtitle: '642 persons',
			search: 'Search persons, companies, emails...',
			icon: 'tabler:address-book'
		},
		projects: {
			title: 'Hermes Hub',
			subtitle: 'Product Development',
			search: 'Search projects, documents, people...',
			icon: 'tabler:cube'
		},
		tasks: {
			title: 'Tasks',
			subtitle: 'All your tasks from connected trackers',
			search: 'Search tasks, projects, trackers, people...',
			icon: 'tabler:checkbox'
		},
		calendar: {
			title: 'Calendar',
			subtitle: 'All your events from connected calendars',
			search: 'Search events, meetings, persons...',
			icon: 'tabler:calendar'
		},
		documents: {
			title: 'Documents',
			subtitle: 'All your documents from connected sources',
			search: 'Search documents, folders, content...',
			icon: 'tabler:file-text'
		},
		notes: {
			title: 'Notes',
			subtitle: 'All your notes from connected sources',
			search: 'Search notes, content, emails...',
			icon: 'tabler:notes'
		},
		knowledge: {
			title: 'Knowledge Graph',
			subtitle: 'Explore relationships across people, projects, documents, messages and tasks.',
			search: 'Search anything in your knowledge graph...',
			icon: 'tabler:share'
		},
		telegram: {
			title: 'Telegram Client',
			subtitle: 'Telegram messaging, policy automation and call intelligence.',
			search: 'Search Telegram chats, policies, calls...',
			icon: 'tabler:brand-telegram'
		},
		whatsapp: {
			title: 'WhatsApp Web',
			subtitle: 'WhatsApp companion sessions, fixture ingestion and live-runtime guardrails.',
			search: 'Search WhatsApp sessions and messages...',
			icon: 'tabler:brand-whatsapp'
		},
		agents: {
			title: 'AI Agents',
			subtitle: 'Your intelligent assistants working across your data and tools',
			search: 'Search agents, capabilities, tasks...',
			icon: 'tabler:sparkles'
		},
		organizations: {
			title: 'Companies',
			subtitle: 'All companies and organizations from your communications',
			search: 'Search companies, industries, locations...',
			icon: 'tabler:building'
		},
		settings: {
			title: 'Settings',
			subtitle: 'Runtime settings and connected accounts.',
			search: 'Search settings and accounts...',
			icon: 'tabler:settings'
		}
	};

	const homeStats = $derived.by(() => {
		const stats: StatCard[] = [];
		if (mailboxHealth) {
			stats.push({ label: 'Messages', value: String(mailboxHealth.total_messages), delta: `+${mailboxHealth.unread}`, icon: 'tabler:mail' });
			stats.push({ label: 'Needs attention', value: String(mailboxHealth.needs_action), delta: `+${mailboxHealth.important}`, icon: 'tabler:alert-circle' });
			stats.push({ label: 'Waiting', value: String(mailboxHealth.waiting), delta: `${mailboxHealth.done} done`, icon: 'tabler:message-reply' });
		}
		stats.push({ label: 'Projects', value: String(projectSummaries.length), delta: 'active', icon: 'tabler:briefcase' });
		stats.push({ label: 'Persons', value: String(persons.length), delta: 'enriched', icon: 'tabler:user-plus' });
		return stats;
	});

	const whatsNew = $derived.by(() => {
		const items: FeedItem[] = [];
		const channelIcons: Record<string, string> = {
			email: 'tabler:mail',
			gmail: 'tabler:brand-gmail',
			icloud: 'tabler:cloud',
			imap: 'tabler:server',
			telegram_user: 'tabler:brand-telegram',
			telegram_bot: 'tabler:brand-telegram',
			whatsapp_web: 'tabler:brand-whatsapp'
		};
		for (const msg of communicationMessages.slice(0, 5)) {
			const sender = msg.sender_display_name || msg.sender || 'Unknown';
			items.push({
				icon: channelIcons[msg.channel_kind] || 'tabler:message',
				title: `New message from ${sender}`,
				meta: msg.subject || msg.body_text_preview,
				time: msg.occurred_at || msg.projected_at,
				tag: msg.subject ? undefined : undefined,
				tone: 'blue'
			});
		}
		return items;
	});

	const peopleTalked = $derived.by(() => {
		const seen = new Set<string>();
		const result: { name: string; meta: string; icon: string }[] = [];
		for (const msg of communicationMessages) {
			const sender = msg.sender_display_name || msg.sender || 'Unknown';
			if (seen.has(sender)) continue;
			seen.add(sender);
			result.push({
				name: sender,
				meta: msg.subject || msg.body_text_preview,
				icon: 'tabler:message'
			});
			if (result.length >= 5) break;
		}
		return result;
	});


	const conversations = $derived.by(() => {
		const channelLabels: Record<string, string> = {
			email: 'Email', gmail: 'Gmail', icloud: 'iCloud', imap: 'IMAP',
			telegram_user: 'Telegram', telegram_bot: 'Telegram',
			whatsapp_web: 'WhatsApp'
		};
		return communicationMessages.map((msg) => ({
			name: msg.sender_display_name || msg.sender || 'Unknown',
			role: msg.sender || '',
			project: msg.subject || msg.body_text_preview,
			channel: channelLabels[msg.channel_kind] || msg.channel_kind,
			time: msg.occurred_at || msg.projected_at,
			preview: msg.body_text_preview
		}));
	});
	const personList = $derived.by(() =>
		persons.map((p) => ({
			name: p.display_name,
			role: p.preferred_channel || 'Contact',
			company: p.email_address,
			status: p.last_interaction_at ? 'Online' : undefined,
			channel: p.preferred_channel ?? undefined
		}))
	);

	const projects = $derived.by(() =>
		projectSummaries.map((ps) => ({
			name: ps.project.name,
			kind: ps.project.kind,
			progress: ps.project.progress_percent,
			tasks: ps.stats.message_count + ps.stats.document_count,
			icon: 'tabler:cube' as const,
			tone: ps.project.status === 'active' ? 'cyan' as const : 'blue' as const
		}))
	);

	const tasks = $derived.by(() => {
		const all: TaskItem[] = [];
		for (const tc of taskCandidates) {
			all.push({
				title: tc.title,
				tracker: tc.source_kind,
				project: tc.project_id || 'Unassigned',
				assignee: tc.assignee_label || 'Unassigned',
				status: tc.review_state === 'suggested' ? 'To Review' : tc.review_state === 'user_confirmed' ? 'Active' : 'Rejected',
				priority: tc.confidence > 0.7 ? 'High' : 'Medium',
				due: tc.due_text || 'No deadline',
				group: tc.review_state === 'suggested' ? 'Review Queue' : 'Active'
			});
		}
		for (const at of activeTasks) {
			all.push({
				title: at.title,
				tracker: at.source_kind,
				project: at.project_id || 'Unassigned',
				assignee: 'Active',
				status: 'Active',
				priority: 'High',
				due: 'Active',
				group: 'Active'
			});
		}
		return all;
	});

	const documents = $derived.by(() =>
		documentProcessingJobs.map((job) => ({
			name: `${job.document_id} (${job.step})`,
			source: 'Hermes Hub',
			project: job.status,
			type: job.step,
			date: job.queued_at,
			size: job.last_error_summary || 'No errors',
			icon: 'tabler:file-text' as const,
			tone: job.status === 'succeeded' ? 'green' as const : job.status === 'failed' ? 'red' as const : 'amber' as const
		}))
	);
	const notes = [
		{ title: 'Hermes Hub - Product Strategy', body: 'Основные принципы: единое пространство памяти, интеграция всех коммуникаций...', source: 'Apple Notes', tag: '#project', time: '10:42', icon: 'tabler:notes' },
		{ title: 'User Research Summary', body: 'Ключевые инсайты из интервью с пользователями...', source: 'Obsidian', tag: '#research', time: '09:15', icon: 'tabler:file-text' },
		{ title: 'Meeting with Maria - 13 May 2024', body: 'Обсудили roadmap, приоритеты и сроки запуска новых функций...', source: 'Gmail', tag: '#meeting', time: '08:27', icon: 'tabler:brand-gmail' },
		{ title: 'Quick Ideas', body: '- AI для автоматической категоризации заметок - Граф связей между проектами...', source: 'Anytype', tag: '#idea', time: '07:58', icon: 'tabler:bulb' },
		{ title: 'Integration Architecture', body: 'Схема интеграции с внешними сервисами и потоками данных...', source: 'Obsidian', tag: '#reference', time: 'May 12, 18:45', icon: 'tabler:file-text' },
		{ title: 'Email: Partnership Opportunity', body: 'Интересное предложение о партнерстве. Нужно обсудить с командой...', source: 'Outlook', tag: '#partnership', time: 'May 12, 16:20', icon: 'tabler:mail' }
	];

	const weekDays = ['MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN'];
	const nowDate = new Date();
	const weekStart = new Date(nowDate);
	weekStart.setDate(nowDate.getDate() - nowDate.getDay() + 1);
	weekStart.setHours(0, 0, 0, 0);
	const weekColumns = weekDays.map((d, i) => {
		const d2 = new Date(weekStart); d2.setDate(weekStart.getDate() + i);
		return `${d} ${d2.getDate()}`;
	});
	const filteredEvents = $derived(calendarEvents.filter(e => {
		const start = new Date(e.start_at);
		const end = new Date(weekStart); end.setDate(weekStart.getDate() + 7);
		return start >= weekStart && start < end;
	}));

	const selectedCommunication = $derived(communicationMessages[selectedConversationIndex] ?? null);
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
	const notificationItems = $derived.by(() => {
		const dismissed = new Set(dismissedNotificationIds);
		const items: NotificationItem[] = demoNotificationItems.filter(
			(notification) => !dismissed.has(notification.id)
		);
		for (const message of telegramMessages.slice(0, 5)) {
			if (message.delivery_state !== 'received') {
				continue;
			}
			const id = `telegram:${message.message_id}`;
			if (dismissed.has(id)) {
				continue;
			}
			items.push({
				id,
				source: 'telegram',
				icon: 'tabler:brand-telegram',
				title: message.chat_title || 'Telegram',
				body: message.text || 'New Telegram message',
				meta: senderLabel(message.sender_display_name || message.sender),
				time: message.occurred_at || message.projected_at,
				messageId: message.message_id,
				accountId: message.account_id,
				providerChatId: message.provider_chat_id,
				targetSection: 'telegram'
			});
		}
		for (const message of communicationMessages.slice(0, 5)) {
			const workflowState = (message as CommunicationMessageSummaryV2).workflow_state;
			if (workflowState !== 'new' && workflowState !== 'needs_action') {
				continue;
			}
			const id = `communication:${message.message_id}`;
			if (dismissed.has(id)) {
				continue;
			}
			items.push({
				id,
				source: 'communications',
				icon: communicationChannelIcon(message.channel_kind),
				title: message.subject || 'New communication',
				body: message.body_text_preview || senderLabel(message.sender),
				meta: senderLabel(message.sender_display_name || message.sender),
				time: message.occurred_at || message.projected_at,
				messageId: message.message_id,
				accountId: message.account_id,
				targetSection: 'unified'
			});
		}
		return items
			.sort((left, right) => new Date(right.time ?? 0).getTime() - new Date(left.time ?? 0).getTime())
			.slice(0, 12);
	});
	const telegramClosureCapabilities = $derived(
		telegramCapabilities?.capabilities.filter((capability) => capability.closure_gate) ?? []
	);
	const telegramBlockedCapabilities = $derived(
		telegramCapabilities?.capabilities.filter((capability) => capability.status === 'blocked') ?? []
	);
	const telegramQrRuntimeBlocked = $derived(
		telegramAuthMethod === 'qr' &&
			telegramCapabilities !== null &&
			!telegramCapabilities.tdjson_runtime_available
	);
	const telegramQrNeedsFormAppCredentials = $derived(
		telegramAuthMethod === 'qr' &&
			!(telegramCapabilities?.telegram_app_credentials_configured ?? false)
	);
	const telegramNeedsFormAppCredentials = $derived(
		telegramAuthMethod === 'phone' || telegramQrNeedsFormAppCredentials
	);
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
	const selectedConversation = $derived(conversations[selectedConversationIndex] ?? conversations[0]);
	const selectedPerson = $derived(personList[selectedPersonIndex] ?? personList[0]);
	const selectedOrganization = $derived(organizations.find(o => o.organization_id === selectedOrganizationId) ?? organizations[0]);
	const orgPeople = $derived.by(() => persons.filter(p => p.linked_projects?.some(pid => selectedOrganization?.display_name && pid.includes(selectedOrganization.display_name))).slice(0, 5));
	const agentCards = $derived(aiAgents.map(agentCardView));
	const selectedAgent = $derived(agentCards[selectedAgentIndex] ?? agentCards[0] ?? null);
	const activeView = $derived(viewCopy[activeWorkspaceView]);
	const selectedGraphNode = $derived(graphNeighborhood?.selected_node ?? null);
	const graphCanvasNodes = $derived(buildGraphCanvasNodes(graphNeighborhood));
	const graphCanvasEdges = $derived(buildGraphCanvasEdges(graphNeighborhood, graphCanvasNodes));
	const selectedGraphProperties = $derived(
		selectedGraphNode ? graphPropertyRows(selectedGraphNode.properties) : []
	);
	const graphNeighborCounts = $derived(graphKindCounts(graphNeighborNodes(graphNeighborhood)));
	const graphFilterChips = $derived(buildGraphFilterChips(graphSummary));
	const selectedProjectSummary = $derived(
		projectSummaries.find((item) => item.project.project_id === selectedProjectId) ??
			projectSummaries[0] ??
			null
	);
	const selectedProjectRecord = $derived(
		selectedProjectDetail?.project ?? selectedProjectSummary?.project ?? null
	);
	const selectedProjectStats = $derived(
		selectedProjectDetail?.stats ?? selectedProjectSummary?.stats ?? emptyProjectStats()
	);
	const relatedProjectSummaries = $derived(
		projectSummaries.filter((item) => item.project.project_id !== selectedProjectRecord?.project_id)
	);
	const suggestedTaskCandidates = $derived(
		taskCandidates.filter((item) => item.review_state === 'suggested')
	);
	const suggestedIdentityCandidates = $derived(
		identityCandidates.filter((item) => item.review_state === 'suggested')
	);
	const confirmedMergeIdentityCandidates = $derived(
		identityCandidates.filter(
			(item) =>
				item.candidate_kind === 'merge_persons' &&
				item.review_state === 'user_confirmed' &&
				!confirmedSplitCandidateForMerge(item)
		)
	);
	const settingsByCategory = $derived(
		groupSettingsByCategory(
			applicationSettings.filter(
				(setting) =>
					setting.setting_key !== 'frontend.sidebar' &&
					setting.setting_key !== FRONTEND_THEME_SETTING_KEY
			)
		)
	);
	const emailProviderAccounts = $derived(
		providerAccounts.filter((account) => ['gmail', 'icloud', 'imap'].includes(account.provider_kind))
	);
	const telegramProviderAccounts = $derived(
		providerAccounts.filter((account) =>
			['telegram_user', 'telegram_bot'].includes(account.provider_kind)
		)
	);
	const whatsappProviderAccounts = $derived(
		providerAccounts.filter((account) => account.provider_kind === 'whatsapp_web')
	);
	const vaultStatus = $derived(status?.vault_status ?? null);
	const isVaultReady = $derived(vaultStatus?.state === 'locked' || vaultStatus?.state === 'unlocked');

	onMount(() => {
		void loadV1Status();
		void loadGraphSummary();
		void loadGraphNodeChoices();
		void loadCommunications();
		void loadDocumentProcessingJobs();
		void loadProjects();
		void loadIdentityCandidates();
		void loadPersons();
		void loadOrganizations();
		void loadTaskReviewState();
		void loadAiWorkspace();
		void loadTelegramWorkspace();
		void loadWhatsappWebWorkspace();
		void loadSettingsWorkspace();
	});

	async function loadV1Status() {
		try {
			status = await fetchV1Status(apiBaseUrl, apiSecret);
			statusError = '';
			if (status.vault_status.state === 'uninitialized') {
				vaultWizardStep = status.vault_status.entropy_progress >= 100 ? 'biometric' : 'intro';
			}
		} catch (error) {
			statusError = error instanceof Error ? error.message : 'Unknown status error';
		}
	}

	function startVaultWizard() {
		vaultWizardStep = 'entropy';
		vaultWizardError = '';
		vaultWizardMessage = '';
	}

	async function handleVaultEntropyMove(event: MouseEvent) {
		if (vaultWizardStep !== 'entropy' || isVaultActionSubmitting) {
			return;
		}
		const previous = vaultLastEntropyEvent;
		const interval = previous ? Math.max(1, event.timeStamp - previous.timestamp_ms) : 1;
		const dx = previous ? event.clientX - previous.x : 0;
		const dy = previous ? event.clientY - previous.y : 0;
		const velocity = Math.hypot(dx, dy) / interval;
		const acceleration = previous ? Math.abs(velocity - previous.velocity) / interval : 0;
		const entropyEvent: VaultEntropyEvent = {
			x: event.clientX,
			y: event.clientY,
			dx,
			dy,
			timestamp_ms: event.timeStamp,
			velocity,
			acceleration,
			interval_ms: interval
		};
		vaultLastEntropyEvent = entropyEvent;
		vaultEntropyEvents = [...vaultEntropyEvents, entropyEvent].slice(-2000);
		vaultEntropyBuffer.push(entropyEvent);
		if (vaultEntropyBuffer.length >= 100) {
			await flushVaultEntropy();
		}
	}

	async function flushVaultEntropy() {
		if (vaultEntropyBuffer.length === 0) {
			return;
		}
		const events = vaultEntropyBuffer;
		vaultEntropyBuffer = [];
		try {
			const vault_status = await collectVaultEntropy(apiBaseUrl, apiSecret, events);
			status = status ? { ...status, vault_status } : status;
			if (vault_status.entropy_progress >= 100) {
				vaultWizardStep = 'biometric';
			}
		} catch (error) {
			vaultWizardError = error instanceof Error ? error.message : 'Vault entropy failed';
		}
	}

	async function createSecureVault() {
		if (isVaultActionSubmitting) {
			return;
		}
		isVaultActionSubmitting = true;
		vaultWizardError = '';
		try {
			await flushVaultEntropy();
			const vault_status = await createVault(apiBaseUrl, apiSecret);
			status = status ? { ...status, vault_status } : status;
			vaultWizardStep = 'recovery';
			vaultWizardMessage = 'Vault created. Export recovery material before continuing.';
		} catch (error) {
			vaultWizardError = error instanceof Error ? error.message : 'Vault create failed';
		} finally {
			isVaultActionSubmitting = false;
		}
	}

	async function unlockSecureVault() {
		if (isVaultActionSubmitting) {
			return;
		}
		isVaultActionSubmitting = true;
		vaultWizardError = '';
		try {
			const vault_status = await unlockVault(apiBaseUrl, apiSecret);
			status = status ? { ...status, vault_status } : status;
			vaultWizardMessage = 'Vault unlocked for this Hermes session.';
		} catch (error) {
			vaultWizardError = error instanceof Error ? error.message : 'Vault unlock failed';
		} finally {
			isVaultActionSubmitting = false;
		}
	}

	async function exportRecoveryMaterial() {
		if (isVaultActionSubmitting) {
			return;
		}
		isVaultActionSubmitting = true;
		vaultWizardError = '';
		try {
			vaultRecovery = await exportVaultRecovery(apiBaseUrl, apiSecret);
			vaultWizardStep = 'done';
			vaultWizardMessage = 'Recovery material exported. Store it outside the app.';
		} catch (error) {
			vaultWizardError = error instanceof Error ? error.message : 'Vault recovery export failed';
		} finally {
			isVaultActionSubmitting = false;
		}
	}

	async function loadSettingsWorkspace() {
		isSettingsLoading = true;
		try {
			const [settingsResponse, accountsResponse] = await Promise.all([
				fetchApplicationSettings(apiBaseUrl, apiSecret),
				fetchProviderAccounts(apiBaseUrl, apiSecret)
			]);
			applicationSettings = settingsResponse.items;
			const frontendLayoutSetting = findFrontendLayoutSetting(settingsResponse.items);
			const frontendSidebarSetting = findFrontendSidebarSetting(settingsResponse.items);
			const frontendThemeSetting = findFrontendThemeSetting(settingsResponse.items);
			layoutSettings = parseLayoutSettings(frontendLayoutSetting?.value ?? null);
			sidebarSettings = parseSidebarSettings(frontendSidebarSetting?.value ?? null);
			themeSettings = parseFrontendThemeSettings(frontendThemeSetting?.value ?? null);
			sidebarDraft = null;
			themeDraft = null;
			const frontendLocaleSetting = findFrontendLocaleSetting(settingsResponse.items);
			if (frontendLocaleSetting?.value === 'ru' || frontendLocaleSetting?.value === 'en') {
				setLocale(frontendLocaleSetting.value);
			}
			layoutError = '';
			sidebarError = '';
			themeError = '';
			providerAccounts = accountsResponse.items;
			settingDrafts = Object.fromEntries(
				settingsResponse.items.map((setting) => [setting.setting_key, settingDraftValue(setting)])
			);
			settingsError = '';
		} catch (error) {
			layoutSettings = defaultLayoutSettings();
			sidebarSettings = defaultSidebarSettings();
			themeSettings = defaultFrontendThemeSettings();
			layoutError = error instanceof Error ? error.message : 'Unknown layout settings error';
			sidebarError = error instanceof Error ? error.message : 'Unknown sidebar settings error';
			themeError = error instanceof Error ? error.message : 'Unknown appearance settings error';
			settingsError = error instanceof Error ? error.message : 'Unknown settings error';
		} finally {
			isSettingsLoading = false;
		}
	}

	async function saveSetting(setting: ApplicationSetting) {
		const draft = settingDrafts[setting.setting_key] ?? '';
		let nextValue: ApplicationSetting['value'];
		try {
			nextValue = settingDraftToValue(setting, draft);
		} catch (error) {
			settingsError = error instanceof Error ? error.message : 'Invalid setting value';
			return;
		}

		savingSettingKey = setting.setting_key;
		try {
			const updated = await saveApplicationSetting(
				apiBaseUrl, apiSecret,
				setting.setting_key,
				nextValue
			);
			applicationSettings = applicationSettings.map((item) =>
				item.setting_key === updated.setting_key ? updated : item
			);
			settingDrafts = {
				...settingDrafts,
				[updated.setting_key]: settingDraftValue(updated)
			};
			if (updated.setting_key === 'frontend.layout') {
				layoutSettings = parseLayoutSettings(updated.value);
				layoutDraft = null;
				layoutError = '';
			}
			if (updated.setting_key === 'frontend.sidebar') {
				sidebarSettings = parseSidebarSettings(updated.value);
				sidebarDraft = null;
				sidebarError = '';
			}
			if (updated.setting_key === FRONTEND_THEME_SETTING_KEY) {
				themeSettings = parseFrontendThemeSettings(updated.value);
				themeDraft = null;
				themeError = '';
			}
			settingsActionMessage = `${updated.label} saved`;
			settingsError = '';
			if (updated.setting_key.startsWith('ai.')) {
				void loadAiWorkspace();
			}
		} catch (error) {
			settingsError = error instanceof Error ? error.message : 'Unknown setting update error';
		} finally {
			savingSettingKey = null;
		}
	}

	async function loadGraphSummary() {
		isGraphSummaryLoading = true;
		try {
			graphSummary = await fetchGraphSummary(apiBaseUrl, apiSecret);
			graphError = '';
		} catch (error) {
			graphError = error instanceof Error ? error.message : 'Unknown graph summary error';
		} finally {
			isGraphSummaryLoading = false;
		}
	}

	async function loadGraphNodeChoices() {
		const requestSequence = ++graphNodeChoicesRequestSequence;
		isGraphNodeChoicesLoading = true;
		try {
			const nodes = await fetchGraphNodes(apiBaseUrl, apiSecret, 20);
			if (requestSequence !== graphNodeChoicesRequestSequence) {
				return;
			}
			graphNodeChoices = nodes;
			graphNodeChoicesError = '';
		} catch (error) {
			if (requestSequence !== graphNodeChoicesRequestSequence) {
				return;
			}
			graphNodeChoices = [];
			graphNodeChoicesError = error instanceof Error ? error.message : 'Unknown graph node picker error';
		} finally {
			if (requestSequence === graphNodeChoicesRequestSequence) {
				isGraphNodeChoicesLoading = false;
			}
		}
	}

	async function runGraphSearch() {
		const requestSequence = ++graphSearchRequestSequence;
		const query = graphSearchQuery.trim();
		graphSearchSubmitted = true;
		lastSubmittedGraphSearchQuery = query;

		if (!query) {
			graphSearchResults = [];
			graphSearchError = '';
			isGraphSearchLoading = false;
			return;
		}

		isGraphSearchLoading = true;
		try {
			const results = await searchGraphNodes(apiBaseUrl, apiSecret, query, 20);
			if (requestSequence !== graphSearchRequestSequence) {
				return;
			}
			graphSearchResults = results;
			graphSearchError = '';
		} catch (error) {
			if (requestSequence !== graphSearchRequestSequence) {
				return;
			}
			graphSearchResults = [];
			graphSearchError = error instanceof Error ? error.message : 'Unknown graph search error';
		} finally {
			if (requestSequence === graphSearchRequestSequence) {
				isGraphSearchLoading = false;
			}
		}
	}

	async function selectGraphNode(node: GraphNode) {
		const requestSequence = ++graphNeighborhoodRequestSequence;
		graphNeighborhoodError = '';
		graphNeighborhood = null;
		isGraphNeighborhoodLoading = true;
		try {
			const neighborhood = await fetchGraphNeighborhood(
				apiBaseUrl, apiSecret,
				node.node_id,
				1
			);
			if (requestSequence !== graphNeighborhoodRequestSequence) {
				return;
			}
			graphNeighborhood = neighborhood;
		} catch (error) {
			if (requestSequence !== graphNeighborhoodRequestSequence) {
				return;
			}
			graphNeighborhood = null;
			graphNeighborhoodError =
				error instanceof Error ? error.message : 'Unknown graph neighborhood error';
		} finally {
			if (requestSequence === graphNeighborhoodRequestSequence) {
				isGraphNeighborhoodLoading = false;
			}
		}
	}


	async function loadCommunicationMessagesFiltered(filterState?: WorkflowState) {
		try {
			isCommunicationsLoading = true;
			communicationsError = '';
			const response = await fetchMailMessages(
				apiBaseUrl, apiSecret,
				undefined, filterState || undefined, undefined, 50
			);
			communicationMessages = response.items as unknown as CommunicationMessageSummary[];
			if (selectedConversationIndex >= communicationMessages.length) {
				selectedConversationIndex = Math.max(0, communicationMessages.length - 1);
			}
			if (communicationMessages.length > 0) {
				await loadCommunicationDetail(communicationMessages[selectedConversationIndex].message_id);
			} else {
				selectedCommunicationDetail = null;
			}
		} catch (error) {
			communicationsError = error instanceof Error ? error.message : 'Unknown communications error';
			selectedCommunicationDetail = null;
		} finally {
			isCommunicationsLoading = false;
		}
	}

	async function loadMessageStateCounts() {
		try {
			const response = await fetchMessageStateCounts(apiBaseUrl, apiSecret);
			mailStateCounts = response.counts;
		} catch {
			mailStateCounts = [];
		}
	}

	async function handleWorkflowStateTransition(messageId: string, newState: WorkflowState) {
		try {
			isMailStateTransitioning = true;
			mailStateError = '';
			await transitionMessageWorkflowState(apiBaseUrl, apiSecret, messageId, newState);
			await loadCommunicationMessagesFiltered(mailStateFilter || undefined);
			await loadMessageStateCounts();
		await loadMailboxHealth();
		await loadTopSenders();
		await loadDrafts();
		await loadThreads();
		} catch (error) {
			mailStateError = error instanceof Error ? error.message : 'State transition failed';
		} finally {
			isMailStateTransitioning = false;
		}
	}


	async function loadDrafts() {
		try { const r = await fetchDrafts(apiBaseUrl, apiSecret); drafts = r.items; } catch { drafts = []; }
	}
	async function loadMailboxHealth() {
		try { mailboxHealth = await fetchMailboxHealth(apiBaseUrl, apiSecret); } catch { mailboxHealth = null; }
	}
	async function loadTopSenders() {
		try { topSenders = await fetchTopSenders(apiBaseUrl, apiSecret); } catch { topSenders = []; }
	}
	async function loadThreads() {
		try { const r = await fetchThreads(apiBaseUrl, apiSecret); threads = r.items; } catch { threads = []; }
	}
	async function handleAnalyzeMessage(messageId: string) {
		try { isAnalyzing = true; aiAnalysisResult = await analyzeMessage(apiBaseUrl, apiSecret, messageId); } catch { aiAnalysisResult = null; } finally { isAnalyzing = false; }
	}
	async function handleSaveDraft() {
		if (!composeForm.draft_id || !composeForm.subject) return;
		try {
			await createDraft(apiBaseUrl, apiSecret, {
				draft_id: composeForm.draft_id,
				account_id: composeForm.account_id || 'gmail-primary',
				to_recipients: composeForm.to_text.split(',').map(s => s.trim()).filter(Boolean),
				cc_recipients: composeForm.cc_text.split(',').map(s => s.trim()).filter(Boolean),
				subject: composeForm.subject,
				body_text: composeForm.body,
				status: 'draft',
			});
			composeForm = { draft_id: '', account_id: '', to_text: '', cc_text: '', subject: '', body: '' };
			isComposeOpen = false;
			await loadDrafts();
		} catch (e) { /* ignore */ }
	}
	async function loadCommunications() {
		isCommunicationsLoading = true;
		try {
			const response = await fetchCommunicationMessages(apiBaseUrl, apiSecret, 50);
			communicationMessages = response.items;
			communicationsError = '';
			if (selectedConversationIndex >= communicationMessages.length) {
				selectedConversationIndex = 0;
			}
			if (communicationMessages.length > 0) {
				await loadCommunicationDetail(communicationMessages[selectedConversationIndex].message_id);
			} else {
				selectedCommunicationDetail = null;
			}
		} catch (error) {
			communicationsError =
				error instanceof Error ? error.message : 'Unknown communications error';
			selectedCommunicationDetail = null;
		} finally {
			isCommunicationsLoading = false;
		}
	}

	async function loadProjects() {
		const requestSequence = ++projectRequestSequence;
		isProjectsLoading = true;
		try {
			const response = await fetchProjects(apiBaseUrl, apiSecret, 25);
			if (requestSequence !== projectRequestSequence) {
				return;
			}
			projectSummaries = response.items;
			projectsError = '';
			const nextProjectId =
				selectedProjectId || response.items[0]?.project.project_id || '';
			selectedProjectId = nextProjectId;
			if (nextProjectId) {
				await loadProjectDetail(nextProjectId, requestSequence);
			} else {
				selectedProjectDetail = null;
			}
		} catch (error) {
			if (requestSequence !== projectRequestSequence) {
				return;
			}
			projectsError = error instanceof Error ? error.message : 'Unknown projects error';
			selectedProjectDetail = null;
		} finally {
			if (requestSequence === projectRequestSequence) {
				isProjectsLoading = false;
			}
		}
	}

	async function loadTaskReviewState() {
		isTasksLoading = true;
		try {
			const [candidateResponse, taskResponse] = await Promise.all([
				fetchTaskCandidates(apiBaseUrl, apiSecret, 50),
				fetchTaskRecords(apiBaseUrl, apiSecret, { limit: 50 })
			]);
			taskCandidates = candidateResponse.items;
			activeTasks = taskResponse.items;
			tasksError = '';
		} catch (error) {
			tasksError = error instanceof Error ? error.message : 'Unknown task candidate error';
		} finally {
			isTasksLoading = false;
		}
	}


	async function loadPersons() {
		isPersonsLoading = true;
		try {
			const response = await fetchPersons(apiBaseUrl, apiSecret);
			persons = response.items;
			personsError = '';
		} catch (error) {
			personsError = error instanceof Error ? error.message : 'Unknown persons error';
			persons = [];
		} finally {
			isPersonsLoading = false;
		}
	}

	async function loadOrganizations() {
		isOrganizationsLoading = true;
		try {
			const response = await fetchOrganizations(apiBaseUrl, apiSecret);
			organizations = response.items;
			organizationsError = '';
		} catch (error) {
			organizationsError = error instanceof Error ? error.message : 'Unknown organizations error';
			organizations = [];
		} finally {
			isOrganizationsLoading = false;
		}
	}

	async function loadCalendar() {
		isCalendarLoading = true;
		try {
			const [accts, events] = await Promise.all([
				fetchCalendarAccounts(apiBaseUrl, apiSecret),
				fetchCalendarEvents(apiBaseUrl, apiSecret, { limit: 200 })
			]);
			calendarAccounts = accts.items;
			calendarEvents = events.items;
			calendarSources = [];
			for (const acct of calendarAccounts) {
				try {
					const srcs = await fetchCalendarSources(apiBaseUrl, apiSecret, acct.account_id);
					calendarSources.push(...srcs.items);
				} catch (_) { /* sources optional */ }
			}
			fetchCalendarWatchtower(apiBaseUrl, apiSecret).then(r => calendarWatchtower = r).catch(() => {});
			calendarError = '';
		} catch (error) {
			calendarError = error instanceof Error ? error.message : 'Calendar load failed';
			calendarAccounts = [];
			calendarEvents = [];
		} finally {
			isCalendarLoading = false;
		}
	}

	function getEventTimeRange(): { from: string; to: string } {
		const now = new Date();
		const from = new Date(now);
		if (calendarViewMode === 'day') { from.setHours(0, 0, 0, 0); }
		else if (calendarViewMode === 'week') { from.setDate(now.getDate() - now.getDay() + 1); from.setHours(0, 0, 0, 0); }
		else { from.setDate(1); from.setHours(0, 0, 0, 0); }
		const to = new Date(from);
		if (calendarViewMode === 'day') to.setDate(to.getDate() + 1);
		else if (calendarViewMode === 'week') to.setDate(to.getDate() + 7);
		else to.setMonth(to.getMonth() + 1);
		return { from: from.toISOString(), to: to.toISOString() };
	}

	async function prepareEvent(evt: CalendarEvent) {
		selectedEvent = evt;
		try {
			const [ctx, brief, agenda] = await Promise.all([
				fetchEventContextPack(apiBaseUrl, apiSecret, evt.event_id),
				fetchEventBrief(apiBaseUrl, apiSecret, evt.event_id),
				fetchEventAgenda(apiBaseUrl, apiSecret, evt.event_id),
			]);
			eventContext = ctx;
			eventBrief = brief;
			eventAgenda = agenda;
		} catch (_) { eventBrief = null; }
	}

	async function completeEvent(evt: CalendarEvent) {
		selectedEvent = evt;
		try {
			const notes = await fetchMeetingNotes(apiBaseUrl, apiSecret, evt.event_id);
			eventContext = { notes: notes.items };
		} catch (_) {}
	}

	async function searchCalendar() {
		if (!calendarSearchQuery.trim()) { calendarSearchResults = []; return; }
		try {
			const result = await searchCalendarEvents(apiBaseUrl, apiSecret, calendarSearchQuery);
			calendarSearchResults = (result.results as CalendarEvent[]) || [];
		} catch (_) { calendarSearchResults = []; }
	}

	async function loadWeeklyBrief() {
		try {
			const brief = await fetchWeeklyBrief(apiBaseUrl, apiSecret);
			weeklyBrief = brief;
		} catch (_) { weeklyBrief = null; }
	}

	async function handleCreateEvent() {
		if (!newEventTitle || !newEventStart || !newEventEnd) return;
		try {
			await createCalendarEvent(apiBaseUrl, apiSecret, {
				title: newEventTitle, start_at: new Date(newEventStart).toISOString(),
				end_at: new Date(newEventEnd).toISOString(), event_type: newEventType
			});
			showNewEventForm = false;
			newEventTitle = '';
			await loadCalendar();
		} catch (e) { calendarError = e instanceof Error ? e.message : 'Create failed'; }
	}

	async function loadAiWorkspace() {
		isAiLoading = true;
		try {
			const [agentResponse, runResponse] = await Promise.all([
				fetchAiAgents(apiBaseUrl, apiSecret),
				fetchAiRuns(apiBaseUrl, apiSecret, 25)
			]);
			aiAgents = agentResponse.items;
			aiRuns = runResponse.items;
			if (selectedAgentIndex >= aiAgents.length) {
				selectedAgentIndex = 0;
			}
			aiError = '';
			try {
				aiStatus = await fetchAiStatus(apiBaseUrl, apiSecret);
			} catch (statusError) {
				aiStatus = null;
				aiError =
					statusError instanceof Error ? statusError.message : 'Unknown AI status error';
			}
		} catch (error) {
			aiError = error instanceof Error ? error.message : 'Unknown AI runtime error';
		} finally {
			isAiLoading = false;
		}
	}

	async function submitAiAnswer() {
		const query = aiQuestion.trim();
		if (!query || isAiAnswerSubmitting) {
			return;
		}
		isAiAnswerSubmitting = true;
		aiError = '';
		try {
			aiAnswerResult = await requestAiAnswer(apiBaseUrl, apiSecret, {
				command_id: `ai-answer-${crypto.randomUUID()}`,
				query,
				agent_id: selectedAgent?.agentId ?? 'MNEMOSYNE'
			});
			await loadAiRunsOnly();
		} catch (error) {
			aiError = error instanceof Error ? error.message : 'Unknown AI answer error';
		} finally {
			isAiAnswerSubmitting = false;
		}
	}

	async function refreshTasksFromAi() {
		const query = aiTaskQuery.trim();
		if (!query || isAiTaskRefreshSubmitting) {
			return;
		}
		isAiTaskRefreshSubmitting = true;
		aiError = '';
		try {
			aiTaskRefreshResult = await refreshAiTaskCandidates(apiBaseUrl, apiSecret, {
				command_id: `ai-task-refresh-${crypto.randomUUID()}`,
				query
			});
			await Promise.all([loadTaskReviewState(), loadAiRunsOnly()]);
		} catch (error) {
			aiError = error instanceof Error ? error.message : 'Unknown AI task refresh error';
		} finally {
			isAiTaskRefreshSubmitting = false;
		}
	}

	async function prepareAiBrief(projectId = selectedProjectRecord?.project_id) {
		const topic = aiMeetingTopic.trim();
		if (!topic || isAiMeetingPrepSubmitting) {
			return;
		}
		isAiMeetingPrepSubmitting = true;
		aiError = '';
		try {
			aiMeetingPrepResult = await requestAiMeetingPrep(apiBaseUrl, apiSecret, {
				command_id: `ai-meeting-prep-${crypto.randomUUID()}`,
				topic,
				project_id: projectId
			});
			setCurrentView('agents');
			await loadAiRunsOnly();
		} catch (error) {
			aiError = error instanceof Error ? error.message : 'Unknown AI meeting prep error';
		} finally {
			isAiMeetingPrepSubmitting = false;
		}
	}

	async function loadAiRunsOnly() {
		try {
			const response = await fetchAiRuns(apiBaseUrl, apiSecret, 25);
			aiRuns = response.items;
		} catch (error) {
			aiError = error instanceof Error ? error.message : 'Unknown AI run history error';
		}
	}

	async function loadIdentityCandidates() {
		isIdentityCandidatesLoading = true;
		try {
			const response = await fetchIdentityCandidates(apiBaseUrl, apiSecret, 50);
			identityCandidates = response.items;
			identityCandidatesError = '';
		} catch (error) {
			identityCandidatesError =
				error instanceof Error ? error.message : 'Unknown identity candidate error';
		} finally {
			isIdentityCandidatesLoading = false;
		}
	}

	async function loadDocumentProcessingJobs() {
		isDocumentProcessingJobsLoading = true;
		try {
			const response = await fetchDocumentProcessingJobs(apiBaseUrl, apiSecret, 50);
			documentProcessingJobs = response.items;
			documentProcessingJobsError = '';
		} catch (error) {
			documentProcessingJobsError =
				error instanceof Error ? error.message : 'Unknown document processing jobs error';
		} finally {
			isDocumentProcessingJobsLoading = false;
		}
	}

	async function reloadSelectedDocumentProcessingDetail() {
		const documentId = selectedDocumentProcessingDetail?.document_id;
		if (!documentId) {
			return;
		}

		try {
			selectedDocumentProcessingDetail = await fetchDocumentProcessing(
				apiBaseUrl, apiSecret,
				documentId
			);
			documentProcessingDetailError = '';
		} catch (error) {
			documentProcessingDetailError =
				error instanceof Error ? error.message : 'Unknown document processing detail error';
		}
	}

	async function retryFailedDocumentProcessingJob(job: DocumentProcessingJob) {
		if (retryingDocumentProcessingJobId === job.job_id) {
			return;
		}

		retryingDocumentProcessingJobId = job.job_id;
		documentProcessingJobsError = '';
		try {
			await retryDocumentProcessingJob(apiBaseUrl, apiSecret, job.job_id, {
				command_id: `document-processing-retry-${Date.now()}-${job.job_id}`
			});
			await loadDocumentProcessingJobs();
			await reloadSelectedDocumentProcessingDetail();
		} catch (error) {
			documentProcessingJobsError =
				error instanceof Error ? error.message : 'Unknown document processing retry error';
		} finally {
			if (retryingDocumentProcessingJobId === job.job_id) {
				retryingDocumentProcessingJobId = null;
			}
		}
	}

	async function setIdentityCandidateReview(
		candidate: PersonIdentityCandidate,
		reviewState: PersonIdentityReviewState
	) {
		try {
			await reviewIdentityCandidate(
				apiBaseUrl, apiSecret,
				candidate.identity_candidate_id,
				reviewState
			);
			await loadIdentityCandidates();
		} catch (error) {
			identityCandidatesError =
				error instanceof Error ? error.message : 'Unknown identity review error';
		}
	}

	async function splitConfirmedIdentityMerge(candidate: PersonIdentityCandidate) {
		const splitCandidate = splitCandidateForConfirmedMerge(candidate);
		if (!splitCandidate) {
			return;
		}

		const commandId = `person-identity-split-${Date.now()}-${candidate.identity_candidate_id}`;
		try {
			await reviewIdentityCandidate(
				apiBaseUrl, apiSecret,
				splitCandidate.identity_candidate_id,
				'user_confirmed',
				commandId
			);
			await loadIdentityCandidates();
		} catch (error) {
			identityCandidatesError =
				error instanceof Error ? error.message : 'Unknown identity split review error';
		}
	}

	async function setTaskCandidateReview(
		candidate: TaskCandidate,
		reviewState: TaskCandidateReviewState
	) {
		try {
			await reviewTaskCandidate(apiBaseUrl, apiSecret, candidate.task_candidate_id, reviewState);
			await loadTaskReviewState();
		} catch (error) {
			tasksError = error instanceof Error ? error.message : 'Unknown task candidate review error';
		}
	}

	async function loadProjectDetail(projectId: string, requestSequence = ++projectRequestSequence) {
		if (!projectId) {
			selectedProjectDetail = null;
			return;
		}
		isProjectsLoading = true;
		try {
			const detail = await fetchProjectDetail(apiBaseUrl, apiSecret, projectId);
			if (requestSequence !== projectRequestSequence) {
				return;
			}
			selectedProjectDetail = detail;
			selectedProjectId = detail.project.project_id;
			projectsError = '';
		} catch (error) {
			if (requestSequence !== projectRequestSequence) {
				return;
			}
			projectsError = error instanceof Error ? error.message : 'Unknown project detail error';
			selectedProjectDetail = null;
		} finally {
			if (requestSequence === projectRequestSequence) {
				isProjectsLoading = false;
			}
		}
	}

	function selectProject(project: ProjectSummary) {
		if (project.project.project_id === selectedProjectId && selectedProjectDetail) {
			return;
		}
		void loadProjectDetail(project.project.project_id);
	}

	async function loadCommunicationDetail(messageId: string) {
		try {
			selectedCommunicationDetail = await fetchCommunicationMessage(
				apiBaseUrl, apiSecret,
				messageId
			);
			communicationsError = '';
		} catch (error) {
			communicationsError =
				error instanceof Error ? error.message : 'Unknown communication detail error';
			selectedCommunicationDetail = null;
		}
	}

	function selectCommunication(index: number) {
		selectedConversationIndex = index;
		const message = communicationMessages[index];
		if (message) {
			void loadCommunicationDetail(message.message_id);
		}
	}

	async function askAiAboutSelectedMessage() {
		const message = selectedCommunicationDetail?.message ?? selectedCommunication;
		if (!message) {
			return;
		}
		aiQuestion = `Answer from local sources for message ${message.message_id}: ${message.subject}`;
		setCurrentView('agents');
		await submitAiAnswer();
	}

	function senderLabel(sender: string) {
		const match = sender.match(/^"?([^"<]+)"?\s*</);
		return (match?.[1] ?? senderEmail(sender) ?? sender).trim();
	}

	function senderEmail(sender: string) {
		const angleMatch = sender.match(/<([^>]+)>/);
		if (angleMatch?.[1]) {
			return angleMatch[1].trim();
		}
		const emailMatch = sender.match(/[^\s<>]+@[^\s<>]+/);
		return emailMatch?.[0]?.trim() ?? sender.trim();
	}

	function messageTime(message: CommunicationMessageSummary | CommunicationMessageDetailItem) {
		return formatDateTime(message.occurred_at ?? message.projected_at);
	}

	function telegramMessageTime(message: TelegramMessage) {
		return formatDateTime(message.occurred_at ?? message.projected_at);
	}

	function whatsappMessageTime(message: WhatsappWebMessage) {
		return formatDateTime(message.occurred_at ?? message.projected_at);
	}

	function emptyProjectStats(): ProjectStats {
		return {
			message_count: 0,
			document_count: 0,
			people_count: 0,
			graph_connection_count: 0,
			latest_activity_at: null
		};
	}

	function formatDateTime(value: string | null) {
		if (!value) {
			return '';
		}
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) {
			return '';
		}
		return new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		}).format(date);
	}

	function groupSettingsByCategory(settings: ApplicationSetting[]) {
		return settings.reduce<Record<string, ApplicationSetting[]>>((groups, setting) => {
			groups[setting.category] = [...(groups[setting.category] ?? []), setting];
			return groups;
		}, {});
	}

	function settingDraftValue(setting: ApplicationSetting) {
		if (setting.value_kind === 'json') {
			return JSON.stringify(setting.value, null, 2);
		}
		return String(setting.value);
	}

	function updateSettingDraft(settingKey: string, value: string) {
		settingDrafts = {
			...settingDrafts,
			[settingKey]: value
		};
		settingsActionMessage = '';
	}

	function inputEventValue(event: Event) {
		return (event.currentTarget as HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement).value;
	}

	function checkboxEventValue(event: Event) {
		return (event.currentTarget as HTMLInputElement).checked ? 'true' : 'false';
	}

	function cloneLayoutSettings(settings: LayoutSettings): LayoutSettings {
		return structuredClone($state.snapshot(settings));
	}

	function cloneSidebarSettings(settings: SidebarSettings): SidebarSettings {
		return structuredClone($state.snapshot(settings));
	}

	function cloneThemeSettings(settings: FrontendThemeSettings): FrontendThemeSettings {
		return structuredClone($state.snapshot(settings));
	}

	function updateThemeDraft(patch: Partial<FrontendThemeSettings>) {
		themeDraft = parseFrontendThemeSettings({
			...cloneThemeSettings(themeDraft ?? themeSettings),
			...patch
		});
		themeError = '';
		settingsActionMessage = '';
	}

	function selectShellBackground(shellBackground: ShellBackgroundId) {
		updateThemeDraft({ shellBackground });
	}

	function updateShellBrightness(event: Event) {
		const backgroundBrightness = Number(inputEventValue(event));
		updateThemeDraft({ backgroundBrightness: backgroundBrightness as ShellBackgroundBrightness });
	}

	function updateGlobalPanelOpacity(event: Event) {
		const panelOpacity = Number(inputEventValue(event));
		updateThemeDraft({ panelOpacity: panelOpacity as PanelOpacity });
	}

	function updateGlobalPanelBlur(event: Event) {
		const panelBlur = Number(inputEventValue(event));
		updateThemeDraft({ panelBlur: panelBlur as PanelBlur });
	}

	function selectShellAccent(accentColor: ShellAccentColorId) {
		updateThemeDraft({ accentColor });
	}

	function shellBackgroundPreviewClass(shellBackground: ShellBackgroundId) {
		return `background-preview ${shellBackgroundClass({
			...effectiveThemeSettings,
			shellBackground
		})}`;
	}

	function shellAccentSwatchClass(accentColor: ShellAccentColorId) {
		return `accent-swatch ${shellAccentClass({
			...effectiveThemeSettings,
			accentColor
		})}`;
	}

	function shellBackgroundLabel(shellBackground: ShellBackgroundId) {
		return shellBackgroundOptions.find((option) => option.id === shellBackground)?.label ?? 'Default';
	}

	function shellAccentLabel(accentColor: ShellAccentColorId) {
		return shellAccentColorOptions.find((option) => option.id === accentColor)?.label ?? 'Teal';
	}

	function themeSettingsHasChanges() {
		return (
			JSON.stringify(parseFrontendThemeSettings(themeDraft ?? themeSettings)) !==
			JSON.stringify(themeSettings)
		);
	}

	function resetThemeSettingsToDefault() {
		themeDraft = defaultFrontendThemeSettings();
		themeError = '';
		settingsActionMessage = '';
	}

	function cancelThemeSettingsEditing() {
		themeDraft = null;
		themeError = '';
	}

	async function saveThemeSettings() {
		const nextSettings = parseFrontendThemeSettings(themeDraft ?? themeSettings);
		isThemeSettingsSaving = true;
		savingSettingKey = FRONTEND_THEME_SETTING_KEY;
		try {
			const updated = await saveFrontendThemeSetting(apiBaseUrl, apiSecret, nextSettings);
			applicationSettings = applicationSettings.some((item) => item.setting_key === updated.setting_key)
				? applicationSettings.map((item) =>
						item.setting_key === updated.setting_key ? updated : item
					)
				: [...applicationSettings, updated];
			themeSettings = parseFrontendThemeSettings(updated.value);
			themeDraft = null;
			settingDrafts = {
				...settingDrafts,
				[updated.setting_key]: settingDraftValue(updated)
			};
			themeError = '';
			settingsError = '';
			settingsActionMessage = _('Appearance saved');
		} catch (error) {
			themeError = error instanceof Error ? error.message : 'Unknown appearance settings update error';
			settingsError = themeError;
		} finally {
			isThemeSettingsSaving = false;
			savingSettingKey = null;
		}
	}

	function sidebarGroupLabel(group: SidebarNavGroup, index: number) {
		return group.label || (group.id === 'communications' ? 'Communications' : `Group ${index + 1}`);
	}

	function sidebarRootIndexForGroup(groupId: string) {
		return effectiveSidebarSettings.rootItemIds.indexOf(sidebarGroupRootId(groupId));
	}

	function sidebarConfigItem(itemId: SidebarItemId) {
		const communicationSectionId = parseCommunicationSidebarItemId(itemId);
		if (communicationSectionId) {
			const section = communicationSections.find((item) => item.id === communicationSectionId);
			return section ? { id: itemId, label: section.label, icon: section.icon } : null;
		}

		const item = primaryNav.find((navItem) => navItem.id === itemId);
		return item ? { id: itemId, label: item.label, icon: item.icon } : null;
	}

	function sidebarItemLabel(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary' ? item.primary.label : item.section.label;
	}

	function sidebarItemIcon(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary' ? item.primary.icon : item.section.icon;
	}

	function sidebarItemBadge(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary' ? item.primary.badge : communicationSectionBadge(item.section.id);
	}

	function sidebarItemTitle(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary' && !item.primary.enabled
			? `${item.primary.label} is not available in the current desktop scope`
			: sidebarItemLabel(item);
	}

	function isSidebarItemDisabled(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary' && !item.primary.enabled;
	}

	function isSidebarItemActive(item: ResolvedSidebarItem<NavItem>) {
		return item.kind === 'primary'
			? currentView === item.primary.id
			: currentView === 'communications' && activeCommunicationSection === item.section.id;
	}

	function isSidebarItemIdActive(itemId: SidebarItemId) {
		const communicationSectionId = parseCommunicationSidebarItemId(itemId);
		if (communicationSectionId) {
			return currentView === 'communications' && activeCommunicationSection === communicationSectionId;
		}
		return currentView === itemId;
	}

	function sidebarGroupHasActiveItem(group: SidebarNavGroup) {
		return group.itemIds.some((itemId) => isSidebarItemIdActive(itemId));
	}

	function sidebarGroupHasSeparatorBefore(group: SidebarNavGroup, itemId: SidebarItemId) {
		return group.itemIds.indexOf(itemId) > 0 && group.separatorBeforeItemIds.includes(itemId);
	}

	function isSidebarGroupExpanded(groupId: string) {
		return expandedSidebarGroupIds.includes(groupId);
	}

	function setSidebarGroupExpanded(groupId: string, expanded: boolean) {
		expandedSidebarGroupIds = expanded
			? Array.from(new Set([...expandedSidebarGroupIds, groupId]))
			: expandedSidebarGroupIds.filter((id) => id !== groupId);
	}

	function toggleSidebarGroup(group: SidebarNavGroup) {
		if (isSidebarRail) {
			activeSidebarRailGroupId = activeSidebarRailGroupId === group.id ? null : group.id;
			return;
		}

		setSidebarGroupExpanded(group.id, !isSidebarGroupExpanded(group.id));
	}

	function closeSidebarRailDropdown() {
		activeSidebarRailGroupId = null;
	}

	function selectSidebarItem(item: ResolvedSidebarItem<NavItem>) {
		if (item.kind === 'primary') {
			setCurrentView(item.primary.id);
			return;
		}

		selectCommunicationSection(item.section.id);
	}

	function sidebarSettingsHasChanges() {
		return sidebarDraft !== null && JSON.stringify(sidebarDraft) !== JSON.stringify(sidebarSettings);
	}

	function sidebarGroupIdFromLabel(label: string) {
		const base =
			label
				.trim()
				.toLowerCase()
				.replace(/[^a-z0-9_-]+/g, '-')
				.replace(/^-+|-+$/g, '') || `group-${effectiveSidebarSettings.groups.length + 1}`;
		const existingIds = new Set(effectiveSidebarSettings.groups.map((group) => group.id));
		if (!existingIds.has(base)) {
			return base;
		}

		let suffix = 2;
		while (existingIds.has(`${base}-${suffix}`)) {
			suffix += 1;
		}
		return `${base}-${suffix}`;
	}

	function updateSidebarDraft(update: (draft: SidebarSettings) => SidebarSettings) {
		const draft = sidebarDraft ?? cloneSidebarSettings(sidebarSettings);
		sidebarDraft = parseSidebarSettings(update(draft));
		sidebarError = '';
		settingsActionMessage = '';
	}

	function updateSidebarGroupLabel(groupId: string, label: string) {
		updateSidebarDraft((draft) => ({
			...draft,
			groups: draft.groups.map((group) =>
				group.id === groupId ? { ...group, label: label.slice(0, 32) } : group
			)
		}));
	}

	function addSidebarGroup() {
		const label = newSidebarGroupLabel.trim().slice(0, 32);
		const groupLabel = label || `Group ${effectiveSidebarSettings.groups.length + 1}`;
		const groupId = sidebarGroupIdFromLabel(groupLabel);
		updateSidebarDraft((draft) => ({
			...draft,
			rootItemIds: [...draft.rootItemIds, sidebarGroupRootId(groupId)],
			groups: [
				...draft.groups,
				{
					id: groupId,
					label: groupLabel,
					icon: 'tabler:folder',
					itemIds: [],
					separatorBeforeItemIds: []
				}
			]
		}));
		setSidebarGroupExpanded(groupId, true);
		newSidebarGroupLabel = '';
	}

	function removeSidebarGroup(groupId: string) {
		if (groupId === 'communications') {
			sidebarError = 'The Communications group can be renamed or reordered, but not removed.';
			return;
		}
		updateSidebarDraft((draft) => {
			if (draft.groups.length <= 1) {
				return draft;
			}

			const groupIndex = draft.groups.findIndex((group) => group.id === groupId);
			if (groupIndex < 0) {
				return draft;
			}

			const groups = draft.groups.map((group) => ({
				...group,
				itemIds: [...group.itemIds],
				separatorBeforeItemIds: [...group.separatorBeforeItemIds]
			}));
			const [removedGroup] = groups.splice(groupIndex, 1);
			if (!removedGroup) {
				return draft;
			}

			let rootItemIds = draft.rootItemIds.filter((rootId) => sidebarGroupIdFromRootId(rootId) !== groupId);
			const communicationsGroupIndex = groups.findIndex((group) => group.id === 'communications');
			for (const itemId of removedGroup.itemIds) {
				const communicationSectionId = parseCommunicationSidebarItemId(itemId);
				if (communicationSectionId && communicationsGroupIndex >= 0) {
					groups[communicationsGroupIndex] = {
						...groups[communicationsGroupIndex],
						itemIds: [...groups[communicationsGroupIndex].itemIds, itemId]
					};
				} else if (!communicationSectionId) {
					rootItemIds = [...rootItemIds, itemId as SidebarRootItemId];
				}
			}

			return { ...draft, rootItemIds, groups };
		});
		expandedSidebarGroupIds = expandedSidebarGroupIds.filter((id) => id !== groupId);
	}

	function moveSidebarGroup(groupId: string, direction: -1 | 1) {
		updateSidebarDraft((draft) => {
			const rootId = sidebarGroupRootId(groupId);
			const rootIndex = draft.rootItemIds.indexOf(rootId);
			const nextIndex = rootIndex + direction;
			if (rootIndex < 0 || nextIndex < 0 || nextIndex >= draft.rootItemIds.length) {
				return draft;
			}

			const rootItemIds = [...draft.rootItemIds];
			[rootItemIds[rootIndex], rootItemIds[nextIndex]] = [rootItemIds[nextIndex], rootItemIds[rootIndex]];
			return { ...draft, rootItemIds };
		});
	}

	function moveSidebarRootItem(rootId: SidebarRootItemId, direction: -1 | 1) {
		updateSidebarDraft((draft) => {
			const rootIndex = draft.rootItemIds.indexOf(rootId);
			const nextIndex = rootIndex + direction;
			if (rootIndex < 0 || nextIndex < 0 || nextIndex >= draft.rootItemIds.length) {
				return draft;
			}
			const rootItemIds = [...draft.rootItemIds];
			[rootItemIds[rootIndex], rootItemIds[nextIndex]] = [rootItemIds[nextIndex], rootItemIds[rootIndex]];
			return { ...draft, rootItemIds };
		});
	}

	function moveSidebarItem(itemId: SidebarItemId, direction: -1 | 1) {
		updateSidebarDraft((draft) => {
			const groups = draft.groups.map((group) => ({
				...group,
				itemIds: [...group.itemIds],
				separatorBeforeItemIds: [...group.separatorBeforeItemIds]
			}));
			const groupIndex = groups.findIndex((group) => group.itemIds.includes(itemId));
			if (groupIndex < 0) {
				return draft;
			}

			const itemIndex = groups[groupIndex].itemIds.indexOf(itemId);
			const nextItemIndex = itemIndex + direction;
			if (nextItemIndex >= 0 && nextItemIndex < groups[groupIndex].itemIds.length) {
				[groups[groupIndex].itemIds[itemIndex], groups[groupIndex].itemIds[nextItemIndex]] = [
					groups[groupIndex].itemIds[nextItemIndex],
					groups[groupIndex].itemIds[itemIndex]
				];
				return { ...draft, groups };
			}

			const nextGroupIndex = groupIndex + direction;
			if (nextGroupIndex < 0 || nextGroupIndex >= groups.length) {
				return draft;
			}

			groups[groupIndex].itemIds = groups[groupIndex].itemIds.filter((id) => id !== itemId);
			groups[groupIndex].separatorBeforeItemIds = groups[groupIndex].separatorBeforeItemIds.filter(
				(id) => id !== itemId
			);
			if (direction < 0) {
				groups[nextGroupIndex].itemIds = [...groups[nextGroupIndex].itemIds, itemId];
			} else {
				groups[nextGroupIndex].itemIds = [itemId, ...groups[nextGroupIndex].itemIds];
			}
			return { ...draft, groups };
		});
	}

	function moveSidebarItemToGroup(itemId: SidebarItemId, targetGroupId: string) {
		updateSidebarDraft((draft) => {
			if (targetGroupId !== 'root' && !draft.groups.some((group) => group.id === targetGroupId)) {
				return draft;
			}

			const groups = draft.groups.map((group) => ({
				...group,
				itemIds: group.itemIds.filter((id) => id !== itemId),
				separatorBeforeItemIds: group.separatorBeforeItemIds.filter((id) => id !== itemId)
			}));
			const rootItemIds = draft.rootItemIds.filter((id) => id !== itemId);

			if (targetGroupId === 'root') {
				if (parseCommunicationSidebarItemId(itemId)) {
					return draft;
				}
				return {
					...draft,
					rootItemIds: [...rootItemIds, itemId as SidebarRootItemId],
					groups
				};
			}

			return {
				...draft,
				rootItemIds,
				groups: groups.map((group) =>
					group.id === targetGroupId ? { ...group, itemIds: [...group.itemIds, itemId] } : group
				)
			};
		});
		if (targetGroupId !== 'root') {
			setSidebarGroupExpanded(targetGroupId, true);
		}
	}

	function toggleSidebarGroupSeparator(groupId: string, itemId: SidebarItemId) {
		updateSidebarDraft((draft) => ({
			...draft,
			groups: draft.groups.map((group) => {
				if (group.id !== groupId || group.itemIds.indexOf(itemId) <= 0) {
					return group;
				}

				const hasSeparator = group.separatorBeforeItemIds.includes(itemId);
				return {
					...group,
					separatorBeforeItemIds: hasSeparator
						? group.separatorBeforeItemIds.filter((id) => id !== itemId)
						: [...group.separatorBeforeItemIds, itemId]
				};
			})
		}));
	}

	function toggleSidebarItemHidden(itemId: SidebarItemId) {
		updateSidebarDraft((draft) => ({
			...draft,
			hiddenItemIds: draft.hiddenItemIds.includes(itemId)
				? draft.hiddenItemIds.filter((id) => id !== itemId)
				: [...draft.hiddenItemIds, itemId]
		}));
	}

	function resetSidebarSettingsToDefault() {
		sidebarDraft = defaultSidebarSettings();
		sidebarError = '';
		settingsActionMessage = '';
	}

	function cancelSidebarSettingsEditing() {
		sidebarDraft = null;
		sidebarError = '';
		newSidebarGroupLabel = '';
	}

	async function saveSidebarSettings() {
		const nextSettings = parseSidebarSettings(sidebarDraft ?? sidebarSettings);
		isSidebarSettingsSaving = true;
		savingSettingKey = 'frontend.sidebar';
		try {
			const updated = await saveFrontendSidebarSetting(apiBaseUrl, apiSecret, nextSettings);
			applicationSettings = applicationSettings.some((item) => item.setting_key === updated.setting_key)
				? applicationSettings.map((item) =>
						item.setting_key === updated.setting_key ? updated : item
					)
				: [...applicationSettings, updated];
			sidebarSettings = parseSidebarSettings(updated.value);
			sidebarDraft = null;
			settingDrafts = {
				...settingDrafts,
				[updated.setting_key]: settingDraftValue(updated)
			};
			sidebarError = '';
			settingsError = '';
			settingsActionMessage = 'Sidebar navigation saved';
		} catch (error) {
			sidebarError = error instanceof Error ? error.message : 'Unknown sidebar settings update error';
			settingsError = sidebarError;
		} finally {
			isSidebarSettingsSaving = false;
			savingSettingKey = null;
		}
	}

	function startLayoutEditing() {
		layoutDraft = cloneLayoutSettings(layoutSettings);
		isLayoutEditing = true;
		isUserMenuOpen = false;
		layoutError = '';
		selectedLayoutWidgetId = null;
	}

	function cancelLayoutEditing() {
		layoutDraft = null;
		isLayoutEditing = false;
		isWidgetDrawerOpen = false;
		selectedLayoutWidgetId = null;
		layoutError = '';
	}

	async function saveLayoutSettings() {
		const nextSettings = parseLayoutSettings(layoutDraft ?? layoutSettings);
		isLayoutSettingsSaving = true;
		savingSettingKey = 'frontend.layout';
		try {
			const updated = await saveFrontendLayoutSetting(apiBaseUrl, apiSecret, nextSettings);
			applicationSettings = applicationSettings.some((item) => item.setting_key === updated.setting_key)
				? applicationSettings.map((item) =>
						item.setting_key === updated.setting_key ? updated : item
					)
				: [...applicationSettings, updated];
			layoutSettings = parseLayoutSettings(updated.value);
			layoutDraft = null;
			isLayoutEditing = false;
			isWidgetDrawerOpen = false;
			selectedLayoutWidgetId = null;
			settingDrafts = {
				...settingDrafts,
				[updated.setting_key]: settingDraftValue(updated)
			};
			layoutError = '';
			settingsError = '';
			settingsActionMessage = _('Widget layout saved');
		} catch (error) {
			layoutError = error instanceof Error ? error.message : 'Unknown layout settings update error';
			settingsError = layoutError;
		} finally {
			isLayoutSettingsSaving = false;
			savingSettingKey = null;
		}
	}

	function toggleUserMenu() {
		isUserMenuOpen = !isUserMenuOpen;
	}

	function closeUserMenu() {
		isUserMenuOpen = false;
	}

	function toggleNotificationsDrawer() {
		isNotificationsDrawerOpen = !isNotificationsDrawerOpen;
		if (isNotificationsDrawerOpen) {
			isUserMenuOpen = false;
		}
	}

	function closeNotificationsDrawer() {
		isNotificationsDrawerOpen = false;
	}

	function dismissNotification(notificationId: string) {
		if (!dismissedNotificationIds.includes(notificationId)) {
			dismissedNotificationIds = [...dismissedNotificationIds, notificationId];
		}
	}

	function toggleNotificationExpanded(notificationId: string) {
		expandedNotificationIds = expandedNotificationIds.includes(notificationId)
			? expandedNotificationIds.filter((id) => id !== notificationId)
			: [...expandedNotificationIds, notificationId];
	}

	function isNotificationExpanded(notificationId: string) {
		return expandedNotificationIds.includes(notificationId);
	}

	function notificationNeedsExpansion(notification: NotificationItem) {
		return notification.body.length > 120;
	}

	async function openNotificationTarget(notification: NotificationItem) {
		isNotificationsDrawerOpen = false;
		const targetSection = notification.targetSection ?? (notification.source === 'telegram' ? 'telegram' : 'unified');
		if (notification.isDemo) {
			selectCommunicationSection(targetSection);
			return;
		}
		if (notification.source === 'telegram') {
			selectCommunicationSection(targetSection);
			if (telegramChats.length === 0) {
				await loadTelegramWorkspace();
			}
			const targetChat = telegramChats.find(
				(chat) =>
					chat.account_id === notification.accountId &&
					chat.provider_chat_id === notification.providerChatId
			);
			if (targetChat) {
				selectTelegramChat(targetChat);
			} else if (notification.providerChatId) {
				selectedTelegramChatId = notification.providerChatId;
			}
			return;
		}

		selectCommunicationSection(targetSection);
		if (!notification.messageId) {
			return;
		}
		const messageIndex = communicationMessages.findIndex(
			(message) => message.message_id === notification.messageId
		);
		if (messageIndex >= 0) {
			selectCommunication(messageIndex);
		} else {
			await loadCommunicationDetail(notification.messageId);
		}
	}

	function exitApplication() {
		isUserMenuOpen = false;
		if (typeof window !== 'undefined') {
			window.close();
		}
	}

	function resetCurrentViewLayout() {
		const layoutViewId = activeLayout?.preset.viewId;
		if (!layoutViewId) {
			return;
		}

		const draft = layoutDraft ?? cloneLayoutSettings(layoutSettings);
		const views = { ...draft.views };
		delete views[layoutViewId];
		layoutDraft = { ...draft, views };
		selectedLayoutWidgetId = null;
		layoutError = '';
	}

	function ensureCurrentViewOverride() {
		const preset = activeLayout?.preset ?? findPresetForView(activeWorkspaceView);
		if (!preset) {
			return null;
		}

		const draft = layoutDraft ?? cloneLayoutSettings(layoutSettings);
		const existingOverride = draft.views[preset.viewId];
		const override = existingOverride ?? {
			presetId: preset.id,
			presetVersion: preset.version,
			hiddenWidgetIds: [],
			zoneOverrides: {},
			orderOverrides: {},
			gridOverrides: {}
		};

		layoutDraft = {
			...draft,
			views: {
				...draft.views,
				[preset.viewId]: override
			}
		};
		layoutError = '';

		return override;
	}

	function updateCurrentViewOverride(update: (override: ViewLayoutOverride) => ViewLayoutOverride) {
		const override = ensureCurrentViewOverride();
		const layoutViewId = activeLayout?.preset.viewId ?? findPresetForView(activeWorkspaceView)?.viewId;
		if (!override || !layoutDraft || !layoutViewId) {
			return;
		}

		layoutDraft = {
			...layoutDraft,
			views: {
				...layoutDraft.views,
				[layoutViewId]: update(override)
			}
		};
	}

	function hideWidget(widgetId: string) {
		updateCurrentViewOverride((override) => {
			if (override.hiddenWidgetIds.includes(widgetId)) {
				return override;
			}

			return {
				...override,
				hiddenWidgetIds: [...override.hiddenWidgetIds, widgetId]
			};
		});
		if (selectedLayoutWidgetId === widgetId) {
			selectedLayoutWidgetId = null;
		}
	}

	function showWidget(widgetId: string) {
		updateCurrentViewOverride((override) => ({
			...override,
			hiddenWidgetIds: override.hiddenWidgetIds.filter((id) => id !== widgetId)
		}));
		isWidgetDrawerOpen = false;
	}

	function openWidgetSettingsDrawer(widgetId: string) {
		selectedLayoutWidgetId = widgetId;
		isWidgetDrawerOpen = false;
	}

	function closeWidgetSettingsDrawer() {
		selectedLayoutWidgetId = null;
	}

	function widgetZoneTitle(zoneId: string) {
		return activeLayout?.zones.find((zone) => zone.id === zoneId)?.title ?? zoneId;
	}

	function widgetGridValue(widgetId: string, dimension: WidgetGridDimension) {
		return activeWidgetById.get(widgetId)?.[dimension] ?? 1;
	}

	function widgetGridMin(widgetId: string, dimension: WidgetGridDimension) {
		const widget = activeWidgetById.get(widgetId);
		if (!widget) {
			return 1;
		}

		return dimension === 'columns' ? widget.minColumns : widget.minRows;
	}

	function widgetGridMax(dimension: WidgetGridDimension) {
		return dimension === 'columns' ? LAYOUT_GRID_COLUMNS : LAYOUT_GRID_MAX_ROWS;
	}

	function normalizeWidgetGridValue(
		widgetId: string,
		dimension: WidgetGridDimension,
		value: number
	) {
		if (!Number.isFinite(value)) {
			return widgetGridValue(widgetId, dimension);
		}

		return Math.max(
			widgetGridMin(widgetId, dimension),
			Math.min(widgetGridMax(dimension), Math.trunc(value))
		);
	}

	function setWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, value: number) {
		const nextValue = normalizeWidgetGridValue(widgetId, dimension, value);
		updateCurrentViewOverride((override) => {
			const currentGridOverride = override.gridOverrides[widgetId] ?? {};

			return {
				...override,
				gridOverrides: {
					...override.gridOverrides,
					[widgetId]: {
						...currentGridOverride,
						[dimension]: nextValue
					}
				}
			};
		});
	}

	function adjustWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, delta: -1 | 1) {
		setWidgetGridValue(widgetId, dimension, widgetGridValue(widgetId, dimension) + delta);
	}

	function handleWidgetGridInput(widgetId: string, dimension: WidgetGridDimension, event: Event) {
		const input = event.currentTarget;
		if (!(input instanceof HTMLInputElement)) {
			return;
		}

		setWidgetGridValue(widgetId, dimension, input.valueAsNumber);
	}

	function widgetPanelSurfaceValue(widgetId: string, setting: WidgetPanelSurfaceSetting) {
		const widget = activeWidgetById.get(widgetId);
		return widget?.[setting] ?? effectiveThemeSettings[setting];
	}

	function widgetPanelSurfaceOverrideValue(widgetId: string, setting: WidgetPanelSurfaceSetting) {
		return activeWidgetById.get(widgetId)?.[setting] ?? null;
	}

	function setWidgetPanelSurfaceValue(
		widgetId: string,
		setting: WidgetPanelSurfaceSetting,
		value: number
	) {
		const allowedValues: readonly number[] =
			setting === 'panelOpacity' ? panelOpacityValues : panelBlurValues;
		if (!allowedValues.includes(value)) {
			return;
		}

		updateCurrentViewOverride((override) => {
			const currentGridOverride = override.gridOverrides[widgetId] ?? {};
			return {
				...override,
				gridOverrides: {
					...override.gridOverrides,
					[widgetId]: {
						...currentGridOverride,
						[setting]: value
					}
				}
			};
		});
	}

	function handleWidgetPanelSurfaceInput(
		widgetId: string,
		setting: WidgetPanelSurfaceSetting,
		event: Event
	) {
		const input = event.currentTarget;
		if (!(input instanceof HTMLInputElement)) {
			return;
		}

		setWidgetPanelSurfaceValue(widgetId, setting, input.valueAsNumber);
	}

	function resetWidgetPanelSurface(widgetId: string) {
		updateCurrentViewOverride((override) => {
			const currentGridOverride = override.gridOverrides[widgetId];
			if (!currentGridOverride) {
				return override;
			}

			const restGridOverride = { ...currentGridOverride };
			delete restGridOverride.panelOpacity;
			delete restGridOverride.panelBlur;
			const nextGridOverrides = { ...override.gridOverrides };
			if (Object.keys(restGridOverride).length === 0) {
				delete nextGridOverrides[widgetId];
			} else {
				nextGridOverrides[widgetId] = restGridOverride;
			}

			return {
				...override,
				gridOverrides: nextGridOverrides
			};
		});
	}

	function resetWidgetGrid(widgetId: string) {
		updateCurrentViewOverride((override) => {
			const currentGridOverride = override.gridOverrides[widgetId];
			if (!currentGridOverride) {
				return override;
			}

			const restGridOverride = { ...currentGridOverride };
			delete restGridOverride.columns;
			delete restGridOverride.rows;
			const nextGridOverrides = { ...override.gridOverrides };
			if (Object.keys(restGridOverride).length === 0) {
				delete nextGridOverrides[widgetId];
			} else {
				nextGridOverrides[widgetId] = restGridOverride;
			}

			return {
				...override,
				gridOverrides: nextGridOverrides
			};
		});
	}

	function moveWidgetInZone(widgetId: string, direction: -1 | 1) {
		const layout = activeLayout;
		if (!layout) return;

		const widget = Object.values(layout.widgetsByZone)
			.flat()
			.find((item) => item.widgetId === widgetId);
		if (!widget) return;

		const zoneWidgets = layout.widgetsByZone[widget.zoneId] ?? [];
		const ids = zoneWidgets.map((item) => item.widgetId);
		const index = ids.indexOf(widgetId);
		const nextIndex = index + direction;
		if (index < 0 || nextIndex < 0 || nextIndex >= ids.length) return;

		const nextIds = [...ids];
		[nextIds[index], nextIds[nextIndex]] = [nextIds[nextIndex], nextIds[index]];

		updateCurrentViewOverride((override) => ({
			...override,
			orderOverrides: {
				...override.orderOverrides,
				[widget.zoneId]: nextIds
			}
		}));
	}

	function isWidgetVisible(widgetId: string) {
		if (!activeLayout) return true;

		return Object.values(activeLayout.widgetsByZone).some((widgets) =>
			widgets.some((widget) => widget.widgetId === widgetId)
		);
	}

	function syncWidgetGridClasses(widgetsById: Map<string, ResolvedWidget>) {
		if (typeof document === 'undefined') {
			return;
		}

		if (window.scrollX !== 0 || window.scrollY !== 0) {
			window.scrollTo(0, 0);
		}

		const workspace = document.querySelector<HTMLElement>('.workspace');
		const workspaceWidth = workspace?.clientWidth ?? window.innerWidth;
		const workspaceRect = workspace?.getBoundingClientRect();
		const statusStrip = document.querySelector<HTMLElement>('.workspace-status-strip');
		const statusRect = statusStrip?.getBoundingClientRect();
		const minimumColumnWidth = 44;
		const availableColumns = Math.max(
			1,
			Math.min(12, Math.floor(workspaceWidth / minimumColumnWidth))
		);

		for (const element of document.querySelectorAll<HTMLElement>('.widget-frame[data-widget-id]')) {
			element.classList.remove(...gridClassNames);

			const widgetId = element.dataset.widgetId;
			if (!widgetId) continue;

			const widget = widgetsById.get(widgetId);
			if (!widget) continue;

			const scrollClass = scrollClassByMode[widget.scrollMode];
			element.classList.add(
				`widget-cols-${widget.columns}`,
				`widget-rows-${widget.rows}`,
				scrollClass
			);
			if (widget.panelOpacity !== undefined) {
				element.classList.add(`widget-panel-opacity-${widget.panelOpacity}`);
			}
			if (widget.panelBlur !== undefined) {
				element.classList.add(`widget-panel-blur-${widget.panelBlur}`);
			}
			element.dataset.widgetColumns = String(widget.columns);
			element.dataset.widgetRows = String(widget.rows);
			element.dataset.widgetMinColumns = String(widget.minColumns);
			element.dataset.widgetMinRows = String(widget.minRows);
			element.dataset.widgetScroll = widget.scrollMode;
			element.dataset.widgetPanelOpacity =
				widget.panelOpacity === undefined ? 'global' : String(widget.panelOpacity);
			element.dataset.widgetPanelBlur = widget.panelBlur === undefined ? 'global' : String(widget.panelBlur);
		}

		requestAnimationFrame(() => {
			if (window.scrollX !== 0 || window.scrollY !== 0) {
				window.scrollTo(0, 0);
			}

			const hiddenByViewport: string[] = [];
			const rightLimit = workspaceRect?.right ?? window.innerWidth;
			const bottomLimit = statusRect?.top ?? workspaceRect?.bottom ?? window.innerHeight;

			for (const element of document.querySelectorAll<HTMLElement>('.widget-frame[data-widget-id]')) {
				const widgetId = element.dataset.widgetId;
				const widget = widgetId ? widgetsById.get(widgetId) : null;
				if (!widget) continue;

				const rect = element.getBoundingClientRect();
				const isFitHidden =
					widget.minColumns > availableColumns ||
					rect.left < (workspaceRect?.left ?? 0) - 1 ||
					rect.right > rightLimit + 1 ||
					rect.bottom > bottomLimit - 1;

				if (isFitHidden) {
					element.classList.add('widget-fit-hidden');
					hiddenByViewport.push(widget.definition.title);
				} else {
					element.classList.remove('widget-fit-hidden');
				}
			}

			for (const element of document.querySelectorAll<HTMLElement>('.widget-frame[data-widget-hide-if-clipped-content][data-widget-id]')) {
				const widgetId = element.dataset.widgetId;
				const widget = widgetId ? widgetsById.get(widgetId) : null;
				const contentElements = Array.from(
					element.querySelectorAll<HTMLElement>('[data-widget-fit-content]')
				);
				if (!widget || contentElements.length === 0 || element.classList.contains('widget-fit-hidden')) {
					continue;
				}

				const rect = element.getBoundingClientRect();
				const isContentClipped = contentElements.some((content) => {
					const contentRect = content.getBoundingClientRect();
					const hasInternalOverflow =
						content.scrollHeight > content.clientHeight + 1 ||
						content.scrollWidth > content.clientWidth + 1;

					return (
						hasInternalOverflow ||
						contentRect.left < rect.left - 1 ||
						contentRect.right > rect.right + 1 ||
						contentRect.top < rect.top - 1 ||
						contentRect.bottom > rect.bottom + 1
					);
				});

				if (isContentClipped) {
					element.classList.add('widget-fit-hidden');
					hiddenByViewport.push(widget.definition.title);
				}
			}

			viewportHiddenWidgetTitles = hiddenByViewport;
		});
	}

	function resolveActiveLayout(viewId: ViewId, settings: LayoutSettings): ResolvedLayout | null {
		const preset = findPresetForView(viewId);
		if (!preset) return null;
		const layoutViewId = preset.viewId;
		return resolveLayout(preset, widgetRegistry, settings.views[layoutViewId]);
	}

	function settingDraftToValue(setting: ApplicationSetting, draft: string): ApplicationSetting['value'] {
		const value = draft.trim();
		if (setting.value_kind === 'integer') {
			const numberValue = Number(value);
			if (!Number.isInteger(numberValue)) {
				throw new Error(`${setting.label} must be an integer`);
			}
			return numberValue;
		}
		if (setting.value_kind === 'boolean') {
			return value === 'true';
		}
		if (setting.value_kind === 'json') {
			return JSON.parse(value);
		}
		return value;
	}

	function settingAllowedValues(setting: ApplicationSetting) {
		const values = setting.metadata.allowed_values;
		if (!Array.isArray(values)) {
			return [];
		}
		return values.filter((value): value is string => typeof value === 'string');
	}

	function settingControl(setting: ApplicationSetting) {
		const control = setting.metadata.ui_control;
		return typeof control === 'string' ? control : '';
	}

	function settingValueText(settingKey: string) {
		const setting = applicationSettings.find((item) => item.setting_key === settingKey);
		if (!setting) {
			return 'not set';
		}
		if (setting.value === null || setting.value === undefined) {
			return 'not set';
		}
		if (typeof setting.value === 'object') {
			return JSON.stringify(setting.value);
		}
		return String(setting.value);
	}

	function settingMetadataFlag(setting: ApplicationSetting, key: string) {
		return setting.metadata[key] === true;
	}

	function settingMetadataText(setting: ApplicationSetting, key: string) {
		const value = setting.metadata[key];
		return typeof value === 'string' && value.trim() ? value.trim() : '';
	}

	function settingHasChanged(setting: ApplicationSetting) {
		return (settingDrafts[setting.setting_key] ?? settingDraftValue(setting)) !== settingDraftValue(setting);
	}

	function settingsCategoryLabel(category: string) {
		return category
			.split('_')
			.flatMap((part) => part.split('-'))
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function accountProviderIcon(providerKind: string) {
		if (providerKind === 'telegram_user' || providerKind === 'telegram_bot') {
			return 'tabler:brand-telegram';
		}
		if (providerKind === 'whatsapp_web') {
			return 'tabler:brand-whatsapp';
		}
		return 'tabler:mail';
	}

	function accountProviderLabel(providerKind: string) {
		return providerKind
			.split('_')
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function accountUpdatedLabel(account: ProviderAccount) {
		return formatDateTime(account.updated_at) || 'Never';
	}

	function agentCardView(agent: AiAgent) {
		const visual = agentVisual(agent.agent_id);
		const runs = aiRuns.filter((run) => run.agent_id === agent.agent_id);
		const completed = runs.filter((run) => run.status === 'completed').length;
		const success = runs.length > 0 ? Math.round((completed / runs.length) * 100) : 0;

		return {
			agentId: agent.agent_id,
			name: agent.display_name,
			summary: agent.role,
			icon: visual.icon,
			tasks: runs.length,
			success: runs.length > 0 ? `${success}%` : 'n/a',
			status: agent.status,
			tone: visual.tone,
			model: agent.default_model
		};
	}

	function agentVisual(agentId: string) {
		switch (agentId) {
			case 'HESTIA':
				return { icon: 'tabler:calendar-stats', tone: 'mint' };
			case 'HERMES':
				return { icon: 'tabler:route', tone: 'blue' };
			case 'MNEMOSYNE':
				return { icon: 'tabler:database-search', tone: 'purple' };
			case 'ATHENA':
				return { icon: 'tabler:target-arrow', tone: 'amber' };
			default:
				return { icon: 'tabler:sparkles', tone: 'cyan' };
		}
	}

	function runStatusLabel(run: AiRun) {
		if (run.status === 'completed') {
			return 'Completed';
		}
		if (run.status === 'failed') {
			return 'Failed';
		}
		return 'Requested';
	}

	function aiRuntimeSummary() {
		if (!aiStatus) {
			return isAiLoading ? 'Loading' : 'Unknown';
		}
		return aiStatus.status === 'ok' ? 'Ready' : 'Unavailable';
	}

	function aiModelSummary() {
		if (!aiStatus) {
			return 'No status';
		}
		return `${aiStatus.chat_model} / ${aiStatus.embedding_model}`;
	}

	function formatDuration(durationMs: number | null | undefined) {
		if (durationMs == null) {
			return 'n/a';
		}
		if (durationMs < 1000) {
			return `${durationMs} ms`;
		}
		return `${(durationMs / 1000).toFixed(1)} s`;
	}

	function safeCitations(value: unknown): AiCitation[] {
		if (!Array.isArray(value)) {
			return [];
		}
		return value.filter(isAiCitation);
	}

	function isAiCitation(value: unknown): value is AiCitation {
		return (
			typeof value === 'object' &&
			value !== null &&
			typeof (value as { source_kind?: unknown }).source_kind === 'string' &&
			typeof (value as { source_id?: unknown }).source_id === 'string' &&
			typeof (value as { title?: unknown }).title === 'string' &&
			typeof (value as { excerpt?: unknown }).excerpt === 'string'
		);
	}

	function formatProjectDate(value: string | null) {
		if (!value) {
			return 'Not set';
		}
		const date = new Date(`${value}T00:00:00`);
		if (Number.isNaN(date.getTime())) {
			return 'Invalid date';
		}
		return new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		}).format(date);
	}

	function formatProjectDateTime(value: string | null) {
		const formatted = formatDateTime(value);
		return formatted || 'No activity';
	}

	function projectStatusLabel(status: string) {
		return status
			.split('_')
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function projectTimelineIcon(item: ProjectTimelineItem) {
		switch (item.item_kind) {
			case 'message':
				return 'tabler:mail';
			case 'document':
				return 'tabler:file-text';
			default:
				return 'tabler:circle-dot';
		}
	}

	function projectDocumentIcon(document: ProjectDocumentSummary) {
		switch (document.document_kind) {
			case 'pdf':
				return 'tabler:file-type-pdf';
			case 'markdown':
				return 'tabler:file-text';
			default:
				return 'tabler:file';
		}
	}

	function projectMessageSender(message: ProjectMessageSummary) {
		return senderLabel(message.sender);
	}

	function formatBytes(sizeBytes: number) {
		if (sizeBytes < 1024) {
			return `${sizeBytes} B`;
		}
		if (sizeBytes < 1024 * 1024) {
			return `${(sizeBytes / 1024).toFixed(1)} KB`;
		}
		return `${(sizeBytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	function attachmentIcon(contentType: string) {
		if (contentType.includes('pdf')) {
			return 'tabler:file-type-pdf';
		}
		if (contentType.startsWith('image/')) {
			return 'tabler:photo';
		}
		if (contentType.includes('spreadsheet') || contentType.includes('excel')) {
			return 'tabler:file-spreadsheet';
		}
		return 'tabler:file';
	}

	function communicationSectionBadge(sectionId: CommunicationSectionId) {
		if (sectionId === 'inbox') {
			return mailboxHealth?.unread ? String(mailboxHealth.unread) : undefined;
		}
		if (sectionId === 'waiting') {
			return mailboxHealth?.waiting ? String(mailboxHealth.waiting) : undefined;
		}
		if (sectionId === 'needs_reply') {
			return mailboxHealth?.needs_action ? String(mailboxHealth.needs_action) : undefined;
		}
		return undefined;
	}

	function communicationSectionWorkflowState(sectionId: CommunicationSectionId): WorkflowState | '' | null {
		switch (sectionId) {
			case 'inbox':
				return 'new';
			case 'waiting':
				return 'waiting';
			case 'needs_reply':
				return 'needs_action';
			case 'unified':
			case 'mail':
				return '';
			default:
				return null;
		}
	}

	function selectCommunicationSection(sectionId: CommunicationSectionId) {
		currentView = 'communications';
		activeCommunicationSection = sectionId;
		if (!isSidebarRail) {
			setSidebarGroupExpanded('communications', true);
		}
		activeSidebarRailGroupId = null;
		isWidgetDrawerOpen = false;

		const workflowState = communicationSectionWorkflowState(sectionId);
		if (workflowState !== null) {
			mailStateFilter = workflowState;
			void loadCommunicationMessagesFiltered(workflowState || undefined);
		}
	}

	function toggleSidebarRail() {
		isSidebarRail = !isSidebarRail;
		activeSidebarRailGroupId = null;
		if (!isSidebarRail && currentView === 'communications') {
			setSidebarGroupExpanded('communications', true);
		}
	}

	function setCurrentView(viewId: AppViewId) {
		currentView = viewId;
		isWidgetDrawerOpen = false;
		activeSidebarRailGroupId = null;
		if (viewId === 'communications') {
			if (!isSidebarRail) {
				setSidebarGroupExpanded('communications', true);
			}
			const workflowState = communicationSectionWorkflowState(activeCommunicationSection);
			if (workflowState !== null) {
				mailStateFilter = workflowState;
				void loadCommunicationMessagesFiltered(workflowState || undefined);
			}
			return;
		}
	}

	function setView(item: NavItem) {
		if (!item.enabled) {
			return;
		}
		setCurrentView(item.id);
	}

	function openAccountDrawer(target: AccountWizardTarget = 'mail') {
		accountWizardKind = target === 'gmail' || target === 'icloud' || target === 'imap' ? 'mail' : target;
		if (target === 'gmail' || target === 'icloud' || target === 'imap') {
			selectMailService(target);
		}
		if (accountWizardKind === 'mail') {
			mailWizardStep = 'provider';
		}
		if (accountWizardKind === 'calendar') {
			calendarWizardStep = 'provider';
		}
		if (accountWizardKind === 'telegram') {
			telegramWizardStep = 'account';
		}
		telegramQrLogin = null;
		telegramQrPassword = '';
		isAccountDrawerOpen = true;
		setupMessage = '';
		setupError = '';
		telegramActionMessage = '';
		telegramError = '';
		whatsappActionMessage = '';
		whatsappError = '';
	}

	function closeAccountDrawer() {
		isAccountDrawerOpen = false;
	}

	function selectMailService(service: MailService) {
		selectedMailService = service;
		setupMessage = '';
		setupError = '';

		if (service === 'gmail') {
			selectedProvider = 'gmail';
			gmailForm = {
				...gmailForm,
				account_id: gmailForm.account_id || 'gmail-primary',
				display_name: gmailForm.display_name || 'Primary Gmail'
			};
			return;
		}

		const preset = mailServicePreset(service);
		selectedProvider = preset.provider;
		imapForm = {
			...imapForm,
			account_id: preset.accountId,
			display_name: preset.displayName,
			host: preset.host,
			port: preset.port,
			tls: true,
			mailbox: imapForm.mailbox || 'INBOX',
			secret_kind: preset.secretKind
		};
	}

	function continueMailWizard() {
		const email = mailAddress.trim();
		if (email) {
			const inferred = inferMailService(email);
			if (inferred) {
				selectMailService(inferred);
			}
			gmailForm = {
				...gmailForm,
				external_account_id: email,
				account_id: gmailForm.account_id || accountIdFromEmail(email, 'gmail'),
				display_name: gmailForm.display_name || email
			};
			imapForm = {
				...imapForm,
				external_account_id: email,
				username: imapForm.username || email,
				account_id: imapForm.account_id || accountIdFromEmail(email, 'mail'),
				display_name: imapForm.display_name || email
			};
		}
		mailWizardStep = 'details';
	}

	function continueCalendarWizard(provider?: CalendarProvider) {
		if (provider) {
			calendarAccountForm = {
				...calendarAccountForm,
				provider,
				account_name: calendarProviderDefaultName(provider)
			};
		}
		calendarWizardStep = 'details';
	}

	function continueTelegramWizard(nextStep: TelegramWizardStep) {
		telegramWizardStep = nextStep;
	}

	function selectTelegramAuthMethod(method: TelegramAuthMethod) {
		telegramAuthMethod = method;
		telegramSetupMode = method === 'fixture' ? 'fixture' : 'live';
		telegramQrLogin = null;
		telegramQrPassword = '';
		if (method === 'qr' && telegramAccountForm.external_account_id === '@telegram_fixture') {
			telegramAccountForm = {
				...telegramAccountForm,
				external_account_id: ''
			};
		} else if (method === 'bot_token') {
			telegramAccountForm = {
				...telegramAccountForm,
				provider_kind: 'telegram_bot'
			};
		}
	}

	function telegramWizardExternalAccountId() {
		return (
			telegramAccountForm.external_account_id.trim() ||
			(telegramAuthMethod === 'qr'
				? `qr-login:${telegramAccountForm.account_id}`
				: telegramAccountForm.account_id)
		);
	}

	function telegramQrStatusLabel(status: string) {
		switch (status) {
			case 'waiting_qr_scan':
				return 'Waiting for QR scan';
			case 'waiting_password':
				return 'Telegram password required';
			case 'ready':
				return 'Telegram authorized';
			case 'expired':
				return 'QR code expired';
			case 'failed':
				return 'QR login failed';
			case 'runtime_unavailable':
				return 'TDLib runtime unavailable';
			default:
				return 'QR login status';
		}
	}

	function applyTelegramQrLoginResult(result: TelegramQrLoginStatusResponse) {
		telegramQrLogin = result;
		if (result.status !== 'ready') {
			return;
		}
		telegramQrPassword = '';

		telegramAccountForm = {
			...telegramAccountForm,
			account_id: result.suggested_account_id ?? telegramAccountForm.account_id,
			display_name: result.suggested_display_name ?? telegramAccountForm.display_name,
			external_account_id:
				result.suggested_external_account_id ?? telegramAccountForm.external_account_id
		};
	}

	function telegramWizardNote() {
		if (telegramAuthMethod === 'fixture') {
			return 'Fixture mode creates local Telegram records for UI and policy testing.';
		}
		if (telegramAuthMethod === 'qr') {
			if (telegramCapabilities !== null && !telegramCapabilities.tdjson_runtime_available) {
				return 'TDLib JSON runtime is not available in the running backend.';
			}
			if (telegramQrNeedsFormAppCredentials) {
				return 'Enter Telegram API ID and API hash to start QR login in this dev session.';
			}
			if (telegramQrLogin?.status === 'waiting_password') {
				return 'Enter the Telegram 2-step verification password to finish local TDLib authorization.';
			}
			return 'Telegram app credentials are configured in the backend environment. QR login is ready.';
		}
		return 'Live credentials are stored in the encrypted database vault. Telegram live runtime remains blocked until the adapter is implemented.';
	}

	function selectProvider(provider: Provider) {
		selectedProvider = provider;
		selectedMailService = provider;
		setupMessage = '';
		setupError = '';

		if (provider === 'icloud') {
			imapForm = {
				...imapForm,
				account_id: imapForm.account_id || 'icloud-primary',
				display_name: imapForm.display_name || 'Primary iCloud',
				host: 'imap.mail.me.com',
				port: 993,
				tls: true,
				mailbox: imapForm.mailbox || 'INBOX',
				secret_kind: 'app_password'
			};
		}
		if (provider === 'imap') {
			imapForm = {
				...imapForm,
				account_id: imapForm.account_id === 'icloud-primary' ? 'imap-primary' : imapForm.account_id,
				display_name:
					imapForm.display_name === 'Primary iCloud' ? 'Primary IMAP' : imapForm.display_name,
				host: imapForm.host === 'imap.mail.me.com' ? '' : imapForm.host,
				secret_kind: 'password'
			};
		}
	}

	function mailServicePreset(service: MailService): {
		provider: Provider;
		accountId: string;
		displayName: string;
		host: string;
		port: number;
		secretKind: 'app_password' | 'password';
	} {
		switch (service) {
			case 'icloud':
				return {
					provider: 'icloud',
					accountId: 'icloud-primary',
					displayName: 'Primary iCloud',
					host: 'imap.mail.me.com',
					port: 993,
					secretKind: 'app_password'
				};
			case 'microsoft':
				return {
					provider: 'imap',
					accountId: 'microsoft-primary',
					displayName: 'Microsoft Mail',
					host: 'outlook.office365.com',
					port: 993,
					secretKind: 'password'
				};
			case 'yahoo':
				return {
					provider: 'imap',
					accountId: 'yahoo-primary',
					displayName: 'Yahoo Mail',
					host: 'imap.mail.yahoo.com',
					port: 993,
					secretKind: 'app_password'
				};
			case 'aol':
				return {
					provider: 'imap',
					accountId: 'aol-primary',
					displayName: 'AOL Mail',
					host: 'imap.aol.com',
					port: 993,
					secretKind: 'app_password'
				};
			default:
				return {
					provider: 'imap',
					accountId: 'imap-primary',
					displayName: 'IMAP Mail',
					host: imapForm.host === 'imap.mail.me.com' ? '' : imapForm.host,
					port: Number(imapForm.port) || 993,
					secretKind: 'password'
				};
		}
	}

	function hasFixedMailServerPreset(service: MailService) {
		return service !== 'imap';
	}

	function mailServiceDisplayName(service: MailService) {
		switch (service) {
			case 'gmail':
				return 'Google';
			case 'icloud':
				return 'iCloud';
			case 'microsoft':
				return 'Microsoft Exchange';
			case 'yahoo':
				return 'Yahoo';
			case 'aol':
				return 'AOL';
			default:
				return 'Other Mail Account';
		}
	}

	function mailServiceIcon(service: MailService) {
		switch (service) {
			case 'gmail':
				return 'tabler:brand-gmail';
			case 'icloud':
				return 'tabler:cloud';
			case 'microsoft':
				return 'tabler:brand-office';
			case 'yahoo':
				return 'tabler:mail';
			case 'aol':
				return 'tabler:mail-bolt';
			default:
				return 'tabler:server';
		}
	}

	function mailServiceAccountPrefix(service: MailService) {
		switch (service) {
			case 'icloud':
				return 'icloud';
			case 'microsoft':
				return 'microsoft';
			case 'yahoo':
				return 'yahoo';
			case 'aol':
				return 'aol';
			case 'gmail':
				return 'gmail';
			default:
				return 'imap';
		}
	}

	function inferMailService(email: string): MailService | null {
		const domain = email.split('@')[1]?.trim().toLowerCase() ?? '';
		if (['gmail.com', 'googlemail.com'].includes(domain)) return 'gmail';
		if (['icloud.com', 'me.com', 'mac.com'].includes(domain)) return 'icloud';
		if (['outlook.com', 'hotmail.com', 'live.com', 'office365.com'].includes(domain)) return 'microsoft';
		if (domain.endsWith('yahoo.com')) return 'yahoo';
		if (domain === 'aol.com') return 'aol';
		return null;
	}

	function accountIdFromEmail(email: string, fallback: string) {
		const normalized = email
			.trim()
			.toLowerCase()
			.replace(/[^a-z0-9]+/g, '-')
			.replace(/^-+|-+$/g, '');
		return normalized ? `${fallback}-${normalized}` : `${fallback}-primary`;
	}

	function calendarProviderDefaultName(provider: CalendarProvider) {
		switch (provider) {
			case 'google':
				return 'Google Calendar';
			case 'microsoft':
				return 'Microsoft Calendar';
			case 'apple':
				return 'Apple Calendar';
			case 'caldav':
				return 'CalDAV Calendar';
			case 'ics':
				return 'ICS Feed';
			default:
				return 'Local Calendar';
		}
	}

	async function startGmailSetup() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			gmailPending = await startGmailOAuthSetup(apiBaseUrl, apiSecret, {
				account_id: gmailForm.account_id,
				display_name: gmailForm.display_name,
				external_account_id: gmailForm.external_account_id,
				client_id: gmailForm.client_id,
				client_secret: gmailForm.client_secret || undefined,
				redirect_uri: gmailForm.redirect_uri
			});
			setupMessage = 'Gmail OAuth grant started';
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Gmail setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function completeGmailSetup() {
		if (!gmailPending) {
			setupError = 'Gmail OAuth grant has not been started';
			return;
		}

		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			const result = await completeGmailOAuthSetup(apiBaseUrl, apiSecret, {
				setup_id: gmailPending.setup_id,
				state: gmailPending.state,
				authorization_code: gmailAuthorizationCode
			});
			setupMessage = `Gmail account ${result.account_id} saved`;
			gmailAuthorizationCode = '';
			gmailPending = null;
			await loadSettingsWorkspace();
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Gmail setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function saveImapAccount() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			const username = imapForm.username.trim();
			const fixedServerPreset = hasFixedMailServerPreset(selectedMailService);
			const externalAccountId = fixedServerPreset
				? username
				: imapForm.external_account_id.trim() || username;
			const accountId = fixedServerPreset
				? accountIdFromEmail(externalAccountId, mailServiceAccountPrefix(selectedMailService))
				: imapForm.account_id.trim() || accountIdFromEmail(externalAccountId, 'imap');
			const displayName = fixedServerPreset
				? externalAccountId || mailServiceDisplayName(selectedMailService)
				: imapForm.display_name.trim() || externalAccountId || mailServiceDisplayName(selectedMailService);
			const host = imapForm.host.trim();
			const mailbox = imapForm.mailbox.trim() || 'INBOX';

			const result = await setupImapAccount(apiBaseUrl, apiSecret, {
				account_id: accountId,
				provider_kind: selectedProvider === 'icloud' ? 'icloud' : 'imap',
				display_name: displayName,
				external_account_id: externalAccountId,
				host,
				port: Number(imapForm.port),
				tls: imapForm.tls,
				mailbox,
				username,
				password: imapForm.password,
				secret_kind: imapForm.secret_kind
			});
			setupMessage = `Mail account ${result.account_id} saved`;
			imapForm = { ...imapForm, password: '' };
			await loadSettingsWorkspace();
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Mail account setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function saveCalendarAccount() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			const result = await createCalendarAccount(apiBaseUrl, apiSecret, {
				provider: calendarAccountForm.provider,
				account_name: calendarAccountForm.account_name,
				email: calendarAccountForm.email.trim() || undefined
			});
			setupMessage = `Calendar account ${result.account_name} saved`;
			await Promise.all([loadCalendar(), loadSettingsWorkspace()]);
		} catch (error) {
			setupError = error instanceof Error ? error.message : 'Calendar account setup failed';
		} finally {
			isSetupSubmitting = false;
		}
	}

	async function loadTelegramWorkspace() {
		isTelegramLoading = true;
		try {
			const [
				capabilityResponse,
				chatResponse,
				messageResponse,
				templateResponse,
				policyResponse,
				callResponse
			] =
				await Promise.all([
					fetchTelegramCapabilities(apiBaseUrl, apiSecret),
					fetchTelegramChats(apiBaseUrl, apiSecret),
					fetchTelegramMessages(apiBaseUrl, apiSecret),
					fetchAutomationTemplates(apiBaseUrl, apiSecret),
					fetchAutomationPolicies(apiBaseUrl, apiSecret),
					fetchTelegramCalls(apiBaseUrl, apiSecret)
				]);

			telegramCapabilities = capabilityResponse;
			telegramChats = chatResponse.items;
			telegramMessages = messageResponse.items;
			automationTemplates = templateResponse.items;
			automationPolicies = policyResponse.items;
			telegramCalls = callResponse.items;

			if (!telegramChats.some((chat) => chat.provider_chat_id === selectedTelegramChatId)) {
				selectedTelegramChatId = telegramChats[0]?.provider_chat_id ?? '';
			}
			if (!telegramCalls.some((call) => call.call_id === selectedTelegramCallId)) {
				selectedTelegramCallId = telegramCalls[0]?.call_id ?? '';
			}
			if (selectedTelegramCallId) {
				await loadSelectedCallTranscript(selectedTelegramCallId);
			} else {
				callTranscript = null;
			}

			telegramError = '';
		} catch (error) {
			telegramError = error instanceof Error ? error.message : 'Unknown Telegram workspace error';
			callTranscript = null;
		} finally {
			isTelegramLoading = false;
		}
	}

	async function loadWhatsappWebWorkspace() {
		isWhatsappLoading = true;
		try {
			const [capabilityResponse, sessionResponse, messageResponse] = await Promise.all([
				fetchWhatsappCapabilities(apiBaseUrl, apiSecret),
				fetchWhatsappWebSessions(apiBaseUrl, apiSecret),
				fetchWhatsappWebMessages(apiBaseUrl, apiSecret)
			]);

			whatsappCapabilities = capabilityResponse;
			whatsappSessions = sessionResponse.items;
			whatsappMessages = messageResponse.items;

			if (!whatsappSessions.some((session) => session.session_id === selectedWhatsappSessionId)) {
				selectedWhatsappSessionId = whatsappSessions[0]?.session_id ?? '';
			}

			whatsappError = '';
		} catch (error) {
			whatsappError = error instanceof Error ? error.message : 'Unknown WhatsApp Web workspace error';
		} finally {
			isWhatsappLoading = false;
		}
	}

	async function setupWhatsappWebFixture() {
		if (isWhatsappActionSubmitting) {
			return;
		}

		isWhatsappActionSubmitting = true;
		whatsappActionMessage = '';
		whatsappError = '';
		setupMessage = '';
		setupError = '';
		try {
			const result = await setupWhatsappWebFixtureAccount(apiBaseUrl, apiSecret, {
				account_id: whatsappAccountForm.account_id,
				provider_kind: 'whatsapp_web',
				display_name: whatsappAccountForm.display_name,
				external_account_id: whatsappAccountForm.external_account_id,
				device_name: whatsappAccountForm.device_name,
				local_state_path: whatsappAccountForm.local_state_path
			});
			selectedWhatsappSessionId = result.session.session_id;
			whatsappMessageForm = {
				...whatsappMessageForm,
				account_id: result.account_id
			};
			whatsappActionMessage = `${providerKindLabel(result.provider_kind)} account ${result.account_id} saved`;
			setupMessage = whatsappActionMessage;
			await Promise.all([loadWhatsappWebWorkspace(), loadSettingsWorkspace()]);
		} catch (error) {
			const message = error instanceof Error ? error.message : 'WhatsApp Web fixture setup failed';
			whatsappError = message;
			setupError = message;
		} finally {
			isWhatsappActionSubmitting = false;
		}
	}

	async function ingestWhatsappWebMessageFixture() {
		if (isWhatsappActionSubmitting) {
			return;
		}

		isWhatsappActionSubmitting = true;
		whatsappActionMessage = '';
		whatsappError = '';
		try {
			const providerMessageId =
				whatsappMessageForm.provider_message_id.trim() || `wa-fixture-msg-${crypto.randomUUID()}`;
			const result = await ingestWhatsappWebFixtureMessage(apiBaseUrl, apiSecret, {
				account_id: whatsappMessageForm.account_id,
				provider_chat_id: whatsappMessageForm.provider_chat_id,
				provider_message_id: providerMessageId,
				chat_title: whatsappMessageForm.chat_title,
				sender_id: whatsappMessageForm.sender_id,
				sender_display_name: whatsappMessageForm.sender_display_name,
				text: whatsappMessageForm.text,
				import_batch_id: whatsappMessageForm.import_batch_id,
				occurred_at: whatsappMessageForm.occurred_at || new Date().toISOString(),
				delivery_state: whatsappMessageForm.delivery_state
			});
			whatsappActionMessage = `WhatsApp Web message ${result.message_id} projected`;
			whatsappMessageForm = {
				...whatsappMessageForm,
				provider_message_id: `wa-fixture-msg-${crypto.randomUUID()}`,
				occurred_at: new Date().toISOString()
			};
			await Promise.all([loadWhatsappWebWorkspace(), loadCommunications()]);
		} catch (error) {
			whatsappError = error instanceof Error ? error.message : 'WhatsApp Web fixture ingest failed';
		} finally {
			isWhatsappActionSubmitting = false;
		}
	}

	async function saveTelegramAccountFromWizard(options: { allowWhileSubmitting?: boolean } = {}) {
		if (isTelegramActionSubmitting && !options.allowWhileSubmitting) {
			return;
		}
		const shouldResetSubmitting = !isTelegramActionSubmitting;

		const isFixtureSetup = telegramAuthMethod === 'fixture';
		const providerKind =
			telegramAuthMethod === 'bot_token'
				? 'telegram_bot'
				: telegramAuthMethod === 'phone' || telegramAuthMethod === 'qr'
					? 'telegram_user'
					: telegramAccountForm.provider_kind;
		const externalAccountId = telegramWizardExternalAccountId();

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		setupMessage = '';
		setupError = '';
		try {
			const result =
				isFixtureSetup
					? await setupTelegramFixtureAccount(apiBaseUrl, apiSecret, {
							account_id: telegramAccountForm.account_id,
							provider_kind: providerKind,
							display_name: telegramAccountForm.display_name,
							external_account_id: externalAccountId,
							tdlib_data_path:
								telegramAuthMethod === 'qr'
									? undefined
									: telegramAccountForm.tdlib_data_path || undefined,
							transcription_enabled:
								telegramAuthMethod === 'qr' ? false : telegramAccountForm.transcription_enabled
						})
					: await setupTelegramAccount(apiBaseUrl, apiSecret, {
							account_id: telegramAccountForm.account_id,
							provider_kind: providerKind,
							display_name: telegramAccountForm.display_name,
							external_account_id: externalAccountId,
							api_id:
								providerKind === 'telegram_user' && telegramAccountForm.api_id.trim()
									? Number(telegramAccountForm.api_id.trim())
									: undefined,
							api_hash:
								providerKind === 'telegram_user'
									? telegramAccountForm.api_hash.trim() || undefined
									: undefined,
							bot_token:
								providerKind === 'telegram_bot'
									? telegramAccountForm.bot_token || undefined
									: undefined,
							session_encryption_key:
								providerKind === 'telegram_user'
									? telegramAccountForm.session_encryption_key || undefined
									: undefined,
							tdlib_data_path:
								telegramAuthMethod === 'qr'
									? undefined
									: telegramAccountForm.tdlib_data_path || undefined,
							transcription_enabled:
								telegramAuthMethod === 'qr' ? false : telegramAccountForm.transcription_enabled
						});
			const runtimeLabel =
				telegramAuthMethod === 'qr' && telegramQrLogin?.status === 'ready'
					? 'saved after QR authorization'
					: result.runtime === 'live_blocked'
						? 'saved as live-blocked'
						: 'saved';
			telegramActionMessage = `${providerKindLabel(result.provider_kind)} account ${result.account_id} ${runtimeLabel}`;
			setupMessage = telegramActionMessage;
			telegramAccountForm = {
				...telegramAccountForm,
				api_hash: '',
				bot_token: '',
				session_encryption_key: ''
			};
			telegramMessageForm = {
				...telegramMessageForm,
				account_id: result.account_id
			};
			automationPolicyForm = {
				...automationPolicyForm,
				account_id: result.account_id
			};
			telegramCallForm = {
				...telegramCallForm,
				account_id: result.account_id
			};
			transcriptForm = {
				...transcriptForm,
				account_id: result.account_id
			};
			await Promise.all([loadTelegramWorkspace(), loadSettingsWorkspace()]);
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Telegram account setup failed';
			telegramError = message;
			setupError = message;
		} finally {
			if (shouldResetSubmitting) {
				isTelegramActionSubmitting = false;
			}
		}
	}

	async function saveReadyTelegramQrAccountFromWizard() {
		if (telegramAuthMethod !== 'qr' || telegramQrLogin?.status !== 'ready') {
			return;
		}
		await saveTelegramAccountFromWizard({ allowWhileSubmitting: true });
	}

	async function startTelegramQrLoginFromWizard() {
		if (isTelegramActionSubmitting) {
			return;
		}

		let capabilities = telegramCapabilities;
		if (!capabilities) {
			try {
				capabilities = await fetchTelegramCapabilities(apiBaseUrl, apiSecret);
				telegramCapabilities = capabilities;
			} catch {
				capabilities = null;
			}
		}

		if (capabilities && !capabilities.tdjson_runtime_available) {
			setupError = 'TDLib JSON runtime is not available in the running backend';
			telegramError = setupError;
			return;
		}

		const apiIdValue = telegramAccountForm.api_id.trim();
		const apiHashValue = telegramAccountForm.api_hash.trim();
		const appCredentialsConfigured = capabilities?.telegram_app_credentials_configured ?? false;
		if (!appCredentialsConfigured && (!apiIdValue || !apiHashValue)) {
			setupError = 'Telegram API ID and API hash are required for QR login in this dev session';
			telegramError = setupError;
			return;
		}
		const parsedApiId = Number(apiIdValue);
		if (apiIdValue && (!Number.isInteger(parsedApiId) || parsedApiId <= 0)) {
			setupError = 'Telegram API ID must be greater than zero';
			telegramError = setupError;
			return;
		}
		const apiId = apiIdValue ? parsedApiId : undefined;

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		setupMessage = '';
		setupError = '';
		telegramQrLogin = null;
		telegramQrPassword = '';

		try {
			const result = await startTelegramQrLogin(apiBaseUrl, apiSecret, {
				account_id: telegramAccountForm.account_id,
				display_name: telegramAccountForm.display_name,
				external_account_id: telegramWizardExternalAccountId(),
				api_id: apiId,
				api_hash: apiHashValue || undefined,
				session_encryption_key: telegramAccountForm.session_encryption_key || undefined,
				tdlib_data_path: undefined,
				transcription_enabled: false
			});
			applyTelegramQrLoginResult(result);
			if (result.status === 'ready') {
				await saveReadyTelegramQrAccountFromWizard();
			} else {
				setupMessage =
					result.status === 'waiting_qr_scan'
						? 'Scan the Telegram QR code to continue'
						: result.message ?? `Telegram QR login status: ${result.status}`;
			}
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Telegram QR login start failed';
			setupError = message;
			telegramError = message;
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	async function submitTelegramQrPasswordFromWizard() {
		if (isTelegramActionSubmitting || !telegramQrLogin) {
			return;
		}

		if (telegramQrLogin.status !== 'waiting_password') {
			setupError = 'Telegram QR login is not waiting for a password';
			telegramError = setupError;
			return;
		}

		if (!telegramQrPassword) {
			setupError = 'Telegram 2-step verification password is required';
			telegramError = setupError;
			return;
		}

		isTelegramActionSubmitting = true;
		setupError = '';
		telegramError = '';
		try {
			const result = await submitTelegramQrLoginPassword(
				apiBaseUrl,
				apiSecret,
				telegramQrLogin.setup_id,
				{ password: telegramQrPassword }
			);
			telegramQrPassword = '';
			applyTelegramQrLoginResult(result);
			if (result.status === 'ready') {
				await saveReadyTelegramQrAccountFromWizard();
			} else {
				setupMessage = result.message ?? `Telegram QR login status: ${result.status}`;
			}
		} catch (error) {
			const message =
				error instanceof Error ? error.message : 'Telegram QR login password submit failed';
			setupError = message;
			telegramError = message;
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	function submitTelegramQrStepFromWizard() {
		if (telegramQrLogin?.status === 'waiting_password') {
			void submitTelegramQrPasswordFromWizard();
			return;
		}
		void startTelegramQrLoginFromWizard();
	}

	async function refreshTelegramQrLoginStatus() {
		if (!telegramQrLogin || isTelegramActionSubmitting) {
			return;
		}

		isTelegramActionSubmitting = true;
		setupError = '';
		telegramError = '';
		try {
			const result = await fetchTelegramQrLoginStatus(
				apiBaseUrl,
				apiSecret,
				telegramQrLogin.setup_id
			);
			applyTelegramQrLoginResult(result);
			if (result.status === 'ready') {
				await saveReadyTelegramQrAccountFromWizard();
			} else {
				setupMessage = result.message ?? `Telegram QR login status: ${result.status}`;
			}
		} catch (error) {
			const message =
				error instanceof Error ? error.message : 'Telegram QR login status request failed';
			setupError = message;
			telegramError = message;
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	async function ingestTelegramMessageFixture() {
		if (isTelegramActionSubmitting) {
			return;
		}

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		try {
			const providerMessageId =
				telegramMessageForm.provider_message_id.trim() || `fixture-msg-${crypto.randomUUID()}`;
			const result = await ingestTelegramFixtureMessage(apiBaseUrl, apiSecret, {
				account_id: telegramMessageForm.account_id,
				provider_chat_id: telegramMessageForm.provider_chat_id,
				provider_message_id: providerMessageId,
				chat_kind: telegramMessageForm.chat_kind,
				chat_title: telegramMessageForm.chat_title,
				sender_id: telegramMessageForm.sender_id,
				sender_display_name: telegramMessageForm.sender_display_name,
				text: telegramMessageForm.text,
				import_batch_id: telegramMessageForm.import_batch_id,
				occurred_at: telegramMessageForm.occurred_at || new Date().toISOString(),
				delivery_state: telegramMessageForm.delivery_state
			});
			selectedTelegramChatId = telegramMessageForm.provider_chat_id;
			telegramActionMessage = `Telegram message ${result.message_id} projected`;
			telegramMessageForm = {
				...telegramMessageForm,
				provider_message_id: `fixture-msg-${crypto.randomUUID()}`,
				occurred_at: new Date().toISOString()
			};
			await Promise.all([loadTelegramWorkspace(), loadCommunications()]);
		} catch (error) {
			telegramError = error instanceof Error ? error.message : 'Telegram fixture ingest failed';
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	async function saveTelegramAutomationTemplate() {
		if (isTelegramActionSubmitting) {
			return;
		}

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		try {
			const template = await saveAutomationTemplate(apiBaseUrl, apiSecret, {
				template_id: automationTemplateForm.template_id,
				name: automationTemplateForm.name,
				body_template: automationTemplateForm.body_template,
				required_variables: splitList(automationTemplateForm.required_variables_text)
			});
			telegramActionMessage = `Template ${template.template_id} saved`;
			automationPolicyForm = {
				...automationPolicyForm,
				template_id: template.template_id
			};
			await loadTelegramWorkspace();
		} catch (error) {
			telegramError = error instanceof Error ? error.message : 'Automation template save failed';
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	async function saveTelegramAutomationPolicy() {
		if (isTelegramActionSubmitting) {
			return;
		}

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		try {
			const policy = await saveAutomationPolicy(apiBaseUrl, apiSecret, {
				policy_id: automationPolicyForm.policy_id,
				template_id: automationPolicyForm.template_id,
				name: automationPolicyForm.name,
				enabled: automationPolicyForm.enabled,
				account_id: automationPolicyForm.account_id,
				allowed_chat_ids: splitList(automationPolicyForm.allowed_chat_ids_text),
				trigger_kind: automationPolicyForm.trigger_kind,
				max_sends_per_hour: Number(automationPolicyForm.max_sends_per_hour),
				quiet_hours: parseJsonObject(automationPolicyForm.quiet_hours_text, 'quiet hours'),
				expires_at: automationPolicyForm.expires_at.trim() || null,
				conditions: parseJsonObject(automationPolicyForm.conditions_text, 'conditions')
			});
			telegramActionMessage = `Policy ${policy.policy_id} saved`;
			telegramSendForm = {
				...telegramSendForm,
				policy_id: policy.policy_id
			};
			await loadTelegramWorkspace();
		} catch (error) {
			telegramError = error instanceof Error ? error.message : 'Automation policy save failed';
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	async function runTelegramAutomationDryRun() {
		if (isTelegramActionSubmitting) {
			return;
		}

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		telegramSendDryRunResult = null;
		try {
			const result = await dryRunTelegramSend(apiBaseUrl, apiSecret, {
				command_id: `telegram-dry-run-${crypto.randomUUID()}`,
				policy_id: telegramSendForm.policy_id,
				provider_chat_id: telegramSendForm.provider_chat_id,
				variables: parseStringMap(telegramSendForm.variables_text, 'variables'),
				source_context: parseJsonObject(telegramSendForm.source_context_text, 'source context')
			});
			telegramSendDryRunResult = result;
			telegramActionMessage = `Dry-run accepted with preview hash ${result.rendered_preview_hash}`;
			await loadTelegramWorkspace();
		} catch (error) {
			telegramError = error instanceof Error ? error.message : 'Telegram send dry-run failed';
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	async function saveTelegramCallFixture() {
		if (isTelegramActionSubmitting) {
			return;
		}

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		try {
			const call = await saveTelegramCall(apiBaseUrl, apiSecret, {
				call_id: telegramCallForm.call_id,
				account_id: telegramCallForm.account_id,
				provider_call_id: telegramCallForm.provider_call_id,
				provider_chat_id: telegramCallForm.provider_chat_id,
				direction: telegramCallForm.direction,
				call_state: telegramCallForm.call_state,
				started_at: telegramCallForm.started_at.trim() || null,
				ended_at: telegramCallForm.ended_at.trim() || null,
				transcription_policy_id: telegramCallForm.transcription_policy_id.trim() || null,
				metadata: parseJsonObject(telegramCallForm.metadata_text, 'call metadata')
			});
			selectedTelegramCallId = call.call_id;
			telegramActionMessage = `Call ${call.call_id} saved`;
			await loadTelegramWorkspace();
		} catch (error) {
			telegramError = error instanceof Error ? error.message : 'Telegram call save failed';
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	async function saveCallTranscriptFixtureFromUi() {
		if (isTelegramActionSubmitting || !selectedTelegramCallId) {
			return;
		}

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		try {
			callTranscript = await saveCallTranscriptFixture(
				apiBaseUrl, apiSecret,
				selectedTelegramCallId,
				{
					transcript_id: transcriptForm.transcript_id,
					account_id: transcriptForm.account_id,
					provider_chat_id: transcriptForm.provider_chat_id,
					source_audio_ref: transcriptForm.source_audio_ref,
					language_code: transcriptForm.language_code || undefined,
					always_on_policy: transcriptForm.always_on_policy
				}
			);
			telegramActionMessage = `Transcript ${callTranscript.transcript_id} saved`;
			await loadTelegramWorkspace();
		} catch (error) {
			telegramError = error instanceof Error ? error.message : 'Call transcript save failed';
		} finally {
			isTelegramActionSubmitting = false;
		}
	}

	async function loadSelectedCallTranscript(callId = selectedTelegramCallId) {
		if (!callId) {
			callTranscript = null;
			return;
		}

		try {
			const response = await fetchCallTranscript(apiBaseUrl, apiSecret, callId);
			callTranscript = response.transcript;
			telegramError = '';
		} catch (error) {
			callTranscript = null;
			telegramError = error instanceof Error ? error.message : 'Call transcript request failed';
		}
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

	function selectWhatsappSession(session: WhatsappWebSession) {
		selectedWhatsappSessionId = session.session_id;
		whatsappMessageForm = {
			...whatsappMessageForm,
			account_id: session.account_id
		};
	}

	function splitList(value: string) {
		return value
			.split(',')
			.map((item) => item.trim())
			.filter(Boolean);
	}

	function parseJsonObject(value: string, field: string): Record<string, unknown> {
		const trimmed = value.trim();
		if (!trimmed) {
			return {};
		}

		const parsed = JSON.parse(trimmed) as unknown;
		if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
			throw new Error(`${field} must be a JSON object`);
		}
		return parsed as Record<string, unknown>;
	}

	function parseStringMap(value: string, field: string): Record<string, string> {
		const parsed = parseJsonObject(value, field);
		return Object.fromEntries(
			Object.entries(parsed).map(([key, rawValue]) => {
				if (typeof rawValue !== 'string') {
					throw new Error(`${field}.${key} must be a string`);
				}
				return [key, rawValue];
			})
		);
	}

	function telegramChatKindValue(value: string): 'private' | 'group' | 'channel' | 'bot' {
		if (value === 'group' || value === 'channel' || value === 'bot') {
			return value;
		}
		return 'private';
	}

	function providerKindLabel(value: string) {
		return value
			.split('_')
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function capabilityLabel(value: string) {
		return value
			.split('_')
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function communicationChannelIcon(channelKind: string) {
		if (channelKind === 'telegram_user' || channelKind === 'telegram_bot') {
			return 'tabler:brand-telegram';
		}
		if (channelKind === 'whatsapp_web') {
			return 'tabler:brand-whatsapp';
		}
		return 'tabler:mail';
	}

	function communicationChannelLabel(channelKind: string) {
		if (channelKind === 'telegram_user') {
			return 'Telegram user';
		}
		if (channelKind === 'telegram_bot') {
			return 'Telegram bot';
		}
		if (channelKind === 'whatsapp_web') {
			return 'WhatsApp Web';
		}
		return 'Email';
	}

	function graphNodeTotal() {
		return graphSummary?.node_counts.reduce((total, item) => total + item.count, 0) ?? 0;
	}

	function graphRelationshipTotal() {
		return graphSummary?.edge_counts.reduce((total, item) => total + item.count, 0) ?? 0;
	}

	function formatNumber(value: number) {
		return new Intl.NumberFormat('en-US').format(value);
	}

	function formatGraphKind(kind: GraphNodeKind | string) {
		return kind
			.split('_')
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function graphNodeKindIcon(kind: GraphNodeKind | string) {
		switch (kind) {
			case 'person':
				return 'tabler:user';
			case 'email_address':
				return 'tabler:mail';
			case 'message':
				return 'tabler:message';
			case 'document':
				return 'tabler:file-text';
			case 'project':
				return 'tabler:cube';
			default:
				return 'tabler:circle-dot';
		}
	}

	function graphNodeKindCount(kind: GraphNodeKind) {
		return graphSummary?.node_counts.find((item) => item.key === kind)?.count ?? 0;
	}

	function graphEvidenceTotal() {
		return graphSummary?.evidence_count ?? 0;
	}

	function buildGraphFilterChips(summary: GraphSummary | null): GraphFilterChip[] {
		const nodeKinds: Array<{ id: GraphNodeKind; label: string }> = [
			{ id: 'person', label: 'People' },
			{ id: 'email_address', label: 'Email Addresses' },
			{ id: 'message', label: 'Messages' },
			{ id: 'document', label: 'Documents' },
			{ id: 'project', label: 'Projects' }
		];

		return [
			{
				id: 'all',
				label: 'All',
				count: summary?.node_counts.reduce((total, item) => total + item.count, 0) ?? 0,
				enabled: true
			},
			...nodeKinds.map((item) => ({
				id: item.id,
				label: item.label,
				count: summary?.node_counts.find((count) => count.key === item.id)?.count ?? 0,
				enabled: false
			}))
		];
	}

	function buildGraphCanvasNodes(neighborhood: GraphNeighborhood | null): GraphCanvasNode[] {
		if (!neighborhood) {
			return [];
		}

		const selected = neighborhood.selected_node;
		const neighbors = neighborhood.nodes
			.filter((node) => node.node_id !== selected.node_id)
			.slice(0, 14);
		const radius = 38;

		return [
			{ ...selected, x: 50, y: 50, isSelected: true, layoutClass: 'graph-node-position-center' },
			...neighbors.map((node, index) => {
				const angle = (Math.PI * 2 * index) / Math.max(neighbors.length, 1) - Math.PI / 2;
				return {
					...node,
					x: 50 + Math.cos(angle) * radius,
					y: 50 + Math.sin(angle) * radius,
					isSelected: false,
					layoutClass: `graph-node-position-${index}`
				};
			})
		];
	}

	function buildGraphCanvasEdges(
		neighborhood: GraphNeighborhood | null,
		canvasNodes: GraphCanvasNode[]
	): GraphCanvasEdge[] {
		if (!neighborhood) {
			return [];
		}

		const positions = new Map(canvasNodes.map((node) => [node.node_id, node]));
		return neighborhood.edges.flatMap((edge) => {
			const source = positions.get(edge.source_node_id);
			const target = positions.get(edge.target_node_id);
			if (!source || !target) {
				return [];
			}
			return [
				{
					...edge,
					x1: source.x,
					y1: source.y,
					x2: target.x,
					y2: target.y,
					label: formatGraphRelationship(edge.relationship_type)
				}
			];
		});
	}

	function graphNeighborNodes(neighborhood: GraphNeighborhood | null): GraphNode[] {
		if (!neighborhood) {
			return [];
		}
		return neighborhood.nodes.filter(
			(node) => node.node_id !== neighborhood.selected_node.node_id
		);
	}

	function graphKindCounts(nodes: GraphNode[]): Array<{ kind: GraphNodeKind; count: number }> {
		const counts = new Map<GraphNodeKind, number>();
		for (const node of nodes) {
			counts.set(node.node_kind, (counts.get(node.node_kind) ?? 0) + 1);
		}
		return Array.from(counts.entries())
			.map(([kind, count]) => ({ kind, count }))
			.sort((left, right) => right.count - left.count || left.kind.localeCompare(right.kind));
	}

	function graphPropertyRows(properties: Record<string, unknown>): GraphPropertyRow[] {
		return Object.entries(properties)
			.map(([key, value]) => ({ key, value: formatGraphPropertyValue(value) }))
			.filter((row) => row.value.length > 0)
			.sort((left, right) => left.key.localeCompare(right.key))
			.slice(0, 8);
	}

	function formatGraphPropertyValue(value: unknown): string {
		if (value === null || value === undefined) {
			return '';
		}
		if (Array.isArray(value)) {
			return value.map(formatGraphPropertyValue).filter(Boolean).join(', ');
		}
		if (typeof value === 'object') {
			return JSON.stringify(value);
		}
		return String(value);
	}

	function formatGraphRelationship(type: GraphRelationshipType | string) {
		return type
			.split('_')
			.filter((part) => !['person', 'email', 'address', 'message'].includes(part))
			.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
			.join(' ');
	}

	function formatGraphTimestamp(value: string | null) {
		if (!value) {
			return 'No projection yet';
		}
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) {
			return 'Invalid timestamp';
		}
		return new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		}).format(date);
	}

	function graphEvidenceLabel(evidence: GraphEvidenceSummary) {
		return `${formatGraphKind(evidence.source_kind)} ${evidence.source_id}`;
	}

	function taskSourceLabel(item: TaskCandidate | Task) {
		return `${item.source_kind[0].toUpperCase()}${item.source_kind.slice(1)} · ${item.source_id}`;
	}

	function taskConfidence(item: TaskCandidate) {
		return `${Math.round(item.confidence * 100)}%`;
	}

	function identityConfidence(item: PersonIdentityCandidate) {
		return `${Math.round(item.confidence * 100)}%`;
	}

	function splitCandidateForConfirmedMerge(candidate: PersonIdentityCandidate) {
		return splitCandidateForMerge(candidate, 'suggested');
	}

	function confirmedSplitCandidateForMerge(candidate: PersonIdentityCandidate) {
		return splitCandidateForMerge(candidate, 'user_confirmed');
	}

	function splitCandidateForMerge(
		candidate: PersonIdentityCandidate,
		reviewState: PersonIdentityReviewState
	) {
		if (!candidate.right_person_id) {
			return null;
		}
		const pairKey = personIdentityPairKey(candidate.left_person_id, candidate.right_person_id);
		return (
			identityCandidates.find(
				(item) =>
					item.candidate_kind === 'split_person' &&
					item.review_state === reviewState &&
					item.right_person_id !== null &&
					personIdentityPairKey(item.left_person_id, item.right_person_id) === pairKey
			) ?? null
		);
	}

	function personIdentityPairKey(leftPersonId: string, rightPersonId: string) {
		return leftPersonId <= rightPersonId
			? `${leftPersonId}:${rightPersonId}`
			: `${rightPersonId}:${leftPersonId}`;
	}

	function taskCreatedTime(value: string | null) {
		if (!value) {
			return '';
		}
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) {
			return 'Unknown date';
		}
		return new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		}).format(date);
	}
	</script>

{#snippet widgetEditChrome(widgetId: string)}
	{#if isLayoutEditing}
		<div class="widget-edit-chrome">
			<button
				type="button"
				class="widget-config-button"
				title={_('Configure widget')}
				aria-label={_('Configure widget')}
				aria-expanded={selectedLayoutWidgetId === widgetId}
				onclick={() => openWidgetSettingsDrawer(widgetId)}
			>
				<Icon icon="tabler:adjustments-horizontal" width="15" height="15" />
			</button>
		</div>
	{/if}
{/snippet}

<svelte:head>
	<title>Hermes Hub</title>
	<meta name="description" content="Hermes Hub desktop personal OS dashboard." />
</svelte:head>

<main class={`desktop-shell view-${activeWorkspaceView} ${shellThemeClass}`} class:sidebar-rail={isSidebarRail}>
	{#if !isVaultReady}
		<section class="vault-onboarding" aria-label="Secure vault onboarding" onmousemove={handleVaultEntropyMove}>
			<div class="vault-panel">
				<div class="vault-panel__header">
					<div class="vault-emblem"><Icon icon="tabler:shield-lock" width="30" height="30" /></div>
					<div>
						<p class="vault-kicker">Hermes Secure Vault</p>
						<h1>Create Your Personal Secure Vault</h1>
					</div>
				</div>

				{#if vaultWizardStep === 'intro'}
					<div class="vault-step">
						<p>Hermes Hub encrypts credentials stored on this Mac. Secrets live in a dedicated host vault under <strong>~/.hermes/vault</strong>; PostgreSQL keeps only non-secret bindings.</p>
						<p class="vault-warning">Если потерять recovery phrase/file, доступ к зашифрованным секретам может стать невозможным.</p>
						<div class="vault-actions">
							<button type="button" onclick={startVaultWizard}>Start Entropy Collection</button>
						</div>
					</div>
				{:else if vaultWizardStep === 'entropy'}
					<div class="vault-step">
						<p>Move your mouse around the screen. Hermes combines OS randomness, timing entropy and mouse movement before creating the master key.</p>
						<div class="vault-entropy-canvas">
							<div class="vault-entropy-meter">
								<span>Entropy</span>
								<strong>{vaultStatus?.entropy_progress ?? 0}%</strong>
							</div>
							<progress class="vault-progress" value={vaultStatus?.entropy_progress ?? 0} max="100"></progress>
							<p>{Math.min(vaultEntropyEvents.length, 2000)} / 2000 events</p>
						</div>
						<div class="vault-actions">
							<button type="button" onclick={createSecureVault} disabled={(vaultStatus?.entropy_progress ?? 0) < 100 || isVaultActionSubmitting}>Create Vault</button>
						</div>
					</div>
				{:else if vaultWizardStep === 'biometric'}
					<div class="vault-step">
						<p>Vault material is ready. In release runtime Hermes will use macOS Keychain as source-of-truth for the master key. Docker dev uses the configured dev key path.</p>
						<div class="vault-actions">
							<button type="button" onclick={createSecureVault} disabled={isVaultActionSubmitting}>Create Vault</button>
							<button type="button" onclick={unlockSecureVault} disabled={isVaultActionSubmitting}>Unlock Existing Vault</button>
						</div>
					</div>
				{:else if vaultWizardStep === 'recovery'}
					<div class="vault-step">
						<p>Export recovery material before continuing. Store the phrase and file safely outside Hermes.</p>
						<p class="vault-warning">Без recovery phrase/file восстановление после переустановки или потери Keychain-доступа невозможно.</p>
						<div class="vault-actions">
							<button type="button" onclick={exportRecoveryMaterial} disabled={isVaultActionSubmitting}>Export Recovery</button>
						</div>
					</div>
				{:else}
					<div class="vault-step">
						<p>Vault is ready. Recovery file: <strong>{vaultRecovery?.path ?? '~/.hermes/vault/hermes-recovery.key'}</strong></p>
						{#if vaultRecovery?.recovery_phrase}
							<div class="vault-recovery-phrase">{vaultRecovery.recovery_phrase}</div>
						{/if}
						<div class="vault-actions">
							<button type="button" onclick={() => void loadV1Status()}>Continue</button>
						</div>
					</div>
				{/if}

				{#if vaultWizardMessage}<p class="vault-state success">{vaultWizardMessage}</p>{/if}
				{#if vaultWizardError}<p class="vault-state error">{vaultWizardError}</p>{/if}
				{#if statusError}<p class="vault-state error">{statusError}</p>{/if}
			</div>
		</section>
	{/if}
	<aside class="sidebar" class:rail={isSidebarRail} aria-label="Hermes Hub navigation">
		<div class="brand">
			<button
				type="button"
				class="brand-mark-button"
				aria-label={isSidebarRail ? 'Expand sidebar' : 'Collapse sidebar'}
				aria-pressed={isSidebarRail}
				title={isSidebarRail ? 'Expand sidebar' : 'Collapse sidebar'}
				onclick={toggleSidebarRail}
			>
				<img src="/assets/hermes-logo-mark.png" alt="" class="brand-mark" />
			</button>
			<div class="brand-copy">
				<p class="brand-name">{_('Hermes Hub')}</p>
				<p class="brand-subtitle">{_('Personal OS')}</p>
			</div>
		</div>

		<nav class="nav-group primary-nav" aria-label="Primary workspaces">
			{#each sidebarRootEntries as entry, entryIndex}
				{#if entry.kind === 'item'}
					{@const item = entry.item}
					<div class="nav-entry">
						<button
							type="button"
							class:active={isSidebarItemActive(item)}
							class:disabled={isSidebarItemDisabled(item)}
							disabled={isSidebarItemDisabled(item)}
							aria-current={isSidebarItemActive(item) ? 'page' : undefined}
							title={sidebarItemTitle(item)}
							onclick={() => selectSidebarItem(item)}
						>
							<Icon icon={sidebarItemIcon(item)} width="18" height="18" />
							<span>{_(sidebarItemLabel(item))}</span>
							{#if sidebarItemBadge(item)}
								<em>{sidebarItemBadge(item)}</em>
							{/if}
						</button>
					</div>
				{:else}
					{@const group = entry.group}
					<div class="nav-entry">
						<button
							type="button"
							class:active={sidebarGroupHasActiveItem(group)}
							class:has-subnav={true}
							aria-current={sidebarGroupHasActiveItem(group) ? 'page' : undefined}
							aria-expanded={isSidebarRail ? activeSidebarRailGroupId === group.id : isSidebarGroupExpanded(group.id)}
							aria-controls={`sidebar-group-${group.id}-sections`}
							aria-haspopup={isSidebarRail ? 'menu' : undefined}
							title={_(sidebarGroupLabel(group, entryIndex))}
							onclick={() => toggleSidebarGroup(group)}
						>
							<Icon icon={group.icon} width="18" height="18" />
							<span>{_(sidebarGroupLabel(group, entryIndex))}</span>
							{#if !isSidebarRail}
								<Icon class="nav-disclosure" icon={isSidebarGroupExpanded(group.id) ? 'tabler:chevron-up' : 'tabler:chevron-down'} width="15" height="15" />
							{/if}
						</button>
						{#if activeSidebarRailGroupId === group.id && isSidebarRail}
							<div
								id={`sidebar-group-${group.id}-sections`}
								class="communications-rail-dropdown"
								aria-label={`${_(sidebarGroupLabel(group, entryIndex))} sections`}
							>
								{#each group.items as item}
									{#if sidebarGroupHasSeparatorBefore(group, item.itemId)}
										<div class="subnav-separator" aria-hidden="true"></div>
									{/if}
									<button
										type="button"
										class="subnav-item"
										class:active={isSidebarItemActive(item)}
										aria-current={isSidebarItemActive(item) ? 'page' : undefined}
										title={sidebarItemTitle(item)}
										disabled={isSidebarItemDisabled(item)}
										onclick={() => selectSidebarItem(item)}
									>
										<Icon icon={sidebarItemIcon(item)} width="16" height="16" />
										<span>{_(sidebarItemLabel(item))}</span>
										{#if sidebarItemBadge(item)}
											<em>{sidebarItemBadge(item)}</em>
										{/if}
									</button>
								{/each}
							</div>
						{/if}
						{#if isSidebarGroupExpanded(group.id) && !isSidebarRail}
							<div
								id={`sidebar-group-${group.id}-sections`}
								class="communications-subnav"
								aria-label={`${_(sidebarGroupLabel(group, entryIndex))} sections`}
							>
								{#each group.items as item}
									{#if sidebarGroupHasSeparatorBefore(group, item.itemId)}
										<div class="subnav-separator" aria-hidden="true"></div>
									{/if}
									<button
										type="button"
										class="subnav-item"
										class:active={isSidebarItemActive(item)}
										aria-current={isSidebarItemActive(item) ? 'page' : undefined}
										title={sidebarItemTitle(item)}
										disabled={isSidebarItemDisabled(item)}
										onclick={() => selectSidebarItem(item)}
									>
										<Icon icon={sidebarItemIcon(item)} width="16" height="16" />
										<span>{_(sidebarItemLabel(item))}</span>
										{#if sidebarItemBadge(item)}
											<em>{sidebarItemBadge(item)}</em>
										{/if}
									</button>
								{/each}
							</div>
						{/if}
					</div>
				{/if}
			{/each}
		</nav>

		<div class="sidebar-tools" aria-label="System navigation">
			<button type="button" class="settings-link" class:active={currentView === 'settings'} title="Open settings" onclick={() => setCurrentView('settings')}>
				<Icon icon="tabler:settings" width="18" height="18" />
				<span>{_('Settings')}</span>
			</button>
		</div>
	</aside>

	{#if isSidebarRail && activeSidebarRailGroupId !== null}
		<button
			type="button"
			class="sidebar-rail-dropdown-backdrop"
			aria-label="Close sidebar menu"
			onclick={closeSidebarRailDropdown}
		></button>
	{/if}

	<section class="workspace" class:layout-editing={isLayoutEditing} aria-label={`${_(activeView.title)} workspace`}>
		<header class="topbar">
			<div class="topbar-title">
				<h1>{_(activeView.title)}</h1>
				<p>{_(activeView.subtitle)}</p>
			</div>
			<div class="top-actions">
				<button
					type="button"
					class="icon-button"
					class:active={isNotificationsDrawerOpen}
					aria-label="Open notifications"
					aria-expanded={isNotificationsDrawerOpen}
					aria-controls="notifications-drawer"
					title="Open notifications"
					onclick={toggleNotificationsDrawer}
				>
					<Icon icon="tabler:bell" width="18" height="18" />
					{#if notificationItems.length > 0}
						<i>{notificationItems.length}</i>
					{/if}
				</button>
				<div class="user-menu-shell">
					<button
						type="button"
						class="menu-button"
						aria-haspopup="menu"
						aria-expanded={isUserMenuOpen}
						aria-controls="user-menu"
						onclick={toggleUserMenu}
						title="Open user menu"
					>
						<Icon icon="tabler:menu-2" width="20" height="20" />
					</button>
					{#if isUserMenuOpen}
						<button type="button" class="user-menu-backdrop" aria-label="Close user menu" onclick={closeUserMenu}></button>
						<div id="user-menu" class="user-menu" role="menu" aria-label="User menu">
							<button type="button" role="menuitem" onclick={startLayoutEditing} disabled={isLayoutEditing}>
								<Icon icon="tabler:layout-dashboard" width="16" height="16" />
								<span>{_('Constructor Mode')}</span>
							</button>
							<button type="button" role="menuitem" onclick={async () => { const loc = $currentLocale === 'en' ? 'ru' : 'en'; setLocale(loc); try { await saveFrontendLocaleSetting(apiBaseUrl, apiSecret, loc); } catch (_) {} }}>
								<Icon icon="tabler:language" width="16" height="16" />
								<span>{$currentLocale === 'en' ? 'Русский' : 'English'}</span>
							</button>
							<div class="user-menu-separator" role="separator"></div>
							<button type="button" role="menuitem" onclick={exitApplication}>
								<Icon icon="tabler:logout" width="16" height="16" />
								<span>{_('Exit')}</span>
							</button>
						</div>
					{/if}
				</div>
			</div>
		</header>

		{#if isNotificationsDrawerOpen}
			<button
				type="button"
				class="notifications-backdrop"
				aria-label="Close notifications"
				onclick={closeNotificationsDrawer}
			></button>
			<aside id="notifications-drawer" class="notifications-drawer" aria-label="Notifications">
				<header>
					<div>
						<h2>{_('Notifications')}</h2>
						<p>{notificationItems.length} active</p>
					</div>
					<button type="button" class="icon-button" aria-label="Close notifications" onclick={closeNotificationsDrawer}>
						<Icon icon="tabler:x" width="18" height="18" />
					</button>
				</header>
				{#if notificationItems.length === 0}
					<div class="notifications-empty">
						<Icon icon="tabler:bell-check" width="28" height="28" />
						<p>{_('No active notifications.')}</p>
					</div>
				{:else}
					<div class="notifications-list">
						{#each notificationItems as notification}
							<article>
								<button
									type="button"
									class="notification-target"
									onclick={() => void openNotificationTarget(notification)}
								>
									<span class="round-icon cyan">
										<Icon icon={notification.icon} width="18" height="18" />
									</span>
									<span>
										<strong>{notification.title}</strong>
										<small>{notification.meta} · {formatDateTime(notification.time)}</small>
										<em class:expanded={isNotificationExpanded(notification.id)}>{notification.body}</em>
									</span>
								</button>
								{#if notificationNeedsExpansion(notification)}
									<button
										type="button"
										class="notification-expand"
										aria-expanded={isNotificationExpanded(notification.id)}
										onclick={() => toggleNotificationExpanded(notification.id)}
									>
										{isNotificationExpanded(notification.id) ? 'Show less' : 'Show full text'}
									</button>
								{/if}
								<button
									type="button"
									class="notification-dismiss"
									aria-label={`Dismiss ${notification.title}`}
									onclick={(event) => {
										event.stopPropagation();
										dismissNotification(notification.id);
									}}
								>
									<Icon icon="tabler:trash" width="16" height="16" />
								</button>
							</article>
						{/each}
					</div>
				{/if}
			</aside>
		{/if}

		{#if isLayoutEditing}
			<div class="layout-edit-controls" role="group" aria-label={_('Widget layout controls')}>
				<button type="button" class="ghost-button" onclick={() => { selectedLayoutWidgetId = null; isWidgetDrawerOpen = true; }}>
					<Icon icon="tabler:plus" width="16" height="16" />
					{_('Add widget')}
				</button>
				<button type="button" class="ghost-button" onclick={cancelLayoutEditing}>{_('Cancel')}</button>
				<button type="button" class="ghost-button" onclick={resetCurrentViewLayout}>{_('Reset')}</button>
				<button
					type="button"
					class="primary-button"
					disabled={isLayoutSettingsSaving || layoutDraft === null}
					onclick={saveLayoutSettings}
				>
					{isLayoutSettingsSaving ? _('Saving') : _('Save')}
				</button>
			</div>
		{/if}

		{#if currentView === 'home'}
			<section class="home-page">
				<div class="hero-row">
					<div
						class="widget-frame"
						class:editing={isLayoutEditing}
						data-widget-id="home-metrics"
						data-widget-hide-if-clipped-content
						data-widget-hidden={!isWidgetVisible('home-metrics')}
					>
						{@render widgetEditChrome('home-metrics')}
						<div class="metric-grid home-metrics" data-widget-fit-content>
							{#each homeStats as metric}
								<article class="metric-card">
									<span>{metric.label}</span>
									<div>
										<strong>{metric.value}</strong>
										<Icon icon={metric.icon} width="26" height="26" />
									</div>
									<small>↑ {metric.delta}</small>
								</article>
							{/each}
							<article class="metric-card focus-card">
								<span>{_('Focus Score')}</span>
								<div class="score-ring"><strong>78</strong></div>
								<small>Good ↑ 5</small>
							</article>
						</div>
					</div>
				</div>

				<div class="dashboard-grid">
					<div
						class="widget-frame"
						class:editing={isLayoutEditing}
						data-widget-id="home-whats-new"
						data-widget-hide-if-clipped-content
						data-widget-hidden={!isWidgetVisible('home-whats-new')}
					>
						{@render widgetEditChrome('home-whats-new')}
						<section class="panel feed-panel" data-widget-fit-content>
							<header class="panel-title-row">
								<div>
									<h2>{_('What\'s New')}</h2>
									<p>{_('Key changes and important updates')}</p>
								</div>
								<button type="button" class="ghost-button" disabled>All Types</button>
							</header>
							<div class="feed-list">
								{#each whatsNew as item}
									<article class="feed-row">
										<span class="round-icon {item.tone}"><Icon icon={item.icon} width="22" height="22" /></span>
										<div>
											<strong>{item.title}</strong>
											<p>{item.meta}</p>
											{#if item.tag}<em>{item.tag}</em>{/if}
										</div>
										<time>{item.time}</time>
									</article>
								{/each}
							</div>
							<button type="button" class="link-row" disabled>View all events <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
						</section>
					</div>

					<div
						class="widget-frame"
						class:editing={isLayoutEditing}
						data-widget-id="home-priorities"
						data-widget-hide-if-clipped-content
						data-widget-hidden={!isWidgetVisible('home-priorities')}
					>
						{@render widgetEditChrome('home-priorities')}
						<section class="panel priorities-panel" data-widget-fit-content>
							<header class="panel-title-row">
								<div>
									<h2>{_('Today\'s Priorities')}</h2>
									<p>{_('Focus on what matters most')}</p>
								</div>
							</header>
							<div class="task-stack">
								{#each tasks.slice(0, 5) as task}
									<label>
										<input type="checkbox" />
										<span>
											<strong>{task.title}</strong>
											<small>{task.assignee} · {task.due}</small>
										</span>
										<em class:high={task.priority === 'High'}>{task.priority}</em>
									</label>
								{/each}
							</div>
							<button type="button" class="link-row" disabled>View all tasks <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
						</section>
					</div>

					<div
						class="widget-frame"
						class:editing={isLayoutEditing}
						data-widget-id="home-upcoming"
						data-widget-hide-if-clipped-content
						data-widget-hidden={!isWidgetVisible('home-upcoming')}
					>
						{@render widgetEditChrome('home-upcoming')}
						<section class="panel schedule-panel" data-widget-fit-content>
							<header class="panel-title-row">
								<div>
									<h2>{_('Upcoming')}</h2>
									<p>{_('Your schedule')}</p>
								</div>
							</header>
							<div class="schedule-list">
								<article><time>Today, May 12</time><strong>14:00 Call with Acme Corp</strong><span>16:30 Review Q2 Report</span></article>
								<article><time>Tomorrow, May 13</time><strong>10:00 Project Hermes - Planning</strong><span>15:00 Design Review</span></article>
							</div>
							<button type="button" class="link-row" disabled>View full calendar <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
						</section>
					</div>

					<aside class="stacked-rail">
						<div
							class="widget-frame"
							class:editing={isLayoutEditing}
							data-widget-id="home-people-talked-to"
							data-widget-hide-if-clipped-content
							data-widget-hidden={!isWidgetVisible('home-people-talked-to')}
						>
							{@render widgetEditChrome('home-people-talked-to')}
							<section class="panel mini-panel" data-widget-fit-content>
								<header class="panel-title-row"><h2>{_('People You Talked To')}</h2><button type="button" class="link-button" disabled>View all</button></header>
								<div class="person-list">
									{#each peopleTalked as person}
										<article>
											<img src="/assets/hermes-reference-avatar.png" alt="" />
											<span><strong>{person.name}</strong><small>{person.meta}</small></span>
											<Icon icon={person.icon} width="18" height="18" />
										</article>
									{/each}
								</div>
							</section>
						</div>
						<div
							class="widget-frame"
							class:editing={isLayoutEditing}
							data-widget-id="home-system-status"
							data-widget-hide-if-clipped-content
							data-widget-hidden={!isWidgetVisible('home-system-status')}
						>
							{@render widgetEditChrome('home-system-status')}
							<section class="panel mini-panel" data-widget-fit-content>
								<header class="panel-title-row"><h2>{_('System Status')}</h2></header>
								<ul class="status-list">
									<li class:online={status}>All systems operational</li>
									<li>AI Agents online <span>5/5</span></li>
									<li>Data synchronized <span>2m ago</span></li>
									<li>Local AI models <span>Ready</span></li>
								</ul>
								{#if statusError}<p class="inline-error">{statusError}</p>{/if}
							</section>
						</div>
					</aside>
				</div>

				<div
					class="widget-frame"
					class:editing={isLayoutEditing}
					data-widget-id="home-active-projects"
					data-widget-hide-if-clipped-content
					data-widget-hidden={!isWidgetVisible('home-active-projects')}
				>
					{@render widgetEditChrome('home-active-projects')}
					<section class="panel full-band">
						<header class="panel-title-row">
							<h2>{_('Active Projects')}</h2>
							<button type="button" class="link-button" onclick={() => setCurrentView('projects')}>View all projects</button>
						</header>
						<div class="project-card-row" data-widget-fit-content>
							{#each projects as project}
								<article class="compact-project">
									<span class="round-icon {project.tone}"><Icon icon={project.icon} width="20" height="20" /></span>
									<div>
										<strong>{project.name}</strong>
										<small>{project.kind}</small>
									</div>
									<progress class="progress" max="100" value={project.progress} aria-label={`${project.name} progress`}>{project.progress}%</progress>
									<em>{project.progress}%</em>
								</article>
							{/each}
							<button type="button" class="new-tile" disabled><Icon icon="tabler:plus" width="22" height="22" />New Project</button>
						</div>
					</section>
				</div>
			</section>
		{:else if isCommunicationMessagesSection}
			<section class="communications-page">
				<div class="view-header">
					<div>
						<h1>{_(activeView.title)}</h1>
						<p>{_(activeView.subtitle)}</p>
					</div>
					<div class="header-actions">
						<button type="button" class="segmented active"><Icon icon="tabler:message" width="16" height="16" /></button>
						<button type="button" class="segmented" disabled><Icon icon="tabler:layout-grid" width="16" height="16" /></button>
						<button type="button" class="primary-button" onclick={() => { composeForm.draft_id = 'draft-' + Date.now(); isComposeOpen = true; }}>New Message</button>
					</div>
				</div>
				<div class="filter-tabs">
					<button type="button" class:active={mailStateFilter === ''} onclick={() => selectCommunicationSection('unified')}>Unified <em>{communicationMessages.length}</em></button>
					<button type="button" class:active={mailStateFilter === 'needs_action'} onclick={() => selectCommunicationSection('needs_reply')}>Needs Reply <em>{mailStateCounts.find(c => c.state === 'needs_action')?.count ?? 0}</em></button>
					<button type="button" class:active={mailStateFilter === 'waiting'} onclick={() => selectCommunicationSection('waiting')}>Waiting <em>{mailStateCounts.find(c => c.state === 'waiting')?.count ?? 0}</em></button>
					<button type="button" class:active={mailStateFilter === 'new'} onclick={() => selectCommunicationSection('inbox')}>Inbox <em>{mailStateCounts.find(c => c.state === 'new')?.count ?? 0}</em></button>
					<button type="button" class:active={mailStateFilter === 'done'} onclick={() => { mailStateFilter = 'done'; void loadCommunicationMessagesFiltered('done'); }}>Done <em>{mailStateCounts.find(c => c.state === 'done')?.count ?? 0}</em></button>
					<button type="button" class:active={mailStateFilter === 'archived'} onclick={() => { mailStateFilter = 'archived'; void loadCommunicationMessagesFiltered('archived'); }}>Archived <em>{mailStateCounts.find(c => c.state === 'archived')?.count ?? 0}</em></button>
				</div>
				<div class="three-pane communications-grid">
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-conversation-list" data-widget-hidden={!isWidgetVisible('communications-conversation-list')}>
						{@render widgetEditChrome('communications-conversation-list')}
						<section class="panel conversation-list">
							<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder="Search conversations..." /></label>
							{#if isCommunicationsLoading}
								<div class="empty-panel">Loading messages...</div>
							{:else if communicationsError}
								<div class="empty-panel error">{communicationsError}</div>
							{:else if communicationMessages.length === 0}
								<div class="empty-panel">No local messages yet.</div>
							{:else}
								{#each communicationMessages as message, index}
									<button type="button" class:active={selectedConversationIndex === index} onclick={() => selectCommunication(index)}>
										<span class="round-icon cyan">
											<Icon icon={communicationChannelIcon(message.channel_kind)} width="22" height="22" />
										</span>
										<img src="/assets/hermes-reference-avatar.png" alt="" />
										<span>
											<strong>{senderLabel(message.sender)}</strong>
											<small>{message.subject}</small>
											<em>{message.body_text_preview}</em>
										</span>
										{#if (message as any).workflow_state}
											<span class="state-badge {(message as any).workflow_state}">{(message as any).workflow_state.replace('_', ' ')}</span>
										{/if}
										<time>{messageTime(message)}</time>
										{#if message.attachment_count > 0}<b>{message.attachment_count}</b>{/if}
									</button>
								{/each}
							{/if}
						</section>
					</div>
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-message-detail" data-widget-hidden={!isWidgetVisible('communications-message-detail')}>
						{@render widgetEditChrome('communications-message-detail')}
						<section class="panel chat-pane">
							{#if selectedCommunication}
								<header>
									<img src="/assets/hermes-reference-avatar.png" alt="" />
									<div><h2>{senderLabel(selectedCommunication.sender)}</h2><p>{selectedCommunication.subject}</p></div>
									<div class="chat-actions">
										<button type="button" onclick={() => void handleWorkflowStateTransition(selectedCommunication.message_id, 'needs_action')} disabled={isMailStateTransitioning} title="Mark as Needs Action"><Icon icon="tabler:alert-triangle" width="17" height="17" /></button>
										<button type="button" onclick={() => void handleWorkflowStateTransition(selectedCommunication.message_id, 'waiting')} disabled={isMailStateTransitioning} title="Mark as Waiting"><Icon icon="tabler:clock-hour-4" width="17" height="17" /></button>
										<button type="button" onclick={() => void handleWorkflowStateTransition(selectedCommunication.message_id, 'done')} disabled={isMailStateTransitioning} title="Mark as Done"><Icon icon="tabler:circle-check" width="17" height="17" /></button>
										<button type="button" onclick={() => void handleWorkflowStateTransition(selectedCommunication.message_id, 'archived')} disabled={isMailStateTransitioning} title="Archive"><Icon icon="tabler:archive" width="17" height="17" /></button>
										<button type="button" onclick={() => void askAiAboutSelectedMessage()} disabled={isAiAnswerSubmitting}><Icon icon="tabler:sparkles" width="17" height="17" /></button>
									</div>
								</header>
								<div class="chat-body">
									{#if aiAnalysisResult && aiAnalysisResult.message_id === selectedCommunication.message_id}
										<article class="ai-analysis-card">
											<strong><Icon icon="tabler:sparkles" width="16" height="16" />AI Analysis</strong>
											{#if aiAnalysisResult.category}<p><em>Category:</em> {aiAnalysisResult.category}</p>{/if}
											{#if aiAnalysisResult.summary}<p><em>Summary:</em> {aiAnalysisResult.summary}</p>{/if}
											{#if aiAnalysisResult.importance_score != null}<p><em>Importance:</em> {aiAnalysisResult.importance_score}/100</p>{/if}
											<p><em>State:</em> <span class="state-badge {aiAnalysisResult.workflow_state}">{aiAnalysisResult.workflow_state.replace('_', ' ')}</span></p>
										</article>
									{/if}
									<div class="date-divider">{messageTime(selectedCommunicationDetail?.message ?? selectedCommunication)}</div>
									<article class="bubble inbound">
										<strong>{selectedCommunication.subject}</strong><br />
										{selectedCommunicationDetail?.message.body_text ?? selectedCommunication.body_text_preview}
										<time>{messageTime(selectedCommunicationDetail?.message ?? selectedCommunication)}</time>
									</article>
									{#each selectedCommunicationDetail?.attachments ?? [] as attachment}
										<article class="attachment-bubble">
											<Icon icon={attachmentIcon(attachment.content_type)} width="34" height="34" />
											<span>
												<strong>{attachment.filename ?? attachment.provider_attachment_id}</strong>
												<small>{formatBytes(attachment.size_bytes)} · {attachment.content_type} · {attachment.scan_status}</small>
											</span>
											<button type="button" disabled><Icon icon="tabler:download" width="16" height="16" /></button>
										</article>
									{/each}
								</div>
								<footer class="composer">
									<input placeholder="Sending is not available yet" disabled />
									<button type="button" disabled><Icon icon="tabler:paperclip" width="17" height="17" /></button>
									<button type="button" disabled><Icon icon="tabler:send" width="18" height="18" /></button>
								</footer>
							{:else}
								<div class="empty-panel fill">Select a local message.</div>
							{/if}
						</section>
					</div>
					<aside class="context-rail">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-sender-profile" data-widget-hidden={!isWidgetVisible('communications-sender-profile')}>
							{@render widgetEditChrome('communications-sender-profile')}
							<section class="panel profile-panel">
								<div class="profile-head"><img src="/assets/hermes-reference-avatar.png" alt="" /><div><h2>{selectedCommunication ? senderLabel(selectedCommunication.sender) : 'No sender selected'}</h2><p>{selectedCommunication ? communicationChannelLabel(selectedCommunication.channel_kind) : 'No channel'}</p><small>{selectedCommunication ? senderEmail(selectedCommunication.sender) : 'No local message selected'}</small></div></div>
								<div class="quick-icons">
									<button type="button" disabled><Icon icon="tabler:mail" width="17" height="17" /></button>
									<button type="button" disabled><Icon icon="tabler:phone" width="17" height="17" /></button>
									<button type="button" disabled><Icon icon="tabler:brand-telegram" width="17" height="17" /></button>
									<button type="button" disabled><Icon icon="tabler:brand-whatsapp" width="17" height="17" /></button>
								</div>
							</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-summary" data-widget-hidden={!isWidgetVisible('communications-summary')}>
							{@render widgetEditChrome('communications-summary')}
							<section class="panel info-card"><h2>Summary</h2><p>{selectedCommunication ? `Stored from ${selectedCommunication.account_id}. Channel ${communicationChannelLabel(selectedCommunication.channel_kind)}. Provider record ${selectedCommunication.provider_record_id}.` : 'Local communication metadata will appear after messages are imported.'}</p><button type="button" class="link-row" disabled>View full profile <Icon icon="tabler:arrow-right" width="15" height="15" /></button></section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-message-metadata" data-widget-hidden={!isWidgetVisible('communications-message-metadata')}>
							{@render widgetEditChrome('communications-message-metadata')}
							<section class="panel info-card"><h2>Message Metadata</h2>{#if selectedCommunication}<ul class="detail-list"><li><Icon icon="tabler:users" width="17" height="17" /> {selectedCommunication.recipients.length} recipients</li><li><Icon icon="tabler:paperclip" width="17" height="17" /> {selectedCommunication.attachment_count} attachments</li><li><Icon icon="tabler:clock" width="17" height="17" /> {messageTime(selectedCommunication)}</li></ul>{:else}<p>No message selected.</p>{/if}</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-related-projects" data-widget-hidden={!isWidgetVisible('communications-related-projects')}>
							{@render widgetEditChrome('communications-related-projects')}
							<section class="panel info-card"><h2>Related Projects</h2>{#each projects.slice(0, 2) as project}<div class="related-row"><span class="round-icon {project.tone}"><Icon icon={project.icon} width="16" height="16" /></span><strong>{project.name}</strong><em>{project.progress}%</em></div>{/each}</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-active-tasks" data-widget-hidden={!isWidgetVisible('communications-active-tasks')}>
							{@render widgetEditChrome('communications-active-tasks')}
							<section class="panel info-card"><h2>Active Tasks</h2>{#each tasks.slice(0, 3) as task}<label class="mini-check"><input type="checkbox" />{task.title}<em>{task.due.split(' ')[0]}</em></label>{/each}</section>
						</div>
					</aside>
				</div>

				{#if isComposeOpen}
					<button type="button" class="drawer-backdrop" onclick={() => (isComposeOpen = false)} aria-label="Close compose"></button>
					<aside class="account-drawer"  aria-label="Compose email">
						<header>
							<div><p>{_('Compose')}</p><h2>New Message</h2></div>
							<button type="button" class="icon-button" onclick={() => (isComposeOpen = false)} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
						</header>
						<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void handleSaveDraft(); }}>
							<label><span>To</span><input bind:value={composeForm.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
							<label><span>CC</span><input bind:value={composeForm.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
							<label><span>Subject</span><input bind:value={composeForm.subject} placeholder="Email subject" autocomplete="off" /></label>
							<label class="wide"><span>Body</span><textarea bind:value={composeForm.body} rows="8" placeholder="Write your message..."></textarea></label>
							<div class="form-actions wide">
								<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
								<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if drafts.length > 0}
					<div class="draft-strip">
						<strong>Drafts ({drafts.length})</strong>
						{#each drafts.slice(0, 3) as draft}
							<button type="button" class="draft-chip" onclick={() => { composeForm = { draft_id: draft.draft_id, account_id: draft.account_id, to_text: draft.to_recipients.join(', '), cc_text: draft.cc_recipients.join(', '), subject: draft.subject, body: draft.body_text }; isComposeOpen = true; }}>
								<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
							</button>
						{/each}
					</div>
				{/if}

				{#if mailboxHealth}
					<div class="health-strip">
						<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{mailboxHealth.needs_action} need action</span>
						<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{mailboxHealth.waiting} waiting</span>
						<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{mailboxHealth.done} done</span>
						<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{mailboxHealth.total_messages} total</span>
						{#if mailboxHealth.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{mailboxHealth.important} important</span>{/if}
					</div>
				{/if}
			</section>
		{:else if activeCommunicationEmptySection}
			<section class="communications-page communication-empty-page">
				<div class="view-header">
					<div class="view-title-with-icon">
						<span class="hero-mark small"><Icon icon={activeCommunicationEmptySection.icon} width="28" height="28" /></span>
						<div>
							<h1>{activeCommunicationEmptySection.label}</h1>
							<p>This Communications workspace is ready in navigation, but no data contract is implemented yet.</p>
						</div>
					</div>
				</div>
				<section class="panel communication-empty-panel">
					<Icon icon={activeCommunicationEmptySection.icon} width="42" height="42" />
					<div>
						<h2>{activeCommunicationEmptySection.label}</h2>
						<p>Hermes Hub will keep this as a second-level Communications context. It is intentionally empty until a scoped backend and UI slice defines real records for this workspace.</p>
					</div>
				</section>
			</section>
		{:else if currentView === 'persons'}
			<section class="persons-page">
				<div class="persons-layout">
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-list" data-widget-hidden={!isWidgetVisible('persons-list')}>
						{@render widgetEditChrome('persons-list')}
						<section class="panel persons-list-panel">
							<header>
								<div><h1>Persons</h1><p>{persons.length} persons</p></div>
								<button type="button" class="primary-button" disabled>New Person</button>
							</header>
							<div class="filter-tabs compact">
								<button type="button" class="active">All</button>
								<button type="button" disabled>People <em>532</em></button>
								<button type="button" disabled>Companies <em>110</em></button>
							</div>
							<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder="Search persons..." /></label>
							{#each personList as person, index}
								<button type="button" class="person-row" class:active={selectedPersonIndex === index} onclick={() => (selectedPersonIndex = index)}>
									<img src="/assets/hermes-reference-avatar.png" alt="" />
									<span><strong>{person.name}</strong><small>{person.role}</small><em>{person.company}</em></span>
									<small>{person.status ?? person.channel ?? 'Email'}</small>
								</button>
							{/each}
						</section>
					</div>
					<section class="person-detail">
						{#if selectedPerson}
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-hero" data-widget-hidden={!isWidgetVisible('persons-hero')}>
							{@render widgetEditChrome('persons-hero')}
							<header class="person-hero panel">
								<img src="/assets/hermes-reference-avatar.png" alt="" />
								<div><h1>{selectedPerson.name}</h1><p>{selectedPerson.role} at {selectedPerson.company}</p><small>{selectedPerson.status ?? selectedPerson.channel ?? 'Contact'}</small></div>
								<div class="chat-actions">
									<button type="button" disabled><Icon icon="tabler:mail" width="17" height="17" /></button>
									<button type="button" disabled><Icon icon="tabler:phone" width="17" height="17" /></button>
									<button type="button" disabled><Icon icon="tabler:video" width="17" height="17" /></button>
									<button type="button" disabled><Icon icon="tabler:brand-whatsapp" width="17" height="17" /></button>
								</div>
							</header>
						</div>
						<div class="section-tabs">
							<button type="button" class="active">Overview</button>
							<button type="button" disabled>Communications</button>
							<button type="button" disabled>Documents <em>24</em></button>
							<button type="button" disabled>Tasks <em>7</em></button>
							<button type="button" disabled>Projects <em>5</em></button>
							<button type="button" disabled>Notes</button>
						</div>
						<div class="person-cards">
							<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-information" data-widget-hidden={!isWidgetVisible('persons-information')}>
								{@render widgetEditChrome('persons-information')}
								<section class="panel info-card">
									<h2>Person Information</h2>
									<ul class="detail-list">
										<li><Icon icon="tabler:mail" width="17" height="17" /> {selectedPerson.company} <em>Primary</em></li>
										<li><Icon icon="tabler:phone" width="17" height="17" /> +1 (555) 123-4567 <em>Mobile</em></li>
										<li><Icon icon="tabler:brand-telegram" width="17" height="17" /> @john.smith <em>Telegram</em></li>
										<li><Icon icon="tabler:map-pin" width="17" height="17" /> New York, USA <em>Local Time: 18:42</em></li>
									</ul>
								</section>
							</div>
							<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-about" data-widget-hidden={!isWidgetVisible('persons-about')}>
								{@render widgetEditChrome('persons-about')}
								<section class="panel info-card"><h2>About</h2><p>John is a strategic consulting partner. We have been working together since 2021 on multiple projects including Hermes Hub and IRIS platform development.</p><div class="tag-cloud"><span>Decision Maker</span><span>Executive</span><span>Strategic</span><span>Tech Enthusiast</span></div></section>
							</div>
							<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-relationship-strength" data-widget-hidden={!isWidgetVisible('persons-relationship-strength')}>
								{@render widgetEditChrome('persons-relationship-strength')}
								<section class="panel info-card"><h2>Relationship Strength</h2><div class="big-score">85</div><strong>Strong</strong><p>Last interaction 2 hours ago</p></section>
							</div>
							<div class="widget-frame span-2" class:editing={isLayoutEditing} data-widget-id="persons-recent-interactions" data-widget-hidden={!isWidgetVisible('persons-recent-interactions')}>
								{@render widgetEditChrome('persons-recent-interactions')}
								<section class="panel info-card span-2"><h2>Recent Interactions</h2>{#each whatsNew.slice(0, 3) as item}<div class="feed-row compact-row"><span class="round-icon {item.tone}"><Icon icon={item.icon} width="18" height="18" /></span><div><strong>{item.title}</strong><p>{item.meta}</p></div><time>{item.time}</time></div>{/each}</section>
							</div>
							<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-active-projects" data-widget-hidden={!isWidgetVisible('persons-active-projects')}>
								{@render widgetEditChrome('persons-active-projects')}
								<section class="panel info-card"><h2>{_('Active Projects')}</h2>{#each projects.slice(0, 3) as project}<div class="related-row"><span class="round-icon {project.tone}"><Icon icon={project.icon} width="16" height="16" /></span><strong>{project.name}</strong><em>{project.progress}%</em></div>{/each}</section>
							</div>
						</div>
						{:else}
							<section class="panel empty-domain-state">
								<Icon icon="tabler:user" width="42" height="42" />
								<div>
									<h2>No person selected</h2>
									<p>Hermes Hub will show relationship memory here after persons are imported from local sources.</p>
								</div>
							</section>
						{/if}
					</section>
					<aside class="stacked-rail">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-ai-summary" data-widget-hidden={!isWidgetVisible('persons-ai-summary')}>
							{@render widgetEditChrome('persons-ai-summary')}
							<section class="panel info-card"><h2>AI Summary</h2><p>John is a key strategic partner and decision maker. You have a strong professional relationship with frequent communication across multiple projects.</p></section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-identity-review" data-widget-hidden={!isWidgetVisible('persons-identity-review')}>
							{@render widgetEditChrome('persons-identity-review')}
							<section class="panel info-card">
								<h2>Person Identity Review</h2>
								<p class="identity-note">Person merges are only suggested and are not applied until confirmed.</p>
								{#if isIdentityCandidatesLoading}
									<p class="inline-copy">Loading identity suggestions…</p>
								{:else if identityCandidatesError}
									<p class="inline-error">{identityCandidatesError}</p>
								{:else if suggestedIdentityCandidates.length === 0 && confirmedMergeIdentityCandidates.length === 0}
									<p class="inline-copy">No identity suggestions right now.</p>
								{:else}
									{#each suggestedIdentityCandidates as candidate}
										<div class="identity-candidate-row">
											<div>
												<strong>{candidate.candidate_kind}</strong>
												<p>{candidate.evidence_summary}</p>
												<small>Left: {candidate.left_person_id}</small>
												<small>Right: {candidate.right_person_id ?? 'N/A'}</small>
												<small>Confidence: {identityConfidence(candidate)} · {candidate.review_state}</small>
											</div>
											<div class="identity-actions">
												<button type="button" onclick={() => void setIdentityCandidateReview(candidate, 'user_confirmed')}>
													<Icon icon="tabler:check" width="15" height="15" />
													Confirm
												</button>
												<button type="button" onclick={() => void setIdentityCandidateReview(candidate, 'user_rejected')}>
													<Icon icon="tabler:x" width="15" height="15" />
													Reject
												</button>
											</div>
										</div>
									{/each}
									{#each confirmedMergeIdentityCandidates as candidate}
										{@const splitCandidate = splitCandidateForConfirmedMerge(candidate)}
										<div class="identity-candidate-row">
											<div>
											<strong>{candidate.candidate_kind}</strong>
											<p>{candidate.evidence_summary}</p>
											<small>Left: {candidate.left_person_id}</small>
											<small>Right: {candidate.right_person_id ?? 'N/A'}</small>
											<small>Confidence: {identityConfidence(candidate)} · {candidate.review_state}</small>
											</div>
											<div class="identity-actions">
												<button
													type="button"
													disabled={splitCandidate === null}
													title={splitCandidate === null
														? 'Refresh identity candidates to create a split review for this confirmed link'
														: undefined}
													onclick={() => void splitConfirmedIdentityMerge(candidate)}
												>
													<Icon icon="tabler:arrows-split" width="15" height="15" />
													Split
												</button>
											</div>
										</div>
									{/each}
								{/if}
							</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-related-documents" data-widget-hidden={!isWidgetVisible('persons-related-documents')}>
							{@render widgetEditChrome('persons-related-documents')}
							<section class="panel info-card"><h2>Related Documents</h2>{#each documents.slice(0, 4) as doc}<div class="doc-mini"><Icon icon={doc.icon} width="20" height="20" /><span><strong>{doc.name}</strong><small>{doc.size} · {doc.date}</small></span></div>{/each}</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="persons-recent-notes" data-widget-hidden={!isWidgetVisible('persons-recent-notes')}>
							{@render widgetEditChrome('persons-recent-notes')}
							<section class="panel info-card"><h2>Recent Notes</h2><p>Discussed expansion to EU market</p><p>Prefers email for official communication</p><p>Interested in AI/ML integration</p></section>
						</div>
					</aside>
				</div>

				{#if isComposeOpen}
					<button type="button" class="drawer-backdrop" onclick={() => (isComposeOpen = false)} aria-label="Close compose"></button>
					<aside class="account-drawer"  aria-label="Compose email">
						<header>
							<div><p>{_('Compose')}</p><h2>New Message</h2></div>
							<button type="button" class="icon-button" onclick={() => (isComposeOpen = false)} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
						</header>
						<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void handleSaveDraft(); }}>
							<label><span>To</span><input bind:value={composeForm.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
							<label><span>CC</span><input bind:value={composeForm.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
							<label><span>Subject</span><input bind:value={composeForm.subject} placeholder="Email subject" autocomplete="off" /></label>
							<label class="wide"><span>Body</span><textarea bind:value={composeForm.body} rows="8" placeholder="Write your message..."></textarea></label>
							<div class="form-actions wide">
								<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
								<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if drafts.length > 0}
					<div class="draft-strip">
						<strong>Drafts ({drafts.length})</strong>
						{#each drafts.slice(0, 3) as draft}
							<button type="button" class="draft-chip" onclick={() => { composeForm = { draft_id: draft.draft_id, account_id: draft.account_id, to_text: draft.to_recipients.join(', '), cc_text: draft.cc_recipients.join(', '), subject: draft.subject, body: draft.body_text }; isComposeOpen = true; }}>
								<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
							</button>
						{/each}
					</div>
				{/if}

				{#if mailboxHealth}
					<div class="health-strip">
						<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{mailboxHealth.needs_action} need action</span>
						<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{mailboxHealth.waiting} waiting</span>
						<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{mailboxHealth.done} done</span>
						<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{mailboxHealth.total_messages} total</span>
						{#if mailboxHealth.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{mailboxHealth.important} important</span>{/if}
					</div>
				{/if}
			</section>
		{:else if currentView === 'projects'}
			<section class="projects-page">
				{#if projectsError && !selectedProjectRecord}
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-hero" data-widget-hidden={!isWidgetVisible('projects-hero')}>
						{@render widgetEditChrome('projects-hero')}
						<section class="panel info-card project-empty-state">
							<Icon icon="tabler:alert-circle" width="28" height="28" />
							<h2>Projects unavailable</h2>
							<p>{projectsError}</p>
							<button type="button" onclick={() => void loadProjects()}>Retry</button>
						</section>
					</div>
				{:else if !selectedProjectRecord}
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-hero" data-widget-hidden={!isWidgetVisible('projects-hero')}>
						{@render widgetEditChrome('projects-hero')}
						<section class="panel info-card project-empty-state">
							<Icon icon="tabler:cube" width="30" height="30" />
							<h2>No projects returned</h2>
							<p>{isProjectsLoading ? 'Loading local projects...' : 'Local project records are empty.'}</p>
						</section>
					</div>
				{:else}
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-hero" data-widget-hidden={!isWidgetVisible('projects-hero')}>
						{@render widgetEditChrome('projects-hero')}
						<header class="project-hero panel">
							<div class="project-logo"><Icon icon="tabler:cube" width="48" height="48" /></div>
							<div>
								<h1>{selectedProjectRecord.name} <em>{projectStatusLabel(selectedProjectRecord.status)}</em></h1>
								<p>{selectedProjectRecord.kind}</p>
								<small>{selectedProjectRecord.description}</small>
							</div>
							<button type="button" class="primary-button" onclick={() => void prepareAiBrief(selectedProjectRecord.project_id)} disabled={isAiMeetingPrepSubmitting}><Icon icon="tabler:calendar-stats" width="16" height="16" />Prepare brief</button>
						</header>
					</div>
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-metadata-strip" data-widget-hidden={!isWidgetVisible('projects-metadata-strip')}>
						{@render widgetEditChrome('projects-metadata-strip')}
						<div class="project-meta-strip panel">
							<article><span>Owner</span><strong>{selectedProjectRecord.owner_display_name}</strong></article>
							<article><span>People</span><strong>{formatNumber(selectedProjectStats.people_count)}</strong></article>
							<article><span>Start Date</span><strong>{formatProjectDate(selectedProjectRecord.start_date)}</strong></article>
							<article><span>Target Date</span><strong>{formatProjectDate(selectedProjectRecord.target_date)}</strong></article>
							<article><span>Progress</span><progress class="progress" max="100" value={selectedProjectRecord.progress_percent} aria-label={`${selectedProjectRecord.name} progress`}>{selectedProjectRecord.progress_percent}%</progress><strong>{selectedProjectRecord.progress_percent}%</strong></article>
						</div>
					</div>
					{#if projectSummaries.length > 1}
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-switcher" data-widget-hidden={!isWidgetVisible('projects-switcher')}>
							{@render widgetEditChrome('projects-switcher')}
							<div class="project-switcher panel">
								{#each projectSummaries as item}
									<button
										type="button"
										class:active={item.project.project_id === selectedProjectRecord.project_id}
										onclick={() => selectProject(item)}
									>
										<Icon icon="tabler:cube" width="16" height="16" />
										<span>{item.project.name}</span>
										<em>{item.project.progress_percent}%</em>
									</button>
								{/each}
							</div>
						</div>
					{/if}
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-section-tabs" data-widget-hidden={!isWidgetVisible('projects-section-tabs')}>
						{@render widgetEditChrome('projects-section-tabs')}
						<div class="section-tabs">
							<button type="button" class="active">Overview</button>
							<button type="button" disabled>Communications <em>{selectedProjectStats.message_count}</em></button>
							<button type="button" disabled>Tasks</button>
							<button type="button" disabled>Documents <em>{selectedProjectStats.document_count}</em></button>
							<button type="button" disabled>Calendar</button>
							<button type="button" disabled>Team <em>{selectedProjectStats.people_count}</em></button>
							<button type="button" disabled>Notes</button>
							<button type="button" disabled>Files</button>
							<button type="button" disabled>Settings</button>
						</div>
					</div>
					{#if projectsError}
						<p class="inline-error">{projectsError}</p>
					{/if}
					<div class="project-dashboard-grid">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-summary" data-widget-hidden={!isWidgetVisible('projects-summary')}>
							{@render widgetEditChrome('projects-summary')}
							<section class="panel info-card">
								<h2>Project Summary</h2>
								<div class="summary-numbers">
									<article><strong>{formatNumber(selectedProjectStats.document_count)}</strong><span>Documents</span></article>
									<article><strong>{formatNumber(selectedProjectStats.message_count)}</strong><span>Messages</span></article>
									<article><strong>{formatNumber(selectedProjectStats.people_count)}</strong><span>People</span></article>
									<article><strong>{formatNumber(selectedProjectStats.graph_connection_count)}</strong><span>Graph links</span></article>
								</div>
							</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-graph-preview" data-widget-hidden={!isWidgetVisible('projects-graph-preview')}>
							{@render widgetEditChrome('projects-graph-preview')}
							<section class="panel graph-card-large">
								<h2>Knowledge Graph</h2>
								<div class="radial-graph">
									<div class="graph-center"><Icon icon="tabler:cube" width="30" height="30" /><span>{selectedProjectRecord.name}</span></div>
									<span class="graph-chip graph-chip-messages">Messages {formatNumber(selectedProjectStats.message_count)}</span>
									<span class="graph-chip graph-chip-documents">Documents {formatNumber(selectedProjectStats.document_count)}</span>
									<span class="graph-chip graph-chip-people">People {formatNumber(selectedProjectStats.people_count)}</span>
									<span class="graph-chip graph-chip-links">Links {formatNumber(selectedProjectStats.graph_connection_count)}</span>
								</div>
							</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-timeline" data-widget-hidden={!isWidgetVisible('projects-timeline')}>
							{@render widgetEditChrome('projects-timeline')}
							<section class="panel info-card">
								<h2>Project Timeline</h2>
								{#if selectedProjectDetail?.timeline.length}
									{#each selectedProjectDetail.timeline as item}
										<div class="timeline-mini">
											<Icon icon={projectTimelineIcon(item)} width="16" height="16" />
											<time>{formatProjectDateTime(item.occurred_at)}</time>
											<strong>{item.title}</strong>
										</div>
									{/each}
								{:else}
									<p class="muted-copy">No timeline items from local sources.</p>
								{/if}
							</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-recent-communications" data-widget-hidden={!isWidgetVisible('projects-recent-communications')}>
							{@render widgetEditChrome('projects-recent-communications')}
							<section class="panel info-card">
								<h2>Recent Communications</h2>
								{#if selectedProjectDetail?.recent_messages.length}
									{#each selectedProjectDetail.recent_messages as message}
										<div class="related-row">
											<span class="round-icon cyan"><Icon icon="tabler:mail" width="16" height="16" /></span>
											<strong>{projectMessageSender(message)}</strong>
											<em>{formatProjectDateTime(message.occurred_at)}</em>
										</div>
									{/each}
								{:else}
									<p class="muted-copy">No linked communications.</p>
								{/if}
							</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-top-documents" data-widget-hidden={!isWidgetVisible('projects-top-documents')}>
							{@render widgetEditChrome('projects-top-documents')}
							<section class="panel info-card">
								<h2>Top Documents</h2>
								{#if selectedProjectDetail?.documents.length}
									{#each selectedProjectDetail.documents as document}
										<div class="doc-mini">
											<Icon icon={projectDocumentIcon(document)} width="20" height="20" />
											<span><strong>{document.title}</strong><small>{document.document_kind} · {formatProjectDateTime(document.imported_at)}</small></span>
										</div>
									{/each}
								{:else}
									<p class="muted-copy">No linked documents.</p>
								{/if}
							</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-source-evidence" data-widget-hidden={!isWidgetVisible('projects-source-evidence')}>
							{@render widgetEditChrome('projects-source-evidence')}
							<section class="panel info-card">
								<h2>Source Evidence</h2>
								<div class="summary-numbers compact">
									<article><strong>{formatNumber(selectedProjectStats.message_count + selectedProjectStats.document_count)}</strong><span>Matched records</span></article>
									<article><strong>{formatProjectDateTime(selectedProjectStats.latest_activity_at)}</strong><span>Last activity</span></article>
								</div>
							</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-open-promises" data-widget-hidden={!isWidgetVisible('projects-open-promises')}>
							{@render widgetEditChrome('projects-open-promises')}
							<section class="panel info-card">
								<h2>Open Promises</h2>
								<p class="muted-copy">No task candidates connected to this project.</p>
								<button type="button" class="link-row" disabled>View all promises <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
							</section>
						</div>
						<aside class="stacked-rail project-side">
							<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-health" data-widget-hidden={!isWidgetVisible('projects-health')}>
								{@render widgetEditChrome('projects-health')}
								<section class="panel info-card">
									<h2>Project Health</h2>
									<div class="health-row"><span>Status</span><strong>{projectStatusLabel(selectedProjectRecord.status)}</strong></div>
									<div class="health-row"><span>Progress</span><strong>{selectedProjectRecord.progress_percent}%</strong></div>
									<div class="health-row"><span>Graph Links</span><strong>{formatNumber(selectedProjectStats.graph_connection_count)}</strong></div>
								</section>
							</div>
							<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-key-people" data-widget-hidden={!isWidgetVisible('projects-key-people')}>
								{@render widgetEditChrome('projects-key-people')}
								<section class="panel info-card">
									<h2>Key People</h2>
									{#if selectedProjectDetail?.key_people.length}
										{#each selectedProjectDetail.key_people as person}
											<div class="person-compact">
												<img src="/assets/hermes-reference-avatar.png" alt="" />
												<span><strong>{person.display_name}</strong><small>{person.email_address}</small></span>
												<em>{formatNumber(person.interaction_count)}</em>
											</div>
										{/each}
									{:else}
										<p class="muted-copy">No linked people.</p>
									{/if}
								</section>
							</div>
							<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="projects-related-projects" data-widget-hidden={!isWidgetVisible('projects-related-projects')}>
								{@render widgetEditChrome('projects-related-projects')}
								<section class="panel info-card">
									<h2>Related Projects</h2>
									{#if relatedProjectSummaries.length}
										{#each relatedProjectSummaries.slice(0, 4) as item}
											<div class="related-row">
												<span class="round-icon cyan"><Icon icon="tabler:cube" width="16" height="16" /></span>
												<strong>{item.project.name}</strong>
												<em>{item.project.progress_percent}%</em>
											</div>
										{/each}
									{:else}
										<p class="muted-copy">No related project records.</p>
									{/if}
								</section>
							</div>
						</aside>
					</div>
				{/if}
			</section>
		{:else if currentView === 'tasks'}
			<section class="tasks-page">
				<div class="view-header">
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:hexagon" width="28" height="28" /></span><div><h1>{_(activeView.title)}</h1><p>{_(activeView.subtitle)}</p></div></div>
					<div class="widget-frame inline-metrics" class:editing={isLayoutEditing} data-widget-id="tasks-metrics" data-widget-hidden={!isWidgetVisible('tasks-metrics')}>
						{@render widgetEditChrome('tasks-metrics')}
						<div class="metric-grid inline-metrics">
							<article class="metric-card">
								<span>Active Tasks</span>
								<strong>{activeTasks.length}</strong>
								<small>Active records</small>
							</article>
							<article class="metric-card">
								<span>Suggested Candidates</span>
								<strong>{suggestedTaskCandidates.length}</strong>
								<small>Ready for review</small>
							</article>
							<article class="metric-card">
								<span>Review State</span>
								<strong>{tasksError ? 'Error' : 'Ready'}</strong>
								<small>{tasksError ? 'Show message below' : 'Live API'}</small>
							</article>
						</div>
					</div>
					<button type="button" class="primary-button" onclick={() => void refreshTasksFromAi()} disabled={isAiTaskRefreshSubmitting}><Icon icon="tabler:sparkles" width="16" height="16" />AI refresh</button>
				</div>
				{#if tasksError}
					<p class="inline-error">{tasksError}</p>
				{/if}
				<div class="tasks-layout">
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="tasks-active-list" data-widget-hidden={!isWidgetVisible('tasks-active-list')}>
						{@render widgetEditChrome('tasks-active-list')}
						<section class="panel task-table">
							<h3 class="task-group">Active Tasks <em>{activeTasks.length}</em></h3>
							<div class="table-head task-table-head"><span>Task</span><span>Source</span><span>Project</span><span>Created</span><span>Status</span></div>
							{#if isTasksLoading}
								<p class="inline-copy">Loading task state…</p>
							{:else if activeTasks.length === 0}
								<p class="inline-copy">No active tasks yet.</p>
							{:else}
								{#each activeTasks as item}
									<label class="task-row"><input type="checkbox" disabled checked /><strong>{item.title}</strong><span>{taskSourceLabel(item)}</span><span>{item.project_id ?? 'Unassigned'}</span><time>{taskCreatedTime(item.created_at)}</time><em>{item.hermes_status}</em></label>
								{/each}
							{/if}

							<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="tasks-candidate-review" data-widget-hidden={!isWidgetVisible('tasks-candidate-review')}>
								{@render widgetEditChrome('tasks-candidate-review')}
								<h3 class="task-group">Review Queue <em>{suggestedTaskCandidates.length}</em></h3>
								<div class="table-head task-table-head"><span>Candidate</span><span>Source</span><span>Project</span><span>Confidence</span><span>Action</span></div>
								{#if isTasksLoading}
									<p class="inline-copy">Loading task candidates…</p>
								{:else if suggestedTaskCandidates.length === 0}
									<p class="inline-copy">No suggested candidates.</p>
								{:else}
									{#each suggestedTaskCandidates as candidate}
										<div class="task-row task-row-actions">
											<strong>{candidate.title}</strong>
											<span>{taskSourceLabel(candidate)}</span>
											<span>{candidate.project_id ?? 'Unassigned'}</span>
											<em>{taskConfidence(candidate)}</em>
											<div class="task-actions">
												<button type="button" onclick={() => void setTaskCandidateReview(candidate, 'user_confirmed')}><Icon icon="tabler:check" width="15" height="15" /> Confirm</button>
												<button type="button" onclick={() => void setTaskCandidateReview(candidate, 'user_rejected')}><Icon icon="tabler:x" width="15" height="15" /> Reject</button>
											</div>
										</div>
									{/each}
								{/if}
							</div>
						</section>
					</div>
					<aside class="stacked-rail">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="tasks-ai-refresh-status" data-widget-hidden={!isWidgetVisible('tasks-ai-refresh-status')}>
							{@render widgetEditChrome('tasks-ai-refresh-status')}
							<section class="panel chart-panel"><h2>Review Stats</h2><div class="donut"><strong>{taskCandidates.length}</strong><span>Suggestions</span></div><ul><li>{`${suggestedTaskCandidates.length} Suggested`}</li><li>{`${activeTasks.length} Active`}</li><li>{`${taskCandidates.length - suggestedTaskCandidates.length - activeTasks.length} Done`}</li></ul></section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="tasks-context" data-widget-hidden={!isWidgetVisible('tasks-context')}>
							{@render widgetEditChrome('tasks-context')}
							<section class="panel info-card">
								<h2>Recent Candidate Signals</h2>
								{#if suggestedTaskCandidates.length === 0}
									<p class="muted-copy">No pending candidate signals.</p>
								{:else}
									{#each suggestedTaskCandidates.slice(0, 5) as candidate}
										<div class="deadline"><span>{candidate.title}</span><time>{candidate.source_kind}</time></div>
									{/each}
								{/if}
							</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="tasks-deadlines-priority" data-widget-hidden={!isWidgetVisible('tasks-deadlines-priority')}>
							{@render widgetEditChrome('tasks-deadlines-priority')}
							<section class="panel info-card"><h2>Active Task Sources</h2>{#each ['message','document'] as source}<div class="bar-row"><span>{source}</span><div><i></i></div></div>{/each}</section>
						</div>
					</aside>
				</div>

				{#if isComposeOpen}
					<button type="button" class="drawer-backdrop" onclick={() => (isComposeOpen = false)} aria-label="Close compose"></button>
					<aside class="account-drawer"  aria-label="Compose email">
						<header>
							<div><p>{_('Compose')}</p><h2>New Message</h2></div>
							<button type="button" class="icon-button" onclick={() => (isComposeOpen = false)} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
						</header>
						<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void handleSaveDraft(); }}>
							<label><span>To</span><input bind:value={composeForm.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
							<label><span>CC</span><input bind:value={composeForm.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
							<label><span>Subject</span><input bind:value={composeForm.subject} placeholder="Email subject" autocomplete="off" /></label>
							<label class="wide"><span>Body</span><textarea bind:value={composeForm.body} rows="8" placeholder="Write your message..."></textarea></label>
							<div class="form-actions wide">
								<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
								<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if drafts.length > 0}
					<div class="draft-strip">
						<strong>Drafts ({drafts.length})</strong>
						{#each drafts.slice(0, 3) as draft}
							<button type="button" class="draft-chip" onclick={() => { composeForm = { draft_id: draft.draft_id, account_id: draft.account_id, to_text: draft.to_recipients.join(', '), cc_text: draft.cc_recipients.join(', '), subject: draft.subject, body: draft.body_text }; isComposeOpen = true; }}>
								<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
							</button>
						{/each}
					</div>
				{/if}

				{#if mailboxHealth}
					<div class="health-strip">
						<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{mailboxHealth.needs_action} need action</span>
						<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{mailboxHealth.waiting} waiting</span>
						<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{mailboxHealth.done} done</span>
						<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{mailboxHealth.total_messages} total</span>
						{#if mailboxHealth.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{mailboxHealth.important} important</span>{/if}
					</div>
				{/if}
			</section>
		{:else if currentView === 'calendar'}
			<section class="calendar-page">
				<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="calendar-toolbar" data-widget-hidden={!isWidgetVisible('calendar-toolbar')}>
					{@render widgetEditChrome('calendar-toolbar')}
					<div class="view-header">
						<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:calendar" width="28" height="28" /></span><div><h1>{_(activeView.title)}</h1><p>{_(activeView.subtitle)}</p></div></div>
						<div class="search-bar">
							<input type="text" placeholder="Search events..." bind:value={calendarSearchQuery} oninput={() => searchCalendar()} />
						</div>
						<div class="section-tabs pill-tabs">
							<button type="button" class:active={calendarViewMode === 'day'} onclick={() => { calendarViewMode = 'day'; loadCalendar(); }}>Day</button>
							<button type="button" class:active={calendarViewMode === 'week'} onclick={() => { calendarViewMode = 'week'; loadCalendar(); }}>Week</button>
							<button type="button" class:active={calendarViewMode === 'month'} onclick={() => { calendarViewMode = 'month'; loadCalendar(); }}>Month</button>
							<button type="button" class:active={calendarViewMode === 'agenda'} onclick={() => { calendarViewMode = 'agenda'; loadCalendar(); }}>Agenda</button>
						</div>
						<button type="button" class="primary-button" onclick={() => showNewEventForm = !showNewEventForm}><Icon icon="tabler:plus" width="16" height="16" /> New Event</button>
						<button type="button" class="ghost-button" onclick={() => openAccountDrawer('calendar')}><Icon icon="tabler:calendar-plus" width="16" height="16" />Add Calendar</button>
						<button type="button" class="ghost-button" onclick={() => { loadCalendar(); loadWeeklyBrief(); }} title="Refresh"><Icon icon="tabler:refresh" width="16" height="16" /></button>
					</div>
				</div>

				{#if showNewEventForm}
					<div class="panel new-event-form">
						<h3>New Event</h3>
						<div class="form-row">
							<input type="text" placeholder="Event title" bind:value={newEventTitle} />
							<select bind:value={newEventType}>
								<option value="meeting">Meeting</option><option value="focus">Focus</option>
								<option value="deadline">Deadline</option><option value="personal">Personal</option>
								<option value="travel">Travel</option><option value="tax">Tax</option>
								<option value="review">Review</option><option value="planning">Planning</option>
							</select>
						</div>
						<div class="form-row">
							<input type="datetime-local" bind:value={newEventStart} />
							<span>→</span>
							<input type="datetime-local" bind:value={newEventEnd} />
						</div>
						<div class="form-actions">
							<button type="button" class="primary-button" onclick={handleCreateEvent}>Create</button>
							<button type="button" class="ghost-button" onclick={() => showNewEventForm = false}>Cancel</button>
						</div>
					</div>
				{/if}

				<div class="filter-bar">
					<span>{calendarAccounts.length} accounts &middot; {calendarEvents.length} events</span>
					{#if calendarError}<span class="error-text">{calendarError}</span>{/if}
					{#if calendarSearchResults.length > 0}
						<span class="search-hint">Search: {calendarSearchResults.length} results for "{calendarSearchQuery}"</span>
					{/if}
				</div>

				<div class="calendar-layout">
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="calendar-week-grid" data-widget-hidden={!isWidgetVisible('calendar-week-grid')}>
						{@render widgetEditChrome('calendar-week-grid')}
						<section class="panel week-board">
							<div class="week-header">{#each weekColumns as day}<strong>{day}</strong>{/each}</div>
							<div class="event-list">
								{#if isCalendarLoading}
									<div class="loading-state">Loading events...</div>
								{:else if (calendarSearchResults.length > 0 ? calendarSearchResults : filteredEvents).length === 0}
									<div class="empty-state">No events</div>
								{:else}
									{#each (calendarSearchResults.length > 0 ? calendarSearchResults : filteredEvents) as evt (evt.event_id)}
										{@const tone = evt.event_type === 'meeting' ? 'blue' : evt.event_type === 'deadline' ? 'red' : evt.event_type === 'focus' ? 'green' : 'neutral'}
										{@const dayLabel = new Date(evt.start_at).toLocaleDateString('en-US', {weekday:'short', day:'numeric'})}
										<div class="event-row {tone}" onclick={() => prepareEvent(evt)} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && prepareEvent(evt)}>
											<span class="event-day">{dayLabel}</span>
											<span class="event-time">{new Date(evt.start_at).toLocaleTimeString([], {hour:'2-digit', minute:'2-digit'})} - {new Date(evt.end_at).toLocaleTimeString([], {hour:'2-digit', minute:'2-digit'})}</span>
											<strong>{evt.title}</strong>
											<span class="event-type-chip">{evt.event_type || 'event'}</span>
											{#if evt.importance_score && evt.importance_score > 0.5}<em class="importance-dot high"></em>{/if}
											{#if evt.readiness_score != null && evt.readiness_score < 0.5}<em class="importance-dot warn"></em>{/if}
										</div>
									{/each}
								{/if}
							</div>
							<footer class="source-footer">
								{#each calendarAccounts as acct}
									<span class="source-badge">{acct.account_name}</span>
								{/each}
							</footer>
						</section>
					</div>
					<aside class="stacked-rail">
						<!-- Weekly Brief -->
						<div class="panel info-card">
							<h2>Weekly Brief <button type="button" class="link-row" onclick={loadWeeklyBrief}><Icon icon="tabler:refresh" width="12" height="12" /></button></h2>
							{#if weeklyBrief}
								<div class="metric-grid tiny">
									<article class="metric-card"><span>Events</span><strong>{weeklyBrief.upcoming_events_this_week as number || 0}</strong></article>
									<article class="metric-card"><span>Overdue</span><strong>{weeklyBrief.overdue_deadlines as number || 0}</strong></article>
									<article class="metric-card"><span>No Notes</span><strong>{weeklyBrief.past_events_without_notes as number || 0}</strong></article>
								</div>
							{:else}
								<p class="muted">Click refresh to load</p>
							{/if}
						</div>

						<!-- Upcoming -->
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="calendar-upcoming" data-widget-hidden={!isWidgetVisible('calendar-upcoming')}>
							{@render widgetEditChrome('calendar-upcoming')}
							<section class="panel info-card">
								<h2>{_('Upcoming')}</h2>
								{#if calendarEvents.length === 0}
									<p class="muted">No upcoming events</p>
								{:else}
									{#each calendarEvents.filter(e => new Date(e.start_at) >= new Date()).slice(0, 8) as evt}
										<div class="deadline" role="button" tabindex="0" onclick={() => prepareEvent(evt)} onkeydown={(e) => e.key === 'Enter' && prepareEvent(evt)}>
											<span>{new Date(evt.start_at).toLocaleDateString('en-US', {weekday:'short', month:'short', day:'numeric'})} &middot; {evt.title}</span>
											<time>{new Date(evt.start_at).toLocaleTimeString([], {hour:'2-digit', minute:'2-digit'})}</time>
										</div>
									{/each}
								{/if}
							</section>
						</div>

						<!-- Event Detail (when selected) -->
						{#if selectedEvent}
							<div class="panel info-card event-detail">
								<h2>{selectedEvent.title} <button type="button" class="ghost-button small" onclick={() => { selectedEvent = null; eventBrief = null; eventContext = null; }}><Icon icon="tabler:x" width="14" height="14" /></button></h2>
								<div class="event-meta">
									<span><Icon icon="tabler:clock" width="14" height="14" /> {new Date(selectedEvent.start_at).toLocaleString()}</span>
									{#if selectedEvent.location}<span><Icon icon="tabler:map-pin" width="14" height="14" /> {selectedEvent.location}</span>{/if}
									<span class="chip {selectedEvent.status}">{selectedEvent.status}</span>
								</div>
								{#if eventBrief}
									<div class="brief-section">
										<h4>Brief</h4>
										{#if (eventBrief.participants as any[])}
											<div class="brief-participants">
												{#each (eventBrief.participants as any[]) as p}
													<span class="participant-chip">{p.name || p.email}</span>
												{/each}
											</div>
										{/if}
										{#if (eventBrief.context as any)?.summary}<p class="muted">{(eventBrief.context as any).summary}</p>{/if}
									</div>
								{/if}
								{#if eventAgenda}
									<div class="brief-section">
										<h4>Agenda</h4>
										{#if eventAgenda.suggested_agenda}
											<ul class="agenda-list">
												{#each (eventAgenda.suggested_agenda as any[]) as item}
													<li>{item}</li>
												{/each}
											</ul>
										{/if}
									</div>
								{/if}
								<div class="event-actions">
									<button type="button" class="primary-button small" onclick={() => selectedEvent && prepareEvent(selectedEvent)}><Icon icon="tabler:brain" width="14" height="14" /> Prepare</button>
									<button type="button" class="ghost-button small" onclick={() => selectedEvent && completeEvent(selectedEvent)}><Icon icon="tabler:check" width="14" height="14" /> Complete</button>
								</div>
							</div>
						{/if}

						<!-- Calendars -->
						<div class="widget-frame stacked-rail" class:editing={isLayoutEditing} data-widget-id="calendar-source-status" data-widget-hidden={!isWidgetVisible('calendar-source-status')}>
							{@render widgetEditChrome('calendar-source-status')}
							<section class="panel info-card">
								<h2>Calendars</h2>
								{#if calendarSources.length === 0}
									{#each calendarAccounts as acct}
										<label class="mini-check"><input type="checkbox" checked disabled />{acct.account_name}<em>{acct.provider}</em></label>
									{/each}
								{:else}
									{#each calendarSources as src}
										<label class="mini-check"><input type="checkbox" checked disabled />{src.name}<em>{src.timezone || ''}</em></label>
									{/each}
								{/if}
							</section>
						</div>
					</aside>
				</div>
			</section>
		{:else if currentView === 'documents'}
			<section class="documents-page">
				<div class="view-header">
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:file-text" width="28" height="28" /></span><div><h1>{_(activeView.title)}</h1><p>{_(activeView.subtitle)}</p></div></div>
					<button type="button" class="primary-button" disabled>Upload</button>
				</div>
				<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="documents-source-cards" data-widget-hidden={!isWidgetVisible('documents-source-cards')}>
					{@render widgetEditChrome('documents-source-cards')}
					<div class="source-strip">
						{#each ['Google Drive 1,243', 'OneDrive 812', 'Dropbox 342', 'Notion 256'] as source, index}
							<article class="source-card"><Icon icon={index === 0 ? 'tabler:brand-google-drive' : index === 1 ? 'tabler:cloud' : index === 2 ? 'tabler:brand-dropbox' : 'tabler:brand-notion'} width="28" height="28" /><span>{source}</span></article>
						{/each}
						<button type="button" class="source-card add" disabled><Icon icon="tabler:plus" width="20" height="20" />Add Source</button>
					</div>
				</div>
				<div class="filter-bar"><button type="button" disabled>All Accounts</button><button type="button" disabled>All Types</button><button type="button" disabled>All Projects</button><button type="button" disabled>All Folders</button><button type="button" disabled>Filters</button></div>
				<div class="docs-layout">
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="documents-navigation" data-widget-hidden={!isWidgetVisible('documents-navigation')}>
						{@render widgetEditChrome('documents-navigation')}
						<aside class="left-panels"><section class="panel info-card"><h2>Smart Collections</h2>{#each ['Recently Added 48', 'Recently Opened 24', 'Important 32', 'Shared with Me 18', 'Requires Review 7', 'Contracts & Legal 23', 'Financial 15'] as item}<div class="collection-row">{item}</div>{/each}</section><section class="panel info-card"><h2>My Folders</h2>{#each ['Hermes Hub', 'Projects', 'Personal', 'Work', 'Archive 2024', 'Clients', 'References'] as folder}<div class="collection-row"><Icon icon="tabler:folder" width="16" height="16" />{folder}</div>{/each}</section></aside>
					</div>
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="documents-list" data-widget-hidden={!isWidgetVisible('documents-list')}>
						{@render widgetEditChrome('documents-list')}
						<section class="panel docs-table">
							<header><h2>Hermes Hub</h2><div class="section-tabs"><button type="button" class="active">Overview</button><button type="button" disabled>Documents <em>142</em></button><button type="button" disabled>Folders <em>16</em></button><button type="button" disabled>Links <em>28</em></button><button type="button" disabled>Activity</button></div></header>
							<div class="category-grid">{#each ['Architecture 23 documents','Product 31 documents','Design 18 documents','Meetings 24 documents','Contracts 12 documents','Research 15 documents','Reports 11 documents','Other 8 documents'] as category}<article><Icon icon="tabler:folder" width="20" height="20" /><span>{category}</span></article>{/each}</div>
							<div class="table-head docs"><span>Name</span><span>Source</span><span>Project</span><span>Type</span><span>Last Modified</span><span>Size</span></div>
							{#each documents as doc}
								<div class="doc-row"><Icon icon={doc.icon} width="19" height="19" /><strong>{doc.name}</strong><span>{doc.source}</span><span>{doc.project}</span><span>{doc.type}</span><time>{doc.date}</time><em>{doc.size}</em></div>
							{/each}
						</section>
					</div>
					<aside class="stacked-rail">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="documents-processing-jobs" data-widget-hidden={!isWidgetVisible('documents-processing-jobs')}>
							{@render widgetEditChrome('documents-processing-jobs')}
							<section class="panel info-card">
								<h2>Document Processing</h2>
								{#if isDocumentProcessingJobsLoading}
									<p class="muted-copy">Loading document processing status…</p>
								{:else if documentProcessingJobsError}
									<p class="muted-copy">{documentProcessingJobsError}</p>
								{:else if documentProcessingJobs.length === 0}
									<p class="muted-copy">No processing jobs yet.</p>
								{:else}
									{#each documentProcessingJobs.slice(0, 5) as job}
										<div class="deadline">
											<div>
												<span>{job.document_id} · {job.step}</span>
												<small>{job.status}{job.last_error_summary ? ` — ${job.last_error_summary}` : ''}</small>
											</div>
											{#if job.status === 'failed'}
												<button
													type="button"
													disabled={retryingDocumentProcessingJobId === job.job_id}
													onclick={() => void retryFailedDocumentProcessingJob(job)}
												>
													Retry
												</button>
											{/if}
										</div>
									{/each}
									{#if documentProcessingDetailError}
										<p class="muted-copy">{documentProcessingDetailError}</p>
									{/if}
								{/if}
							</section>
						</div>
						<div class="widget-frame stacked-rail" class:editing={isLayoutEditing} data-widget-id="documents-related-context" data-widget-hidden={!isWidgetVisible('documents-related-context')}>
							{@render widgetEditChrome('documents-related-context')}
							<section class="panel chart-panel"><h2>Documents Insights</h2><strong>2,653</strong><span>Total Documents</span><div class="donut small"><strong>24%</strong></div></section>
							<section class="panel info-card"><h2>Document Types</h2>{#each ['PDF 1,234 (46%)', 'Documents 623 (23%)', 'Spreadsheets 312 (12%)', 'Presentations 198 (7%)', 'Images 142 (5%)'] as item}<div class="bar-row"><span>{item}</span><div><i></i></div></div>{/each}</section>
							<section class="panel info-card"><h2>Recent Activity</h2>{#each personList.slice(1,5) as person}<div class="person-compact"><img src="/assets/hermes-reference-avatar.png" alt="" /><span><strong>{person.name}</strong><small>updated a document</small></span></div>{/each}</section>
						</div>
					</aside>
				</div>

				{#if isComposeOpen}
					<button type="button" class="drawer-backdrop" onclick={() => (isComposeOpen = false)} aria-label="Close compose"></button>
					<aside class="account-drawer"  aria-label="Compose email">
						<header>
							<div><p>{_('Compose')}</p><h2>New Message</h2></div>
							<button type="button" class="icon-button" onclick={() => (isComposeOpen = false)} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
						</header>
						<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void handleSaveDraft(); }}>
							<label><span>To</span><input bind:value={composeForm.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
							<label><span>CC</span><input bind:value={composeForm.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
							<label><span>Subject</span><input bind:value={composeForm.subject} placeholder="Email subject" autocomplete="off" /></label>
							<label class="wide"><span>Body</span><textarea bind:value={composeForm.body} rows="8" placeholder="Write your message..."></textarea></label>
							<div class="form-actions wide">
								<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
								<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if drafts.length > 0}
					<div class="draft-strip">
						<strong>Drafts ({drafts.length})</strong>
						{#each drafts.slice(0, 3) as draft}
							<button type="button" class="draft-chip" onclick={() => { composeForm = { draft_id: draft.draft_id, account_id: draft.account_id, to_text: draft.to_recipients.join(', '), cc_text: draft.cc_recipients.join(', '), subject: draft.subject, body: draft.body_text }; isComposeOpen = true; }}>
								<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
							</button>
						{/each}
					</div>
				{/if}

				{#if mailboxHealth}
					<div class="health-strip">
						<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{mailboxHealth.needs_action} need action</span>
						<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{mailboxHealth.waiting} waiting</span>
						<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{mailboxHealth.done} done</span>
						<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{mailboxHealth.total_messages} total</span>
						{#if mailboxHealth.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{mailboxHealth.important} important</span>{/if}
					</div>
				{/if}
			</section>
		{:else if currentView === 'notes'}
			<section class="notes-page">
				<div class="notes-layout">
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="notes-source-filters" data-widget-hidden={!isWidgetVisible('notes-source-filters')}>
						{@render widgetEditChrome('notes-source-filters')}
						<aside class="left-panels">
							<section class="panel info-card"><h2>Sources</h2>{#each ['Apple Notes 1,243','Obsidian 872','Anytype 532','Gmail 1,156','Outlook 623'] as source}<div class="collection-row">{source}</div>{/each}<button type="button" class="link-row" disabled>Add Source</button></section>
							<section class="panel info-card"><h2>Collections</h2>{#each ['Inbox 231','Starred 128','Today 89','To Review 74','Personal 312','Projects 482','Ideas 156','Research 203','Archive 1,024'] as item}<div class="collection-row">{item}</div>{/each}</section>
							<section class="panel info-card"><h2>Tags</h2><div class="tag-cloud"><span># project 342</span><span># idea 156</span><span># meeting 213</span><span># research 182</span><span># reference 98</span><span># design 76</span></div></section>
						</aside>
					</div>
					<section class="notes-main">
						<div class="view-header">
							<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:notes" width="28" height="28" /></span><div><h1>Notes</h1><p>All your notes from connected sources</p></div></div>
							<div class="metric-grid inline-metrics tiny">{#each [{label:'Total Notes',value:'4,426',delta:'18%'},{label:'This Week',value:'128',delta:'12%'},{label:'Unprocessed',value:'89',delta:'5%'}] as metric}<article class="metric-card"><span>{metric.label}</span><strong>{metric.value}</strong><small>↑ {metric.delta}</small></article>{/each}</div>
							<button type="button" class="primary-button" disabled>New Note</button>
						</div>
						<div class="filter-bar"><button type="button" disabled>All Sources</button><button type="button" disabled>All Types</button><button type="button" disabled>All Collections</button><button type="button" disabled>All Tags</button><button type="button" disabled>Filters</button><button type="button" disabled>Sort: Updated</button></div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="notes-list" data-widget-hidden={!isWidgetVisible('notes-list')}>
							{@render widgetEditChrome('notes-list')}
							<section class="notes-list panel">
								<h3>Today</h3>{#each notes.slice(0,4) as note}<article><Icon icon={note.icon} width="22" height="22" /><div><strong>{note.title}</strong><p>{note.body}</p><small>{note.source} · {note.time}</small></div><em>{note.tag}</em></article>{/each}
								<h3>Yesterday</h3>{#each notes.slice(4) as note}<article><Icon icon={note.icon} width="22" height="22" /><div><strong>{note.title}</strong><p>{note.body}</p><small>{note.source} · {note.time}</small></div><em>{note.tag}</em></article>{/each}
							</section>
						</div>
					</section>
					<aside class="stacked-rail">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="notes-metadata" data-widget-hidden={!isWidgetVisible('notes-metadata')}>
							{@render widgetEditChrome('notes-metadata')}
							<section class="panel chart-panel"><h2>Notes Insights</h2><div class="donut"><strong>4,426</strong><span>Total Notes</span></div></section>
						</div>
						<div class="widget-frame stacked-rail" class:editing={isLayoutEditing} data-widget-id="notes-related-projects-documents" data-widget-hidden={!isWidgetVisible('notes-related-projects-documents')}>
							{@render widgetEditChrome('notes-related-projects-documents')}
							<section class="panel info-card"><h2>Activity</h2>{#each ['You created a note','Maria Petrova shared a note','Email processed','Note linked to project'] as item}<div class="deadline"><span>{item}</span><time>10:42</time></div>{/each}</section>
							<section class="panel info-card"><h2>Unprocessed Items</h2>{#each ['23 Emails','34 Apple Notes','12 Attachments','8 Web Clippings'] as item}<div class="collection-row">{item}</div>{/each}<button type="button" class="link-row" disabled>Process All</button></section>
						</div>
					</aside>
				</div>

				{#if isComposeOpen}
					<button type="button" class="drawer-backdrop" onclick={() => (isComposeOpen = false)} aria-label="Close compose"></button>
					<aside class="account-drawer"  aria-label="Compose email">
						<header>
							<div><p>{_('Compose')}</p><h2>New Message</h2></div>
							<button type="button" class="icon-button" onclick={() => (isComposeOpen = false)} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
						</header>
						<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void handleSaveDraft(); }}>
							<label><span>To</span><input bind:value={composeForm.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
							<label><span>CC</span><input bind:value={composeForm.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
							<label><span>Subject</span><input bind:value={composeForm.subject} placeholder="Email subject" autocomplete="off" /></label>
							<label class="wide"><span>Body</span><textarea bind:value={composeForm.body} rows="8" placeholder="Write your message..."></textarea></label>
							<div class="form-actions wide">
								<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
								<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if drafts.length > 0}
					<div class="draft-strip">
						<strong>Drafts ({drafts.length})</strong>
						{#each drafts.slice(0, 3) as draft}
							<button type="button" class="draft-chip" onclick={() => { composeForm = { draft_id: draft.draft_id, account_id: draft.account_id, to_text: draft.to_recipients.join(', '), cc_text: draft.cc_recipients.join(', '), subject: draft.subject, body: draft.body_text }; isComposeOpen = true; }}>
								<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
							</button>
						{/each}
					</div>
				{/if}

				{#if mailboxHealth}
					<div class="health-strip">
						<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{mailboxHealth.needs_action} need action</span>
						<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{mailboxHealth.waiting} waiting</span>
						<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{mailboxHealth.done} done</span>
						<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{mailboxHealth.total_messages} total</span>
						{#if mailboxHealth.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{mailboxHealth.important} important</span>{/if}
					</div>
				{/if}
			</section>
		{:else if currentView === 'knowledge'}
			<section class="knowledge-page">
				<div class="graph-filter-tabs">
					{#each graphFilterChips as item}
						<button
							type="button"
							class:active={item.id === 'all'}
							disabled={!item.enabled}
							title={item.enabled ? `${item.label} graph view` : `${item.label} filtering is not available in this read-only slice`}
						>
							{item.label}
							{#if item.count !== null}<em>{formatNumber(item.count)}</em>{/if}
						</button>
					{/each}
					<button type="button" disabled title="Projection rebuild requires a command API boundary">
						<Icon icon="tabler:refresh" width="15" height="15" />
						Rebuild
					</button>
				</div>

				<div class="knowledge-layout">
					<section class="panel graph-workbench">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-toolbar" data-widget-hidden={!isWidgetVisible('knowledge-toolbar')}>
							{@render widgetEditChrome('knowledge-toolbar')}
							<div class="graph-toolbar">
								<form
									class="graph-search-form"
									onsubmit={(event) => {
										event.preventDefault();
										void runGraphSearch();
									}}
								>
									<Icon icon="tabler:search" width="17" height="17" />
									<input
										bind:value={graphSearchQuery}
										placeholder="Search graph nodes..."
										aria-label="Search graph nodes"
									/>
									<button type="submit" disabled={isGraphSearchLoading || !graphSearchQuery.trim()}>
										{isGraphSearchLoading ? 'Searching' : 'Search'}
									</button>
								</form>
								<button type="button" disabled title="Pan and zoom engine is not part of this slice">
									<Icon icon="tabler:hand-click" width="16" height="16" />
								</button>
								<button type="button" disabled title="Depth is fixed to 1 by the current graph API contract">
									Depth 1
								</button>
							</div>
						</div>

						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-search-results" data-widget-hidden={!isWidgetVisible('knowledge-search-results')}>
							{@render widgetEditChrome('knowledge-search-results')}
							<div
								class="graph-search-strip"
								aria-live="polite"
								aria-busy={isGraphSearchLoading || isGraphNodeChoicesLoading}
							>
								{#if graphSearchError}
									<div class="graph-strip-message error">
										<span>{graphSearchError}</span>
										<button type="button" onclick={() => void runGraphSearch()}>Retry</button>
									</div>
								{:else if graphSearchResults.length > 0}
									<div class="graph-picker">
										<div class="graph-picker-head">
											<span>Search results</span>
											<em>{formatNumber(graphSearchResults.length)}</em>
										</div>
										<div class="graph-result-row" aria-label="Graph search results">
											{#each graphSearchResults as node}
												<button
													type="button"
													class:active={selectedGraphNode?.node_id === node.node_id}
													onclick={() => void selectGraphNode(node)}
												>
													<Icon icon={graphNodeKindIcon(node.node_kind)} width="16" height="16" />
													<span>{node.label}</span>
													<em>{formatGraphKind(node.node_kind)}</em>
												</button>
											{/each}
										</div>
									</div>
								{:else if graphSearchSubmitted && lastSubmittedGraphSearchQuery}
									<div class="graph-strip-message">
										<span>No graph nodes found for "{lastSubmittedGraphSearchQuery}".</span>
									</div>
								{:else if graphNodeChoicesError}
									<div class="graph-strip-message error">
										<span>{graphNodeChoicesError}</span>
										<button type="button" onclick={() => void loadGraphNodeChoices()}>Retry</button>
									</div>
								{:else if graphNodeChoices.length > 0}
									<div class="graph-picker">
										<div class="graph-picker-head">
											<span>Suggested nodes</span>
											<em>{formatNumber(graphNodeChoices.length)}</em>
										</div>
										<div class="graph-result-row" aria-label="Suggested graph nodes">
											{#each graphNodeChoices as node}
												<button
													type="button"
													class:active={selectedGraphNode?.node_id === node.node_id}
													onclick={() => void selectGraphNode(node)}
												>
													<Icon icon={graphNodeKindIcon(node.node_kind)} width="16" height="16" />
													<span>{node.label}</span>
													<em>{formatGraphKind(node.node_kind)}</em>
												</button>
											{/each}
										</div>
									</div>
								{:else if isGraphNodeChoicesLoading}
									<div class="graph-strip-message">
										<span>Loading selectable graph nodes.</span>
									</div>
								{:else}
									<div class="graph-strip-message">
										<span>No selectable graph nodes returned by the local projection.</span>
									</div>
								{/if}
							</div>
						</div>

						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-graph-canvas" data-widget-hidden={!isWidgetVisible('knowledge-graph-canvas')}>
							{@render widgetEditChrome('knowledge-graph-canvas')}
							<div class="knowledge-canvas" aria-busy={isGraphNeighborhoodLoading}>
								{#if graphError && !graphSummary}
									<div class="graph-state-card error">
										<Icon icon="tabler:alert-triangle" width="26" height="26" />
										<h2>Graph summary unavailable</h2>
										<p>{graphError}</p>
										<button type="button" onclick={() => void loadGraphSummary()}>Retry summary</button>
									</div>
								{:else if isGraphSummaryLoading && !graphSummary}
									<div class="graph-state-card">
										<Icon icon="tabler:loader-2" width="26" height="26" />
										<h2>Loading graph summary</h2>
										<p>Reading local graph projection metadata.</p>
									</div>
								{:else if graphSummary?.is_empty}
									<div class="graph-state-card">
										<Icon icon="tabler:database-off" width="26" height="26" />
										<h2>No graph projection yet</h2>
										<p>Import persons, messages or documents, then run the existing projection smoke command to create graph data.</p>
									</div>
								{:else if graphNeighborhood}
									<svg class="graph-edge-layer" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
										{#each graphCanvasEdges as edge}
											<line
												x1={edge.x1}
												y1={edge.y1}
												x2={edge.x2}
												y2={edge.y2}
												class:reviewed={edge.review_state === 'system_accepted' || edge.review_state === 'user_confirmed'}
											/>
										{/each}
										{#each graphCanvasEdges as edge}
											<text
												class="graph-edge-label"
												class:reviewed={edge.review_state === 'system_accepted' || edge.review_state === 'user_confirmed'}
												x={(edge.x1 + edge.x2) / 2}
												y={(edge.y1 + edge.y2) / 2}
											>
												{edge.label}
											</text>
										{/each}
									</svg>
									{#each graphCanvasNodes as node}
										<button
											type="button"
											class="graph-node {node.layoutClass}"
											class:kind-person={node.node_kind === 'person'}
											class:kind-email_address={node.node_kind === 'email_address'}
											class:kind-message={node.node_kind === 'message'}
											class:kind-document={node.node_kind === 'document'}
											class:selected={node.isSelected}
											onclick={() => void selectGraphNode(node)}
											title={`${node.label} - ${formatGraphKind(node.node_kind)}`}
										>
											<Icon icon={graphNodeKindIcon(node.node_kind)} width={node.isSelected ? 28 : 21} height={node.isSelected ? 28 : 21} />
											<strong>{node.label}</strong>
											<span>{formatGraphKind(node.node_kind)}</span>
										</button>
									{/each}
								{:else}
									<div class="graph-state-card">
										<img src="/assets/hermes-logo-mark.png" alt="" />
										<h2>Select a graph node</h2>
										<p>{formatNumber(graphNodeTotal())} nodes and {formatNumber(graphRelationshipTotal())} connections are available from the local projection. Use Suggested nodes or search to load a neighborhood.</p>
									</div>
								{/if}
								{#if isGraphNeighborhoodLoading}
									<div class="graph-loading-overlay" role="status" aria-live="polite">
										<Icon icon="tabler:loader-2" width="22" height="22" />
										<span>Loading neighborhood</span>
									</div>
								{/if}
							</div>
						</div>

						<footer class="graph-status-bar">
							<span>Projection: {formatGraphTimestamp(graphSummary?.latest_projection_at ?? null)}</span>
							<span>Evidence: {formatNumber(graphEvidenceTotal())}</span>
							{#if graphNeighborhood?.truncated}<span>Neighborhood truncated at {graphNeighborhood.edge_limit} edges</span>{/if}
							{#if graphNeighborhood?.evidence_truncated}<span>Evidence truncated at {graphNeighborhood.evidence_limit} rows</span>{/if}
						</footer>
					</section>

					<aside class="stacked-rail">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-node-inspector" data-widget-hidden={!isWidgetVisible('knowledge-node-inspector')}>
							{@render widgetEditChrome('knowledge-node-inspector')}
							<section class="panel info-card">
								<h2>Selected Node</h2>
								{#if selectedGraphNode}
									<div class="doc-mini">
										<Icon icon={graphNodeKindIcon(selectedGraphNode.node_kind)} width="24" height="24" />
										<span>
											<strong>{selectedGraphNode.label}</strong>
											<small>{formatGraphKind(selectedGraphNode.node_kind)}</small>
										</span>
									</div>
									<ul class="detail-list node-detail-list">
										<li>Stable key <em>{selectedGraphNode.stable_key}</em></li>
										<li>Created <em>{formatGraphTimestamp(selectedGraphNode.created_at)}</em></li>
										<li>Updated <em>{formatGraphTimestamp(selectedGraphNode.updated_at)}</em></li>
										{#each selectedGraphProperties as row}
											<li>{formatGraphKind(row.key)} <em>{row.value}</em></li>
										{/each}
									</ul>
								{:else}
									<p>Select a graph node to inspect metadata and evidence.</p>
								{/if}
								{#if graphNeighborhoodError}
									<p class="inline-error" role="status" aria-live="polite">{graphNeighborhoodError}</p>
								{/if}
							</section>
						</div>

						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-connections" data-widget-hidden={!isWidgetVisible('knowledge-connections')}>
							{@render widgetEditChrome('knowledge-connections')}
							<section class="panel info-card">
								<h2>Connections</h2>
								{#if graphNeighborCounts.length > 0}
									{#each graphNeighborCounts as item}
										<div class="collection-row">
											<span>{formatGraphKind(item.kind)}</span>
											<em>{item.count}</em>
										</div>
									{/each}
								{:else}
									<p>No returned connections.</p>
								{/if}
							</section>
						</div>

						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-evidence-context" data-widget-hidden={!isWidgetVisible('knowledge-evidence-context')}>
							{@render widgetEditChrome('knowledge-evidence-context')}
							<section class="panel info-card">
								<h2>Evidence</h2>
								{#if graphNeighborhood?.evidence.length}
									{#each graphNeighborhood.evidence.slice(0, 5) as evidence}
										<div class="evidence-row">
											<strong>{formatGraphKind(evidence.source_kind)}</strong>
											<p>{evidence.excerpt ?? graphEvidenceLabel(evidence)}</p>
										</div>
									{/each}
								{:else}
									<p>Evidence appears after selecting a node with returned edges.</p>
								{/if}
							</section>
						</div>

						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="knowledge-graph-summary" data-widget-hidden={!isWidgetVisible('knowledge-graph-summary')}>
							{@render widgetEditChrome('knowledge-graph-summary')}
							<section class="panel info-card">
								<h2>Graph Statistics</h2>
								<div class="summary-numbers compact">
									<article><strong>{formatNumber(graphNodeTotal())}</strong><span>Nodes</span></article>
									<article><strong>{formatNumber(graphRelationshipTotal())}</strong><span>Connections</span></article>
									<article><strong>{formatNumber(graphEvidenceTotal())}</strong><span>Evidence</span></article>
									<article><strong>{formatNumber(graphNodeKindCount('person'))}</strong><span>People</span></article>
								</div>
								{#if graphError}<p class="inline-error">{graphError}</p>{/if}
							</section>
						</div>
					</aside>
				</div>

				{#if isComposeOpen}
					<button type="button" class="drawer-backdrop" onclick={() => (isComposeOpen = false)} aria-label="Close compose"></button>
					<aside class="account-drawer"  aria-label="Compose email">
						<header>
							<div><p>{_('Compose')}</p><h2>New Message</h2></div>
							<button type="button" class="icon-button" onclick={() => (isComposeOpen = false)} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
						</header>
						<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void handleSaveDraft(); }}>
							<label><span>To</span><input bind:value={composeForm.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
							<label><span>CC</span><input bind:value={composeForm.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
							<label><span>Subject</span><input bind:value={composeForm.subject} placeholder="Email subject" autocomplete="off" /></label>
							<label class="wide"><span>Body</span><textarea bind:value={composeForm.body} rows="8" placeholder="Write your message..."></textarea></label>
							<div class="form-actions wide">
								<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
								<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if drafts.length > 0}
					<div class="draft-strip">
						<strong>Drafts ({drafts.length})</strong>
						{#each drafts.slice(0, 3) as draft}
							<button type="button" class="draft-chip" onclick={() => { composeForm = { draft_id: draft.draft_id, account_id: draft.account_id, to_text: draft.to_recipients.join(', '), cc_text: draft.cc_recipients.join(', '), subject: draft.subject, body: draft.body_text }; isComposeOpen = true; }}>
								<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
							</button>
						{/each}
					</div>
				{/if}

				{#if mailboxHealth}
					<div class="health-strip">
						<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{mailboxHealth.needs_action} need action</span>
						<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{mailboxHealth.waiting} waiting</span>
						<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{mailboxHealth.done} done</span>
						<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{mailboxHealth.total_messages} total</span>
						{#if mailboxHealth.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{mailboxHealth.important} important</span>{/if}
					</div>
				{/if}
			</section>
		{:else if currentView === 'communications' && activeCommunicationSection === 'telegram'}
			<section class="telegram-page communications-page">
				<div class="view-header">
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:brand-telegram" width="28" height="28" /></span><div><h1>{_(activeView.title)}</h1><p>{_(activeView.subtitle)}</p></div></div>
					<button type="button" class="primary-button" onclick={() => openAccountDrawer('telegram')}><Icon icon="tabler:plus" width="16" height="16" />Add Account</button>
					<button type="button" class="primary-button" onclick={() => void loadTelegramWorkspace()} disabled={isTelegramLoading}><Icon icon="tabler:refresh" width="16" height="16" />Refresh</button>
				</div>

				<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="telegram-account-status" data-widget-hidden={!isWidgetVisible('telegram-account-status')}>
					{@render widgetEditChrome('telegram-account-status')}
					<div class="metric-grid">
						<article class="metric-card"><span>Chats</span><strong>{telegramChats.length}</strong><small>{selectedTelegramChat?.sync_state ?? 'not synced'}</small></article>
						<article class="metric-card"><span>Messages</span><strong>{telegramMessages.length}</strong><small>Projected channel records</small></article>
						<article class="metric-card"><span>Templates</span><strong>{automationTemplates.length}</strong><small>UI-approved only</small></article>
						<article class="metric-card"><span>Policies</span><strong>{automationPolicies.length}</strong><small>{automationPolicies.filter((policy) => policy.enabled).length} enabled</small></article>
						<article class="metric-card"><span>Calls</span><strong>{telegramCalls.length}</strong><small>{selectedTelegramCall?.call_state ?? 'no history'}</small></article>
						<article class="metric-card"><span>Transcript</span><strong>{callTranscript?.transcript_status ?? 'none'}</strong><small>{callTranscript?.stt_provider ?? 'fixture STT'}</small></article>
					</div>
				</div>

				{#if telegramActionMessage}
					<p class="setup-state success">{telegramActionMessage}</p>
				{/if}
				{#if telegramError}
					<p class="inline-error">{telegramError}</p>
				{/if}

				<div class="three-pane communications-grid telegram-grid">
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="telegram-chat-list" data-widget-hidden={!isWidgetVisible('telegram-chat-list')}>
						{@render widgetEditChrome('telegram-chat-list')}
						<section class="panel conversation-list">
							<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder="Search Telegram chats..." /></label>
							{#if isTelegramLoading && telegramChats.length === 0}
								<div class="empty-panel">Loading Telegram state...</div>
							{:else if telegramChats.length === 0}
								<div class="empty-panel">No Telegram chats projected yet.</div>
							{:else}
								{#each telegramChats as chat}
									<button type="button" class:active={selectedTelegramChat?.provider_chat_id === chat.provider_chat_id} onclick={() => selectTelegramChat(chat)}>
										<span class="round-icon cyan"><Icon icon="tabler:brand-telegram" width="22" height="22" /></span>
										<img src="/assets/hermes-reference-avatar.png" alt="" />
										<span>
											<strong>{chat.title}</strong>
											<small>{chat.account_id} · {chat.chat_kind}</small>
											<em>{chat.sync_state}</em>
										</span>
										<time>{formatDateTime(chat.last_message_at ?? chat.updated_at)}</time>
									</button>
								{/each}
							{/if}
						</section>
					</div>

					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="telegram-message-thread" data-widget-hidden={!isWidgetVisible('telegram-message-thread')}>
						{@render widgetEditChrome('telegram-message-thread')}
						<section class="panel chat-pane telegram-chat-pane">
							{#if selectedTelegramChat}
								<header>
									<span class="round-icon cyan"><Icon icon="tabler:brand-telegram" width="24" height="24" /></span>
									<div><h2>{selectedTelegramChat.title}</h2><p>{selectedTelegramChat.account_id} · {selectedTelegramChat.provider_chat_id}</p></div>
									<div class="chat-actions">
										<button type="button" disabled title="1:1 audio call controls are backend-foundation only in this Telegram foundation"><Icon icon="tabler:phone" width="17" height="17" /></button>
										<button type="button" disabled title="Video calls are outside this Telegram foundation"><Icon icon="tabler:video" width="17" height="17" /></button>
										<button type="button" onclick={() => void loadTelegramWorkspace()} disabled={isTelegramLoading}><Icon icon="tabler:refresh" width="17" height="17" /></button>
									</div>
								</header>
								<div class="chat-body">
									{#if aiAnalysisResult && aiAnalysisResult.message_id === selectedCommunication.message_id}
										<article class="ai-analysis-card">
											<strong><Icon icon="tabler:sparkles" width="16" height="16" />AI Analysis</strong>
											{#if aiAnalysisResult.category}<p><em>Category:</em> {aiAnalysisResult.category}</p>{/if}
											{#if aiAnalysisResult.summary}<p><em>Summary:</em> {aiAnalysisResult.summary}</p>{/if}
											{#if aiAnalysisResult.importance_score != null}<p><em>Importance:</em> {aiAnalysisResult.importance_score}/100</p>{/if}
											<p><em>State:</em> <span class="state-badge {aiAnalysisResult.workflow_state}">{aiAnalysisResult.workflow_state.replace('_', ' ')}</span></p>
										</article>
									{/if}
									{#if selectedTelegramMessages.length === 0}
										<div class="empty-panel fill">No messages for this chat.</div>
									{:else}
										{#each selectedTelegramMessages.slice().reverse() as message}
											<article class="bubble" class:outbound={message.delivery_state === 'sent' || message.delivery_state === 'send_dry_run'} class:inbound={message.delivery_state !== 'sent' && message.delivery_state !== 'send_dry_run'}>
												<strong>{message.sender_display_name ?? message.sender}</strong><br />
												{message.text}
												<time>{telegramMessageTime(message)}</time>
											</article>
										{/each}
									{/if}
								</div>
								<form class="telegram-inline-form" onsubmit={(event) => { event.preventDefault(); void ingestTelegramMessageFixture(); }}>
									<input bind:value={telegramMessageForm.provider_message_id} placeholder="Provider message ID" autocomplete="off" />
									<input bind:value={telegramMessageForm.sender_display_name} placeholder="Sender" autocomplete="off" />
									<input bind:value={telegramMessageForm.text} placeholder="Fixture message text" autocomplete="off" />
									<button type="submit" disabled={isTelegramActionSubmitting || !telegramMessageForm.text.trim()}><Icon icon="tabler:send" width="17" height="17" />Ingest</button>
								</form>
							{:else}
								<div class="empty-panel fill">Create a Telegram fixture account and ingest a message.</div>
							{/if}
						</section>
					</div>

					<aside class="stacked-rail telegram-rail">
						<div class="widget-frame stacked-rail" class:editing={isLayoutEditing} data-widget-id="telegram-sync-controls" data-widget-hidden={!isWidgetVisible('telegram-sync-controls')}>
							{@render widgetEditChrome('telegram-sync-controls')}
							<section class="panel info-card">
							<h2>Account Setup</h2>
							<div class="setup-summary-card">
								<span class="round-icon purple"><Icon icon="tabler:brand-telegram" width="22" height="22" /></span>
								<div>
									<strong>{telegramProviderAccounts.length} Telegram accounts</strong>
									<p>{telegramProviderAccounts.length ? 'User and bot records are available for ingestion and policies.' : 'No Telegram account record is configured yet.'}</p>
								</div>
							</div>
							<div class="form-actions wide">
								<button type="button" onclick={() => openAccountDrawer('telegram')} disabled={isTelegramActionSubmitting}>Open Wizard</button>
							</div>
							</section>

							<section class="panel info-card">
							<h2>Runtime Guardrails</h2>
							<div class="health-row"><span>Mode</span><strong>{telegramCapabilities?.runtime_mode ?? 'unknown'}</strong></div>
							{#if telegramClosureCapabilities.length}
								<ul class="detail-list">
									{#each telegramClosureCapabilities as capability}
										<li>{capabilityLabel(capability.capability)}<em>{capability.status}</em></li>
									{/each}
								</ul>
							{:else}
								<p>Capability contract is not loaded yet.</p>
							{/if}
							{#if telegramBlockedCapabilities.length}
								<div class="evidence-row">
									<strong>Blocked Live Runtime</strong>
									<p>{telegramBlockedCapabilities.map((capability) => capabilityLabel(capability.capability)).join(', ')}</p>
								</div>
							{/if}
							{#if telegramCapabilities?.unsupported_features.length}
								<div class="evidence-row">
									<strong>Telegram Scope</strong>
									<p>{telegramCapabilities.unsupported_features.map(capabilityLabel).join(', ')}</p>
								</div>
							{/if}
							</section>

							<section class="panel info-card">
							<h2>Template</h2>
							<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void saveTelegramAutomationTemplate(); }}>
								<label><span>Template ID</span><input bind:value={automationTemplateForm.template_id} autocomplete="off" /></label>
								<label><span>Name</span><input bind:value={automationTemplateForm.name} autocomplete="off" /></label>
								<label class="wide"><span>Body</span><textarea bind:value={automationTemplateForm.body_template} rows="3"></textarea></label>
								<label class="wide"><span>Required variables</span><input bind:value={automationTemplateForm.required_variables_text} autocomplete="off" /></label>
								<div class="form-actions wide"><button type="submit" disabled={isTelegramActionSubmitting}>Save Template</button></div>
							</form>
							{#if automationTemplates.length}
								<ul class="detail-list">
									{#each automationTemplates.slice(0, 3) as template}
										<li>{template.name}<em>{template.template_id}</em></li>
									{/each}
								</ul>
							{/if}
							</section>

							<section class="panel info-card">
							<h2>Policy</h2>
							<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void saveTelegramAutomationPolicy(); }}>
								<label><span>Policy ID</span><input bind:value={automationPolicyForm.policy_id} autocomplete="off" /></label>
								<label><span>Template ID</span><input bind:value={automationPolicyForm.template_id} autocomplete="off" /></label>
								<label><span>Name</span><input bind:value={automationPolicyForm.name} autocomplete="off" /></label>
								<label><span>Account ID</span><input bind:value={automationPolicyForm.account_id} autocomplete="off" /></label>
								<label class="wide"><span>Allowed chat IDs</span><input bind:value={automationPolicyForm.allowed_chat_ids_text} autocomplete="off" /></label>
								<label><span>Trigger</span><input bind:value={automationPolicyForm.trigger_kind} autocomplete="off" /></label>
								<label><span>Max/hour</span><input bind:value={automationPolicyForm.max_sends_per_hour} type="number" min="1" max="100" /></label>
								<label class="wide"><span>Quiet hours JSON</span><textarea bind:value={automationPolicyForm.quiet_hours_text} rows="2"></textarea></label>
								<label class="wide"><span>Conditions JSON</span><textarea bind:value={automationPolicyForm.conditions_text} rows="2"></textarea></label>
								<label class="checkbox-row"><input bind:checked={automationPolicyForm.enabled} type="checkbox" /><span>Enabled</span></label>
								<div class="form-actions"><button type="submit" disabled={isTelegramActionSubmitting}>Save Policy</button></div>
							</form>
							</section>

							<section class="panel info-card">
							<h2>Dry Run</h2>
							<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void runTelegramAutomationDryRun(); }}>
								<label><span>Policy ID</span><input bind:value={telegramSendForm.policy_id} autocomplete="off" /></label>
								<label><span>Chat ID</span><input bind:value={telegramSendForm.provider_chat_id} autocomplete="off" /></label>
								<label class="wide"><span>Variables JSON</span><textarea bind:value={telegramSendForm.variables_text} rows="3"></textarea></label>
								<label class="wide"><span>Source context JSON</span><textarea bind:value={telegramSendForm.source_context_text} rows="2"></textarea></label>
								<div class="form-actions wide"><button type="submit" disabled={isTelegramActionSubmitting}>Run Dry-run</button></div>
							</form>
							{#if telegramSendDryRunResult}
								<div class="evidence-row">
									<strong>{telegramSendDryRunResult.status}</strong>
									<p>{telegramSendDryRunResult.rendered_text}</p>
									<small>{telegramSendDryRunResult.rendered_preview_hash}</small>
								</div>
							{/if}
							</section>
						</div>

						<div class="widget-frame stacked-rail" class:editing={isLayoutEditing} data-widget-id="telegram-selected-chat-metadata" data-widget-hidden={!isWidgetVisible('telegram-selected-chat-metadata')}>
							{@render widgetEditChrome('telegram-selected-chat-metadata')}
							<section class="panel info-card">
							<h2>Calls</h2>
							{#if telegramCalls.length}
								{#each telegramCalls.slice(0, 4) as call}
									<button type="button" class="collection-row as-button" class:active={selectedTelegramCall?.call_id === call.call_id} onclick={() => selectTelegramCall(call)}>
										<span>{call.provider_chat_id}</span>
										<em>{call.call_state}</em>
									</button>
								{/each}
							{:else}
								<p>No calls saved.</p>
							{/if}
							<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void saveTelegramCallFixture(); }}>
								<label><span>Call ID</span><input bind:value={telegramCallForm.call_id} autocomplete="off" /></label>
								<label><span>Provider call ID</span><input bind:value={telegramCallForm.provider_call_id} autocomplete="off" /></label>
								<label><span>Account ID</span><input bind:value={telegramCallForm.account_id} autocomplete="off" /></label>
								<label><span>Chat ID</span><input bind:value={telegramCallForm.provider_chat_id} autocomplete="off" /></label>
								<label><span>Direction</span><select bind:value={telegramCallForm.direction}><option value="incoming">Incoming</option><option value="outgoing">Outgoing</option></select></label>
								<label><span>State</span><select bind:value={telegramCallForm.call_state}><option value="ringing">Ringing</option><option value="active">Active</option><option value="ended">Ended</option><option value="missed">Missed</option><option value="declined">Declined</option><option value="failed">Failed</option></select></label>
								<label class="wide"><span>Metadata JSON</span><textarea bind:value={telegramCallForm.metadata_text} rows="2"></textarea></label>
								<div class="form-actions wide"><button type="submit" disabled={isTelegramActionSubmitting}>Save Call</button></div>
							</form>
							</section>

							<section class="panel info-card">
							<h2>Transcript</h2>
							{#if selectedTelegramCall}
								<div class="health-row"><span>Selected call</span><strong>{selectedTelegramCall.call_id}</strong></div>
							{/if}
							{#if callTranscript}
								<div class="evidence-row">
									<strong>{callTranscript.transcript_status} · {callTranscript.stt_provider}</strong>
									<p>{callTranscript.transcript_text}</p>
								</div>
							{:else}
								<p>No transcript for selected call.</p>
							{/if}
							<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void saveCallTranscriptFixtureFromUi(); }}>
								<label><span>Transcript ID</span><input bind:value={transcriptForm.transcript_id} autocomplete="off" /></label>
								<label><span>Audio ref</span><input bind:value={transcriptForm.source_audio_ref} autocomplete="off" /></label>
								<label><span>Language</span><input bind:value={transcriptForm.language_code} autocomplete="off" /></label>
								<label class="checkbox-row"><input bind:checked={transcriptForm.always_on_policy} type="checkbox" /><span>Always-on policy</span></label>
								<div class="form-actions wide"><button type="submit" disabled={isTelegramActionSubmitting || !selectedTelegramCallId}>Save Transcript</button></div>
							</form>
							</section>
						</div>
					</aside>
				</div>

				{#if isComposeOpen}
					<button type="button" class="drawer-backdrop" onclick={() => (isComposeOpen = false)} aria-label="Close compose"></button>
					<aside class="account-drawer"  aria-label="Compose email">
						<header>
							<div><p>{_('Compose')}</p><h2>New Message</h2></div>
							<button type="button" class="icon-button" onclick={() => (isComposeOpen = false)} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
						</header>
						<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void handleSaveDraft(); }}>
							<label><span>To</span><input bind:value={composeForm.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
							<label><span>CC</span><input bind:value={composeForm.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
							<label><span>Subject</span><input bind:value={composeForm.subject} placeholder="Email subject" autocomplete="off" /></label>
							<label class="wide"><span>Body</span><textarea bind:value={composeForm.body} rows="8" placeholder="Write your message..."></textarea></label>
							<div class="form-actions wide">
								<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
								<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if drafts.length > 0}
					<div class="draft-strip">
						<strong>Drafts ({drafts.length})</strong>
						{#each drafts.slice(0, 3) as draft}
							<button type="button" class="draft-chip" onclick={() => { composeForm = { draft_id: draft.draft_id, account_id: draft.account_id, to_text: draft.to_recipients.join(', '), cc_text: draft.cc_recipients.join(', '), subject: draft.subject, body: draft.body_text }; isComposeOpen = true; }}>
								<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
							</button>
						{/each}
					</div>
				{/if}

				{#if mailboxHealth}
					<div class="health-strip">
						<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{mailboxHealth.needs_action} need action</span>
						<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{mailboxHealth.waiting} waiting</span>
						<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{mailboxHealth.done} done</span>
						<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{mailboxHealth.total_messages} total</span>
						{#if mailboxHealth.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{mailboxHealth.important} important</span>{/if}
					</div>
				{/if}
			</section>
		{:else if currentView === 'communications' && activeCommunicationSection === 'whatsapp'}
			<section class="whatsapp-page communications-page">
				<div class="view-header">
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:brand-whatsapp" width="28" height="28" /></span><div><h1>{_(activeView.title)}</h1><p>{_(activeView.subtitle)}</p></div></div>
					<button type="button" class="primary-button" onclick={() => openAccountDrawer('whatsapp')}><Icon icon="tabler:plus" width="16" height="16" />Add Account</button>
					<button type="button" class="primary-button" onclick={() => void loadWhatsappWebWorkspace()} disabled={isWhatsappLoading}><Icon icon="tabler:refresh" width="16" height="16" />Refresh</button>
				</div>

				<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="whatsapp-session-status" data-widget-hidden={!isWidgetVisible('whatsapp-session-status')}>
					{@render widgetEditChrome('whatsapp-session-status')}
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
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="whatsapp-account-session-metadata" data-widget-hidden={!isWidgetVisible('whatsapp-account-session-metadata')}>
						{@render widgetEditChrome('whatsapp-account-session-metadata')}
						<section class="panel conversation-list">
							<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder="Search WhatsApp sessions..." /></label>
							{#if isWhatsappLoading && whatsappSessions.length === 0}
								<div class="empty-panel">Loading WhatsApp Web state...</div>
							{:else if whatsappSessions.length === 0}
								<div class="empty-panel">No WhatsApp Web sessions saved yet.</div>
							{:else}
								{#each whatsappSessions as session}
									<button type="button" class:active={selectedWhatsappSession?.session_id === session.session_id} onclick={() => selectWhatsappSession(session)}>
										<span class="round-icon cyan"><Icon icon="tabler:brand-whatsapp" width="22" height="22" /></span>
										<img src="/assets/hermes-reference-avatar.png" alt="" />
										<span>
											<strong>{session.device_name}</strong>
											<small>{session.account_id} · {session.companion_runtime}</small>
											<em>{session.link_state}</em>
										</span>
										<time>{formatDateTime(session.last_sync_at ?? session.updated_at)}</time>
									</button>
								{/each}
							{/if}
						</section>
					</div>

					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="whatsapp-chat-message-surface" data-widget-hidden={!isWidgetVisible('whatsapp-chat-message-surface')}>
						{@render widgetEditChrome('whatsapp-chat-message-surface')}
						<section class="panel chat-pane whatsapp-chat-pane">
							{#if selectedWhatsappSession}
								<header>
									<span class="round-icon cyan"><Icon icon="tabler:brand-whatsapp" width="24" height="24" /></span>
									<div><h2>{selectedWhatsappSession.device_name}</h2><p>{selectedWhatsappSession.account_id} · {selectedWhatsappSession.link_state}</p></div>
									<div class="chat-actions">
										<button type="button" disabled title="Live WhatsApp Web runtime is blocked in WhatsApp foundation"><Icon icon="tabler:world" width="17" height="17" /></button>
										<button type="button" disabled title="Outbound WhatsApp sends require a future policy and runtime contract"><Icon icon="tabler:send-off" width="17" height="17" /></button>
										<button type="button" onclick={() => void loadWhatsappWebWorkspace()} disabled={isWhatsappLoading}><Icon icon="tabler:refresh" width="17" height="17" /></button>
									</div>
								</header>
								<div class="chat-body">
									{#if aiAnalysisResult && aiAnalysisResult.message_id === selectedCommunication.message_id}
										<article class="ai-analysis-card">
											<strong><Icon icon="tabler:sparkles" width="16" height="16" />AI Analysis</strong>
											{#if aiAnalysisResult.category}<p><em>Category:</em> {aiAnalysisResult.category}</p>{/if}
											{#if aiAnalysisResult.summary}<p><em>Summary:</em> {aiAnalysisResult.summary}</p>{/if}
											{#if aiAnalysisResult.importance_score != null}<p><em>Importance:</em> {aiAnalysisResult.importance_score}/100</p>{/if}
											<p><em>State:</em> <span class="state-badge {aiAnalysisResult.workflow_state}">{aiAnalysisResult.workflow_state.replace('_', ' ')}</span></p>
										</article>
									{/if}
									{#if selectedWhatsappMessages.length === 0}
										<div class="empty-panel fill">No WhatsApp Web messages for this session.</div>
									{:else}
										{#each selectedWhatsappMessages.slice().reverse() as message}
											<article class="bubble" class:outbound={message.delivery_state === 'sent' || message.delivery_state === 'send_dry_run'} class:inbound={message.delivery_state !== 'sent' && message.delivery_state !== 'send_dry_run'}>
												<strong>{message.sender_display_name ?? message.sender}</strong><br />
												{message.text}
												<time>{whatsappMessageTime(message)}</time>
											</article>
										{/each}
									{/if}
								</div>
								<form class="telegram-inline-form" onsubmit={(event) => { event.preventDefault(); void ingestWhatsappWebMessageFixture(); }}>
									<input bind:value={whatsappMessageForm.provider_message_id} placeholder="Provider message ID" autocomplete="off" />
									<input bind:value={whatsappMessageForm.sender_display_name} placeholder="Sender" autocomplete="off" />
									<input bind:value={whatsappMessageForm.text} placeholder="Fixture message text" autocomplete="off" />
									<button type="submit" disabled={isWhatsappActionSubmitting || !whatsappMessageForm.text.trim()}><Icon icon="tabler:send" width="17" height="17" />Ingest</button>
								</form>
							{:else}
								<div class="empty-panel fill">Create a WhatsApp Web fixture account before ingesting messages.</div>
							{/if}
						</section>
					</div>

					<aside class="stacked-rail whatsapp-rail">
						<div class="widget-frame stacked-rail" class:editing={isLayoutEditing} data-widget-id="whatsapp-sync-controls" data-widget-hidden={!isWidgetVisible('whatsapp-sync-controls')}>
							{@render widgetEditChrome('whatsapp-sync-controls')}
							<section class="panel info-card">
								<h2>Account Setup</h2>
								<div class="setup-summary-card">
									<span class="round-icon green"><Icon icon="tabler:brand-whatsapp" width="22" height="22" /></span>
									<div>
										<strong>{whatsappProviderAccounts.length} WhatsApp accounts</strong>
										<p>{whatsappProviderAccounts.length ? 'Companion session records are available for fixture ingestion.' : 'No WhatsApp Web account record is configured yet.'}</p>
									</div>
								</div>
								<div class="form-actions wide">
									<button type="button" onclick={() => openAccountDrawer('whatsapp')} disabled={isWhatsappActionSubmitting}>Open Wizard</button>
								</div>
							</section>

							<section class="panel info-card">
								<h2>Runtime Guardrails</h2>
								<div class="health-row"><span>Mode</span><strong>{whatsappCapabilities?.runtime_mode ?? 'unknown'}</strong></div>
								{#if whatsappClosureCapabilities.length}
									<ul class="detail-list">
										{#each whatsappClosureCapabilities as capability}
											<li>{capabilityLabel(capability.capability)}<em>{capability.status}</em></li>
										{/each}
									</ul>
								{:else}
									<p>Capability contract is not loaded yet.</p>
								{/if}
								{#if whatsappBlockedCapabilities.length}
									<div class="evidence-row">
										<strong>Live Scope</strong>
										<p>{whatsappBlockedCapabilities.map((capability) => capabilityLabel(capability.capability)).join(', ')}</p>
									</div>
								{/if}
								{#if whatsappCapabilities?.unsupported_features.length}
									<div class="evidence-row">
										<strong>Unsupported</strong>
										<p>{whatsappCapabilities.unsupported_features.map(capabilityLabel).join(', ')}</p>
									</div>
								{/if}
							</section>

							<section class="panel info-card">
								<h2>Fixture Message</h2>
								<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void ingestWhatsappWebMessageFixture(); }}>
									<label><span>Account ID</span><input bind:value={whatsappMessageForm.account_id} autocomplete="off" /></label>
									<label><span>Chat ID</span><input bind:value={whatsappMessageForm.provider_chat_id} autocomplete="off" /></label>
									<label><span>Chat title</span><input bind:value={whatsappMessageForm.chat_title} autocomplete="off" /></label>
									<label><span>Sender ID</span><input bind:value={whatsappMessageForm.sender_id} autocomplete="off" /></label>
									<label><span>Sender</span><input bind:value={whatsappMessageForm.sender_display_name} autocomplete="off" /></label>
									<label class="wide"><span>Text</span><textarea bind:value={whatsappMessageForm.text} rows="3"></textarea></label>
									<div class="form-actions wide"><button type="submit" disabled={isWhatsappActionSubmitting || !whatsappMessageForm.text.trim()}>Ingest Fixture</button></div>
								</form>
							</section>
						</div>
					</aside>
				</div>

				{#if isComposeOpen}
					<button type="button" class="drawer-backdrop" onclick={() => (isComposeOpen = false)} aria-label="Close compose"></button>
					<aside class="account-drawer"  aria-label="Compose email">
						<header>
							<div><p>{_('Compose')}</p><h2>New Message</h2></div>
							<button type="button" class="icon-button" onclick={() => (isComposeOpen = false)} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
						</header>
						<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void handleSaveDraft(); }}>
							<label><span>To</span><input bind:value={composeForm.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
							<label><span>CC</span><input bind:value={composeForm.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
							<label><span>Subject</span><input bind:value={composeForm.subject} placeholder="Email subject" autocomplete="off" /></label>
							<label class="wide"><span>Body</span><textarea bind:value={composeForm.body} rows="8" placeholder="Write your message..."></textarea></label>
							<div class="form-actions wide">
								<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
								<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if drafts.length > 0}
					<div class="draft-strip">
						<strong>Drafts ({drafts.length})</strong>
						{#each drafts.slice(0, 3) as draft}
							<button type="button" class="draft-chip" onclick={() => { composeForm = { draft_id: draft.draft_id, account_id: draft.account_id, to_text: draft.to_recipients.join(', '), cc_text: draft.cc_recipients.join(', '), subject: draft.subject, body: draft.body_text }; isComposeOpen = true; }}>
								<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
							</button>
						{/each}
					</div>
				{/if}

				{#if mailboxHealth}
					<div class="health-strip">
						<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{mailboxHealth.needs_action} need action</span>
						<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{mailboxHealth.waiting} waiting</span>
						<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{mailboxHealth.done} done</span>
						<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{mailboxHealth.total_messages} total</span>
						{#if mailboxHealth.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{mailboxHealth.important} important</span>{/if}
					</div>
				{/if}
			</section>
		{:else if currentView === 'settings'}
			<section class="settings-page">
				<div class="view-header">
					<div class="view-title-with-icon">
						<span class="hero-mark small"><Icon icon="tabler:settings" width="28" height="28" /></span>
						<div><h1>{_(activeView.title)}</h1><p>{_(activeView.subtitle)}</p></div>
					</div>
					<button type="button" class="primary-button" onclick={() => void loadSettingsWorkspace()} disabled={isSettingsLoading}>
						<Icon icon="tabler:refresh" width="16" height="16" />Refresh
					</button>
				</div>

				<div class="metric-grid settings-metrics">
					<article class="metric-card"><span>{_('Settings')}</span><strong>{applicationSettings.length}</strong><small>Editable runtime values</small></article>
					<article class="metric-card"><span>Accounts</span><strong>{providerAccounts.length}</strong><small>Email, Telegram, WhatsApp</small></article>
					<article class="metric-card"><span>Mail</span><strong>{emailProviderAccounts.length}</strong><small>Gmail, iCloud, IMAP</small></article>
					<article class="metric-card"><span>Telegram</span><strong>{telegramProviderAccounts.length}</strong><small>User and bot records</small></article>
					<article class="metric-card"><span>WhatsApp</span><strong>{whatsappProviderAccounts.length}</strong><small>Web sessions</small></article>
					<article class="metric-card"><span>Secrets</span><strong>Vault</strong><small>Values stay out of settings</small></article>
				</div>

				{#if settingsActionMessage}
					<p class="setup-state success">{settingsActionMessage}</p>
				{/if}
				{#if settingsError}
					<p class="inline-error">{settingsError}</p>
				{/if}

				<div class="section-tabs settings-tabs" aria-label="Settings sections">
					<button type="button" class:active={selectedSettingsSection === 'appearance'} onclick={() => (selectedSettingsSection = 'appearance')}>
						<Icon icon="tabler:palette" width="16" height="16" />{_('Appearance')}
					</button>
					<button type="button" class:active={selectedSettingsSection === 'application'} onclick={() => (selectedSettingsSection = 'application')}>
						<Icon icon="tabler:adjustments-horizontal" width="16" height="16" />Application
					</button>
					<button type="button" class:active={selectedSettingsSection === 'sidebar'} onclick={() => (selectedSettingsSection = 'sidebar')}>
						<Icon icon="tabler:layout-sidebar" width="16" height="16" />Sidebar
					</button>
					<button type="button" class:active={selectedSettingsSection === 'accounts'} onclick={() => (selectedSettingsSection = 'accounts')}>
						<Icon icon="tabler:users" width="16" height="16" />Accounts <em>{providerAccounts.length}</em>
					</button>
					<button type="button" class:active={selectedSettingsSection === 'language'} onclick={() => (selectedSettingsSection = 'language')}>
						<Icon icon="tabler:language" width="16" height="16" />Language
					</button>
				</div>

				{#if selectedSettingsSection === 'appearance'}
					<div class="settings-layout appearance-settings-layout">
						<section class="panel settings-list-panel settings-primary-pane appearance-settings-panel">
							<header class="panel-title-row">
								<div>
									<h2>{_('Interface Appearance')}</h2>
									<p>{_('Choose shell background, brightness and application accent color.')}</p>
								</div>
								<div class="appearance-settings-actions">
									<button type="button" onclick={cancelThemeSettingsEditing} disabled={!themeSettingsHasChanges() || isThemeSettingsSaving}>
										<Icon icon="tabler:arrow-back-up" width="16" height="16" />{_('Cancel')}
									</button>
									<button type="button" onclick={resetThemeSettingsToDefault} disabled={isThemeSettingsSaving}>
										<Icon icon="tabler:restore" width="16" height="16" />{_('Default')}
									</button>
									<button type="button" class="primary-button" onclick={() => void saveThemeSettings()} disabled={!themeSettingsHasChanges() || isThemeSettingsSaving}>
										<Icon icon="tabler:device-floppy" width="16" height="16" />{isThemeSettingsSaving ? _('Saving') : _('Save')}
									</button>
								</div>
							</header>

							{#if themeError}
								<p class="inline-error">{themeError}</p>
							{/if}

							<div class="appearance-config">
								<section class="appearance-section">
									<header>
										<div>
											<h3>{_('Shell Background')}</h3>
											<p>{_('Background image for the desktop shell.')}</p>
										</div>
										<strong>{_(shellBackgroundLabel(effectiveThemeSettings.shellBackground))}</strong>
									</header>
									<div class="background-option-grid">
										{#each shellBackgroundOptions as option}
											<button
												type="button"
												class:active={effectiveThemeSettings.shellBackground === option.id}
												aria-pressed={effectiveThemeSettings.shellBackground === option.id}
												onclick={() => selectShellBackground(option.id)}
											>
												<span class={shellBackgroundPreviewClass(option.id)}></span>
												<span>{_(option.label)}</span>
											</button>
										{/each}
									</div>
								</section>

								<section class="appearance-section">
									<header>
										<div>
											<h3>{_('Background Brightness')}</h3>
											<p>{_('Controls image brightness without changing layout geometry.')}</p>
										</div>
										<strong>{effectiveThemeSettings.backgroundBrightness}%</strong>
									</header>
									<div class="brightness-control">
										<input
											type="range"
											min="30"
											max="100"
											step="10"
											value={effectiveThemeSettings.backgroundBrightness}
											list="shell-background-brightness-values"
											aria-label={_('Background Brightness')}
											oninput={updateShellBrightness}
										/>
										<datalist id="shell-background-brightness-values">
											{#each shellBackgroundBrightnessValues as value}
												<option value={value}></option>
											{/each}
										</datalist>
										<div class="brightness-scale">
											<span>30%</span>
											<span>100%</span>
										</div>
									</div>
								</section>

								<section class="appearance-section">
									<header>
										<div>
											<h3>{_('Panel Transparency')}</h3>
											<p>{_('Global opacity for widget panels and cards.')}</p>
										</div>
										<strong>{effectiveThemeSettings.panelOpacity}%</strong>
									</header>
									<div class="brightness-control">
										<input
											type="range"
											min="40"
											max="100"
											step="10"
											value={effectiveThemeSettings.panelOpacity}
											list="panel-opacity-values"
											aria-label={_('Panel Transparency')}
											oninput={updateGlobalPanelOpacity}
										/>
										<datalist id="panel-opacity-values">
											{#each panelOpacityValues as value}
												<option value={value}></option>
											{/each}
										</datalist>
										<div class="brightness-scale">
											<span>40%</span>
											<span>100%</span>
										</div>
									</div>
								</section>

								<section class="appearance-section">
									<header>
										<div>
											<h3>{_('Panel Blur')}</h3>
											<p>{_('Global backdrop blur for widget panels and cards.')}</p>
										</div>
										<strong>{effectiveThemeSettings.panelBlur}px</strong>
									</header>
									<div class="brightness-control">
										<input
											type="range"
											min="0"
											max="24"
											step="4"
											value={effectiveThemeSettings.panelBlur}
											list="panel-blur-values"
											aria-label={_('Panel Blur')}
											oninput={updateGlobalPanelBlur}
										/>
										<datalist id="panel-blur-values">
											{#each panelBlurValues as value}
												<option value={value}></option>
											{/each}
										</datalist>
										<div class="brightness-scale">
											<span>0px</span>
											<span>24px</span>
										</div>
									</div>
								</section>

								<section class="appearance-section">
									<header>
										<div>
											<h3>{_('Application Color')}</h3>
											<p>{_('Accent color for controls, borders and active states.')}</p>
										</div>
										<strong>{_(shellAccentLabel(effectiveThemeSettings.accentColor))}</strong>
									</header>
									<div class="accent-swatch-grid">
										{#each shellAccentColorOptions as option}
											<button
												type="button"
												class={shellAccentSwatchClass(option.id)}
												class:active={effectiveThemeSettings.accentColor === option.id}
												aria-pressed={effectiveThemeSettings.accentColor === option.id}
												onclick={() => selectShellAccent(option.id)}
											>
												<span class="accent-swatch-dot"></span>
												<span>{_(option.label)}</span>
											</button>
										{/each}
									</div>
								</section>
							</div>
						</section>

						<aside class="settings-rail appearance-preview-rail">
							<section class="panel info-card">
								<h2>{_('Current Appearance')}</h2>
								<div class="health-row"><span>{_('Background')}</span><strong>{_(shellBackgroundLabel(effectiveThemeSettings.shellBackground))}</strong></div>
								<div class="health-row"><span>{_('Brightness')}</span><strong>{effectiveThemeSettings.backgroundBrightness}%</strong></div>
								<div class="health-row"><span>{_('Color')}</span><strong>{_(shellAccentLabel(effectiveThemeSettings.accentColor))}</strong></div>
								<div class="health-row"><span>{_('Panel Transparency')}</span><strong>{effectiveThemeSettings.panelOpacity}%</strong></div>
								<div class="health-row"><span>{_('Panel Blur')}</span><strong>{effectiveThemeSettings.panelBlur}px</strong></div>
							</section>
							<section class="panel info-card">
								<h2>{_('Storage')}</h2>
								<ul class="detail-list">
									<li>{_('Stored as declared frontend.theme setting')}<em>JSON</em></li>
									<li>{_('No private content or secrets')}<em>ADR-0054</em></li>
									<li>{_('No inline styles')}<em>CSS</em></li>
								</ul>
							</section>
						</aside>
					</div>
				{:else if selectedSettingsSection === 'language'}
					<div class="settings-layout">
						<section class="panel settings-list-panel settings-primary-pane">
							<header class="panel-title-row">
								<div><h2>Interface Language</h2><p>Choose the display language for the Hermes Hub interface.</p></div>
							</header>
							<div class="settings-category-list">
								<div class="setting-row">
									<span>Language</span>
									<div class="setting-control">
										<select value={$currentLocale} onchange={async (event) => { const el = event.target; if (el instanceof HTMLSelectElement) { const loc = el.value as "en" | "ru"; setLocale(loc); try { await saveFrontendLocaleSetting(apiBaseUrl, apiSecret, loc); } catch (e) { settingsError = e instanceof Error ? e.message : 'Failed to save locale'; } } }}>
											<option value="en">English</option>
											<option value="ru">Русский</option>
										</select>
									</div>
								</div>
							</div>
						</section>
						<aside class="settings-rail">
							<section class="panel info-card">
								<h2>About</h2>
								<p>Language preference is stored in memory for the current session. A persistent locale setting can be added later.</p>
							</section>
						</aside>
					</div>
				{:else if selectedSettingsSection === 'application'}
					<div class="settings-layout">
						<section class="panel settings-list-panel settings-primary-pane">
								<header class="panel-title-row">
									<div><h2>Application Settings</h2><p>All non-secret settings except database connectivity; secret-like keys are rejected.</p></div>
								</header>
								{#if isSettingsLoading && applicationSettings.length === 0}
									<div class="empty-panel fill">Loading settings...</div>
								{:else if Object.entries(settingsByCategory).length === 0}
									<div class="empty-panel fill">No application settings are declared yet.</div>
								{:else}
									<div class="settings-category-list">
										{#each Object.entries(settingsByCategory) as [category, settings]}
											<section class="settings-category">
												<header>
													<h3>{settingsCategoryLabel(category)}</h3>
													<span>{settings.length}</span>
												</header>
												{#each settings as setting}
													<form class="setting-row" onsubmit={(event) => { event.preventDefault(); void saveSetting(setting); }}>
														<div class="setting-copy">
															<strong>{setting.label}</strong>
															<p>{setting.description}</p>
															<div class="setting-meta-row">
																<code>{setting.setting_key}</code>
																{#if settingMetadataFlag(setting, 'bootstrap')}
																	<em>Bootstrap</em>
																{/if}
																{#if settingMetadataFlag(setting, 'restart_required')}
																	<em>Restart</em>
																{/if}
																{#if settingMetadataText(setting, 'env_var')}
																	<em>{settingMetadataText(setting, 'env_var')}</em>
																{/if}
															</div>
														</div>
														<div class="setting-control">
															{#if settingAllowedValues(setting).length}
																<select value={settingDrafts[setting.setting_key] ?? settingDraftValue(setting)} disabled={!setting.is_editable} onchange={(event) => updateSettingDraft(setting.setting_key, inputEventValue(event))}>
																	{#each settingAllowedValues(setting) as value}
																		<option value={value}>{settingsCategoryLabel(value)}</option>
																	{/each}
																</select>
															{:else if setting.value_kind === 'boolean'}
																<label class="setting-toggle">
																	<input type="checkbox" checked={(settingDrafts[setting.setting_key] ?? settingDraftValue(setting)) === 'true'} disabled={!setting.is_editable} onchange={(event) => updateSettingDraft(setting.setting_key, checkboxEventValue(event))} />
																	<span>{(settingDrafts[setting.setting_key] ?? settingDraftValue(setting)) === 'true' ? 'Enabled' : 'Disabled'}</span>
																</label>
															{:else if setting.value_kind === 'integer'}
																<input type="number" value={settingDrafts[setting.setting_key] ?? settingDraftValue(setting)} min={String(setting.metadata.min ?? '')} max={String(setting.metadata.max ?? '')} step={String(setting.metadata.step ?? 1)} disabled={!setting.is_editable} oninput={(event) => updateSettingDraft(setting.setting_key, inputEventValue(event))} />
															{:else if setting.value_kind === 'json' || settingControl(setting) === 'textarea'}
																<textarea value={settingDrafts[setting.setting_key] ?? settingDraftValue(setting)} disabled={!setting.is_editable} rows="4" oninput={(event) => updateSettingDraft(setting.setting_key, inputEventValue(event))}></textarea>
															{:else}
																<input value={settingDrafts[setting.setting_key] ?? settingDraftValue(setting)} placeholder={String(setting.metadata.placeholder ?? '')} disabled={!setting.is_editable} oninput={(event) => updateSettingDraft(setting.setting_key, inputEventValue(event))} />
															{/if}
															<button type="submit" disabled={!setting.is_editable || savingSettingKey === setting.setting_key || !settingHasChanged(setting)}>
																{savingSettingKey === setting.setting_key ? 'Saving' : 'Save'}
															</button>
														</div>
													</form>
												{/each}
											</section>
										{/each}
									</div>
								{/if}
						</section>

						<aside class="settings-rail">
							<section class="panel info-card">
								<h2>Runtime Source</h2>
								<div class="health-row"><span>Backend bind</span><strong>{settingValueText('server.http_addr')}</strong></div>
								<div class="health-row"><span>Frontend API</span><strong>{settingValueText('frontend.api_base_url')}</strong></div>
								<div class="health-row"><span>AI URL</span><strong>{settingValueText('ai.ollama_base_url')}</strong></div>
								<div class="health-row"><span>Chat</span><strong>{settingValueText('ai.chat_model')}</strong></div>
								<div class="health-row"><span>Embedding</span><strong>{settingValueText('ai.embedding_model')}</strong></div>
							</section>
							<section class="panel info-card">
								<h2>Boundaries</h2>
								<ul class="detail-list">
									<li>PostgreSQL stores declared setting values<em>JSONB</em></li>
									<li>Database URL stays outside the panel<em>Bootstrap</em></li>
									<li>API token and vault key stay outside DB<em>Secret boundary</em></li>
									<li>Credentials stay in encrypted vault<em>No secret values</em></li>
									<li>Settings updates are audited<em>No values in audit</em></li>
								</ul>
							</section>
						</aside>
					</div>
				{:else if selectedSettingsSection === 'sidebar'}
					<div class="settings-layout sidebar-settings-layout">
						<section class="panel settings-list-panel settings-primary-pane sidebar-settings-panel">
							<header class="panel-title-row">
								<div>
									<h2>Sidebar Navigation</h2>
									<p>Configure workspace groups, order and hidden domains. Communications sources stay inside Communications.</p>
								</div>
								<div class="sidebar-settings-actions">
									<button type="button" onclick={cancelSidebarSettingsEditing} disabled={!sidebarSettingsHasChanges() || isSidebarSettingsSaving}>
										<Icon icon="tabler:arrow-back-up" width="16" height="16" />Cancel
									</button>
									<button type="button" onclick={resetSidebarSettingsToDefault} disabled={isSidebarSettingsSaving}>
										<Icon icon="tabler:restore" width="16" height="16" />Default
									</button>
									<button type="button" class="primary-button" onclick={() => void saveSidebarSettings()} disabled={!sidebarSettingsHasChanges() || isSidebarSettingsSaving}>
										<Icon icon="tabler:device-floppy" width="16" height="16" />{isSidebarSettingsSaving ? 'Saving' : 'Save'}
									</button>
								</div>
							</header>

							{#if sidebarError}
								<p class="inline-error">{sidebarError}</p>
							{/if}

							<form class="sidebar-group-create" onsubmit={(event) => { event.preventDefault(); addSidebarGroup(); }}>
								<label>
									<span>New group</span>
									<input bind:value={newSidebarGroupLabel} placeholder="Focus, Library, Planning" autocomplete="off" />
								</label>
								<button type="submit">
									<Icon icon="tabler:plus" width="16" height="16" />Create Group
								</button>
							</form>

							<div class="sidebar-config-list">
								<section class="sidebar-config-group">
									<header>
										<label>
											<span>Root level</span>
											<input value="Sidebar root" disabled autocomplete="off" />
										</label>
									</header>

									<div class="sidebar-config-items">
										{#each effectiveSidebarSettings.rootItemIds as rootId, rootIndex}
											{@const groupId = sidebarGroupIdFromRootId(rootId)}
											{#if groupId}
												{@const group = effectiveSidebarSettings.groups.find((item) => item.id === groupId)}
												{#if group}
													<div class="sidebar-config-item group-node">
														<div class="sidebar-config-item-main">
															<span class="round-icon green"><Icon icon={group.icon} width="18" height="18" /></span>
															<div>
																<strong>{_(sidebarGroupLabel(group, rootIndex))}</strong>
																<small>Expandable group · {group.itemIds.length} items</small>
															</div>
														</div>
														<div class="sidebar-config-item-controls">
															<button type="button" aria-label={`Move ${_(sidebarGroupLabel(group, rootIndex))} up`} title="Move group up" onclick={() => moveSidebarGroup(group.id, -1)} disabled={rootIndex === 0}>
																<Icon icon="tabler:arrow-up" width="16" height="16" />
															</button>
															<button type="button" aria-label={`Move ${_(sidebarGroupLabel(group, rootIndex))} down`} title="Move group down" onclick={() => moveSidebarGroup(group.id, 1)} disabled={rootIndex === effectiveSidebarSettings.rootItemIds.length - 1}>
																<Icon icon="tabler:arrow-down" width="16" height="16" />
															</button>
															<button type="button" aria-label={`Remove ${_(sidebarGroupLabel(group, rootIndex))} group`} title="Remove group" onclick={() => removeSidebarGroup(group.id)} disabled={group.id === 'communications'}>
																<Icon icon="tabler:trash" width="16" height="16" />
															</button>
														</div>
													</div>
												{/if}
											{:else}
												{@const item = sidebarConfigItem(rootId as SidebarItemId)}
												{#if item}
													{@const isHidden = effectiveSidebarSettings.hiddenItemIds.includes(item.id)}
													<div class="sidebar-config-item" class:hidden={isHidden}>
														<div class="sidebar-config-item-main">
															<span class="round-icon cyan"><Icon icon={item.icon} width="18" height="18" /></span>
															<div>
																<strong>{item.label}</strong>
																<small>{isHidden ? 'Hidden from sidebar' : 'Root domain'}</small>
															</div>
														</div>
														<div class="sidebar-config-item-controls">
															<select aria-label={`Move ${item.label}`} value="root" onchange={(event) => moveSidebarItemToGroup(item.id, inputEventValue(event))}>
																<option value="root">Root level</option>
																{#each effectiveSidebarSettings.groups as targetGroup, targetIndex}
																	<option value={targetGroup.id}>{_(sidebarGroupLabel(targetGroup, targetIndex))}</option>
																{/each}
															</select>
															<button type="button" aria-label={`Move ${item.label} up`} title="Move item up" onclick={() => moveSidebarRootItem(rootId, -1)} disabled={rootIndex === 0}>
																<Icon icon="tabler:arrow-up" width="16" height="16" />
															</button>
															<button type="button" aria-label={`Move ${item.label} down`} title="Move item down" onclick={() => moveSidebarRootItem(rootId, 1)} disabled={rootIndex === effectiveSidebarSettings.rootItemIds.length - 1}>
																<Icon icon="tabler:arrow-down" width="16" height="16" />
															</button>
															<button type="button" class:active={!isHidden} aria-pressed={!isHidden} onclick={() => toggleSidebarItemHidden(item.id)}>
																<Icon icon={isHidden ? 'tabler:eye' : 'tabler:eye-off'} width="16" height="16" />{isHidden ? 'Show' : 'Hide'}
															</button>
														</div>
													</div>
												{/if}
											{/if}
										{/each}
									</div>
								</section>

								{#each effectiveSidebarSettings.groups as group, groupIndex}
									<section class="sidebar-config-group">
										<header>
											<label>
												<span>Group label</span>
												<input
													value={group.label}
													placeholder={groupIndex === 0 ? 'Primary' : `Group ${groupIndex + 1}`}
													autocomplete="off"
													oninput={(event) => updateSidebarGroupLabel(group.id, inputEventValue(event))}
												/>
											</label>
											<div class="sidebar-config-group-actions">
												<button type="button" aria-label={`Move ${_(sidebarGroupLabel(group, groupIndex))} group up`} title="Move group up" onclick={() => moveSidebarGroup(group.id, -1)} disabled={sidebarRootIndexForGroup(group.id) <= 0}>
													<Icon icon="tabler:arrow-up" width="16" height="16" />
												</button>
												<button type="button" aria-label={`Move ${_(sidebarGroupLabel(group, groupIndex))} group down`} title="Move group down" onclick={() => moveSidebarGroup(group.id, 1)} disabled={sidebarRootIndexForGroup(group.id) === effectiveSidebarSettings.rootItemIds.length - 1}>
													<Icon icon="tabler:arrow-down" width="16" height="16" />
												</button>
												<button type="button" aria-label={`Remove ${_(sidebarGroupLabel(group, groupIndex))} group`} title="Remove group" onclick={() => removeSidebarGroup(group.id)} disabled={group.id === 'communications'}>
													<Icon icon="tabler:trash" width="16" height="16" />
												</button>
											</div>
										</header>

										<div class="sidebar-config-items">
											{#if group.itemIds.length === 0}
												<div class="empty-panel">No items in this group.</div>
											{:else}
												{#each group.itemIds as itemId, itemIndex}
													{@const item = sidebarConfigItem(itemId)}
													{#if item}
														{@const isHidden = effectiveSidebarSettings.hiddenItemIds.includes(item.id)}
														{@const hasSeparator = sidebarGroupHasSeparatorBefore(group, item.id)}
														<div class="sidebar-config-item" class:hidden={isHidden}>
															<div class="sidebar-config-item-main">
																<span class="round-icon cyan"><Icon icon={item.icon} width="18" height="18" /></span>
																<div>
																	<strong>{item.label}</strong>
																	<small>{isHidden ? 'Hidden from sidebar' : 'Visible domain'}</small>
																</div>
															</div>
															<div class="sidebar-config-item-controls">
																<select aria-label={`Move ${item.label} to group`} value={group.id} onchange={(event) => moveSidebarItemToGroup(item.id, inputEventValue(event))}>
																	{#if !parseCommunicationSidebarItemId(item.id)}
																		<option value="root">Root level</option>
																	{/if}
																	{#each effectiveSidebarSettings.groups as targetGroup, targetIndex}
																		<option value={targetGroup.id}>{_(sidebarGroupLabel(targetGroup, targetIndex))}</option>
																	{/each}
																</select>
																<button type="button" aria-label={`Move ${item.label} up`} title="Move item up" onclick={() => moveSidebarItem(item.id, -1)}>
																	<Icon icon="tabler:arrow-up" width="16" height="16" />
																</button>
																<button type="button" aria-label={`Move ${item.label} down`} title="Move item down" onclick={() => moveSidebarItem(item.id, 1)}>
																	<Icon icon="tabler:arrow-down" width="16" height="16" />
																</button>
																<button
																	type="button"
																	class="separator-toggle"
																	class:active={hasSeparator}
																	aria-pressed={hasSeparator}
																	aria-label={hasSeparator ? `Remove divider before ${item.label}` : `Add divider before ${item.label}`}
																	title={hasSeparator ? `Remove divider before ${item.label}` : `Add divider before ${item.label}`}
																	onclick={() => toggleSidebarGroupSeparator(group.id, item.id)}
																	disabled={itemIndex === 0}
																>
																	<Icon icon="tabler:separator-horizontal" width="16" height="16" />Divider
																</button>
																<button type="button" class:active={!isHidden} aria-pressed={!isHidden} onclick={() => toggleSidebarItemHidden(item.id)}>
																	<Icon icon={isHidden ? 'tabler:eye' : 'tabler:eye-off'} width="16" height="16" />{isHidden ? 'Show' : 'Hide'}
																</button>
															</div>
														</div>
													{/if}
												{/each}
											{/if}
										</div>
									</section>
								{/each}
							</div>
						</section>

						<aside class="settings-rail sidebar-settings-summary">
							<section class="panel info-card">
								<h2>Preview</h2>
								<ul class="sidebar-preview-list">
									{#each sidebarRootEntries as entry, entryIndex}
										<li>
											{#if entry.kind === 'group'}
												<strong>{_(sidebarGroupLabel(entry.group, entryIndex))}</strong>
												<span>{entry.group.items.map((item) => sidebarItemLabel(item)).join(', ') || 'Empty group'}</span>
											{:else}
												<strong>{_(sidebarItemLabel(entry.item))}</strong>
												<span>Root domain</span>
											{/if}
										</li>
									{/each}
								</ul>
							</section>

							<section class="panel info-card">
								<h2>Hidden</h2>
								{#if sidebarHiddenNavItems.length === 0}
									<p>No domains are hidden.</p>
								{:else}
									<ul class="detail-list">
										{#each sidebarHiddenNavItems as item}
											<li>{item.label}<button type="button" onclick={() => toggleSidebarItemHidden(item.id)}>Show</button></li>
										{/each}
									</ul>
								{/if}
							</section>

							<section class="panel info-card">
								<h2>Rules</h2>
								<ul class="detail-list">
									<li>Default keeps the current sidebar order<em>Preset</em></li>
									<li>Communications sources stay nested<em>Context</em></li>
									<li>Hidden domains stay recoverable here<em>Safe</em></li>
									<li>Settings store no message content<em>Privacy</em></li>
								</ul>
							</section>
						</aside>
					</div>
				{:else}
					<div class="settings-account-layout">
						<section class="panel account-section">
								<header class="panel-title-row">
									<div><h2>Mail Accounts</h2><p>Gmail OAuth, iCloud app-password and generic IMAP records.</p></div>
									<button type="button" class="primary-button" onclick={() => openAccountDrawer('mail')}><Icon icon="tabler:plus" width="16" height="16" />Add Mail</button>
								</header>
								<div class="account-card-grid">
									{#if emailProviderAccounts.length === 0}
										<div class="empty-panel fill">No mail accounts configured.</div>
									{:else}
										{#each emailProviderAccounts as account}
											<article class="account-card">
												<span class="round-icon cyan"><Icon icon={accountProviderIcon(account.provider_kind)} width="22" height="22" /></span>
												<div>
													<strong>{account.display_name}</strong>
													<p>{account.external_account_id || account.account_id}</p>
													<small>{accountProviderLabel(account.provider_kind)} · updated {accountUpdatedLabel(account)}</small>
												</div>
												<code>{account.account_id}</code>
											</article>
										{/each}
									{/if}
								</div>
						</section>

						<section class="panel account-section">
								<header class="panel-title-row">
									<div><h2>Calendar Accounts</h2><p>Local and external calendar metadata accounts.</p></div>
									<button type="button" class="primary-button" onclick={() => openAccountDrawer('calendar')}><Icon icon="tabler:calendar-plus" width="16" height="16" />Add Calendar</button>
								</header>
								<div class="account-card-grid">
									{#if calendarAccounts.length === 0}
										<div class="empty-panel fill">No calendar accounts configured.</div>
									{:else}
										{#each calendarAccounts as account}
											<article class="account-card">
												<span class="round-icon green"><Icon icon="tabler:calendar" width="22" height="22" /></span>
												<div>
													<strong>{account.account_name}</strong>
													<p>{account.email || account.account_id}</p>
													<small>{account.provider} · updated {formatDateTime(account.updated_at)}</small>
												</div>
												<code>{account.account_id}</code>
											</article>
										{/each}
									{/if}
								</div>
						</section>

						<section class="panel account-section">
								<header class="panel-title-row">
									<div><h2>Telegram Accounts</h2><p>User and bot accounts used by Telegram ingestion and automation policies.</p></div>
									<button type="button" class="primary-button" onclick={() => openAccountDrawer('telegram')}><Icon icon="tabler:brand-telegram" width="16" height="16" />Add Telegram</button>
								</header>
								<div class="account-card-grid">
									{#if telegramProviderAccounts.length === 0}
										<div class="empty-panel fill">No Telegram accounts configured.</div>
									{:else}
										{#each telegramProviderAccounts as account}
											<article class="account-card">
												<span class="round-icon purple"><Icon icon={accountProviderIcon(account.provider_kind)} width="22" height="22" /></span>
												<div>
													<strong>{account.display_name}</strong>
													<p>{account.external_account_id || account.account_id}</p>
													<small>{accountProviderLabel(account.provider_kind)} · updated {accountUpdatedLabel(account)}</small>
												</div>
												<code>{account.account_id}</code>
											</article>
										{/each}
									{/if}
								</div>
						</section>

						<section class="panel account-section">
								<header class="panel-title-row">
									<div><h2>Other Provider Accounts</h2><p>WhatsApp Web and future communication providers.</p></div>
									<button type="button" class="primary-button" onclick={() => openAccountDrawer('whatsapp')}><Icon icon="tabler:brand-whatsapp" width="16" height="16" />Add WhatsApp</button>
								</header>
								<div class="account-card-grid">
									{#if whatsappProviderAccounts.length === 0}
										<div class="empty-panel fill">No WhatsApp Web accounts configured.</div>
									{:else}
										{#each whatsappProviderAccounts as account}
											<article class="account-card">
												<span class="round-icon green"><Icon icon={accountProviderIcon(account.provider_kind)} width="22" height="22" /></span>
												<div>
													<strong>{account.display_name}</strong>
													<p>{account.external_account_id || account.account_id}</p>
													<small>{accountProviderLabel(account.provider_kind)} · updated {accountUpdatedLabel(account)}</small>
												</div>
												<code>{account.account_id}</code>
											</article>
										{/each}
									{/if}
								</div>
						</section>
					</div>
				{/if}
			</section>
		{:else if currentView === 'agents'}
			<section class="agents-page">
				<div class="view-header">
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:robot" width="28" height="28" /></span><div><h1>{_(activeView.title)}</h1><p>{_(activeView.subtitle)}</p></div></div>
					<button type="button" class="primary-button" onclick={() => void loadAiWorkspace()} disabled={isAiLoading}><Icon icon="tabler:refresh" width="16" height="16" />Refresh</button>
				</div>
				<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-runtime-metrics" data-widget-hidden={!isWidgetVisible('ai-runtime-metrics')}>
					{@render widgetEditChrome('ai-runtime-metrics')}
					<div class="metric-grid agent-metrics">
						<article class="metric-card"><span>Runtime</span><strong>{aiRuntimeSummary()}</strong><small>{aiStatus?.version ? `Ollama ${aiStatus.version}` : 'Ollama'}</small></article>
						<article class="metric-card"><span>Agents</span><strong>{aiAgents.length}</strong><small>{aiAgents.length ? 'Registered' : 'Not loaded'}</small></article>
						<article class="metric-card"><span>Run History</span><strong>{aiRuns.length}</strong><small>Persisted runs</small></article>
						<article class="metric-card"><span>Embedding</span><strong>{aiStatus?.embedding_dimension ?? 0}</strong><small>{aiStatus?.embedding_model ?? 'No model'}</small></article>
						<article class="metric-card"><span>Suggested Tasks</span><strong>{suggestedTaskCandidates.length}</strong><small>Review queue</small></article>
						<article class="metric-card"><span>Latest Duration</span><strong>{formatDuration(aiRuns[0]?.duration_ms)}</strong><small>{aiRuns[0]?.agent_id ?? 'No runs'}</small></article>
					</div>
				</div>
				{#if aiError}
					<p class="inline-error">{aiError}</p>
				{/if}
				<div class="filter-bar"><button type="button" class="active">Local Agents</button><button type="button" disabled>{aiModelSummary()}</button><button type="button" disabled>{aiStatus?.chat_model_available ? 'Chat model ready' : 'Chat model missing'}</button><button type="button" disabled>{aiStatus?.embedding_model_available ? 'Embedding ready' : 'Embedding missing'}</button></div>
				<div class="agents-layout">
					<section class="agent-main">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-agent-list" data-widget-hidden={!isWidgetVisible('ai-agent-list')}>
							{@render widgetEditChrome('ai-agent-list')}
							<div class="agent-grid">
								{#if isAiLoading && agentCards.length === 0}
									<div class="graph-strip-message"><span>Loading local AI agents.</span></div>
								{:else if agentCards.length === 0}
									<div class="graph-strip-message"><span>No V3 agents returned by the backend.</span></div>
								{:else}
									{#each agentCards as agent, index}
										<button type="button" class="agent-card panel" class:active={selectedAgentIndex === index} onclick={() => (selectedAgentIndex = index)}>
											<span class="round-icon {agent.tone}"><Icon icon={agent.icon} width="22" height="22" /></span>
											<div><strong>{agent.name}</strong><p>{agent.summary}</p><em>{agent.status}</em></div>
											<footer><span>{agent.tasks} runs</span><span>{agent.success} success</span></footer>
										</button>
									{/each}
								{/if}
							</div>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-selected-agent-detail" data-widget-hidden={!isWidgetVisible('ai-selected-agent-detail')}>
							{@render widgetEditChrome('ai-selected-agent-detail')}
							<section class="panel agent-detail">
								{#if selectedAgent}
									<header><span class="round-icon {selectedAgent.tone}"><Icon icon={selectedAgent.icon} width="26" height="26" /></span><div><h2>{selectedAgent.name}</h2><em>{selectedAgent.model}</em></div></header>
									<div class="section-tabs"><button type="button" class="active">Overview</button><button type="button" disabled>Run History</button><button type="button" disabled>Citations</button><button type="button" disabled>Settings</button></div>
									<div class="agent-detail-grid"><p>{selectedAgent.summary}. This V3 agent reads local memory projections, retrieves citations and records every run in the backend.</p><div class="spark-chart"></div><ul>{#each ['Ollama Runtime','pgvector Retrieval','Source Citations','Run Provenance','Review Queue'] as capability}<li><Icon icon="tabler:circle-check" width="16" height="16" />{capability}</li>{/each}</ul></div>
								{:else}
									<header><span class="round-icon cyan"><Icon icon="tabler:robot-off" width="26" height="26" /></span><div><h2>No agent selected</h2><em>Backend status required</em></div></header>
								{/if}
								<div class="ai-workflow-grid">
									<form class="ai-workflow-block" onsubmit={(event) => { event.preventDefault(); void submitAiAnswer(); }}>
										<label><span>Ask AI</span><textarea bind:value={aiQuestion} rows="4"></textarea></label>
										<button type="submit" disabled={isAiAnswerSubmitting || !aiQuestion.trim()}><Icon icon="tabler:sparkles" width="16" height="16" />Ask</button>
									</form>
									<form class="ai-workflow-block" onsubmit={(event) => { event.preventDefault(); void prepareAiBrief(); }}>
										<label><span>Prepare brief</span><textarea bind:value={aiMeetingTopic} rows="4"></textarea></label>
										<button type="submit" disabled={isAiMeetingPrepSubmitting || !aiMeetingTopic.trim()}><Icon icon="tabler:calendar-stats" width="16" height="16" />Prepare</button>
									</form>
									<form class="ai-workflow-block" onsubmit={(event) => { event.preventDefault(); void refreshTasksFromAi(); }}>
										<label><span>Task extraction</span><textarea bind:value={aiTaskQuery} rows="4"></textarea></label>
										<button type="submit" disabled={isAiTaskRefreshSubmitting || !aiTaskQuery.trim()}><Icon icon="tabler:checkbox" width="16" height="16" />Refresh candidates</button>
									</form>
								</div>
								{#if aiAnswerResult}
									<div class="ai-result-block">
										<h3>Answer</h3>
										<p>{aiAnswerResult.answer}</p>
										<div class="citation-list">
											{#each aiAnswerResult.citations as citation}
												<div class="citation-row"><strong>{citation.title}</strong><span>{citation.source_kind}:{citation.source_id}</span><p>{citation.excerpt}</p></div>
											{/each}
										</div>
									</div>
								{/if}
								{#if aiMeetingPrepResult}
									<div class="ai-result-block">
										<h3>Meeting Brief</h3>
										<p>{aiMeetingPrepResult.briefing}</p>
										<div class="citation-list">
											{#each aiMeetingPrepResult.citations as citation}
												<div class="citation-row"><strong>{citation.title}</strong><span>{citation.source_kind}:{citation.source_id}</span><p>{citation.excerpt}</p></div>
											{/each}
										</div>
									</div>
								{/if}
								{#if aiTaskRefreshResult}
									<div class="ai-result-block">
										<h3>Task Candidates</h3>
										<p>{aiTaskRefreshResult.created_count} suggested candidates refreshed. Review them in Tasks.</p>
									</div>
								{/if}
							</section>
						</div>
					</section>
					<aside class="stacked-rail">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-runtime-status" data-widget-hidden={!isWidgetVisible('ai-runtime-status')}>
							{@render widgetEditChrome('ai-runtime-status')}
							<section class="panel info-card"><h2>Runtime</h2><div class="health-row"><span>Status</span><strong>{aiRuntimeSummary()}</strong></div><div class="health-row"><span>Chat</span><strong>{aiStatus?.chat_model ?? 'unknown'}</strong></div><div class="health-row"><span>Embedding</span><strong>{aiStatus?.embedding_model ?? 'unknown'}</strong></div></section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-run-history" data-widget-hidden={!isWidgetVisible('ai-run-history')}>
							{@render widgetEditChrome('ai-run-history')}
							<section class="panel info-card"><h2>Run History</h2>{#if aiRuns.length}{#each aiRuns.slice(0,6) as run}<div class="deadline"><span>{run.agent_id} · {runStatusLabel(run)}</span><time>{formatDateTime(run.started_at)} · {formatDuration(run.duration_ms)}</time></div>{/each}{:else}<p>No AI runs persisted yet.</p>{/if}</section>
						</div>
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="ai-citations" data-widget-hidden={!isWidgetVisible('ai-citations')}>
							{@render widgetEditChrome('ai-citations')}
							<section class="panel info-card"><h2>Latest Citations</h2>{#if aiRuns[0] && safeCitations(aiRuns[0].citations).length}{#each safeCitations(aiRuns[0].citations).slice(0,3) as citation}<div class="evidence-row"><strong>{citation.title}</strong><p>{citation.excerpt}</p></div>{/each}{:else}<p>Citations appear after an answer or briefing run.</p>{/if}</section>
						</div>
					</aside>
				</div>

				{#if isComposeOpen}
					<button type="button" class="drawer-backdrop" onclick={() => (isComposeOpen = false)} aria-label="Close compose"></button>
					<aside class="account-drawer"  aria-label="Compose email">
						<header>
							<div><p>{_('Compose')}</p><h2>New Message</h2></div>
							<button type="button" class="icon-button" onclick={() => (isComposeOpen = false)} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
						</header>
						<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void handleSaveDraft(); }}>
							<label><span>To</span><input bind:value={composeForm.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
							<label><span>CC</span><input bind:value={composeForm.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
							<label><span>Subject</span><input bind:value={composeForm.subject} placeholder="Email subject" autocomplete="off" /></label>
							<label class="wide"><span>Body</span><textarea bind:value={composeForm.body} rows="8" placeholder="Write your message..."></textarea></label>
							<div class="form-actions wide">
								<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
								<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if drafts.length > 0}
					<div class="draft-strip">
						<strong>Drafts ({drafts.length})</strong>
						{#each drafts.slice(0, 3) as draft}
							<button type="button" class="draft-chip" onclick={() => { composeForm = { draft_id: draft.draft_id, account_id: draft.account_id, to_text: draft.to_recipients.join(', '), cc_text: draft.cc_recipients.join(', '), subject: draft.subject, body: draft.body_text }; isComposeOpen = true; }}>
								<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
							</button>
						{/each}
					</div>
				{/if}

				{#if mailboxHealth}
					<div class="health-strip">
						<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{mailboxHealth.needs_action} need action</span>
						<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{mailboxHealth.waiting} waiting</span>
						<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{mailboxHealth.done} done</span>
						<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{mailboxHealth.total_messages} total</span>
						{#if mailboxHealth.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{mailboxHealth.important} important</span>{/if}
					</div>
				{/if}
			</section>
		{:else if currentView === 'organizations'}
			<section class="organizations-page">
				<div class="view-header"><div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:building" width="28" height="28" /></span><div><h1>Companies</h1><p>All companies and organizations from your communications</p></div></div></div>
				{#if organizationsError}
					<p class="inline-error">{organizationsError}</p>
				{/if}
				<div class="org-layout">
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="organizations-list" data-widget-hidden={!isWidgetVisible('organizations-list')}>
						{@render widgetEditChrome('organizations-list')}
						<section class="panel org-list-panel">
							<header class="panel-title-row"><h2>All Companies ({organizations.length})</h2></header>
							{#if isOrganizationsLoading && organizations.length === 0}
								<div class="graph-strip-message"><span>Loading companies.</span></div>
							{:else if organizations.length === 0}
								<div class="graph-strip-message"><span>No companies yet.</span></div>
							{:else}
								{#each organizations as org}
									<button type="button" class="org-row" class:active={selectedOrganizationId === org.organization_id} onclick={() => (selectedOrganizationId = org.organization_id)}>
										<span class="round-icon blue"><Icon icon="tabler:building" width="20" height="20" /></span>
										<div>
											<strong>{org.display_name}</strong>
											<p>{org.industry || 'Unknown industry'}{#if org.country} · {org.country}{/if}</p>
										</div>
										<small>{org.status}{#if org.watchlist} · ⚠ watchlist{/if}</small>
									</button>
								{/each}
							{/if}
						</section>
					</div>
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="organizations-detail" data-widget-hidden={!isWidgetVisible('organizations-detail')}>
						{@render widgetEditChrome('organizations-detail')}
						<section class="panel org-detail-panel">
							{#if selectedOrganization}
								<header>
									<span class="round-icon blue"><Icon icon="tabler:building" width="26" height="26" /></span>
									<div><h2>{selectedOrganization.display_name}</h2><em>{selectedOrganization.industry || 'Unknown industry'}{#if selectedOrganization.country} · {selectedOrganization.country}{/if}</em></div>
								</header>
								<div class="org-detail-grid">
									<div class="info-card"><h3>Status</h3><span class="status-chip {selectedOrganization.status}">{selectedOrganization.status}</span>{#if selectedOrganization.health_status}<span class="health-chip">{selectedOrganization.health_status}</span>{/if}{#if selectedOrganization.watchlist}<span class="health-chip important">Watchlist</span>{/if}</div>
									{#if selectedOrganization.description}
										<div class="info-card"><h3>About</h3><p>{selectedOrganization.description}</p></div>
									{/if}
									<div class="info-card"><h3>Details</h3>
										{#if selectedOrganization.website}<div class="detail-row"><span>Website</span><strong>{selectedOrganization.website}</strong></div>{/if}
										{#if selectedOrganization.legal_name}<div class="detail-row"><span>Legal name</span><strong>{selectedOrganization.legal_name}</strong></div>{/if}
										{#if selectedOrganization.registration_number}<div class="detail-row"><span>Registration</span><strong>{selectedOrganization.registration_number}</strong></div>{/if}
										{#if selectedOrganization.vat}<div class="detail-row"><span>VAT</span><strong>{selectedOrganization.vat}</strong></div>{/if}
										<div class="detail-row"><span>Interactions</span><strong>{selectedOrganization.interaction_count}</strong></div>
										<div class="detail-row"><span>Priority</span><strong>{selectedOrganization.priority || 'normal'}</strong></div>
									</div>
									{#if orgPeople.length > 0}
										<div class="info-card"><h3>Key People</h3>
											{#each orgPeople as person}
												<div class="person-mini"><span class="round-icon"><Icon icon="tabler:user" width="16" height="16" /></span><strong>{person.display_name}</strong><small>{person.email_address}</small></div>
											{/each}
										</div>
									{/if}
								</div>
							{:else}
								<header><span class="round-icon"><Icon icon="tabler:building-off" width="26" height="26" /></span><div><h2>No company selected</h2><em>Select a company from the list</em></div></header>
							{/if}
						</section>
					</div>
				</div>
			</section>

		{:else if currentView === 'timeline'}
			<section class="timeline-page">
				<div class="view-header"><div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:timeline-event" width="28" height="28" /></span><div><h1>Timeline</h1><p>Chronological activity across connected sources.</p></div></div></div>
				<div class="timeline-layout">
					<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="timeline-stream" data-widget-hidden={!isWidgetVisible('timeline-stream')}>
						{@render widgetEditChrome('timeline-stream')}
						<section class="panel feed-panel large-timeline">
							<header class="panel-title-row"><h2>Today</h2><button type="button" class="ghost-button" disabled>All Events</button></header>
							{#each communicationMessages.slice(0, 20) as msg, index}
						<article class="timeline-event-row">
							<span class="rail-dot"></span>
							<span class="round-icon blue"><Icon icon="tabler:message" width="20" height="20" /></span>
							<div>
								<strong>{msg.sender_display_name || msg.sender || 'Unknown'}</strong>
								<p>{msg.subject || msg.body_text_preview}</p>
								<time>{msg.occurred_at || msg.projected_at}</time>
							</div>
						</article>
						{/each}
						</section>
					</div>
					<aside class="stacked-rail">
						<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="timeline-filters" data-widget-hidden={!isWidgetVisible('timeline-filters')}>
							{@render widgetEditChrome('timeline-filters')}
							<section class="panel info-card"><h2>Timeline Filters</h2>{#each ['Messages','Documents','Tasks','Calendar','Notes','Decisions'] as item}<label class="mini-check"><input type="checkbox" checked />{item}</label>{/each}</section>
						</div>
					</aside>
				</div>

				{#if isComposeOpen}
					<button type="button" class="drawer-backdrop" onclick={() => (isComposeOpen = false)} aria-label="Close compose"></button>
					<aside class="account-drawer"  aria-label="Compose email">
						<header>
							<div><p>{_('Compose')}</p><h2>New Message</h2></div>
							<button type="button" class="icon-button" onclick={() => (isComposeOpen = false)} aria-label="Close"><Icon icon="tabler:x" width="18" height="18" /></button>
						</header>
						<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void handleSaveDraft(); }}>
							<label><span>To</span><input bind:value={composeForm.to_text} placeholder="recipient@example.com" autocomplete="off" /></label>
							<label><span>CC</span><input bind:value={composeForm.cc_text} placeholder="cc@example.com" autocomplete="off" /></label>
							<label><span>Subject</span><input bind:value={composeForm.subject} placeholder="Email subject" autocomplete="off" /></label>
							<label class="wide"><span>Body</span><textarea bind:value={composeForm.body} rows="8" placeholder="Write your message..."></textarea></label>
							<div class="form-actions wide">
								<button type="submit" class="primary-button"><Icon icon="tabler:device-floppy" width="16" height="16" />Save Draft</button>
								<button type="button" disabled><Icon icon="tabler:send" width="16" height="16" />Send</button>
							</div>
						</form>
					</aside>
				{/if}

				{#if drafts.length > 0}
					<div class="draft-strip">
						<strong>Drafts ({drafts.length})</strong>
						{#each drafts.slice(0, 3) as draft}
							<button type="button" class="draft-chip" onclick={() => { composeForm = { draft_id: draft.draft_id, account_id: draft.account_id, to_text: draft.to_recipients.join(', '), cc_text: draft.cc_recipients.join(', '), subject: draft.subject, body: draft.body_text }; isComposeOpen = true; }}>
								<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
							</button>
						{/each}
					</div>
				{/if}

				{#if mailboxHealth}
					<div class="health-strip">
						<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{mailboxHealth.needs_action} need action</span>
						<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{mailboxHealth.waiting} waiting</span>
						<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{mailboxHealth.done} done</span>
						<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{mailboxHealth.total_messages} total</span>
						{#if mailboxHealth.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{mailboxHealth.important} important</span>{/if}
					</div>
				{/if}
			</section>
	{/if}
		<div class="workspace-status-strip" aria-live="polite">
			<strong>{_(activeView.title)}</strong>
			<span>
				{#if hiddenWidgetTitles.length === 0 && viewportHiddenWidgetTitles.length === 0}
					No hidden widgets
				{:else}
					Hidden: {[...hiddenWidgetTitles, ...viewportHiddenWidgetTitles].join(', ')}
				{/if}
			</span>
		</div>
	</section>

	{#if isLayoutEditing && selectedLayoutWidget}
		<div class="layout-widget-drawer" role="dialog" aria-label={_('Widget settings')}>
			<header>
				<div>
					<p>{_('Widget settings')}</p>
					<h2>{_(selectedLayoutWidget.definition.title)}</h2>
				</div>
				<button
					type="button"
					class="icon-button"
					onclick={closeWidgetSettingsDrawer}
					title={_('Close widget settings')}
					aria-label={_('Close widget settings')}
				>
					<Icon icon="tabler:x" width="16" height="16" />
				</button>
			</header>

			<div class="layout-widget-meta">
				<span>{_('Widget ID')}<strong>{selectedLayoutWidget.widgetId}</strong></span>
				<span>{_('Zone')}<strong>{_(widgetZoneTitle(selectedLayoutWidget.zoneId))}</strong></span>
			</div>

			<section class="layout-widget-size-panel" aria-label={_('Widget grid size')}>
				<div class="layout-widget-grid-row">
					<div class="widget-grid-field">
						<span>{_('Cols')}</span>
						<div class="widget-grid-stepper">
							<button
								type="button"
								title={_('Decrease columns')}
								aria-label={_('Decrease columns')}
								disabled={widgetGridValue(selectedLayoutWidget.widgetId, 'columns') <= widgetGridMin(selectedLayoutWidget.widgetId, 'columns')}
								onclick={() => adjustWidgetGridValue(selectedLayoutWidget.widgetId, 'columns', -1)}
							>
								<Icon icon="tabler:minus" width="12" height="12" />
							</button>
							<input
								type="number"
								min={widgetGridMin(selectedLayoutWidget.widgetId, 'columns')}
								max={widgetGridMax('columns')}
								value={widgetGridValue(selectedLayoutWidget.widgetId, 'columns')}
								aria-label={_('Widget columns')}
								onchange={(event) => handleWidgetGridInput(selectedLayoutWidget.widgetId, 'columns', event)}
							/>
							<button
								type="button"
								title={_('Increase columns')}
								aria-label={_('Increase columns')}
								disabled={widgetGridValue(selectedLayoutWidget.widgetId, 'columns') >= widgetGridMax('columns')}
								onclick={() => adjustWidgetGridValue(selectedLayoutWidget.widgetId, 'columns', 1)}
							>
								<Icon icon="tabler:plus" width="12" height="12" />
							</button>
						</div>
						<small>{widgetGridMin(selectedLayoutWidget.widgetId, 'columns')} - {widgetGridMax('columns')}</small>
					</div>

					<div class="widget-grid-field">
						<span>{_('Rows')}</span>
						<div class="widget-grid-stepper">
							<button
								type="button"
								title={_('Decrease rows')}
								aria-label={_('Decrease rows')}
								disabled={widgetGridValue(selectedLayoutWidget.widgetId, 'rows') <= widgetGridMin(selectedLayoutWidget.widgetId, 'rows')}
								onclick={() => adjustWidgetGridValue(selectedLayoutWidget.widgetId, 'rows', -1)}
							>
								<Icon icon="tabler:minus" width="12" height="12" />
							</button>
							<input
								type="number"
								min={widgetGridMin(selectedLayoutWidget.widgetId, 'rows')}
								max={widgetGridMax('rows')}
								value={widgetGridValue(selectedLayoutWidget.widgetId, 'rows')}
								aria-label={_('Widget rows')}
								onchange={(event) => handleWidgetGridInput(selectedLayoutWidget.widgetId, 'rows', event)}
							/>
							<button
								type="button"
								title={_('Increase rows')}
								aria-label={_('Increase rows')}
								disabled={widgetGridValue(selectedLayoutWidget.widgetId, 'rows') >= widgetGridMax('rows')}
								onclick={() => adjustWidgetGridValue(selectedLayoutWidget.widgetId, 'rows', 1)}
							>
								<Icon icon="tabler:plus" width="12" height="12" />
							</button>
						</div>
						<small>{widgetGridMin(selectedLayoutWidget.widgetId, 'rows')} - {widgetGridMax('rows')}</small>
					</div>
				</div>
			</section>

			<section class="layout-widget-surface-panel" aria-label={_('Widget panel surface')}>
				<header>
					<div>
						<h3>{_('Panel Surface')}</h3>
						<p>{_('Override transparency and blur for this widget only.')}</p>
					</div>
					<button
						type="button"
						onclick={() => resetWidgetPanelSurface(selectedLayoutWidget.widgetId)}
						disabled={
							widgetPanelSurfaceOverrideValue(selectedLayoutWidget.widgetId, 'panelOpacity') === null &&
							widgetPanelSurfaceOverrideValue(selectedLayoutWidget.widgetId, 'panelBlur') === null
						}
					>
						<Icon icon="tabler:restore" width="14" height="14" />{_('Reset')}
					</button>
				</header>

				<div class="layout-widget-grid-row">
					<div class="widget-grid-field">
						<span>{_('Opacity')}</span>
						<div class="widget-surface-slider">
							<input
								type="range"
								min="40"
								max="100"
								step="10"
								value={widgetPanelSurfaceValue(selectedLayoutWidget.widgetId, 'panelOpacity')}
								aria-label={_('Widget panel opacity')}
								oninput={(event) => handleWidgetPanelSurfaceInput(selectedLayoutWidget.widgetId, 'panelOpacity', event)}
							/>
						</div>
						<small>
							{widgetPanelSurfaceValue(selectedLayoutWidget.widgetId, 'panelOpacity')}%
							{widgetPanelSurfaceOverrideValue(selectedLayoutWidget.widgetId, 'panelOpacity') === null ? ` ${_('Global')}` : ` ${_('Override')}`}
						</small>
					</div>

					<div class="widget-grid-field">
						<span>{_('Blur')}</span>
						<div class="widget-surface-slider">
							<input
								type="range"
								min="0"
								max="24"
								step="4"
								value={widgetPanelSurfaceValue(selectedLayoutWidget.widgetId, 'panelBlur')}
								aria-label={_('Widget panel blur')}
								oninput={(event) => handleWidgetPanelSurfaceInput(selectedLayoutWidget.widgetId, 'panelBlur', event)}
							/>
						</div>
						<small>
							{widgetPanelSurfaceValue(selectedLayoutWidget.widgetId, 'panelBlur')}px
							{widgetPanelSurfaceOverrideValue(selectedLayoutWidget.widgetId, 'panelBlur') === null ? ` ${_('Global')}` : ` ${_('Override')}`}
						</small>
					</div>
				</div>
			</section>

			<div class="layout-widget-actions">
				<button type="button" onclick={() => moveWidgetInZone(selectedLayoutWidget.widgetId, -1)}>
					<Icon icon="tabler:arrow-up" width="14" height="14" />{_('Move widget up')}
				</button>
				<button type="button" onclick={() => moveWidgetInZone(selectedLayoutWidget.widgetId, 1)}>
					<Icon icon="tabler:arrow-down" width="14" height="14" />{_('Move widget down')}
				</button>
				<button type="button" onclick={() => resetWidgetGrid(selectedLayoutWidget.widgetId)}>
					<Icon icon="tabler:restore" width="14" height="14" />{_('Reset widget size')}
				</button>
				<button type="button" class="danger" onclick={() => hideWidget(selectedLayoutWidget.widgetId)}>
					<Icon icon="tabler:eye-off" width="14" height="14" />{_('Hide widget')}
				</button>
			</div>
		</div>
	{/if}

	{#if isLayoutEditing && isWidgetDrawerOpen}
		<div class="widget-drawer" role="dialog" aria-label="Add widget">
			<header>
				<h2>Add widget</h2>
				<button
					type="button"
					class="icon-button"
					onclick={() => (isWidgetDrawerOpen = false)}
					title="Close add widget drawer"
					aria-label="Close add widget drawer"
				>
					<Icon icon="tabler:x" width="16" height="16" />
				</button>
			</header>
			<div class="widget-drawer-list">
				{#each addableWidgetsForCurrentView as widget}
					<button type="button" onclick={() => showWidget(widget.id)}>
						<strong>{widget.title}</strong>
						<span>{widget.defaultZone}</span>
					</button>
				{:else}
					<p>No widgets available.</p>
				{/each}
			</div>
		</div>
	{/if}
</main>

{#if isAccountDrawerOpen}
	<button
		type="button"
		class="drawer-backdrop modal-backdrop"
		aria-label="Close account setup"
		onclick={closeAccountDrawer}
	></button>
	<div class="account-modal" role="dialog" aria-modal="true" aria-labelledby="account-setup-heading">
		<header>
			<div>
				<p>{accountWizardKind === 'mail' ? 'Mail Account' : accountWizardKind === 'calendar' ? 'Calendar Account' : accountWizardKind === 'telegram' ? 'Telegram Account' : 'WhatsApp Account'}</p>
				<h2 id="account-setup-heading">{accountWizardKind === 'mail' ? 'New Mail Account' : accountWizardKind === 'calendar' ? 'New Calendar Account' : accountWizardKind === 'telegram' ? 'New Telegram Account' : 'New WhatsApp Account'}</h2>
			</div>
			<button type="button" class="icon-button" onclick={closeAccountDrawer} aria-label="Close">
				<Icon icon="tabler:x" width="18" height="18" />
			</button>
		</header>

		{#if accountWizardKind === 'mail'}
			<div class="wizard-progress" aria-label="Mail setup steps">
				<span class:active={mailWizardStep === 'provider'}>1. Service</span>
				<span class:active={mailWizardStep === 'details'}>2. Details</span>
			</div>
			{#if mailWizardStep === 'provider'}
				<form class="wizard-step" onsubmit={(event) => { event.preventDefault(); continueMailWizard(); }}>
					<label class="wide wizard-email-field"><span>Email address</span><input bind:value={mailAddress} type="email" placeholder="name@example.com" autocomplete="email" /></label>
					<div class="wizard-divider"><span>or choose mail service</span></div>
					<div class="wizard-choice-list">
						<button type="button" class:active={selectedMailService === 'icloud'} onclick={() => selectMailService('icloud')}><Icon icon="tabler:cloud" width="34" height="34" /><strong>iCloud</strong></button>
						<button type="button" class:active={selectedMailService === 'microsoft'} onclick={() => selectMailService('microsoft')}><Icon icon="tabler:brand-office" width="34" height="34" /><strong>Microsoft Exchange</strong></button>
						<button type="button" class:active={selectedMailService === 'gmail'} onclick={() => selectMailService('gmail')}><Icon icon="tabler:brand-gmail" width="34" height="34" /><strong>Google</strong></button>
						<button type="button" class:active={selectedMailService === 'yahoo'} onclick={() => selectMailService('yahoo')}><Icon icon="tabler:mail" width="34" height="34" /><strong>Yahoo</strong></button>
						<button type="button" class:active={selectedMailService === 'aol'} onclick={() => selectMailService('aol')}><Icon icon="tabler:mail-bolt" width="34" height="34" /><strong>AOL</strong></button>
						<button type="button" class:active={selectedMailService === 'imap'} onclick={() => selectMailService('imap')}><Icon icon="tabler:server" width="34" height="34" /><strong>Other Mail Account</strong></button>
					</div>
					<div class="wizard-actions">
						<button type="submit" class="primary-button">Continue</button>
					</div>
				</form>
			{:else}
				<div class="wizard-step">
					<button type="button" class="wizard-back" onclick={() => (mailWizardStep = 'provider')}><Icon icon="tabler:arrow-left" width="15" height="15" />Service</button>
					{#if selectedMailService === 'gmail'}
						<form class="setup-form" onsubmit={(event) => event.preventDefault()}>
							<label><span>Account ID</span><input bind:value={gmailForm.account_id} autocomplete="off" /></label>
							<label><span>Display name</span><input bind:value={gmailForm.display_name} autocomplete="off" /></label>
							<label><span>Gmail address</span><input bind:value={gmailForm.external_account_id} autocomplete="email" /></label>
							<label><span>OAuth client ID</span><input bind:value={gmailForm.client_id} autocomplete="off" /></label>
							<label><span>OAuth client secret</span><input bind:value={gmailForm.client_secret} type="password" autocomplete="off" /></label>
							<label class="wide"><span>Redirect URI</span><input bind:value={gmailForm.redirect_uri} autocomplete="off" /></label>
							<div class="form-actions wide"><button type="button" onclick={startGmailSetup} disabled={isSetupSubmitting}>Start OAuth</button></div>
						</form>
						{#if gmailPending}
							<div class="oauth-box">
								<a href={gmailPending.authorization_url} target="_blank" rel="noreferrer">Open Google consent</a>
								<label><span>Authorization code</span><input bind:value={gmailAuthorizationCode} autocomplete="off" /></label>
								<button type="button" onclick={completeGmailSetup} disabled={isSetupSubmitting}>Complete Gmail</button>
							</div>
						{/if}
					{:else if hasFixedMailServerPreset(selectedMailService)}
						<div class="setup-summary-card" aria-label="Selected mail service">
							<span class="round-icon cyan"><Icon icon={mailServiceIcon(selectedMailService)} width="18" height="18" /></span>
							<div>
								<strong>{mailServiceDisplayName(selectedMailService)}</strong>
							</div>
						</div>
						<form class="setup-form compact-form" onsubmit={(event) => event.preventDefault()}>
							<label><span>Login</span><input bind:value={imapForm.username} autocomplete="username" /></label>
							<label><span>Password</span><input bind:value={imapForm.password} type="password" autocomplete="current-password" /></label>
							<div class="form-actions wide"><button type="button" onclick={saveImapAccount} disabled={isSetupSubmitting}>Save Account</button></div>
						</form>
					{:else}
						<form class="setup-form" onsubmit={(event) => event.preventDefault()}>
							<label><span>Account ID</span><input bind:value={imapForm.account_id} autocomplete="off" /></label>
							<label><span>Display name</span><input bind:value={imapForm.display_name} autocomplete="off" /></label>
							<label><span>Email address</span><input bind:value={imapForm.external_account_id} autocomplete="email" /></label>
							<label><span>Username</span><input bind:value={imapForm.username} autocomplete="username" /></label>
							<label><span>Host</span><input bind:value={imapForm.host} autocomplete="off" /></label>
							<label><span>Port</span><input bind:value={imapForm.port} type="number" min="1" max="65535" /></label>
							<label><span>Mailbox</span><input bind:value={imapForm.mailbox} autocomplete="off" /></label>
							<label><span>Password</span><input bind:value={imapForm.password} type="password" autocomplete="current-password" /></label>
							<label class="checkbox-row"><input bind:checked={imapForm.tls} type="checkbox" /><span>TLS</span></label>
							<div class="form-actions"><button type="button" onclick={saveImapAccount} disabled={isSetupSubmitting}>Save Account</button></div>
						</form>
					{/if}
				</div>
			{/if}
		{:else if accountWizardKind === 'calendar'}
			<div class="wizard-progress" aria-label="Calendar setup steps">
				<span class:active={calendarWizardStep === 'provider'}>1. Provider</span>
				<span class:active={calendarWizardStep === 'details'}>2. Details</span>
			</div>
			{#if calendarWizardStep === 'provider'}
				<div class="wizard-step">
					<div class="wizard-choice-grid">
						<button type="button" onclick={() => continueCalendarWizard('local')}><Icon icon="tabler:calendar" width="28" height="28" /><strong>Local</strong></button>
						<button type="button" onclick={() => continueCalendarWizard('google')}><Icon icon="tabler:brand-google" width="28" height="28" /><strong>Google Calendar</strong></button>
						<button type="button" onclick={() => continueCalendarWizard('microsoft')}><Icon icon="tabler:brand-office" width="28" height="28" /><strong>Microsoft 365</strong></button>
						<button type="button" onclick={() => continueCalendarWizard('apple')}><Icon icon="tabler:apple" width="28" height="28" /><strong>Apple Calendar</strong></button>
						<button type="button" onclick={() => continueCalendarWizard('caldav')}><Icon icon="tabler:server" width="28" height="28" /><strong>CalDAV</strong></button>
						<button type="button" onclick={() => continueCalendarWizard('ics')}><Icon icon="tabler:rss" width="28" height="28" /><strong>ICS Feed</strong></button>
					</div>
				</div>
			{:else}
				<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void saveCalendarAccount(); }}>
					<button type="button" class="wizard-back wide" onclick={() => (calendarWizardStep = 'provider')}><Icon icon="tabler:arrow-left" width="15" height="15" />Provider</button>
					<label><span>Provider</span><select bind:value={calendarAccountForm.provider}><option value="local">Local</option><option value="google">Google Calendar</option><option value="microsoft">Microsoft 365</option><option value="apple">Apple Calendar</option><option value="caldav">CalDAV</option><option value="ics">ICS Feed</option></select></label>
					<label><span>Account name</span><input bind:value={calendarAccountForm.account_name} autocomplete="off" /></label>
					<label class="wide"><span>Email or owner</span><input bind:value={calendarAccountForm.email} autocomplete="email" /></label>
					<div class="form-actions wide"><button type="submit" disabled={isSetupSubmitting || !calendarAccountForm.account_name.trim()}>Save Calendar</button></div>
				</form>
			{/if}
		{:else if accountWizardKind === 'telegram'}
			<div class="wizard-progress" aria-label="Telegram setup steps">
				<span class:active={telegramWizardStep === 'account'}>1. Account</span>
				<span class:active={telegramWizardStep === 'auth'}>2. Login</span>
				<span class:active={telegramWizardStep === 'details'}>3. Details</span>
			</div>
			{#if telegramWizardStep === 'account'}
				<div class="wizard-step">
					<div class="wizard-choice-grid two">
						<button type="button" class:active={telegramAccountForm.provider_kind === 'telegram_user'} onclick={() => { telegramAccountForm = { ...telegramAccountForm, provider_kind: 'telegram_user' }; selectTelegramAuthMethod('phone'); continueTelegramWizard('auth'); }}><Icon icon="tabler:user" width="30" height="30" /><strong>User Account</strong><span>Phone or QR login</span></button>
						<button type="button" class:active={telegramAccountForm.provider_kind === 'telegram_bot'} onclick={() => { telegramAccountForm = { ...telegramAccountForm, provider_kind: 'telegram_bot' }; selectTelegramAuthMethod('bot_token'); continueTelegramWizard('auth'); }}><Icon icon="tabler:robot" width="30" height="30" /><strong>Bot Account</strong><span>Bot token</span></button>
					</div>
				</div>
			{:else if telegramWizardStep === 'auth'}
				<div class="wizard-step">
					<button type="button" class="wizard-back" onclick={() => (telegramWizardStep = 'account')}><Icon icon="tabler:arrow-left" width="15" height="15" />Account</button>
					<div class="wizard-choice-grid">
						{#if telegramAccountForm.provider_kind === 'telegram_user'}
							<button type="button" class:active={telegramAuthMethod === 'phone'} onclick={() => { selectTelegramAuthMethod('phone'); continueTelegramWizard('details'); }}><Icon icon="tabler:phone" width="28" height="28" /><strong>Phone Number</strong></button>
							<button type="button" class:active={telegramAuthMethod === 'qr'} onclick={() => { selectTelegramAuthMethod('qr'); continueTelegramWizard('details'); }}><Icon icon="tabler:qrcode" width="28" height="28" /><strong>QR Code</strong></button>
						{/if}
						{#if telegramAccountForm.provider_kind === 'telegram_bot'}
							<button type="button" class:active={telegramAuthMethod === 'bot_token'} onclick={() => { selectTelegramAuthMethod('bot_token'); continueTelegramWizard('details'); }}><Icon icon="tabler:key" width="28" height="28" /><strong>Bot Token</strong></button>
						{/if}
						<button type="button" class:active={telegramAuthMethod === 'fixture'} onclick={() => { selectTelegramAuthMethod('fixture'); continueTelegramWizard('details'); }}><Icon icon="tabler:flask" width="28" height="28" /><strong>Fixture</strong></button>
					</div>
				</div>
			{:else}
					<form class="setup-form" onsubmit={(event) => { event.preventDefault(); telegramAuthMethod === 'qr' ? submitTelegramQrStepFromWizard() : void saveTelegramAccountFromWizard(); }}>
					<button type="button" class="wizard-back wide" onclick={() => (telegramWizardStep = 'auth')}><Icon icon="tabler:arrow-left" width="15" height="15" />Login</button>
					{#if telegramAuthMethod !== 'qr'}
						<label><span>Account ID</span><input bind:value={telegramAccountForm.account_id} autocomplete="off" /></label>
						<label><span>Display name</span><input bind:value={telegramAccountForm.display_name} autocomplete="off" /></label>
					{/if}
					{#if telegramAuthMethod === 'phone'}
						<label class="wide"><span>Phone number</span><input bind:value={telegramAccountForm.external_account_id} autocomplete="tel" placeholder="+15551234567" /></label>
						{:else if telegramAuthMethod === 'qr'}
							<div class="qr-login-panel wide" class:large={Boolean(telegramQrLogin?.qr_svg)}>
								{#if telegramQrLogin?.qr_svg}
									<div class="qr-svg" aria-label="Telegram QR code">{@html telegramQrLogin.qr_svg}</div>
								{:else}
									<Icon icon="tabler:qrcode" width="58" height="58" />
								{/if}
								<div>
									<strong>{telegramQrLogin ? telegramQrStatusLabel(telegramQrLogin.status) : 'QR login'}</strong>
									<p>{telegramQrLogin?.message ?? 'Telegram QR code is ready to be generated for this account.'}</p>
									{#if telegramQrLogin?.qr_link}
										<a href={telegramQrLogin.qr_link}>Open Telegram login link</a>
									{/if}
								</div>
							</div>
							{#if telegramQrLogin?.status === 'waiting_password'}
								<label class="wide"><span>Telegram Cloud Password</span><input bind:value={telegramQrPassword} type="password" autocomplete="current-password" /></label>
							{/if}
					{:else}
						<label class="wide"><span>External ID</span><input bind:value={telegramAccountForm.external_account_id} autocomplete="off" /></label>
					{/if}
					{#if telegramNeedsFormAppCredentials}
						<label><span>API ID</span><input bind:value={telegramAccountForm.api_id} inputmode="numeric" autocomplete="off" /></label>
						<label><span>API hash</span><input bind:value={telegramAccountForm.api_hash} type="password" autocomplete="off" /></label>
						<label class="wide"><span>Session encryption key</span><input bind:value={telegramAccountForm.session_encryption_key} type="password" autocomplete="off" /></label>
					{/if}
					{#if telegramAuthMethod === 'bot_token'}
						<label class="wide"><span>Bot token</span><input bind:value={telegramAccountForm.bot_token} type="password" autocomplete="off" /></label>
					{/if}
						<div class="wizard-note wide">
							{telegramWizardNote()}
						</div>
						<div class="form-actions wide">
							{#if telegramAuthMethod === 'qr'}
								{#if telegramQrLogin?.status === 'waiting_password'}
									<button type="submit" disabled={isTelegramActionSubmitting || !telegramQrPassword}>Continue</button>
								{:else}
									<button type="submit" disabled={isTelegramActionSubmitting || telegramQrLogin?.status === 'ready' || telegramQrRuntimeBlocked}>Start QR Login</button>
								{/if}
								<button type="button" onclick={refreshTelegramQrLoginStatus} disabled={isTelegramActionSubmitting || !telegramQrLogin || telegramQrLogin.status === 'ready'}>Refresh Status</button>
								{#if telegramQrLogin?.status === 'ready'}
									<button type="button" onclick={() => void saveTelegramAccountFromWizard()} disabled={isTelegramActionSubmitting}>Save Account</button>
								{/if}
							{:else}
								<button type="submit" disabled={isTelegramActionSubmitting}>{telegramAuthMethod === 'fixture' ? 'Save Fixture' : 'Save Live Record'}</button>
							{/if}
						</div>
					</form>
			{/if}
		{:else if accountWizardKind === 'whatsapp'}
			<form class="setup-form" onsubmit={(event) => { event.preventDefault(); void setupWhatsappWebFixture(); }}>
				<label><span>Account ID</span><input bind:value={whatsappAccountForm.account_id} autocomplete="off" /></label>
				<label><span>Display name</span><input bind:value={whatsappAccountForm.display_name} autocomplete="off" /></label>
				<label><span>External ID</span><input bind:value={whatsappAccountForm.external_account_id} autocomplete="off" /></label>
				<label><span>Device name</span><input bind:value={whatsappAccountForm.device_name} autocomplete="off" /></label>
				<label class="wide"><span>Local state path</span><input bind:value={whatsappAccountForm.local_state_path} autocomplete="off" /></label>
				<div class="wizard-note wide">WhatsApp Web live runtime remains blocked; this creates a fixture companion-session record.</div>
				<div class="form-actions wide"><button type="submit" disabled={isWhatsappActionSubmitting}>Save Fixture</button></div>
			</form>
		{/if}

		{#if setupMessage}<p class="setup-state success">{setupMessage}</p>{/if}
		{#if setupError}<p class="setup-state error">{setupError}</p>{/if}
	</div>
{/if}
