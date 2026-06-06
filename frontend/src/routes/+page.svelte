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
		fetchTasks,
		fetchV4Capabilities,
		fetchV5Capabilities,
		fetchTelegramCalls,
		fetchTelegramChats,
		fetchTelegramMessages,
		fetchWhatsappWebMessages,
		fetchWhatsappWebSessions,
		fetchProjects,
		fetchV1Status,
		ingestTelegramFixtureMessage,
		ingestWhatsappWebFixtureMessage,
		refreshAiTaskCandidates,
		reviewIdentityCandidate,
		reviewTaskCandidate,
		requestAiAnswer,
		requestAiMeetingPrep,
		retryDocumentProcessingJob,
		saveApplicationSetting,
		searchGraphNodes,
		saveAutomationPolicy,
		saveAutomationTemplate,
		saveCallTranscriptFixture,
		saveTelegramCall,
		setupImapAccount,
		setupTelegramFixtureAccount,
		setupWhatsappWebFixtureAccount,
		startGmailOAuthSetup,
		type ActiveTask,
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
		type ContactIdentityCandidate,
		type ContactIdentityReviewState,
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
		type TelegramCall,
		type TelegramChat,
		type TelegramMessage,
		type TelegramProviderKind,
		type TelegramSendDryRunResponse,
		type V4CapabilitiesResponse,
		type V5CapabilitiesResponse,
		type WhatsappWebMessage,
		type WhatsappWebSession,
		type V1Status
	} from '$lib/api';
	import { onMount } from 'svelte';

	type Provider = 'gmail' | 'icloud' | 'imap';
	type ViewId =
		| 'home'
		| 'communications'
		| 'timeline'
		| 'contacts'
		| 'projects'
		| 'tasks'
		| 'calendar'
		| 'documents'
		| 'notes'
		| 'knowledge'
		| 'telegram'
		| 'whatsapp'
		| 'agents'
		| 'settings';

	type NavItem = {
		id: ViewId;
		label: string;
		icon: string;
		badge?: string;
		enabled: boolean;
	};

	type ShortcutItem = {
		label: string;
		icon: string;
		badge?: string;
	};

	type StatCard = {
		label: string;
		value: string;
		delta: string;
		icon: string;
		tone?: string;
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
	const apiToken = import.meta.env.VITE_HERMES_LOCAL_API_TOKEN ?? 'change-me-local-api-token';
	let actorId = $state(import.meta.env.VITE_HERMES_ACTOR_ID ?? 'desktop-shell');

	let currentView = $state<ViewId>('home');
	let searchQuery = $state('');
	let status = $state<V1Status | null>(null);
	let statusError = $state('');
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
	let selectedCommunicationDetail = $state<CommunicationMessageDetail | null>(null);
	let communicationsError = $state('');
	let isCommunicationsLoading = $state(false);
	let projectSummaries = $state<ProjectSummary[]>([]);
	let selectedProjectDetail = $state<ProjectDetail | null>(null);
	let selectedProjectId = $state('');
	let projectsError = $state('');
	let isProjectsLoading = $state(false);
	let taskCandidates = $state<TaskCandidate[]>([]);
	let activeTasks = $state<ActiveTask[]>([]);
	let documentProcessingJobs = $state<DocumentProcessingJob[]>([]);
	let selectedDocumentProcessingDetail = $state<DocumentProcessingRecord | null>(null);
	let documentProcessingDetailError = $state('');
	let isDocumentProcessingJobsLoading = $state(false);
	let retryingDocumentProcessingJobId = $state<string | null>(null);
	let documentProcessingJobsError = $state('');
	let isTasksLoading = $state(false);
	let tasksError = $state('');
	let identityCandidates = $state<ContactIdentityCandidate[]>([]);
	let identityCandidatesError = $state('');
	let isIdentityCandidatesLoading = $state(false);
	let projectRequestSequence = 0;
	let selectedConversationIndex = $state(0);
	let selectedContactIndex = $state(0);
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
	let telegramChats = $state<TelegramChat[]>([]);
	let telegramMessages = $state<TelegramMessage[]>([]);
	let automationTemplates = $state<AutomationTemplate[]>([]);
	let automationPolicies = $state<AutomationPolicy[]>([]);
	let telegramCalls = $state<TelegramCall[]>([]);
	let v4Capabilities = $state<V4CapabilitiesResponse | null>(null);
	let selectedTelegramChatId = $state('');
	let selectedTelegramCallId = $state('');
	let callTranscript = $state<CallTranscript | null>(null);
	let telegramError = $state('');
	let telegramActionMessage = $state('');
	let isTelegramLoading = $state(false);
	let isTelegramActionSubmitting = $state(false);
	let telegramSendDryRunResult = $state<TelegramSendDryRunResponse | null>(null);
	let telegramAccountForm = $state({
		account_id: 'telegram-primary',
		provider_kind: 'telegram_user' as TelegramProviderKind,
		display_name: 'Primary Telegram',
		external_account_id: '@telegram_fixture',
		tdlib_data_path: 'docker/data/telegram/telegram-primary',
		transcription_enabled: true
	});
	let telegramMessageForm = $state({
		account_id: 'telegram-primary',
		provider_chat_id: 'fixture-chat-1',
		provider_message_id: 'fixture-msg-1',
		chat_kind: 'private' as 'private' | 'group' | 'channel' | 'bot',
		chat_title: 'V4 Planning',
		sender_id: 'telegram-fixture-user',
		sender_display_name: 'Telegram Fixture',
		text: 'V4 fixture Telegram message for policy and graph smoke coverage.',
		import_batch_id: 'telegram-fixture-ui',
		occurred_at: new Date().toISOString(),
		delivery_state: 'received' as 'received' | 'sent' | 'send_dry_run' | 'send_blocked'
	});
	let automationTemplateForm = $state({
		template_id: 'template-v4-followup',
		name: 'V4 Follow-up',
		body_template: 'Hi {{name}}, I will follow up about {{topic}}.',
		required_variables_text: 'name, topic'
	});
	let automationPolicyForm = $state({
		policy_id: 'policy-v4-followup',
		template_id: 'template-v4-followup',
		name: 'V4 follow-up allowlist',
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
		policy_id: 'policy-v4-followup',
		provider_chat_id: 'fixture-chat-1',
		variables_text: '{ "name": "Maria", "topic": "V4 Telegram client" }',
		source_context_text: '{ "source": "desktop_ui_fixture" }'
	});
	let telegramCallForm = $state({
		call_id: 'call-v4-fixture-1',
		account_id: 'telegram-primary',
		provider_call_id: 'provider-call-v4-fixture-1',
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
	let v5Capabilities = $state<V5CapabilitiesResponse | null>(null);
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
		chat_title: 'V5 Planning',
		sender_id: 'wa-fixture-user',
		sender_display_name: 'WhatsApp Fixture',
		text: 'V5 fixture WhatsApp Web message for local memory and graph recall.',
		import_batch_id: 'whatsapp-web-fixture-ui',
		occurred_at: new Date().toISOString(),
		delivery_state: 'received' as 'received' | 'sent' | 'send_dry_run' | 'send_blocked'
	});
	let transcriptForm = $state({
		transcript_id: 'transcript-v4-fixture-1',
		account_id: 'telegram-primary',
		provider_chat_id: 'fixture-chat-1',
		source_audio_ref: 'docker/data/calls/fixture-call.wav',
		language_code: 'en',
		always_on_policy: true
	});
	let applicationSettings = $state<ApplicationSetting[]>([]);
	let providerAccounts = $state<ProviderAccount[]>([]);
	let settingDrafts = $state<Record<string, string>>({});
	let settingsError = $state('');
	let settingsActionMessage = $state('');
	let isSettingsLoading = $state(false);
	let savingSettingKey = $state<string | null>(null);
	let selectedSettingsSection = $state<'application' | 'accounts'>('application');

	const primaryNav: NavItem[] = [
		{ id: 'home', label: 'Home', icon: 'tabler:home', enabled: true },
		{ id: 'communications', label: 'Communications', icon: 'tabler:messages', badge: '23', enabled: true },
		{ id: 'timeline', label: 'Timeline', icon: 'tabler:timeline-event', enabled: true },
		{ id: 'contacts', label: 'Contacts', icon: 'tabler:address-book', enabled: true },
		{ id: 'projects', label: 'Projects', icon: 'tabler:briefcase', enabled: true },
		{ id: 'tasks', label: 'Tasks', icon: 'tabler:checkbox', enabled: true },
		{ id: 'calendar', label: 'Calendar', icon: 'tabler:calendar', enabled: true },
		{ id: 'documents', label: 'Documents', icon: 'tabler:file-text', enabled: true },
		{ id: 'notes', label: 'Notes', icon: 'tabler:notes', enabled: true },
		{ id: 'knowledge', label: 'Knowledge Graph', icon: 'tabler:share', enabled: true },
		{ id: 'telegram', label: 'Telegram', icon: 'tabler:brand-telegram', enabled: true },
		{ id: 'whatsapp', label: 'WhatsApp', icon: 'tabler:brand-whatsapp', enabled: true },
		{ id: 'agents', label: 'AI Agents', icon: 'tabler:sparkles', enabled: true },
		{ id: 'settings', label: 'Settings', icon: 'tabler:settings', enabled: true }
	];

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
		contacts: {
			title: 'Contacts',
			subtitle: '642 contacts',
			search: 'Search contacts, companies, emails...',
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
			search: 'Search events, meetings, contacts...',
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
			subtitle: 'V4 messaging, policy automation and call intelligence.',
			search: 'Search Telegram chats, policies, calls...',
			icon: 'tabler:brand-telegram'
		},
		whatsapp: {
			title: 'WhatsApp Web',
			subtitle: 'V5 companion sessions, fixture ingestion and live-runtime guardrails.',
			search: 'Search WhatsApp sessions and messages...',
			icon: 'tabler:brand-whatsapp'
		},
		agents: {
			title: 'AI Agents',
			subtitle: 'Your intelligent assistants working across your data and tools',
			search: 'Search agents, capabilities, tasks...',
			icon: 'tabler:sparkles'
		},
		settings: {
			title: 'Settings',
			subtitle: 'Runtime settings and connected accounts.',
			search: 'Search settings and accounts...',
			icon: 'tabler:settings'
		}
	};

	const shortcutsByView: Record<ViewId, ShortcutItem[]> = {
		home: [
			{ label: 'Inbox', icon: 'tabler:inbox', badge: '12' },
			{ label: 'Starred', icon: 'tabler:star' },
			{ label: 'Waiting', icon: 'tabler:clock-hour-4', badge: '3' },
			{ label: 'Requires Reply', icon: 'tabler:message-reply', badge: '5' },
			{ label: 'Mentions', icon: 'tabler:at' },
			{ label: 'Trash', icon: 'tabler:trash' }
		],
		communications: [
			{ label: 'Inbox', icon: 'tabler:inbox', badge: '12' },
			{ label: 'Starred', icon: 'tabler:star' },
			{ label: 'Waiting', icon: 'tabler:clock-hour-4' },
			{ label: 'Requires Reply', icon: 'tabler:message-reply', badge: '3' },
			{ label: 'Mentions', icon: 'tabler:at' },
			{ label: 'Spam', icon: 'tabler:shield-x', badge: '4' },
			{ label: 'Archive', icon: 'tabler:archive' }
		],
		timeline: [
			{ label: 'Today', icon: 'tabler:calendar-time', badge: '18' },
			{ label: 'Messages', icon: 'tabler:message' },
			{ label: 'Documents', icon: 'tabler:file-text' },
			{ label: 'Decisions', icon: 'tabler:git-pull-request' }
		],
		contacts: [
			{ label: 'All People', icon: 'tabler:users', badge: '642' },
			{ label: 'Companies', icon: 'tabler:building', badge: '128' },
			{ label: 'Clients', icon: 'tabler:shield-check' },
			{ label: 'Partners', icon: 'tabler:users-group' },
			{ label: 'Team', icon: 'tabler:user-check' },
			{ label: 'Vendors', icon: 'tabler:briefcase' },
			{ label: 'Archived', icon: 'tabler:archive' }
		],
		projects: [
			{ label: 'My Projects', icon: 'tabler:briefcase', badge: '12' },
			{ label: 'Active', icon: 'tabler:chart-bar', badge: '7' },
			{ label: 'Planning', icon: 'tabler:calendar-plus' },
			{ label: 'On Hold', icon: 'tabler:clock-pause' },
			{ label: 'Completed', icon: 'tabler:rosette-discount-check' },
			{ label: 'Archived', icon: 'tabler:archive' }
		],
		tasks: [
			{ label: 'My Tasks', icon: 'tabler:checkbox', badge: '12' },
			{ label: 'Assigned to Me', icon: 'tabler:user-check', badge: '7' },
			{ label: 'Waiting', icon: 'tabler:clock', badge: '5' },
			{ label: 'Due Today', icon: 'tabler:calendar-exclamation', badge: '3' },
			{ label: 'This Week', icon: 'tabler:calendar-week', badge: '9' },
			{ label: 'High Priority', icon: 'tabler:star', badge: '4' },
			{ label: 'Completed', icon: 'tabler:heart-check' }
		],
		calendar: [
			{ label: 'My Agenda', icon: 'tabler:calendar-stats', badge: '12' },
			{ label: 'Team Meetings', icon: 'tabler:star', badge: '7' },
			{ label: 'Focus Time', icon: 'tabler:shield-half', badge: '5' },
			{ label: 'Important', icon: 'tabler:shield-star', badge: '3' },
			{ label: 'Travel', icon: 'tabler:plane', badge: '2' },
			{ label: 'Birthdays', icon: 'tabler:calendar-heart' }
		],
		documents: [
			{ label: 'Recent', icon: 'tabler:inbox', badge: '24' },
			{ label: 'Starred', icon: 'tabler:star', badge: '8' },
			{ label: 'Shared with me', icon: 'tabler:shield-check', badge: '12' },
			{ label: 'Contracts', icon: 'tabler:briefcase' },
			{ label: 'Reports', icon: 'tabler:report' },
			{ label: 'Presentations', icon: 'tabler:presentation' },
			{ label: 'Archive', icon: 'tabler:archive' },
			{ label: 'Trash', icon: 'tabler:trash' }
		],
		notes: [
			{ label: 'Inbox', icon: 'tabler:inbox', badge: '12' },
			{ label: 'Starred', icon: 'tabler:star', badge: '8' },
			{ label: 'Today', icon: 'tabler:calendar-check', badge: '5' },
			{ label: 'Personal', icon: 'tabler:folder', badge: '7' },
			{ label: 'Work', icon: 'tabler:folder', badge: '9' },
			{ label: 'Ideas', icon: 'tabler:bulb', badge: '4' },
			{ label: 'Archive', icon: 'tabler:archive' }
		],
		knowledge: [
			{ label: 'My Graphs', icon: 'tabler:heart-handshake', badge: '12' },
			{ label: 'Recent', icon: 'tabler:star', badge: '24' },
			{ label: 'Favorites', icon: 'tabler:star', badge: '8' },
			{ label: 'Important', icon: 'tabler:shield-star', badge: '15' },
			{ label: 'Shared with me', icon: 'tabler:star', badge: '7' },
			{ label: 'Trash', icon: 'tabler:trash' }
		],
		telegram: [
			{ label: 'Chats', icon: 'tabler:messages', badge: 'V4' },
			{ label: 'Policies', icon: 'tabler:shield-check' },
			{ label: 'Templates', icon: 'tabler:template' },
			{ label: 'Calls', icon: 'tabler:phone-call' },
			{ label: 'Transcripts', icon: 'tabler:file-text' },
			{ label: 'Audit', icon: 'tabler:clipboard-list' }
		],
		whatsapp: [
			{ label: 'Sessions', icon: 'tabler:devices', badge: 'V5' },
			{ label: 'Messages', icon: 'tabler:messages' },
			{ label: 'Fixture', icon: 'tabler:flask' },
			{ label: 'Guardrails', icon: 'tabler:shield-lock' },
			{ label: 'Provenance', icon: 'tabler:git-branch' }
		],
		agents: [
			{ label: 'My Agents', icon: 'tabler:robot', badge: '12' },
			{ label: 'Active Tasks', icon: 'tabler:star', badge: '8' },
			{ label: 'Automations', icon: 'tabler:settings-automation', badge: '6' },
			{ label: 'Templates', icon: 'tabler:template', badge: '15' },
			{ label: 'Logs', icon: 'tabler:clipboard-list' },
			{ label: 'Settings', icon: 'tabler:settings' }
		],
		settings: [
			{ label: 'Application', icon: 'tabler:adjustments-horizontal', badge: 'DB' },
			{ label: 'Accounts', icon: 'tabler:users' },
			{ label: 'AI Runtime', icon: 'tabler:sparkles' },
			{ label: 'Security', icon: 'tabler:shield-lock' }
		]
	};

	const homeStats: StatCard[] = [
		{ label: 'New Events', value: '47', delta: '18%', icon: 'tabler:chart-bar' },
		{ label: 'Needs Attention', value: '4', delta: '2', icon: 'tabler:alert-circle' },
		{ label: 'Waiting For Reply', value: '3', delta: '1', icon: 'tabler:message-reply' },
		{ label: 'New Documents', value: '2', delta: '1', icon: 'tabler:file-text' },
		{ label: 'New Contacts', value: '1', delta: '1', icon: 'tabler:user-plus' }
	];

	const whatsNew: FeedItem[] = [
		{ icon: 'tabler:mail', title: 'New email from John Smith', meta: 'Re: Project Hermes - Next Steps', time: '14:32', tag: 'Project Hermes', tone: 'blue' },
		{ icon: 'tabler:brand-telegram', title: 'Telegram message from Maria Petrova', meta: 'Can you review the new mockups?', time: '14:15', tag: 'Design', tone: 'blue' },
		{ icon: 'tabler:brand-whatsapp', title: 'WhatsApp from Accountant', meta: 'Please send me the VAT report for Q2', time: '13:47', tag: 'Finance', tone: 'green' },
		{ icon: 'tabler:file-text', title: 'Document uploaded', meta: 'Contract_Smith_Partners.pdf', time: '11:28', tag: 'Smith & Partners', tone: 'slate' },
		{ icon: 'tabler:calendar-check', title: 'Meeting completed', meta: 'Project Hermes - Weekly Sync', time: '10:42', tag: '45m · 6 participants', tone: 'mint' }
	];

	const peopleTalked = [
		{ name: 'John Smith', meta: 'Re: Project Hermes - Next Steps', icon: 'tabler:mail' },
		{ name: 'Maria Petrova', meta: 'Can you review the new mockups?', icon: 'tabler:brand-telegram' },
		{ name: 'Accountant', meta: 'VAT report for Q2', icon: 'tabler:brand-whatsapp' },
		{ name: 'IRIS Team', meta: 'Updated roadmap v2.0', icon: 'tabler:brand-telegram' },
		{ name: 'Elena Rodriguez', meta: 'Document request', icon: 'tabler:brand-whatsapp' }
	];

	const conversations: Conversation[] = [
		{ name: 'John Smith', role: 'CEO at Smith & Partners', project: 'Hermes Project', channel: 'Email', time: '14:32', unread: '2', preview: "Sounds good! Let's schedule a call for tomorrow" },
		{ name: 'Maria Petrova', role: 'Lead Designer', project: 'Design Discussion', channel: 'Telegram', time: '14:15', unread: '1', preview: 'Here are the mockups for the new dashboard' },
		{ name: 'Acme Corp - Legal', role: 'Contract Review', project: 'Contract Review', channel: 'Email', time: '13:47', preview: 'Please review the attached contract' },
		{ name: 'Accountant', role: 'Finance', project: 'VAT & Taxes', channel: 'WhatsApp', time: '12:21', unread: '3', preview: 'We need the VAT report for Q2' },
		{ name: 'IRIS Team', role: 'Team Channel', project: 'Project Updates', channel: 'Telegram', time: '11:08', preview: 'Alex: Updated the roadmap for v2.0' },
		{ name: 'GitHub', role: 'Hermes Hub', project: 'Hermes Hub', channel: 'Email', time: 'Yesterday', preview: 'Pull request #128 was merged' }
	];

	const contactList: Person[] = [
		{ name: 'John Smith', role: 'CEO', company: 'Smith & Partners', status: 'Online' },
		{ name: 'Maria Petrova', role: 'Lead Designer', company: 'Acme Corp', channel: 'Telegram' },
		{ name: 'Michael Brown', role: 'CTO', company: 'TechFlow Inc.', status: 'Online' },
		{ name: 'Elena Rodriguez', role: 'Project Manager', company: 'IRIS Solutions', status: 'Online' },
		{ name: 'David Wilson', role: 'Product Owner', company: 'Acme Corp', channel: 'Email' },
		{ name: 'Anna Becker', role: 'Marketing Director', company: 'Vision Labs', status: 'Online' },
		{ name: 'Accountant', role: 'Finance', company: 'Personal', channel: 'WhatsApp' },
		{ name: 'IRIS Team', role: 'Team Channel', company: 'IRIS Solution', channel: 'Telegram' }
	];

	const projects: ProjectItem[] = [
		{ name: 'Hermes Hub', kind: 'Product Development', progress: 75, tasks: 23, icon: 'tabler:cube', tone: 'cyan' },
		{ name: 'Acme Integration', kind: 'Client Project', progress: 45, tasks: 12, icon: 'tabler:cube', tone: 'blue' },
		{ name: 'Q3 Marketing Campaign', kind: 'Marketing', progress: 60, tasks: 17, icon: 'tabler:hexagon', tone: 'purple' },
		{ name: 'Personal Finance', kind: 'Personal Project', progress: 30, tasks: 8, icon: 'tabler:home-dollar', tone: 'mint' }
	];

	const tasks: TaskItem[] = [
		{ title: 'Review Q2 financial report', tracker: 'Jira Cloud', project: 'Hermes Hub', assignee: 'Maria Petrova', status: 'In Review', priority: 'High', due: 'Today 14:00', group: 'Due Today' },
		{ title: 'Fix authentication flow bug', tracker: 'YouTrack', project: 'Platform Core', assignee: 'Alex Morgan', status: 'In Progress', priority: 'High', due: 'Today 16:00', group: 'Due Today' },
		{ title: 'Prepare design system update', tracker: 'ClickUp', project: 'Design System', assignee: 'Elena Rodriguez', status: 'To Do', priority: 'Medium', due: 'Today 18:00', group: 'Due Today' },
		{ title: 'Implement plugin architecture', tracker: 'Jira Cloud', project: 'Hermes Hub', assignee: 'John Smith', status: 'In Progress', priority: 'High', due: 'May 16', group: 'This Week' },
		{ title: 'API rate limiting', tracker: 'YouTrack', project: 'Backend Services', assignee: 'Alex Morgan', status: 'To Do', priority: 'Medium', due: 'May 16', group: 'This Week' },
		{ title: 'Update user documentation', tracker: 'ClickUp', project: 'Documentation', assignee: 'Maria Petrova', status: 'In Review', priority: 'Medium', due: 'May 17', group: 'This Week' },
		{ title: 'Setup monitoring alerts', tracker: 'Jira Cloud', project: 'DevOps', assignee: 'John Smith', status: 'To Do', priority: 'Medium', due: 'May 17', group: 'This Week' },
		{ title: 'Refactor notification module', tracker: 'YouTrack', project: 'Platform Core', assignee: 'Elena Rodriguez', status: 'In Progress', priority: 'Low', due: 'May 18', group: 'This Week' },
		{ title: 'Mobile app dark mode', tracker: 'Jira Cloud', project: 'Mobile App', assignee: 'Maria Petrova', status: 'To Do', priority: 'Low', due: 'May 24', group: 'Later' }
	];

	const documents = [
		{ name: 'Hermes_Hub_Architecture_v1.2.pdf', source: 'Google Drive', project: 'Hermes Hub', type: 'PDF', date: 'May 13, 2024', size: '2.4 MB', icon: 'tabler:file-type-pdf', tone: 'red' },
		{ name: 'Product_Roadmap_2024.xlsx', source: 'OneDrive', project: 'Hermes Hub', type: 'Excel', date: 'May 12, 2024', size: '1.1 MB', icon: 'tabler:file-spreadsheet', tone: 'green' },
		{ name: 'Meeting_Notes_Design_System.md', source: 'Dropbox', project: 'Design System', type: 'Markdown', date: 'May 9, 2024', size: '45 KB', icon: 'tabler:file-text', tone: 'blue' },
		{ name: 'Contract_Acme_Corp_v2.pdf', source: 'Google Drive', project: 'Acme Integration', type: 'PDF', date: 'May 10, 2024', size: '1.8 MB', icon: 'tabler:file-type-pdf', tone: 'red' },
		{ name: 'User_Research_Summary.pdf', source: 'Notion', project: 'Website Redesign', type: 'PDF', date: 'May 7, 2024', size: '3.2 MB', icon: 'tabler:file-description', tone: 'slate' },
		{ name: 'API_Documentation_v1.0.pdf', source: 'Dropbox', project: 'Platform Core', type: 'PDF', date: 'May 6, 2024', size: '5.7 MB', icon: 'tabler:file-type-pdf', tone: 'red' },
		{ name: 'Q2_Financial_Report.xlsx', source: 'OneDrive', project: 'Finance', type: 'Excel', date: 'May 5, 2024', size: '980 KB', icon: 'tabler:file-spreadsheet', tone: 'green' }
	];

	const notes = [
		{ title: 'Hermes Hub - Product Strategy', body: 'Основные принципы: единое пространство памяти, интеграция всех коммуникаций...', source: 'Apple Notes', tag: '#project', time: '10:42', icon: 'tabler:notes' },
		{ title: 'User Research Summary', body: 'Ключевые инсайты из интервью с пользователями...', source: 'Obsidian', tag: '#research', time: '09:15', icon: 'tabler:file-text' },
		{ title: 'Meeting with Maria - 13 May 2024', body: 'Обсудили roadmap, приоритеты и сроки запуска новых функций...', source: 'Gmail', tag: '#meeting', time: '08:27', icon: 'tabler:brand-gmail' },
		{ title: 'Quick Ideas', body: '- AI для автоматической категоризации заметок - Граф связей между проектами...', source: 'Anytype', tag: '#idea', time: '07:58', icon: 'tabler:bulb' },
		{ title: 'Integration Architecture', body: 'Схема интеграции с внешними сервисами и потоками данных...', source: 'Obsidian', tag: '#reference', time: 'May 12, 18:45', icon: 'tabler:file-text' },
		{ title: 'Email: Partnership Opportunity', body: 'Интересное предложение о партнерстве. Нужно обсудить с командой...', source: 'Outlook', tag: '#partnership', time: 'May 12, 16:20', icon: 'tabler:mail' }
	];

	const weekColumns = ['MON 12', 'TUE 13', 'WED 14', 'THU 15', 'FRI 16', 'SAT 17', 'SUN 18'];
	const calendarBlocks = [
		{ day: 0, top: 24, height: 52, title: 'Team Standup', meta: 'Google Calendar', tone: 'blue' },
		{ day: 0, top: 118, height: 70, title: 'Project Hermes Planning', meta: 'Microsoft 365', tone: 'green' },
		{ day: 1, top: 70, height: 70, title: 'Focus Time', meta: 'Microsoft 365', tone: 'green' },
		{ day: 1, top: 168, height: 52, title: 'Platform Core Sync', meta: 'YouTrack', tone: 'purple' },
		{ day: 2, top: 24, height: 52, title: 'Product Review', meta: 'Google Calendar', tone: 'blue' },
		{ day: 2, top: 118, height: 62, title: 'Engineering Sync', meta: 'Microsoft 365', tone: 'green' },
		{ day: 3, top: 56, height: 72, title: 'YouTrack: Daily Standup', meta: 'YouTrack', tone: 'purple' },
		{ day: 3, top: 166, height: 74, title: 'Architecture Discussion', meta: 'Microsoft 365', tone: 'green' },
		{ day: 4, top: 24, height: 58, title: 'All Hands', meta: 'Google Calendar', tone: 'blue' },
		{ day: 4, top: 140, height: 62, title: 'Sprint Planning', meta: 'YouTrack', tone: 'purple' },
		{ day: 5, top: 24, height: 298, title: 'Hackathon', meta: 'Personal', tone: 'amber' }
	];

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
	const v4ClosureCapabilities = $derived(
		v4Capabilities?.capabilities.filter((capability) => capability.closure_gate) ?? []
	);
	const v4BlockedCapabilities = $derived(
		v4Capabilities?.capabilities.filter((capability) => capability.status === 'blocked') ?? []
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
	const v5ClosureCapabilities = $derived(
		v5Capabilities?.capabilities.filter((capability) => capability.closure_gate) ?? []
	);
	const v5BlockedCapabilities = $derived(
		v5Capabilities?.capabilities.filter((capability) => capability.status === 'blocked') ?? []
	);
	const selectedConversation = $derived(conversations[selectedConversationIndex] ?? conversations[0]);
	const selectedContact = $derived(contactList[selectedContactIndex] ?? contactList[0]);
	const agentCards = $derived(aiAgents.map(agentCardView));
	const selectedAgent = $derived(agentCards[selectedAgentIndex] ?? agentCards[0] ?? null);
	const activeView = $derived(viewCopy[currentView]);
	const activeShortcuts = $derived(shortcutsByView[currentView]);
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
				item.candidate_kind === 'merge_contacts' &&
				item.review_state === 'user_confirmed' &&
				!confirmedSplitCandidateForMerge(item)
		)
	);
	const settingsByCategory = $derived(groupSettingsByCategory(applicationSettings));
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

	onMount(() => {
		void loadV1Status();
		void loadGraphSummary();
		void loadGraphNodeChoices();
		void loadCommunications();
		void loadDocumentProcessingJobs();
		void loadProjects();
		void loadIdentityCandidates();
		void loadTaskReviewState();
		void loadAiWorkspace();
		void loadTelegramWorkspace();
		void loadWhatsappWebWorkspace();
		void loadSettingsWorkspace();
	});

	async function loadV1Status() {
		try {
			status = await fetchV1Status(apiBaseUrl, apiToken, actorId);
			statusError = '';
		} catch (error) {
			statusError = error instanceof Error ? error.message : 'Unknown status error';
		}
	}

	async function loadSettingsWorkspace() {
		isSettingsLoading = true;
		try {
			const [settingsResponse, accountsResponse] = await Promise.all([
				fetchApplicationSettings(apiBaseUrl, apiToken, actorId),
				fetchProviderAccounts(apiBaseUrl, apiToken, actorId)
			]);
			applicationSettings = settingsResponse.items;
			providerAccounts = accountsResponse.items;
			applyLoadedFrontendSettings(settingsResponse.items);
			settingDrafts = Object.fromEntries(
				settingsResponse.items.map((setting) => [setting.setting_key, settingDraftValue(setting)])
			);
			settingsError = '';
		} catch (error) {
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
				apiBaseUrl,
				apiToken,
				actorId,
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
			settingsActionMessage = `${updated.label} saved`;
			settingsError = '';
			if (updated.setting_key.startsWith('ai.')) {
				void loadAiWorkspace();
			}
			if (updated.setting_key === 'frontend.actor_id') {
				applyLoadedFrontendSettings([updated]);
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
			graphSummary = await fetchGraphSummary(apiBaseUrl, apiToken, actorId);
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
			const nodes = await fetchGraphNodes(apiBaseUrl, apiToken, actorId, 20);
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
			const results = await searchGraphNodes(apiBaseUrl, apiToken, actorId, query, 20);
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
				apiBaseUrl,
				apiToken,
				actorId,
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

	async function loadCommunications() {
		isCommunicationsLoading = true;
		try {
			const response = await fetchCommunicationMessages(apiBaseUrl, apiToken, actorId, 50);
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
			const response = await fetchProjects(apiBaseUrl, apiToken, actorId, 25);
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
				fetchTaskCandidates(apiBaseUrl, apiToken, actorId, 50),
				fetchTasks(apiBaseUrl, apiToken, actorId, 50)
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

	async function loadAiWorkspace() {
		isAiLoading = true;
		try {
			const [agentResponse, runResponse] = await Promise.all([
				fetchAiAgents(apiBaseUrl, apiToken, actorId),
				fetchAiRuns(apiBaseUrl, apiToken, actorId, 25)
			]);
			aiAgents = agentResponse.items;
			aiRuns = runResponse.items;
			if (selectedAgentIndex >= aiAgents.length) {
				selectedAgentIndex = 0;
			}
			aiError = '';
			try {
				aiStatus = await fetchAiStatus(apiBaseUrl, apiToken, actorId);
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
			aiAnswerResult = await requestAiAnswer(apiBaseUrl, apiToken, actorId, {
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
			aiTaskRefreshResult = await refreshAiTaskCandidates(apiBaseUrl, apiToken, actorId, {
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
			aiMeetingPrepResult = await requestAiMeetingPrep(apiBaseUrl, apiToken, actorId, {
				command_id: `ai-meeting-prep-${crypto.randomUUID()}`,
				topic,
				project_id: projectId
			});
			currentView = 'agents';
			await loadAiRunsOnly();
		} catch (error) {
			aiError = error instanceof Error ? error.message : 'Unknown AI meeting prep error';
		} finally {
			isAiMeetingPrepSubmitting = false;
		}
	}

	async function loadAiRunsOnly() {
		try {
			const response = await fetchAiRuns(apiBaseUrl, apiToken, actorId, 25);
			aiRuns = response.items;
		} catch (error) {
			aiError = error instanceof Error ? error.message : 'Unknown AI run history error';
		}
	}

	async function loadIdentityCandidates() {
		isIdentityCandidatesLoading = true;
		try {
			const response = await fetchIdentityCandidates(apiBaseUrl, apiToken, actorId, 50);
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
			const response = await fetchDocumentProcessingJobs(apiBaseUrl, apiToken, actorId, 50);
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
				apiBaseUrl,
				apiToken,
				actorId,
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
			await retryDocumentProcessingJob(apiBaseUrl, apiToken, actorId, job.job_id, {
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
		candidate: ContactIdentityCandidate,
		reviewState: ContactIdentityReviewState
	) {
		try {
			await reviewIdentityCandidate(
				apiBaseUrl,
				apiToken,
				actorId,
				candidate.identity_candidate_id,
				reviewState
			);
			await loadIdentityCandidates();
		} catch (error) {
			identityCandidatesError =
				error instanceof Error ? error.message : 'Unknown identity review error';
		}
	}

	async function splitConfirmedIdentityMerge(candidate: ContactIdentityCandidate) {
		const splitCandidate = splitCandidateForConfirmedMerge(candidate);
		if (!splitCandidate) {
			return;
		}

		const commandId = `contact-identity-split-${Date.now()}-${candidate.identity_candidate_id}`;
		try {
			await reviewIdentityCandidate(
				apiBaseUrl,
				apiToken,
				actorId,
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
			await reviewTaskCandidate(apiBaseUrl, apiToken, actorId, candidate.task_candidate_id, reviewState);
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
			const detail = await fetchProjectDetail(apiBaseUrl, apiToken, actorId, projectId);
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
				apiBaseUrl,
				apiToken,
				actorId,
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
		currentView = 'agents';
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

	function applyLoadedFrontendSettings(settings: ApplicationSetting[]) {
		const configuredActorId = stringSettingValue(settings, 'frontend.actor_id');
		if (configuredActorId) {
			actorId = configuredActorId;
		}
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

	function stringSettingValue(settings: ApplicationSetting[], settingKey: string) {
		const value = settings.find((setting) => setting.setting_key === settingKey)?.value;
		return typeof value === 'string' && value.trim() ? value.trim() : '';
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

	function setView(item: NavItem) {
		if (!item.enabled) {
			return;
		}
		currentView = item.id;
		searchQuery = '';
	}

	function openAccountDrawer() {
		isAccountDrawerOpen = true;
		setupMessage = '';
		setupError = '';
	}

	function closeAccountDrawer() {
		isAccountDrawerOpen = false;
	}

	function selectProvider(provider: Provider) {
		selectedProvider = provider;
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

	async function startGmailSetup() {
		isSetupSubmitting = true;
		setupMessage = '';
		setupError = '';

		try {
			gmailPending = await startGmailOAuthSetup(apiBaseUrl, apiToken, actorId, {
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
			const result = await completeGmailOAuthSetup(apiBaseUrl, apiToken, actorId, {
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
			const result = await setupImapAccount(apiBaseUrl, apiToken, actorId, {
				account_id: imapForm.account_id,
				provider_kind: selectedProvider === 'icloud' ? 'icloud' : 'imap',
				display_name: imapForm.display_name,
				external_account_id: imapForm.external_account_id,
				host: imapForm.host,
				port: Number(imapForm.port),
				tls: imapForm.tls,
				mailbox: imapForm.mailbox,
				username: imapForm.username,
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
					fetchV4Capabilities(apiBaseUrl, apiToken, actorId),
					fetchTelegramChats(apiBaseUrl, apiToken, actorId),
					fetchTelegramMessages(apiBaseUrl, apiToken, actorId),
					fetchAutomationTemplates(apiBaseUrl, apiToken, actorId),
					fetchAutomationPolicies(apiBaseUrl, apiToken, actorId),
					fetchTelegramCalls(apiBaseUrl, apiToken, actorId)
				]);

			v4Capabilities = capabilityResponse;
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
				fetchV5Capabilities(apiBaseUrl, apiToken, actorId),
				fetchWhatsappWebSessions(apiBaseUrl, apiToken, actorId),
				fetchWhatsappWebMessages(apiBaseUrl, apiToken, actorId)
			]);

			v5Capabilities = capabilityResponse;
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
		try {
			const result = await setupWhatsappWebFixtureAccount(apiBaseUrl, apiToken, actorId, {
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
			await Promise.all([loadWhatsappWebWorkspace(), loadSettingsWorkspace()]);
		} catch (error) {
			whatsappError = error instanceof Error ? error.message : 'WhatsApp Web fixture setup failed';
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
			const result = await ingestWhatsappWebFixtureMessage(apiBaseUrl, apiToken, actorId, {
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

	async function setupTelegramFixture() {
		if (isTelegramActionSubmitting) {
			return;
		}

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		try {
			const result = await setupTelegramFixtureAccount(apiBaseUrl, apiToken, actorId, {
				account_id: telegramAccountForm.account_id,
				provider_kind: telegramAccountForm.provider_kind,
				display_name: telegramAccountForm.display_name,
				external_account_id: telegramAccountForm.external_account_id,
				tdlib_data_path: telegramAccountForm.tdlib_data_path || undefined,
				transcription_enabled: telegramAccountForm.transcription_enabled
			});
			telegramActionMessage = `${providerKindLabel(result.provider_kind)} account ${result.account_id} saved`;
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
			telegramError = error instanceof Error ? error.message : 'Telegram fixture setup failed';
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
			const result = await ingestTelegramFixtureMessage(apiBaseUrl, apiToken, actorId, {
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

	async function saveV4AutomationTemplate() {
		if (isTelegramActionSubmitting) {
			return;
		}

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		try {
			const template = await saveAutomationTemplate(apiBaseUrl, apiToken, actorId, {
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

	async function saveV4AutomationPolicy() {
		if (isTelegramActionSubmitting) {
			return;
		}

		isTelegramActionSubmitting = true;
		telegramActionMessage = '';
		telegramError = '';
		try {
			const policy = await saveAutomationPolicy(apiBaseUrl, apiToken, actorId, {
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
			const result = await dryRunTelegramSend(apiBaseUrl, apiToken, actorId, {
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
			const call = await saveTelegramCall(apiBaseUrl, apiToken, actorId, {
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
				apiBaseUrl,
				apiToken,
				actorId,
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
			const response = await fetchCallTranscript(apiBaseUrl, apiToken, actorId, callId);
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
			{ ...selected, x: 50, y: 50, isSelected: true },
			...neighbors.map((node, index) => {
				const angle = (Math.PI * 2 * index) / Math.max(neighbors.length, 1) - Math.PI / 2;
				return {
					...node,
					x: 50 + Math.cos(angle) * radius,
					y: 50 + Math.sin(angle) * radius,
					isSelected: false
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

	function taskSourceLabel(item: TaskCandidate | ActiveTask) {
		return `${item.source_kind[0].toUpperCase()}${item.source_kind.slice(1)} · ${item.source_id}`;
	}

	function taskConfidence(item: TaskCandidate) {
		return `${Math.round(item.confidence * 100)}%`;
	}

	function identityConfidence(item: ContactIdentityCandidate) {
		return `${Math.round(item.confidence * 100)}%`;
	}

	function splitCandidateForConfirmedMerge(candidate: ContactIdentityCandidate) {
		return splitCandidateForMerge(candidate, 'suggested');
	}

	function confirmedSplitCandidateForMerge(candidate: ContactIdentityCandidate) {
		return splitCandidateForMerge(candidate, 'user_confirmed');
	}

	function splitCandidateForMerge(
		candidate: ContactIdentityCandidate,
		reviewState: ContactIdentityReviewState
	) {
		if (!candidate.right_contact_id) {
			return null;
		}
		const pairKey = contactIdentityPairKey(candidate.left_contact_id, candidate.right_contact_id);
		return (
			identityCandidates.find(
				(item) =>
					item.candidate_kind === 'split_contact' &&
					item.review_state === reviewState &&
					item.right_contact_id !== null &&
					contactIdentityPairKey(item.left_contact_id, item.right_contact_id) === pairKey
			) ?? null
		);
	}

	function contactIdentityPairKey(leftContactId: string, rightContactId: string) {
		return leftContactId <= rightContactId
			? `${leftContactId}:${rightContactId}`
			: `${rightContactId}:${leftContactId}`;
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

<svelte:head>
	<title>Hermes Hub</title>
	<meta name="description" content="Hermes Hub desktop personal OS dashboard." />
</svelte:head>

<main class="desktop-shell view-{currentView}">
	<aside class="sidebar" aria-label="Hermes Hub navigation">
		<div class="brand">
			<img src="/assets/hermes-logo-mark.png" alt="" class="brand-mark" />
			<div>
				<p class="brand-name">Hermes Hub</p>
				<p class="brand-subtitle">Personal OS</p>
			</div>
		</div>

		<nav class="nav-group" aria-label="Primary">
			{#each primaryNav as item}
				<button
					type="button"
					class:active={currentView === item.id}
					class:disabled={!item.enabled}
					disabled={!item.enabled}
					title={item.enabled ? item.label : `${item.label} is not available in the current desktop scope`}
					onclick={() => setView(item)}
				>
					<Icon icon={item.icon} width="18" height="18" />
					<span>{item.label}</span>
					{#if item.badge}
						<em>{item.badge}</em>
					{/if}
				</button>
			{/each}
		</nav>

		<div class="nav-separator"></div>

		<section class="shortcuts" aria-label="Shortcuts">
			<p>Shortcuts</p>
			<nav class="nav-group">
				{#each activeShortcuts as item}
					<button type="button" class="shortcut" disabled title={`${item.label} shortcut is not implemented yet`}>
						<Icon icon={item.icon} width="18" height="18" />
						<span>{item.label}</span>
						{#if item.badge}
							<em>{item.badge}</em>
						{/if}
					</button>
				{/each}
			</nav>
		</section>

		<div class="profile-card">
			<img src="/assets/hermes-reference-avatar.png" alt="Alex Morgan" />
			<div>
				<strong>Alex Morgan</strong>
				<span>Focus Mode</span>
			</div>
			<Icon icon="tabler:chevron-down" width="16" height="16" />
		</div>

		<div class="sidebar-tools" aria-label="Settings shortcuts">
			<button type="button" class:active={currentView === 'settings'} title="Open settings" onclick={() => (currentView = 'settings')}>
				<Icon icon="tabler:settings" width="18" height="18" />
			</button>
			<button type="button" disabled title="Help is not available yet">
				<Icon icon="tabler:help-circle" width="18" height="18" />
			</button>
			<button type="button" disabled title="Apps are not available yet">
				<Icon icon="tabler:layout-grid" width="18" height="18" />
			</button>
		</div>
	</aside>

	<section class="workspace" aria-label={`${activeView.title} workspace`}>
		<header class="topbar">
			<label class="search-box">
				<Icon icon="tabler:search" width="18" height="18" />
				<input bind:value={searchQuery} placeholder={activeView.search} aria-label={activeView.search} />
				<span class="kbd">⌘ K</span>
			</label>
			<div class="top-actions">
				<button type="button" disabled>
					<Icon icon="tabler:terminal-2" width="16" height="16" />
					Command Palette
					<span class="kbd">⌘ P</span>
				</button>
				<button type="button" class="icon-button" disabled title="Notifications are not implemented yet">
					<Icon icon="tabler:bell" width="18" height="18" />
					<i>2</i>
				</button>
				<button type="button" class="avatar-button" onclick={openAccountDrawer} title="Open account setup">
					<img src="/assets/hermes-logo-mark.png" alt="" />
				</button>
			</div>
		</header>

		{#if currentView === 'home'}
			<section class="home-page">
				<div class="hero-row">
					<div class="greeting">
						<div class="hero-mark"><img src="/assets/hermes-logo-mark.png" alt="" /></div>
						<div>
							<h1>{activeView.title}</h1>
							<p>{activeView.subtitle}</p>
						</div>
					</div>
					<div class="metric-grid home-metrics">
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
							<span>Focus Score</span>
							<div class="score-ring"><strong>78</strong></div>
							<small>Good ↑ 5</small>
						</article>
					</div>
				</div>

				<div class="dashboard-grid">
					<section class="panel feed-panel">
						<header class="panel-title-row">
							<div>
								<h2>What's New</h2>
								<p>Key changes and important updates</p>
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

					<section class="panel priorities-panel">
						<header class="panel-title-row">
							<div>
								<h2>Today's Priorities</h2>
								<p>Focus on what matters most</p>
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

					<section class="panel schedule-panel">
						<header class="panel-title-row">
							<div>
								<h2>Upcoming</h2>
								<p>Your schedule</p>
							</div>
						</header>
						<div class="schedule-list">
							<article><time>Today, May 12</time><strong>14:00 Call with Acme Corp</strong><span>16:30 Review Q2 Report</span></article>
							<article><time>Tomorrow, May 13</time><strong>10:00 Project Hermes - Planning</strong><span>15:00 Design Review</span></article>
						</div>
						<button type="button" class="link-row" disabled>View full calendar <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
					</section>

					<aside class="stacked-rail">
						<section class="panel mini-panel">
							<header class="panel-title-row"><h2>People You Talked To</h2><button type="button" class="link-button" disabled>View all</button></header>
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
						<section class="panel mini-panel">
							<header class="panel-title-row"><h2>System Status</h2></header>
							<ul class="status-list">
								<li class:online={status}>All systems operational</li>
								<li>AI Agents online <span>5/5</span></li>
								<li>Data synchronized <span>2m ago</span></li>
								<li>Local AI models <span>Ready</span></li>
							</ul>
							{#if statusError}<p class="inline-error">{statusError}</p>{/if}
						</section>
					</aside>
				</div>

				<section class="panel full-band">
					<header class="panel-title-row">
						<h2>Active Projects</h2>
						<button type="button" class="link-button" onclick={() => (currentView = 'projects')}>View all projects</button>
					</header>
					<div class="project-card-row">
						{#each projects as project}
							<article class="compact-project">
								<span class="round-icon {project.tone}"><Icon icon={project.icon} width="20" height="20" /></span>
								<div>
									<strong>{project.name}</strong>
									<small>{project.kind}</small>
								</div>
								<div class="progress"><span style={`width: ${project.progress}%`}></span></div>
								<em>{project.progress}%</em>
							</article>
						{/each}
						<button type="button" class="new-tile" disabled><Icon icon="tabler:plus" width="22" height="22" />New Project</button>
					</div>
				</section>
			</section>
		{:else if currentView === 'communications'}
			<section class="communications-page">
				<div class="view-header">
					<div>
						<h1>{activeView.title}</h1>
						<p>{activeView.subtitle}</p>
					</div>
					<div class="header-actions">
						<button type="button" class="segmented active"><Icon icon="tabler:message" width="16" height="16" /></button>
						<button type="button" class="segmented" disabled><Icon icon="tabler:layout-grid" width="16" height="16" /></button>
						<button type="button" class="primary-button" disabled>New Message</button>
					</div>
				</div>
				<div class="filter-tabs">
					<button type="button" class="active">All <em>{communicationMessages.length}</em></button>
					<button type="button" disabled>People <em>0</em></button>
					<button type="button" disabled>Unread <em>0</em></button>
					<button type="button" disabled>Requires Reply <em>0</em></button>
					<button type="button" disabled>Waiting <em>0</em></button>
					<button type="button" disabled>More <Icon icon="tabler:chevron-down" width="14" height="14" /></button>
				</div>
				<div class="three-pane communications-grid">
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
									<time>{messageTime(message)}</time>
									{#if message.attachment_count > 0}<b>{message.attachment_count}</b>{/if}
								</button>
							{/each}
						{/if}
					</section>
					<section class="panel chat-pane">
						{#if selectedCommunication}
							<header>
								<img src="/assets/hermes-reference-avatar.png" alt="" />
								<div><h2>{senderLabel(selectedCommunication.sender)}</h2><p>{selectedCommunication.subject}</p></div>
								<div class="chat-actions">
									<button type="button" onclick={() => void askAiAboutSelectedMessage()} disabled={isAiAnswerSubmitting}><Icon icon="tabler:sparkles" width="17" height="17" /></button>
									<button type="button" disabled><Icon icon="tabler:phone" width="17" height="17" /></button>
									<button type="button" disabled><Icon icon="tabler:video" width="17" height="17" /></button>
									<button type="button" disabled><Icon icon="tabler:info-circle" width="17" height="17" /></button>
								</div>
							</header>
							<div class="chat-body">
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
					<aside class="context-rail">
						<section class="panel profile-panel">
							<div class="profile-head"><img src="/assets/hermes-reference-avatar.png" alt="" /><div><h2>{selectedCommunication ? senderLabel(selectedCommunication.sender) : 'No sender selected'}</h2><p>{selectedCommunication ? communicationChannelLabel(selectedCommunication.channel_kind) : 'No channel'}</p><small>{selectedCommunication ? senderEmail(selectedCommunication.sender) : 'No local message selected'}</small></div></div>
							<div class="quick-icons">
								<button type="button" disabled><Icon icon="tabler:mail" width="17" height="17" /></button>
								<button type="button" disabled><Icon icon="tabler:phone" width="17" height="17" /></button>
								<button type="button" disabled><Icon icon="tabler:brand-telegram" width="17" height="17" /></button>
								<button type="button" disabled><Icon icon="tabler:brand-whatsapp" width="17" height="17" /></button>
							</div>
						</section>
						<section class="panel info-card"><h2>Summary</h2><p>{selectedCommunication ? `Stored from ${selectedCommunication.account_id}. Channel ${communicationChannelLabel(selectedCommunication.channel_kind)}. Provider record ${selectedCommunication.provider_record_id}.` : 'Local communication metadata will appear after messages are imported.'}</p><button type="button" class="link-row" disabled>View full profile <Icon icon="tabler:arrow-right" width="15" height="15" /></button></section>
						<section class="panel info-card"><h2>Message Metadata</h2>{#if selectedCommunication}<ul class="detail-list"><li><Icon icon="tabler:users" width="17" height="17" /> {selectedCommunication.recipients.length} recipients</li><li><Icon icon="tabler:paperclip" width="17" height="17" /> {selectedCommunication.attachment_count} attachments</li><li><Icon icon="tabler:clock" width="17" height="17" /> {messageTime(selectedCommunication)}</li></ul>{:else}<p>No message selected.</p>{/if}</section>
						<section class="panel info-card"><h2>Related Projects</h2>{#each projects.slice(0, 2) as project}<div class="related-row"><span class="round-icon {project.tone}"><Icon icon={project.icon} width="16" height="16" /></span><strong>{project.name}</strong><em>{project.progress}%</em></div>{/each}</section>
						<section class="panel info-card"><h2>Active Tasks</h2>{#each tasks.slice(0, 3) as task}<label class="mini-check"><input type="checkbox" />{task.title}<em>{task.due.split(' ')[0]}</em></label>{/each}</section>
					</aside>
				</div>
			</section>
		{:else if currentView === 'contacts'}
			<section class="contacts-page">
				<div class="contacts-layout">
					<section class="panel contacts-list-panel">
						<header>
							<div><h1>Contacts</h1><p>642 contacts</p></div>
							<button type="button" class="primary-button" disabled>New Contact</button>
						</header>
						<div class="filter-tabs compact">
							<button type="button" class="active">All</button>
							<button type="button" disabled>People <em>532</em></button>
							<button type="button" disabled>Companies <em>110</em></button>
						</div>
						<label class="local-search"><Icon icon="tabler:search" width="17" height="17" /><input placeholder="Search contacts..." /></label>
						{#each contactList as contact, index}
							<button type="button" class="contact-row" class:active={selectedContactIndex === index} onclick={() => (selectedContactIndex = index)}>
								<img src="/assets/hermes-reference-avatar.png" alt="" />
								<span><strong>{contact.name}</strong><small>{contact.role}</small><em>{contact.company}</em></span>
								<small>{contact.status ?? contact.channel ?? 'Email'}</small>
							</button>
						{/each}
					</section>
					<section class="contact-detail">
						<header class="contact-hero panel">
							<img src="/assets/hermes-reference-avatar.png" alt="" />
							<div><h1>{selectedContact.name}</h1><p>{selectedContact.role} at {selectedContact.company}</p><small>Online</small></div>
							<div class="chat-actions">
								<button type="button" disabled><Icon icon="tabler:mail" width="17" height="17" /></button>
								<button type="button" disabled><Icon icon="tabler:phone" width="17" height="17" /></button>
								<button type="button" disabled><Icon icon="tabler:video" width="17" height="17" /></button>
								<button type="button" disabled><Icon icon="tabler:brand-whatsapp" width="17" height="17" /></button>
							</div>
						</header>
						<div class="section-tabs">
							<button type="button" class="active">Overview</button>
							<button type="button" disabled>Communications</button>
							<button type="button" disabled>Documents <em>24</em></button>
							<button type="button" disabled>Tasks <em>7</em></button>
							<button type="button" disabled>Projects <em>5</em></button>
							<button type="button" disabled>Notes</button>
						</div>
						<div class="contact-cards">
							<section class="panel info-card">
								<h2>Contact Information</h2>
								<ul class="detail-list">
									<li><Icon icon="tabler:mail" width="17" height="17" /> jsmith@smithpartners.com <em>Work</em></li>
									<li><Icon icon="tabler:phone" width="17" height="17" /> +1 (555) 123-4567 <em>Mobile</em></li>
									<li><Icon icon="tabler:brand-telegram" width="17" height="17" /> @john.smith <em>Telegram</em></li>
									<li><Icon icon="tabler:map-pin" width="17" height="17" /> New York, USA <em>Local Time: 18:42</em></li>
								</ul>
							</section>
							<section class="panel info-card"><h2>About</h2><p>John is a strategic consulting partner. We have been working together since 2021 on multiple projects including Hermes Hub and IRIS platform development.</p><div class="tag-cloud"><span>Decision Maker</span><span>Executive</span><span>Strategic</span><span>Tech Enthusiast</span></div></section>
							<section class="panel info-card"><h2>Relationship Strength</h2><div class="big-score">85</div><strong>Strong</strong><p>Last interaction 2 hours ago</p></section>
							<section class="panel info-card span-2"><h2>Recent Interactions</h2>{#each whatsNew.slice(0, 3) as item}<div class="feed-row compact-row"><span class="round-icon {item.tone}"><Icon icon={item.icon} width="18" height="18" /></span><div><strong>{item.title}</strong><p>{item.meta}</p></div><time>{item.time}</time></div>{/each}</section>
							<section class="panel info-card"><h2>Active Projects</h2>{#each projects.slice(0, 3) as project}<div class="related-row"><span class="round-icon {project.tone}"><Icon icon={project.icon} width="16" height="16" /></span><strong>{project.name}</strong><em>{project.progress}%</em></div>{/each}</section>
						</div>
					</section>
					<aside class="stacked-rail">
						<section class="panel info-card"><h2>AI Summary</h2><p>John is a key strategic partner and decision maker. You have a strong professional relationship with frequent communication across multiple projects.</p></section>
						<section class="panel info-card">
							<h2>Contact Identity Review</h2>
							<p class="identity-note">Contact merges are only suggested and are not applied until confirmed.</p>
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
											<small>Left: {candidate.left_contact_id}</small>
											<small>Right: {candidate.right_contact_id ?? 'N/A'}</small>
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
											<small>Left: {candidate.left_contact_id}</small>
											<small>Right: {candidate.right_contact_id ?? 'N/A'}</small>
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
						<section class="panel info-card"><h2>Related Documents</h2>{#each documents.slice(0, 4) as doc}<div class="doc-mini"><Icon icon={doc.icon} width="20" height="20" /><span><strong>{doc.name}</strong><small>{doc.size} · {doc.date}</small></span></div>{/each}</section>
						<section class="panel info-card"><h2>Recent Notes</h2><p>Discussed expansion to EU market</p><p>Prefers email for official communication</p><p>Interested in AI/ML integration</p></section>
					</aside>
				</div>
			</section>
		{:else if currentView === 'projects'}
			<section class="projects-page">
				{#if projectsError && !selectedProjectRecord}
					<section class="panel info-card project-empty-state">
						<Icon icon="tabler:alert-circle" width="28" height="28" />
						<h2>Projects unavailable</h2>
						<p>{projectsError}</p>
						<button type="button" onclick={() => void loadProjects()}>Retry</button>
					</section>
				{:else if !selectedProjectRecord}
					<section class="panel info-card project-empty-state">
						<Icon icon="tabler:cube" width="30" height="30" />
						<h2>No projects returned</h2>
						<p>{isProjectsLoading ? 'Loading local projects...' : 'Local project records are empty.'}</p>
					</section>
				{:else}
					<header class="project-hero panel">
						<div class="project-logo"><Icon icon="tabler:cube" width="48" height="48" /></div>
						<div>
							<h1>{selectedProjectRecord.name} <em>{projectStatusLabel(selectedProjectRecord.status)}</em></h1>
							<p>{selectedProjectRecord.kind}</p>
							<small>{selectedProjectRecord.description}</small>
						</div>
						<button type="button" class="primary-button" onclick={() => void prepareAiBrief(selectedProjectRecord.project_id)} disabled={isAiMeetingPrepSubmitting}><Icon icon="tabler:calendar-stats" width="16" height="16" />Prepare brief</button>
					</header>
					<div class="project-meta-strip panel">
						<article><span>Owner</span><strong>{selectedProjectRecord.owner_display_name}</strong></article>
						<article><span>People</span><strong>{formatNumber(selectedProjectStats.people_count)}</strong></article>
						<article><span>Start Date</span><strong>{formatProjectDate(selectedProjectRecord.start_date)}</strong></article>
						<article><span>Target Date</span><strong>{formatProjectDate(selectedProjectRecord.target_date)}</strong></article>
						<article><span>Progress</span><div class="progress"><span style={`width: ${selectedProjectRecord.progress_percent}%`}></span></div><strong>{selectedProjectRecord.progress_percent}%</strong></article>
					</div>
					{#if projectSummaries.length > 1}
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
					{/if}
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
					{#if projectsError}
						<p class="inline-error">{projectsError}</p>
					{/if}
					<div class="project-dashboard-grid">
						<section class="panel info-card">
							<h2>Project Summary</h2>
							<div class="summary-numbers">
								<article><strong>{formatNumber(selectedProjectStats.document_count)}</strong><span>Documents</span></article>
								<article><strong>{formatNumber(selectedProjectStats.message_count)}</strong><span>Messages</span></article>
								<article><strong>{formatNumber(selectedProjectStats.people_count)}</strong><span>People</span></article>
								<article><strong>{formatNumber(selectedProjectStats.graph_connection_count)}</strong><span>Graph links</span></article>
							</div>
						</section>
						<section class="panel graph-card-large">
							<h2>Knowledge Graph</h2>
							<div class="radial-graph">
								<div class="graph-center"><Icon icon="tabler:cube" width="30" height="30" /><span>{selectedProjectRecord.name}</span></div>
								<span class="graph-chip" style="--x:18%; --y:24%">Messages {formatNumber(selectedProjectStats.message_count)}</span>
								<span class="graph-chip" style="--x:68%; --y:22%">Documents {formatNumber(selectedProjectStats.document_count)}</span>
								<span class="graph-chip" style="--x:18%; --y:68%">People {formatNumber(selectedProjectStats.people_count)}</span>
								<span class="graph-chip" style="--x:66%; --y:70%">Links {formatNumber(selectedProjectStats.graph_connection_count)}</span>
							</div>
						</section>
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
						<section class="panel info-card">
							<h2>Source Evidence</h2>
							<div class="summary-numbers compact">
								<article><strong>{formatNumber(selectedProjectStats.message_count + selectedProjectStats.document_count)}</strong><span>Matched records</span></article>
								<article><strong>{formatProjectDateTime(selectedProjectStats.latest_activity_at)}</strong><span>Last activity</span></article>
							</div>
						</section>
						<section class="panel info-card">
							<h2>Open Promises</h2>
							<p class="muted-copy">No task candidates connected to this project.</p>
							<button type="button" class="link-row" disabled>View all promises <Icon icon="tabler:arrow-right" width="15" height="15" /></button>
						</section>
						<aside class="stacked-rail project-side">
							<section class="panel info-card">
								<h2>Project Health</h2>
								<div class="health-row"><span>Status</span><strong>{projectStatusLabel(selectedProjectRecord.status)}</strong></div>
								<div class="health-row"><span>Progress</span><strong>{selectedProjectRecord.progress_percent}%</strong></div>
								<div class="health-row"><span>Graph Links</span><strong>{formatNumber(selectedProjectStats.graph_connection_count)}</strong></div>
							</section>
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
						</aside>
					</div>
				{/if}
			</section>
		{:else if currentView === 'tasks'}
			<section class="tasks-page">
				<div class="view-header">
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:hexagon" width="28" height="28" /></span><div><h1>{activeView.title}</h1><p>{activeView.subtitle}</p></div></div>
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
					<button type="button" class="primary-button" onclick={() => void refreshTasksFromAi()} disabled={isAiTaskRefreshSubmitting}><Icon icon="tabler:sparkles" width="16" height="16" />AI refresh</button>
				</div>
				{#if tasksError}
					<p class="inline-error">{tasksError}</p>
				{/if}
				<div class="tasks-layout">
					<section class="panel task-table">
						<h3 class="task-group">Active Tasks <em>{activeTasks.length}</em></h3>
							<div class="table-head task-table-head"><span>Task</span><span>Source</span><span>Project</span><span>Created</span><span>Status</span></div>
						{#if isTasksLoading}
							<p class="inline-copy">Loading task state…</p>
						{:else if activeTasks.length === 0}
							<p class="inline-copy">No active tasks yet.</p>
						{:else}
							{#each activeTasks as item}
								<label class="task-row"><input type="checkbox" disabled checked /><strong>{item.title}</strong><span>{taskSourceLabel(item)}</span><span>{item.project_id ?? 'Unassigned'}</span><time>{taskCreatedTime(item.created_at)}</time><em>{item.status}</em></label>
							{/each}
						{/if}

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
					</section>
					<aside class="stacked-rail">
						<section class="panel chart-panel"><h2>Review Stats</h2><div class="donut"><strong>{taskCandidates.length}</strong><span>Suggestions</span></div><ul><li>{`${suggestedTaskCandidates.length} Suggested`}</li><li>{`${activeTasks.length} Active`}</li><li>{`${taskCandidates.length - suggestedTaskCandidates.length - activeTasks.length} Done`}</li></ul></section>
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
						<section class="panel info-card"><h2>Active Task Sources</h2>{#each ['message','document'] as source}<div class="bar-row"><span>{source}</span><div><i></i></div></div>{/each}</section>
					</aside>
				</div>
			</section>
		{:else if currentView === 'calendar'}
			<section class="calendar-page">
				<div class="view-header">
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:calendar" width="28" height="28" /></span><div><h1>{activeView.title}</h1><p>{activeView.subtitle}</p></div></div>
					<div class="section-tabs pill-tabs"><button type="button" disabled>Day</button><button type="button" class="active">Week</button><button type="button" disabled>Month</button><button type="button" disabled>Agenda</button></div>
					<button type="button" class="primary-button" disabled>New Event</button>
				</div>
				<div class="filter-bar"><button type="button" disabled>All Accounts (8)</button><button type="button" disabled>All Calendars (24)</button><button type="button" disabled>All Event Types</button><button type="button" disabled>Filters</button></div>
				<div class="calendar-layout">
					<section class="panel week-board">
						<div class="week-header">{#each weekColumns as day}<strong>{day}</strong>{/each}</div>
						<div class="time-grid">
							{#each calendarBlocks as block}
								<article class="event-block {block.tone}" style={`--day:${block.day}; --top:${block.top}px; --height:${block.height}px`}><strong>{block.title}</strong><span>{block.meta}</span></article>
							{/each}
							<div class="now-line"><span>11:42</span></div>
						</div>
						<footer>Legend: <span>Google Calendar</span><span>Microsoft 365</span><span>YouTrack</span><span>Personal</span></footer>
					</section>
					<aside class="stacked-rail">
						<section class="panel info-card"><h2>Upcoming Events</h2>{#each ['1:1 with Maria', 'Roadmap Review', 'Product Review', 'Engineering Sync', 'Architecture Discussion'] as event, index}<div class="deadline"><span>{index < 2 ? 'Today' : 'Tomorrow'} · {event}</span><time>{index + 9}:00</time></div>{/each}</section>
						<section class="panel info-card"><h2>Calendars</h2>{#each ['Google Work', 'Google Personal', 'Microsoft Work', 'YouTrack Events'] as item}<label class="mini-check"><input type="checkbox" checked />{item}<em></em></label>{/each}<button type="button" class="link-row" disabled>Add Calendar</button></section>
						<section class="panel info-card"><h2>Time Insights</h2>{#each ['Meetings 18h 30m', 'Focus Time 12h 15m', 'Personal 8h 45m', 'Other 3h 30m'] as item}<div class="bar-row"><span>{item}</span><div><i></i></div></div>{/each}</section>
					</aside>
				</div>
			</section>
		{:else if currentView === 'documents'}
			<section class="documents-page">
				<div class="view-header">
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:file-text" width="28" height="28" /></span><div><h1>{activeView.title}</h1><p>{activeView.subtitle}</p></div></div>
					<button type="button" class="primary-button" disabled>Upload</button>
				</div>
				<div class="source-strip">
					{#each ['Google Drive 1,243', 'OneDrive 812', 'Dropbox 342', 'Notion 256'] as source, index}
						<article class="source-card"><Icon icon={index === 0 ? 'tabler:brand-google-drive' : index === 1 ? 'tabler:cloud' : index === 2 ? 'tabler:brand-dropbox' : 'tabler:brand-notion'} width="28" height="28" /><span>{source}</span></article>
					{/each}
					<button type="button" class="source-card add" disabled><Icon icon="tabler:plus" width="20" height="20" />Add Source</button>
				</div>
				<div class="filter-bar"><button type="button" disabled>All Accounts</button><button type="button" disabled>All Types</button><button type="button" disabled>All Projects</button><button type="button" disabled>All Folders</button><button type="button" disabled>Filters</button></div>
				<div class="docs-layout">
					<aside class="left-panels"><section class="panel info-card"><h2>Smart Collections</h2>{#each ['Recently Added 48', 'Recently Opened 24', 'Important 32', 'Shared with Me 18', 'Requires Review 7', 'Contracts & Legal 23', 'Financial 15'] as item}<div class="collection-row">{item}</div>{/each}</section><section class="panel info-card"><h2>My Folders</h2>{#each ['Hermes Hub', 'Projects', 'Personal', 'Work', 'Archive 2024', 'Clients', 'References'] as folder}<div class="collection-row"><Icon icon="tabler:folder" width="16" height="16" />{folder}</div>{/each}</section></aside>
					<section class="panel docs-table">
						<header><h2>Hermes Hub</h2><div class="section-tabs"><button type="button" class="active">Overview</button><button type="button" disabled>Documents <em>142</em></button><button type="button" disabled>Folders <em>16</em></button><button type="button" disabled>Links <em>28</em></button><button type="button" disabled>Activity</button></div></header>
						<div class="category-grid">{#each ['Architecture 23 documents','Product 31 documents','Design 18 documents','Meetings 24 documents','Contracts 12 documents','Research 15 documents','Reports 11 documents','Other 8 documents'] as category}<article><Icon icon="tabler:folder" width="20" height="20" /><span>{category}</span></article>{/each}</div>
						<div class="table-head docs"><span>Name</span><span>Source</span><span>Project</span><span>Type</span><span>Last Modified</span><span>Size</span></div>
						{#each documents as doc}
							<div class="doc-row"><Icon icon={doc.icon} width="19" height="19" /><strong>{doc.name}</strong><span>{doc.source}</span><span>{doc.project}</span><span>{doc.type}</span><time>{doc.date}</time><em>{doc.size}</em></div>
						{/each}
					</section>
					<aside class="stacked-rail">
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
						<section class="panel chart-panel"><h2>Documents Insights</h2><strong>2,653</strong><span>Total Documents</span><div class="donut small"><strong>24%</strong></div></section>
						<section class="panel info-card"><h2>Document Types</h2>{#each ['PDF 1,234 (46%)', 'Documents 623 (23%)', 'Spreadsheets 312 (12%)', 'Presentations 198 (7%)', 'Images 142 (5%)'] as item}<div class="bar-row"><span>{item}</span><div><i></i></div></div>{/each}</section>
						<section class="panel info-card"><h2>Recent Activity</h2>{#each contactList.slice(1,5) as person}<div class="person-compact"><img src="/assets/hermes-reference-avatar.png" alt="" /><span><strong>{person.name}</strong><small>updated a document</small></span></div>{/each}</section>
					</aside>
				</div>
			</section>
		{:else if currentView === 'notes'}
			<section class="notes-page">
				<div class="notes-layout">
					<aside class="left-panels">
						<section class="panel info-card"><h2>Sources</h2>{#each ['Apple Notes 1,243','Obsidian 872','Anytype 532','Gmail 1,156','Outlook 623'] as source}<div class="collection-row">{source}</div>{/each}<button type="button" class="link-row" disabled>Add Source</button></section>
						<section class="panel info-card"><h2>Collections</h2>{#each ['Inbox 231','Starred 128','Today 89','To Review 74','Personal 312','Projects 482','Ideas 156','Research 203','Archive 1,024'] as item}<div class="collection-row">{item}</div>{/each}</section>
						<section class="panel info-card"><h2>Tags</h2><div class="tag-cloud"><span># project 342</span><span># idea 156</span><span># meeting 213</span><span># research 182</span><span># reference 98</span><span># design 76</span></div></section>
					</aside>
					<section class="notes-main">
						<div class="view-header">
							<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:notes" width="28" height="28" /></span><div><h1>Notes</h1><p>All your notes from connected sources</p></div></div>
							<div class="metric-grid inline-metrics tiny">{#each [{label:'Total Notes',value:'4,426',delta:'18%'},{label:'This Week',value:'128',delta:'12%'},{label:'Unprocessed',value:'89',delta:'5%'}] as metric}<article class="metric-card"><span>{metric.label}</span><strong>{metric.value}</strong><small>↑ {metric.delta}</small></article>{/each}</div>
							<button type="button" class="primary-button" disabled>New Note</button>
						</div>
						<div class="filter-bar"><button type="button" disabled>All Sources</button><button type="button" disabled>All Types</button><button type="button" disabled>All Collections</button><button type="button" disabled>All Tags</button><button type="button" disabled>Filters</button><button type="button" disabled>Sort: Updated</button></div>
						<section class="notes-list panel">
							<h3>Today</h3>{#each notes.slice(0,4) as note}<article><Icon icon={note.icon} width="22" height="22" /><div><strong>{note.title}</strong><p>{note.body}</p><small>{note.source} · {note.time}</small></div><em>{note.tag}</em></article>{/each}
							<h3>Yesterday</h3>{#each notes.slice(4) as note}<article><Icon icon={note.icon} width="22" height="22" /><div><strong>{note.title}</strong><p>{note.body}</p><small>{note.source} · {note.time}</small></div><em>{note.tag}</em></article>{/each}
						</section>
					</section>
					<aside class="stacked-rail">
						<section class="panel chart-panel"><h2>Notes Insights</h2><div class="donut"><strong>4,426</strong><span>Total Notes</span></div></section>
						<section class="panel info-card"><h2>Activity</h2>{#each ['You created a note','Maria Petrova shared a note','Email processed','Note linked to project'] as item}<div class="deadline"><span>{item}</span><time>10:42</time></div>{/each}</section>
						<section class="panel info-card"><h2>Unprocessed Items</h2>{#each ['23 Emails','34 Apple Notes','12 Attachments','8 Web Clippings'] as item}<div class="collection-row">{item}</div>{/each}<button type="button" class="link-row" disabled>Process All</button></section>
					</aside>
				</div>
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
									<p>Import contacts, messages or documents, then run the existing projection smoke command to create graph data.</p>
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
								</svg>
								{#each graphCanvasEdges as edge}
									<span
										class="graph-edge-label"
										style={`left:${(edge.x1 + edge.x2) / 2}%; top:${(edge.y1 + edge.y2) / 2}%`}
									>
										{edge.label}
									</span>
								{/each}
								{#each graphCanvasNodes as node}
									<button
										type="button"
										class="graph-node"
										class:kind-person={node.node_kind === 'person'}
										class:kind-email_address={node.node_kind === 'email_address'}
										class:kind-message={node.node_kind === 'message'}
										class:kind-document={node.node_kind === 'document'}
										class:selected={node.isSelected}
										style={`left:${node.x}%; top:${node.y}%`}
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

						<footer class="graph-status-bar">
							<span>Projection: {formatGraphTimestamp(graphSummary?.latest_projection_at ?? null)}</span>
							<span>Evidence: {formatNumber(graphEvidenceTotal())}</span>
							{#if graphNeighborhood?.truncated}<span>Neighborhood truncated at {graphNeighborhood.edge_limit} edges</span>{/if}
							{#if graphNeighborhood?.evidence_truncated}<span>Evidence truncated at {graphNeighborhood.evidence_limit} rows</span>{/if}
						</footer>
					</section>

					<aside class="stacked-rail">
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
					</aside>
				</div>
			</section>
		{:else if currentView === 'telegram'}
			<section class="telegram-page communications-page">
				<div class="view-header">
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:brand-telegram" width="28" height="28" /></span><div><h1>{activeView.title}</h1><p>{activeView.subtitle}</p></div></div>
					<button type="button" class="primary-button" onclick={() => void loadTelegramWorkspace()} disabled={isTelegramLoading}><Icon icon="tabler:refresh" width="16" height="16" />Refresh</button>
				</div>

				<div class="metric-grid">
					<article class="metric-card"><span>Chats</span><strong>{telegramChats.length}</strong><small>{selectedTelegramChat?.sync_state ?? 'not synced'}</small></article>
					<article class="metric-card"><span>Messages</span><strong>{telegramMessages.length}</strong><small>Projected channel records</small></article>
					<article class="metric-card"><span>Templates</span><strong>{automationTemplates.length}</strong><small>UI-approved only</small></article>
					<article class="metric-card"><span>Policies</span><strong>{automationPolicies.length}</strong><small>{automationPolicies.filter((policy) => policy.enabled).length} enabled</small></article>
					<article class="metric-card"><span>Calls</span><strong>{telegramCalls.length}</strong><small>{selectedTelegramCall?.call_state ?? 'no history'}</small></article>
					<article class="metric-card"><span>Transcript</span><strong>{callTranscript?.transcript_status ?? 'none'}</strong><small>{callTranscript?.stt_provider ?? 'fixture STT'}</small></article>
				</div>

				{#if telegramActionMessage}
					<p class="setup-state success">{telegramActionMessage}</p>
				{/if}
				{#if telegramError}
					<p class="inline-error">{telegramError}</p>
				{/if}

				<div class="three-pane communications-grid telegram-grid">
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

					<section class="panel chat-pane telegram-chat-pane">
						{#if selectedTelegramChat}
							<header>
								<span class="round-icon cyan"><Icon icon="tabler:brand-telegram" width="24" height="24" /></span>
								<div><h2>{selectedTelegramChat.title}</h2><p>{selectedTelegramChat.account_id} · {selectedTelegramChat.provider_chat_id}</p></div>
								<div class="chat-actions">
									<button type="button" disabled title="1:1 audio call controls are backend-foundation only in this V4 slice"><Icon icon="tabler:phone" width="17" height="17" /></button>
									<button type="button" disabled title="Video calls are V4.x"><Icon icon="tabler:video" width="17" height="17" /></button>
									<button type="button" onclick={() => void loadTelegramWorkspace()} disabled={isTelegramLoading}><Icon icon="tabler:refresh" width="17" height="17" /></button>
								</div>
							</header>
							<div class="chat-body">
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

					<aside class="stacked-rail telegram-rail">
						<section class="panel info-card">
							<h2>Account Setup</h2>
							<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void setupTelegramFixture(); }}>
								<label><span>Account ID</span><input bind:value={telegramAccountForm.account_id} autocomplete="off" /></label>
								<label><span>Provider</span><select bind:value={telegramAccountForm.provider_kind}><option value="telegram_user">User</option><option value="telegram_bot">Bot</option></select></label>
								<label><span>Display name</span><input bind:value={telegramAccountForm.display_name} autocomplete="off" /></label>
								<label><span>External ID</span><input bind:value={telegramAccountForm.external_account_id} autocomplete="off" /></label>
								<label class="wide"><span>TDLib data path</span><input bind:value={telegramAccountForm.tdlib_data_path} autocomplete="off" /></label>
								<label class="checkbox-row"><input bind:checked={telegramAccountForm.transcription_enabled} type="checkbox" /><span>Transcription enabled</span></label>
								<div class="form-actions"><button type="submit" disabled={isTelegramActionSubmitting}>Save Fixture</button></div>
							</form>
						</section>

						<section class="panel info-card">
							<h2>Runtime Guardrails</h2>
							<div class="health-row"><span>Mode</span><strong>{v4Capabilities?.runtime_mode ?? 'unknown'}</strong></div>
							{#if v4ClosureCapabilities.length}
								<ul class="detail-list">
									{#each v4ClosureCapabilities as capability}
										<li>{capabilityLabel(capability.capability)}<em>{capability.status}</em></li>
									{/each}
								</ul>
							{:else}
								<p>Capability contract is not loaded yet.</p>
							{/if}
							{#if v4BlockedCapabilities.length}
								<div class="evidence-row">
									<strong>Blocked Live Runtime</strong>
									<p>{v4BlockedCapabilities.map((capability) => capabilityLabel(capability.capability)).join(', ')}</p>
								</div>
							{/if}
							{#if v4Capabilities?.unsupported_features.length}
								<div class="evidence-row">
									<strong>V4.x Scope</strong>
									<p>{v4Capabilities.unsupported_features.map(capabilityLabel).join(', ')}</p>
								</div>
							{/if}
						</section>

						<section class="panel info-card">
							<h2>Template</h2>
							<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void saveV4AutomationTemplate(); }}>
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
							<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void saveV4AutomationPolicy(); }}>
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
					</aside>
				</div>
			</section>
		{:else if currentView === 'whatsapp'}
			<section class="whatsapp-page communications-page">
				<div class="view-header">
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:brand-whatsapp" width="28" height="28" /></span><div><h1>{activeView.title}</h1><p>{activeView.subtitle}</p></div></div>
					<button type="button" class="primary-button" onclick={() => void loadWhatsappWebWorkspace()} disabled={isWhatsappLoading}><Icon icon="tabler:refresh" width="16" height="16" />Refresh</button>
				</div>

				<div class="metric-grid">
					<article class="metric-card"><span>Sessions</span><strong>{whatsappSessions.length}</strong><small>{selectedWhatsappSession?.link_state ?? 'not linked'}</small></article>
					<article class="metric-card"><span>Messages</span><strong>{whatsappMessages.length}</strong><small>Canonical WhatsApp Web records</small></article>
					<article class="metric-card"><span>Runtime</span><strong>{v5Capabilities?.runtime_mode ?? 'unknown'}</strong><small>Fixture/manual foundation</small></article>
					<article class="metric-card"><span>Blocked</span><strong>{v5BlockedCapabilities.length}</strong><small>Live runtime remains blocked</small></article>
				</div>

				{#if whatsappActionMessage}
					<p class="setup-state success">{whatsappActionMessage}</p>
				{/if}
				{#if whatsappError}
					<p class="inline-error">{whatsappError}</p>
				{/if}

				<div class="three-pane communications-grid whatsapp-grid">
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

					<section class="panel chat-pane whatsapp-chat-pane">
						{#if selectedWhatsappSession}
							<header>
								<span class="round-icon cyan"><Icon icon="tabler:brand-whatsapp" width="24" height="24" /></span>
								<div><h2>{selectedWhatsappSession.device_name}</h2><p>{selectedWhatsappSession.account_id} · {selectedWhatsappSession.link_state}</p></div>
								<div class="chat-actions">
									<button type="button" disabled title="Live WhatsApp Web runtime is blocked in V5 foundation"><Icon icon="tabler:world" width="17" height="17" /></button>
									<button type="button" disabled title="Outbound WhatsApp sends require a future policy and runtime contract"><Icon icon="tabler:send-off" width="17" height="17" /></button>
									<button type="button" onclick={() => void loadWhatsappWebWorkspace()} disabled={isWhatsappLoading}><Icon icon="tabler:refresh" width="17" height="17" /></button>
								</div>
							</header>
							<div class="chat-body">
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

					<aside class="stacked-rail whatsapp-rail">
						<section class="panel info-card">
							<h2>Account Setup</h2>
							<form class="setup-form compact-form" onsubmit={(event) => { event.preventDefault(); void setupWhatsappWebFixture(); }}>
								<label><span>Account ID</span><input bind:value={whatsappAccountForm.account_id} autocomplete="off" /></label>
								<label><span>Display name</span><input bind:value={whatsappAccountForm.display_name} autocomplete="off" /></label>
								<label><span>External ID</span><input bind:value={whatsappAccountForm.external_account_id} autocomplete="off" /></label>
								<label><span>Device name</span><input bind:value={whatsappAccountForm.device_name} autocomplete="off" /></label>
								<label class="wide"><span>Local state path</span><input bind:value={whatsappAccountForm.local_state_path} autocomplete="off" /></label>
								<div class="form-actions wide"><button type="submit" disabled={isWhatsappActionSubmitting}>Save Fixture</button></div>
							</form>
						</section>

						<section class="panel info-card">
							<h2>Runtime Guardrails</h2>
							<div class="health-row"><span>Mode</span><strong>{v5Capabilities?.runtime_mode ?? 'unknown'}</strong></div>
							{#if v5ClosureCapabilities.length}
								<ul class="detail-list">
									{#each v5ClosureCapabilities as capability}
										<li>{capabilityLabel(capability.capability)}<em>{capability.status}</em></li>
									{/each}
								</ul>
							{:else}
								<p>Capability contract is not loaded yet.</p>
							{/if}
							{#if v5BlockedCapabilities.length}
								<div class="evidence-row">
									<strong>Live Scope</strong>
									<p>{v5BlockedCapabilities.map((capability) => capabilityLabel(capability.capability)).join(', ')}</p>
								</div>
							{/if}
							{#if v5Capabilities?.unsupported_features.length}
								<div class="evidence-row">
									<strong>Unsupported</strong>
									<p>{v5Capabilities.unsupported_features.map(capabilityLabel).join(', ')}</p>
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
					</aside>
				</div>
			</section>
		{:else if currentView === 'settings'}
			<section class="settings-page">
				<div class="view-header">
					<div class="view-title-with-icon">
						<span class="hero-mark small"><Icon icon="tabler:settings" width="28" height="28" /></span>
						<div><h1>{activeView.title}</h1><p>{activeView.subtitle}</p></div>
					</div>
					<button type="button" class="primary-button" onclick={() => void loadSettingsWorkspace()} disabled={isSettingsLoading}>
						<Icon icon="tabler:refresh" width="16" height="16" />Refresh
					</button>
				</div>

				<div class="metric-grid settings-metrics">
					<article class="metric-card"><span>Settings</span><strong>{applicationSettings.length}</strong><small>Editable runtime values</small></article>
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
					<button type="button" class:active={selectedSettingsSection === 'application'} onclick={() => (selectedSettingsSection = 'application')}>
						<Icon icon="tabler:adjustments-horizontal" width="16" height="16" />Application
					</button>
					<button type="button" class:active={selectedSettingsSection === 'accounts'} onclick={() => (selectedSettingsSection = 'accounts')}>
						<Icon icon="tabler:users" width="16" height="16" />Accounts <em>{providerAccounts.length}</em>
					</button>
				</div>

				{#if selectedSettingsSection === 'application'}
					<div class="settings-layout">
						<section class="panel settings-list-panel">
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

						<aside class="stacked-rail settings-rail">
							<section class="panel info-card">
								<h2>Runtime Source</h2>
								<div class="health-row"><span>Backend bind</span><strong>{settingValueText('server.http_addr')}</strong></div>
								<div class="health-row"><span>Frontend API</span><strong>{settingValueText('frontend.api_base_url')}</strong></div>
								<div class="health-row"><span>Actor</span><strong>{settingValueText('frontend.actor_id')}</strong></div>
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
				{:else}
					<div class="settings-account-layout">
						<section class="panel account-section">
							<header class="panel-title-row">
								<div><h2>Mail Accounts</h2><p>Gmail OAuth, iCloud app-password and generic IMAP records.</p></div>
								<button type="button" class="primary-button" onclick={openAccountDrawer}><Icon icon="tabler:plus" width="16" height="16" />Add Mail</button>
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
								<div><h2>Telegram Accounts</h2><p>User and bot accounts used by Telegram ingestion and automation policies.</p></div>
								<button type="button" class="primary-button" onclick={() => (currentView = 'telegram')}><Icon icon="tabler:brand-telegram" width="16" height="16" />Setup</button>
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
								<button type="button" class="primary-button" onclick={() => (currentView = 'whatsapp')}><Icon icon="tabler:brand-whatsapp" width="16" height="16" />Setup</button>
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
					<div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:robot" width="28" height="28" /></span><div><h1>{activeView.title}</h1><p>{activeView.subtitle}</p></div></div>
					<button type="button" class="primary-button" onclick={() => void loadAiWorkspace()} disabled={isAiLoading}><Icon icon="tabler:refresh" width="16" height="16" />Refresh</button>
				</div>
				<div class="metric-grid agent-metrics">
					<article class="metric-card"><span>Runtime</span><strong>{aiRuntimeSummary()}</strong><small>{aiStatus?.version ? `Ollama ${aiStatus.version}` : 'Ollama'}</small></article>
					<article class="metric-card"><span>Agents</span><strong>{aiAgents.length}</strong><small>{aiAgents.length ? 'Registered' : 'Not loaded'}</small></article>
					<article class="metric-card"><span>Run History</span><strong>{aiRuns.length}</strong><small>Persisted runs</small></article>
					<article class="metric-card"><span>Embedding</span><strong>{aiStatus?.embedding_dimension ?? 0}</strong><small>{aiStatus?.embedding_model ?? 'No model'}</small></article>
					<article class="metric-card"><span>Suggested Tasks</span><strong>{suggestedTaskCandidates.length}</strong><small>Review queue</small></article>
					<article class="metric-card"><span>Latest Duration</span><strong>{formatDuration(aiRuns[0]?.duration_ms)}</strong><small>{aiRuns[0]?.agent_id ?? 'No runs'}</small></article>
				</div>
				{#if aiError}
					<p class="inline-error">{aiError}</p>
				{/if}
				<div class="filter-bar"><button type="button" class="active">Local Agents</button><button type="button" disabled>{aiModelSummary()}</button><button type="button" disabled>{aiStatus?.chat_model_available ? 'Chat model ready' : 'Chat model missing'}</button><button type="button" disabled>{aiStatus?.embedding_model_available ? 'Embedding ready' : 'Embedding missing'}</button></div>
				<div class="agents-layout">
					<section class="agent-main">
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
					</section>
					<aside class="stacked-rail">
						<section class="panel info-card"><h2>Runtime</h2><div class="health-row"><span>Status</span><strong>{aiRuntimeSummary()}</strong></div><div class="health-row"><span>Chat</span><strong>{aiStatus?.chat_model ?? 'unknown'}</strong></div><div class="health-row"><span>Embedding</span><strong>{aiStatus?.embedding_model ?? 'unknown'}</strong></div></section>
						<section class="panel info-card"><h2>Run History</h2>{#if aiRuns.length}{#each aiRuns.slice(0,6) as run}<div class="deadline"><span>{run.agent_id} · {runStatusLabel(run)}</span><time>{formatDateTime(run.started_at)} · {formatDuration(run.duration_ms)}</time></div>{/each}{:else}<p>No AI runs persisted yet.</p>{/if}</section>
						<section class="panel info-card"><h2>Latest Citations</h2>{#if aiRuns[0] && safeCitations(aiRuns[0].citations).length}{#each safeCitations(aiRuns[0].citations).slice(0,3) as citation}<div class="evidence-row"><strong>{citation.title}</strong><p>{citation.excerpt}</p></div>{/each}{:else}<p>Citations appear after an answer or briefing run.</p>{/if}</section>
					</aside>
				</div>
			</section>
		{:else}
			<section class="timeline-page">
				<div class="view-header"><div class="view-title-with-icon"><span class="hero-mark small"><Icon icon="tabler:timeline-event" width="28" height="28" /></span><div><h1>Timeline</h1><p>Chronological activity across connected sources.</p></div></div></div>
				<div class="timeline-layout">
					<section class="panel feed-panel large-timeline">
						<header class="panel-title-row"><h2>Today</h2><button type="button" class="ghost-button" disabled>All Events</button></header>
						{#each whatsNew.concat(whatsNew) as item, index}<article class="timeline-event-row"><time>{18 - index}:42</time><span class="rail-dot"></span><span class="round-icon {item.tone}"><Icon icon={item.icon} width="20" height="20" /></span><div><strong>{item.title}</strong><p>{item.meta}</p>{#if item.tag}<em>{item.tag}</em>{/if}</div></article>{/each}
					</section>
					<aside class="stacked-rail"><section class="panel info-card"><h2>Timeline Filters</h2>{#each ['Messages','Documents','Tasks','Calendar','Notes','Decisions'] as item}<label class="mini-check"><input type="checkbox" checked />{item}</label>{/each}</section></aside>
				</div>
			</section>
		{/if}
	</section>
</main>

{#if isAccountDrawerOpen}
	<button
		type="button"
		class="drawer-backdrop"
		aria-label="Close account setup"
		onclick={closeAccountDrawer}
	></button>
	<aside class="account-drawer" aria-labelledby="account-setup-heading">
		<header>
			<div>
				<p>Provider Accounts</p>
				<h2 id="account-setup-heading">Add Account</h2>
			</div>
			<button type="button" class="icon-button" onclick={closeAccountDrawer} aria-label="Close">
				<Icon icon="tabler:x" width="18" height="18" />
			</button>
		</header>

		<div class="provider-tabs" aria-label="Account provider">
			<button type="button" class:active={selectedProvider === 'gmail'} onclick={() => selectProvider('gmail')}>Gmail</button>
			<button type="button" class:active={selectedProvider === 'icloud'} onclick={() => selectProvider('icloud')}>iCloud</button>
			<button type="button" class:active={selectedProvider === 'imap'} onclick={() => selectProvider('imap')}>Raw IMAP</button>
		</div>

		{#if selectedProvider === 'gmail'}
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

		{#if setupMessage}<p class="setup-state success">{setupMessage}</p>{/if}
		{#if setupError}<p class="setup-state error">{setupError}</p>{/if}
	</aside>
{/if}

<style>
	:global(*) {
		box-sizing: border-box;
	}

	:global(body) {
		margin: 0;
		min-width: 1280px;
		background: #02090b;
		color: #eefefb;
		font-family:
			Inter, 'SF Pro Display', ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont,
			'Segoe UI', sans-serif;
		letter-spacing: 0;
	}

	:global(button),
	:global(input),
	:global(textarea) {
		font: inherit;
		letter-spacing: 0;
	}

	:global(button) {
		border: 0;
		cursor: pointer;
	}

	:global(button:disabled) {
		cursor: not-allowed;
	}

	:global(button:focus-visible),
	:global(input:focus-visible) {
		outline: 1px solid rgba(45, 240, 206, 0.62);
		outline-offset: 2px;
	}

	:global(textarea:focus-visible) {
		outline: 1px solid rgba(45, 240, 206, 0.62);
		outline-offset: 2px;
	}

	.desktop-shell {
		display: grid;
		grid-template-columns: 224px minmax(1040px, 1fr);
		gap: 16px;
		height: 100vh;
		min-height: 760px;
		overflow: hidden;
		padding: 0 14px 16px 0;
		background:
			radial-gradient(circle at 72% 2%, rgba(23, 122, 121, 0.14), transparent 34%),
			linear-gradient(180deg, rgba(7, 28, 32, 0.88), rgba(2, 9, 11, 0.98) 46%),
			#02090b;
	}

	.sidebar {
		display: grid;
		grid-template-rows: auto auto auto 1fr auto auto;
		min-height: 100vh;
		padding: 20px 12px 14px;
		border: 1px solid rgba(37, 224, 197, 0.14);
		border-left: 0;
		border-radius: 0 18px 34px 0;
		background:
			linear-gradient(180deg, rgba(4, 26, 29, 0.96), rgba(2, 13, 16, 0.98)),
			#020d10;
		box-shadow: inset -1px 0 0 rgba(255, 255, 255, 0.03), 18px 0 48px rgba(0, 0, 0, 0.28);
	}

	.brand,
	.profile-card,
	.topbar,
	.top-actions,
	.panel-title-row,
	.view-header,
	.view-title-with-icon,
	.metric-card div,
	.feed-row,
	.related-row,
	.person-compact,
	.doc-mini {
		display: flex;
		align-items: center;
	}

	.brand {
		gap: 12px;
		padding: 2px 8px 28px;
	}

	.brand-mark {
		width: 42px;
		height: 42px;
		object-fit: contain;
		filter: drop-shadow(0 0 16px rgba(37, 224, 197, 0.55));
	}

	.brand-name,
	.brand-subtitle,
	h1,
	h2,
	h3,
	p {
		margin: 0;
	}

	.brand-name {
		color: #f2fffd;
		font-size: 15px;
		font-weight: 600;
		text-transform: uppercase;
	}

	.brand-subtitle {
		margin-top: 2px;
		color: #849ca0;
		font-size: 10px;
		font-weight: 700;
		text-transform: uppercase;
	}

	.nav-group {
		display: grid;
		gap: 5px;
	}

	.nav-group button {
		display: grid;
		grid-template-columns: 22px 1fr auto;
		align-items: center;
		gap: 10px;
		width: 100%;
		min-height: 38px;
		border: 1px solid transparent;
		border-radius: 7px;
		background: transparent;
		color: #c6d7d7;
		padding: 0 9px;
		text-align: left;
	}

	.nav-group button.active {
		border-color: rgba(40, 236, 205, 0.45);
		background: linear-gradient(90deg, rgba(12, 112, 93, 0.62), rgba(8, 54, 54, 0.42));
		box-shadow: inset 0 0 24px rgba(28, 221, 188, 0.14);
		color: #40f3d1;
	}

	.nav-group button.disabled,
	.nav-group button.shortcut {
		opacity: 0.9;
	}

	.nav-group em,
	.filter-tabs em,
	.section-tabs em,
	.kbd {
		border-radius: 999px;
		font-style: normal;
	}

	.nav-group em {
		background: rgba(33, 218, 183, 0.16);
		color: #39f2d0;
		font-size: 11px;
		padding: 3px 8px;
	}

	.nav-separator {
		height: 1px;
		margin: 20px 0 18px;
		background: rgba(129, 202, 194, 0.11);
	}

	.shortcuts {
		padding: 0 8px;
	}

	.shortcuts > p {
		color: #8ea4a6;
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
		margin-bottom: 12px;
	}

	.profile-card {
		gap: 11px;
		margin: 22px -12px 0;
		padding: 12px 22px;
		border-top: 1px solid rgba(78, 157, 151, 0.15);
		border-bottom: 1px solid rgba(78, 157, 151, 0.1);
		background: rgba(9, 30, 33, 0.58);
	}

	.profile-card img,
	.conversation-list img,
	.chat-pane header img,
	.profile-head img,
	.contacts-list-panel img,
	.contact-hero img,
	.person-list img,
	.person-compact img {
		border-radius: 50%;
		object-fit: cover;
	}

	.profile-card img {
		width: 42px;
		height: 42px;
	}

	.profile-card div {
		min-width: 0;
		flex: 1;
	}

	.profile-card strong {
		display: block;
		color: #f2fffd;
		font-size: 13px;
		font-weight: 600;
	}

	.profile-card span {
		color: #29e2c2;
		font-size: 12px;
	}

	.sidebar-tools {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
		padding-top: 14px;
	}

	.sidebar-tools button,
	.icon-button,
	.chat-actions button,
	.quick-icons button,
	.segmented {
		display: inline-grid;
		place-items: center;
		width: 34px;
		height: 34px;
		border: 1px solid rgba(130, 211, 205, 0.15);
		border-radius: 8px;
		background: rgba(5, 24, 27, 0.72);
		color: #b6cdcc;
	}

	.sidebar-tools button.active {
		border-color: rgba(45, 240, 206, 0.42);
		background: rgba(25, 154, 132, 0.2);
		color: #2df0ce;
	}

	.workspace {
		display: grid;
		grid-template-rows: 58px minmax(0, 1fr);
		gap: 10px;
		min-width: 0;
		min-height: 0;
		overflow: hidden;
		padding-top: 15px;
	}

	.topbar {
		justify-content: space-between;
		gap: 18px;
	}

	.search-box,
	.local-search,
	.composer {
		display: flex;
		align-items: center;
		border: 1px solid rgba(111, 205, 195, 0.17);
		background: rgba(9, 31, 35, 0.76);
		color: #8ba4a5;
	}

	.search-box {
		width: min(560px, 44vw);
		height: 38px;
		border-radius: 8px;
		padding: 0 10px;
		box-shadow: inset 0 0 28px rgba(24, 189, 164, 0.04);
	}

	input {
		min-width: 0;
		flex: 1;
		border: 0;
		outline: 0;
		background: transparent;
		color: #edfffc;
	}

	.search-box input,
	.local-search input,
	.composer input {
		padding: 0 10px;
		font-size: 13px;
	}

	.kbd {
		border: 1px solid rgba(145, 214, 206, 0.12);
		background: rgba(255, 255, 255, 0.03);
		color: #90a6a6;
		font-size: 11px;
		line-height: 1;
		padding: 5px 7px;
	}

	.top-actions {
		justify-content: flex-end;
		gap: 9px;
	}

	.top-actions > button:not(.icon-button):not(.avatar-button) {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		height: 38px;
		border: 1px solid rgba(111, 205, 195, 0.16);
		border-radius: 8px;
		background: rgba(6, 23, 27, 0.8);
		color: #ccdedd;
		padding: 0 10px;
		font-size: 12px;
	}

	.icon-button {
		position: relative;
	}

	.icon-button i {
		position: absolute;
		top: -5px;
		right: -5px;
		display: grid;
		place-items: center;
		min-width: 16px;
		height: 16px;
		border-radius: 999px;
		background: #ff3b52;
		color: #fff;
		font-size: 9px;
		font-style: normal;
	}

	.avatar-button {
		display: grid;
		place-items: center;
		width: 38px;
		height: 38px;
		border: 1px solid rgba(45, 240, 206, 0.2);
		border-radius: 50%;
		background: rgba(28, 174, 151, 0.16);
	}

	.avatar-button img {
		width: 31px;
		height: 31px;
		object-fit: contain;
	}

	.panel,
	.metric-card,
	.source-card {
		border: 1px solid rgba(82, 204, 190, 0.16);
		background:
			linear-gradient(180deg, rgba(8, 29, 33, 0.94), rgba(5, 22, 25, 0.9)),
			#06181b;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.035);
	}

	.panel {
		border-radius: 8px;
		overflow: hidden;
	}

	.panel-title-row {
		justify-content: space-between;
		gap: 12px;
		min-height: 48px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.12);
		padding: 0 16px;
	}

	.panel-title-row h2,
	.info-card h2,
	.chart-panel h2 {
		color: #f6fffe;
		font-size: 16px;
		font-weight: 500;
	}

	.panel-title-row p {
		margin-top: 4px;
		color: #8fa8a7;
		font-size: 12px;
	}

	.home-page,
	.communications-page,
	.contacts-page,
	.projects-page,
	.tasks-page,
	.calendar-page,
	.documents-page,
	.notes-page,
	.knowledge-page,
	.agents-page,
	.settings-page,
	.timeline-page {
		min-height: 0;
		overflow: auto;
		padding-right: 2px;
		scrollbar-color: rgba(45, 240, 206, 0.25) transparent;
	}

	.hero-row {
		display: grid;
		grid-template-columns: 300px minmax(640px, 1fr);
		align-items: center;
		gap: 14px;
		min-height: 102px;
	}

	.greeting {
		display: flex;
		align-items: center;
		gap: 16px;
	}

	.hero-mark {
		display: grid;
		place-items: center;
		width: 66px;
		height: 66px;
		border: 1px solid rgba(40, 236, 205, 0.38);
		border-radius: 50%;
		background:
			linear-gradient(180deg, rgba(13, 84, 78, 0.5), rgba(5, 29, 31, 0.5)),
			#041214;
		box-shadow: 0 0 36px rgba(30, 230, 196, 0.2);
		color: #2df0ce;
	}

	.hero-mark.small {
		width: 54px;
		height: 54px;
	}

	.hero-mark img {
		width: 58px;
		height: 58px;
		object-fit: contain;
	}

	h1 {
		color: #ffffff;
		font-size: 24px;
		font-weight: 560;
		line-height: 1.18;
	}

	.greeting p,
	.view-header p,
	.project-hero p,
	.project-hero small,
	.info-card p,
	.chart-panel span,
	.feed-row p,
	.feed-row small,
	.conversation-list small,
	.conversation-list em,
	.chat-pane p,
	.profile-head p,
	.profile-head small,
	.contacts-list-panel p,
	.contacts-list-panel small,
	.contacts-list-panel em,
	.doc-mini small,
	.notes-list p,
	.notes-list small,
	.agent-card p,
	.agent-card footer,
	.detail-list,
	.collection-row {
		color: #91a8a8;
	}

	.greeting p,
	.view-header p {
		margin-top: 8px;
		font-size: 13px;
	}

	.metric-grid {
		display: grid;
		grid-template-columns: repeat(6, minmax(110px, 1fr));
		gap: 10px;
	}

	.home-metrics {
		grid-template-columns: repeat(6, minmax(110px, 1fr));
	}

	.metric-card {
		min-height: 84px;
		border-radius: 8px;
		padding: 13px 12px;
	}

	.metric-card span {
		display: block;
		color: #93a9a9;
		font-size: 10px;
		text-transform: uppercase;
	}

	.metric-card div {
		justify-content: space-between;
		margin-top: 10px;
		color: #25e0c5;
	}

	.metric-card strong {
		color: #ffffff;
		font-size: 24px;
		font-weight: 500;
	}

	.metric-card small {
		display: block;
		margin-top: 5px;
		color: #2ef1cd;
		font-size: 11px;
	}

	.score-ring,
	.big-score,
	.donut {
		display: grid;
		place-items: center;
		border: 4px solid rgba(45, 235, 204, 0.82);
		border-right-color: rgba(42, 139, 167, 0.8);
		border-bottom-color: rgba(236, 183, 70, 0.8);
		border-radius: 50%;
	}

	.score-ring {
		width: 48px;
		height: 48px;
		margin: 0 !important;
	}

	.score-ring strong {
		font-size: 14px;
	}

	.dashboard-grid {
		display: grid;
		grid-template-columns: minmax(380px, 1.05fr) minmax(300px, 0.83fr) minmax(280px, 0.76fr) 270px;
		gap: 12px;
		min-height: 456px;
	}

	.feed-list,
	.task-stack,
	.schedule-list,
	.person-list,
	.status-list,
	.info-card,
	.chart-panel {
		display: grid;
		gap: 10px;
		padding: 14px;
	}

	.feed-row {
		display: grid;
		grid-template-columns: 36px minmax(0, 1fr) auto;
		gap: 12px;
		min-height: 64px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.08);
	}

	.feed-row:last-child {
		border-bottom: 0;
	}

	.feed-row strong,
	.related-row strong,
	.person-compact strong,
	.doc-mini strong,
	.deadline span,
	.collection-row,
	.notes-list strong,
	.agent-card strong {
		color: #f6fffe;
		font-size: 13px;
		font-weight: 560;
	}

	.feed-row p {
		margin-top: 4px;
		font-size: 12px;
	}

	.feed-row em,
	.tag-cloud span,
	.task-stack em,
	.task-row em,
	.task-row b,
	.project-hero em,
	.agent-card em,
	.notes-list em {
		display: inline-block;
		border-radius: 5px;
		background: rgba(33, 218, 183, 0.14);
		color: #35edce;
		font-size: 10px;
		font-style: normal;
		padding: 3px 7px;
	}

	.feed-row time,
	.related-row em,
	.deadline time,
	.doc-row time,
	.doc-row em,
	.task-row time {
		color: #a6bbbb;
		font-size: 11px;
		font-style: normal;
	}

	.round-icon {
		display: grid;
		place-items: center;
		width: 34px;
		height: 34px;
		border-radius: 50%;
		background: rgba(28, 199, 197, 0.17);
		color: #2eeed0;
	}

	.round-icon.blue {
		background: rgba(31, 138, 214, 0.2);
		color: #30d8ff;
	}

	.round-icon.green,
	.round-icon.mint {
		background: rgba(35, 204, 142, 0.18);
		color: #2df0a4;
	}

	.round-icon.purple {
		background: rgba(161, 87, 220, 0.22);
		color: #cf9dff;
	}

	.round-icon.amber,
	.round-icon.orange {
		background: rgba(236, 183, 70, 0.18);
		color: #eeb84b;
	}

	.round-icon.pink {
		background: rgba(236, 93, 157, 0.2);
		color: #ff9ccd;
	}

	.round-icon.red {
		background: rgba(236, 70, 70, 0.18);
		color: #ff6969;
	}

	.link-row {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		min-height: 34px;
		background: transparent;
		color: #2ef1cd;
		font-size: 12px;
	}

	.link-button,
	.ghost-button {
		background: transparent;
		color: #9bb1b0;
		font-size: 11px;
	}

	.ghost-button {
		height: 30px;
		border: 1px solid rgba(45, 240, 206, 0.18);
		border-radius: 6px;
		background: rgba(33, 167, 144, 0.1);
		color: #3ae9cb;
		padding: 0 10px;
	}

	.task-stack label,
	.mini-check {
		display: grid;
		grid-template-columns: 18px minmax(0, 1fr) auto;
		align-items: center;
		gap: 10px;
		color: #d9eeec;
		font-size: 12px;
	}

	input[type='checkbox'] {
		width: 15px;
		height: 15px;
		margin: 0;
		accent-color: #27d8bd;
	}

	.task-stack small {
		display: block;
		margin-top: 4px;
		color: #91a8a8;
		font-size: 11px;
	}

	.high {
		background: rgba(226, 55, 70, 0.17) !important;
		color: #ff6b74 !important;
	}

	.schedule-list article {
		display: grid;
		gap: 8px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.08);
		padding-bottom: 14px;
	}

	.schedule-list time {
		color: #2ef1cd;
		font-size: 12px;
	}

	.schedule-list strong,
	.schedule-list span {
		color: #f5fffe;
		font-size: 13px;
		font-weight: 500;
	}

	.stacked-rail,
	.left-panels {
		display: grid;
		gap: 12px;
		align-content: start;
		min-width: 0;
	}

	.status-list {
		margin: 0;
		list-style: none;
	}

	.person-list article {
		display: grid;
		grid-template-columns: 44px minmax(0, 1fr) 20px;
		align-items: center;
		gap: 10px;
		min-height: 56px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.08);
	}

	.person-list img {
		width: 44px;
		height: 44px;
	}

	.person-list strong,
	.person-list small {
		display: block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.person-list strong {
		color: #fff;
		font-size: 13px;
		font-weight: 560;
	}

	.person-list small {
		margin-top: 4px;
		color: #91a8a8;
		font-size: 11px;
	}

	.status-list li {
		position: relative;
		display: flex;
		justify-content: space-between;
		padding-left: 14px;
		color: #2df0ce;
		font-size: 12px;
	}

	.status-list li::before {
		position: absolute;
		top: 6px;
		left: 0;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #2df0ce;
		content: '';
	}

	.inline-error {
		color: #ff9fa7 !important;
		font-size: 11px !important;
		line-height: 1.4;
	}

	.full-band {
		margin-top: 12px;
	}

	.project-card-row {
		display: grid;
		grid-template-columns: repeat(5, minmax(0, 1fr));
		gap: 12px;
		padding: 14px;
	}

	.compact-project,
	.new-tile {
		display: grid;
		grid-template-columns: 38px 1fr;
		align-items: center;
		gap: 10px;
		min-height: 98px;
		border: 1px solid rgba(111, 205, 195, 0.12);
		border-radius: 8px;
		background: rgba(12, 40, 44, 0.52);
		padding: 12px;
	}

	.compact-project strong {
		color: #f7fffd;
		font-size: 13px;
	}

	.compact-project small {
		display: block;
		color: #91a8a8;
		font-size: 11px;
	}

	.progress {
		grid-column: 2;
		height: 6px;
		border-radius: 999px;
		background: rgba(66, 130, 126, 0.22);
		overflow: hidden;
	}

	.progress span {
		display: block;
		height: 100%;
		border-radius: inherit;
		background: linear-gradient(90deg, #27d0b3, #54f0d4);
	}

	.compact-project em {
		grid-column: 2;
		color: #cffff6;
		font-size: 12px;
		font-style: normal;
	}

	.new-tile {
		grid-template-columns: 1fr;
		place-items: center;
		color: #dcefed;
		background: rgba(3, 17, 20, 0.38);
		border-style: dashed;
	}

	.view-header {
		justify-content: space-between;
		gap: 18px;
		min-height: 64px;
		margin-bottom: 12px;
		padding: 0 12px;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.primary-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		height: 38px;
		min-width: 118px;
		border-radius: 8px;
		background: linear-gradient(180deg, rgba(34, 183, 160, 0.82), rgba(15, 106, 94, 0.88));
		color: #eafffb;
		font-size: 13px;
		font-weight: 560;
	}

	.segmented.active,
	.filter-tabs button.active,
	.section-tabs button.active,
	.graph-filter-tabs button.active,
	.filter-bar button.active {
		border-color: rgba(45, 240, 206, 0.48);
		background: rgba(25, 154, 132, 0.24);
		color: #2df0ce;
	}

	.filter-tabs,
	.filter-bar,
	.section-tabs,
	.graph-filter-tabs {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 42px;
		margin-bottom: 12px;
		padding: 4px 10px;
		border: 1px solid rgba(82, 204, 190, 0.12);
		border-radius: 8px;
		background: rgba(5, 22, 25, 0.62);
	}

	.filter-tabs button,
	.filter-bar button,
	.section-tabs button,
	.graph-filter-tabs button {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		height: 32px;
		border: 1px solid transparent;
		border-radius: 7px;
		background: transparent;
		color: #a7bbba;
		padding: 0 12px;
		font-size: 12px;
	}

	.filter-tabs em,
	.section-tabs em {
		background: rgba(142, 174, 174, 0.16);
		color: #d5e7e5;
		padding: 2px 7px;
	}

	.three-pane {
		display: grid;
		grid-template-columns: 350px minmax(430px, 1fr) 320px;
		gap: 12px;
		min-height: 690px;
	}

	.conversation-list {
		display: grid;
		align-content: start;
		padding: 10px;
	}

	.local-search {
		height: 38px;
		border-radius: 8px;
		padding: 0 10px;
		margin-bottom: 10px;
	}

	.conversation-list button,
	.contacts-list-panel .contact-row {
		position: relative;
		display: grid;
		grid-template-columns: 34px 42px minmax(0, 1fr) auto;
		gap: 10px;
		align-items: center;
		width: 100%;
		min-height: 78px;
		border: 1px solid transparent;
		border-radius: 8px;
		background: transparent;
		color: #e6f7f5;
		padding: 9px 10px;
		text-align: left;
	}

	.conversation-list button.active,
	.contacts-list-panel .contact-row.active {
		border-color: rgba(45, 240, 206, 0.24);
		background: rgba(25, 109, 100, 0.24);
	}

	.conversation-list img {
		width: 42px;
		height: 42px;
	}

	.conversation-list strong,
	.contacts-list-panel strong {
		display: block;
		color: #fff;
		font-size: 14px;
		font-weight: 560;
	}

	.conversation-list small,
	.contacts-list-panel small,
	.contacts-list-panel em {
		display: block;
		margin-top: 5px;
		font-size: 11px;
		font-style: normal;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.conversation-list time {
		align-self: start;
		color: #a9bcbb;
		font-size: 11px;
	}

	.conversation-list b {
		position: absolute;
		right: 12px;
		bottom: 24px;
		display: grid;
		place-items: center;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: rgba(45, 240, 206, 0.28);
		color: #42f3d5;
		font-size: 10px;
	}

	.empty-panel {
		display: grid;
		place-items: center;
		min-height: 120px;
		border: 1px dashed rgba(113, 205, 196, 0.18);
		border-radius: 8px;
		color: #a9bcbb;
		font-size: 13px;
		line-height: 1.4;
		padding: 18px;
		text-align: center;
	}

	.empty-panel.error {
		color: #ffb4b4;
		border-color: rgba(255, 87, 87, 0.26);
		background: rgba(106, 31, 42, 0.18);
	}

	.empty-panel.fill {
		min-height: 100%;
		border: 0;
	}

	.chat-pane {
		display: grid;
		grid-template-rows: auto 1fr auto;
		min-height: 0;
	}

	.chat-pane > header,
	.contact-hero {
		display: grid;
		grid-template-columns: auto 1fr auto;
		align-items: center;
		gap: 12px;
		min-height: 68px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.12);
		padding: 12px 16px;
	}

	.chat-pane header img {
		width: 48px;
		height: 48px;
	}

	.chat-pane h2,
	.profile-head h2 {
		color: #fff;
		font-size: 20px;
		font-weight: 560;
	}

	.chat-actions {
		display: flex;
		gap: 8px;
	}

	.chat-body {
		display: grid;
		align-content: start;
		gap: 12px;
		min-height: 0;
		overflow: auto;
		padding: 18px 16px;
	}

	.date-divider {
		color: #95aaa9;
		font-size: 12px;
		text-align: center;
	}

	.bubble,
	.attachment-bubble {
		position: relative;
		max-width: 60%;
		border: 1px solid rgba(111, 205, 195, 0.12);
		border-radius: 8px;
		background: rgba(18, 46, 51, 0.74);
		color: #effffd;
		font-size: 14px;
		line-height: 1.45;
		padding: 12px 14px 20px;
	}

	.bubble.outbound {
		justify-self: end;
		border-color: rgba(45, 240, 206, 0.28);
		background: rgba(19, 116, 98, 0.48);
	}

	.bubble time,
	.attachment-bubble time {
		position: absolute;
		right: 10px;
		bottom: 5px;
		color: #a6bbbb;
		font-size: 10px;
	}

	.attachment-bubble {
		display: grid;
		grid-template-columns: 44px 1fr auto;
		align-items: center;
		gap: 12px;
		max-width: 54%;
	}

	.attachment-bubble small {
		display: block;
		margin-top: 5px;
		color: #a6bbbb;
	}

	.attachment-bubble button {
		color: #b7cdcc;
		background: transparent;
	}

	.composer {
		gap: 8px;
		height: 64px;
		margin: 0 16px 16px;
		border-radius: 10px;
		padding: 0 10px;
	}

	.composer button {
		display: grid;
		place-items: center;
		width: 34px;
		height: 34px;
		border-radius: 8px;
		background: rgba(34, 183, 160, 0.16);
		color: #39f0d0;
	}

	.context-rail {
		display: grid;
		gap: 12px;
		align-content: start;
	}

	.profile-panel {
		padding: 16px;
	}

	.profile-head {
		display: grid;
		grid-template-columns: 58px 1fr;
		gap: 12px;
		align-items: center;
	}

	.profile-head img {
		width: 58px;
		height: 58px;
	}

	.quick-icons {
		display: flex;
		gap: 8px;
		margin-top: 16px;
	}

	.info-card p {
		font-size: 13px;
		line-height: 1.48;
	}

	.related-row {
		display: grid;
		grid-template-columns: 30px 1fr auto;
		gap: 9px;
		min-height: 34px;
	}

	.mini-check em {
		color: #2df0ce;
		font-size: 11px;
		font-style: normal;
	}

	.contacts-layout,
	.docs-layout,
	.notes-layout {
		display: grid;
		grid-template-columns: 320px minmax(520px, 1fr) 310px;
		gap: 12px;
		min-height: 0;
	}

	.contacts-list-panel {
		padding: 12px;
	}

	.contacts-list-panel header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 12px;
	}

	.contacts-list-panel .contact-row {
		grid-template-columns: 44px 1fr auto;
		min-height: 76px;
	}

	.contacts-list-panel img {
		width: 44px;
		height: 44px;
	}

	.contact-detail,
	.notes-main,
	.agent-main {
		display: grid;
		gap: 12px;
		align-content: start;
		min-width: 0;
	}

	.contact-hero img {
		width: 92px;
		height: 92px;
	}

	.contact-hero {
		grid-template-columns: 92px 1fr auto;
	}

	.contact-cards {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 12px;
	}

	.span-2 {
		grid-column: span 2;
	}

	.detail-list {
		display: grid;
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: 12px;
	}

	.detail-list li {
		display: grid;
		grid-template-columns: 20px 1fr auto;
		align-items: center;
		gap: 8px;
	}

	.detail-list em {
		color: #a6bbbb;
		font-style: normal;
	}

	.node-detail-list li {
		grid-template-columns: minmax(0, 1fr) auto;
	}

	.tag-cloud {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.big-score {
		width: 72px;
		height: 72px;
		color: #fff;
		font-size: 24px;
	}

	.project-hero {
		display: grid;
		grid-template-columns: 82px 1fr auto;
		align-items: center;
		gap: 18px;
		min-height: 152px;
		padding: 18px;
	}

	.project-logo {
		display: grid;
		place-items: center;
		width: 78px;
		height: 78px;
		border: 1px solid rgba(45, 240, 206, 0.28);
		border-radius: 50%;
		background: rgba(18, 116, 112, 0.2);
		color: #24d8ff;
	}

	.project-hero h1 {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.project-meta-strip {
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		margin-top: 12px;
		padding: 14px;
	}

	.project-meta-strip article {
		border-right: 1px solid rgba(102, 189, 180, 0.1);
		padding: 0 14px;
	}

	.project-meta-strip article:last-child {
		border-right: 0;
	}

	.project-meta-strip span {
		display: block;
		color: #91a8a8;
		font-size: 12px;
	}

	.project-meta-strip strong {
		display: block;
		margin-top: 8px;
		color: #fff;
		font-size: 14px;
		font-weight: 500;
	}

	.project-empty-state {
		display: grid;
		place-items: center;
		min-height: 360px;
		color: #9fb6b4;
		text-align: center;
	}

	.project-empty-state h2 {
		margin: 0;
		color: #f1fffe;
		font-size: 20px;
		font-weight: 560;
	}

	.project-empty-state p,
	.muted-copy {
		margin: 0;
		color: #91a8a8;
		font-size: 12px;
		line-height: 1.5;
	}

	.project-empty-state button {
		min-height: 34px;
		border: 1px solid rgba(45, 240, 206, 0.24);
		border-radius: 6px;
		background: rgba(31, 161, 142, 0.12);
		color: #2ef1cd;
		padding: 0 14px;
	}

	.project-switcher {
		display: flex;
		gap: 8px;
		margin-top: 12px;
		padding: 8px;
		overflow-x: auto;
	}

	.project-switcher button {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		min-height: 34px;
		border: 1px solid rgba(102, 189, 180, 0.12);
		border-radius: 6px;
		background: rgba(8, 32, 36, 0.76);
		color: #b7cdca;
		padding: 0 10px;
		white-space: nowrap;
	}

	.project-switcher button.active {
		border-color: rgba(45, 240, 206, 0.5);
		background: rgba(25, 153, 131, 0.24);
		color: #2ef1cd;
	}

	.project-switcher em {
		color: #91a8a8;
		font-size: 11px;
		font-style: normal;
	}

	.project-dashboard-grid {
		display: grid;
		grid-template-columns: 0.8fr 1.45fr 0.8fr 310px;
		gap: 12px;
	}

	.project-dashboard-grid .project-side {
		grid-column: 4;
		grid-row: 1 / span 4;
	}

	.graph-card-large {
		min-height: 330px;
		padding: 14px;
	}

	.radial-graph {
		position: relative;
		height: 260px;
		margin-top: 12px;
		background:
			linear-gradient(rgba(45, 240, 206, 0.035) 1px, transparent 1px),
			linear-gradient(90deg, rgba(45, 240, 206, 0.025) 1px, transparent 1px);
		background-size: 30px 30px;
	}

	.graph-center,
	.knowledge-core {
		position: absolute;
		display: grid;
		place-items: center;
		border: 1px solid rgba(45, 240, 206, 0.84);
		border-radius: 50%;
		background: rgba(13, 126, 113, 0.72);
		box-shadow: 0 0 34px rgba(45, 240, 206, 0.42);
		color: #dffffa;
	}

	.graph-center {
		top: 50%;
		left: 50%;
		width: 76px;
		height: 76px;
		transform: translate(-50%, -50%);
	}

	.graph-center span {
		position: absolute;
		top: 84px;
		width: 110px;
		text-align: center;
		font-size: 12px;
	}

	.graph-chip {
		position: absolute;
		left: var(--x);
		top: var(--y);
		border: 1px solid rgba(45, 240, 206, 0.18);
		border-radius: 999px;
		background: rgba(5, 30, 34, 0.82);
		color: #dffffa;
		font-size: 11px;
		padding: 5px 8px;
	}

	.summary-numbers {
		display: grid;
		gap: 12px;
	}

	.summary-numbers.compact {
		grid-template-columns: repeat(2, 1fr);
	}

	.summary-numbers article {
		display: grid;
		gap: 4px;
	}

	.summary-numbers strong {
		color: #fff;
		font-size: 22px;
		font-weight: 500;
	}

	.summary-numbers span {
		color: #91a8a8;
		font-size: 11px;
	}

	.timeline-mini,
	.health-row,
	.deadline {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 10px;
		align-items: start;
		border-bottom: 1px solid rgba(102, 189, 180, 0.08);
		padding-bottom: 9px;
	}

	.timeline-mini strong {
		color: #f6fffe;
		font-size: 12px;
		font-weight: 500;
	}

	.timeline-mini time {
		color: #dcefed;
		font-size: 11px;
	}

	.health-row {
		grid-template-columns: 1fr auto;
		border: 1px solid rgba(111, 205, 195, 0.1);
		border-radius: 7px;
		padding: 10px;
	}

	.health-row span {
		color: #91a8a8;
	}

	.health-row strong {
		color: #2df0ce;
	}

	.person-compact {
		display: grid;
		grid-template-columns: 36px 1fr auto;
		gap: 10px;
		min-height: 40px;
	}

	.person-compact img {
		width: 36px;
		height: 36px;
	}

	.person-compact small,
	.doc-mini small {
		display: block;
		margin-top: 3px;
		font-size: 11px;
	}

	.person-compact em {
		color: #bde8e4;
		font-size: 11px;
		font-style: normal;
	}

	.inline-metrics {
		flex: 1;
		grid-template-columns: repeat(5, minmax(92px, 1fr));
	}

	.inline-metrics.tiny {
		max-width: 460px;
		grid-template-columns: repeat(3, 1fr);
	}

	.tasks-layout,
	.calendar-layout,
	.knowledge-layout,
	.agents-layout,
	.settings-layout,
	.timeline-layout {
		display: grid;
		grid-template-columns: minmax(740px, 1fr) 310px;
		gap: 12px;
	}

	.task-table {
		min-height: 680px;
	}

	.table-head,
	.task-row,
	.doc-row {
		display: grid;
		align-items: center;
		min-height: 46px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.08);
		padding: 0 16px;
	}

	.table-head {
		grid-template-columns: 2fr 0.9fr 1.2fr 1.2fr 0.9fr 0.8fr 0.9fr;
		color: #9fb6b5;
		font-size: 11px;
	}

	.task-row {
		grid-template-columns: 22px 2fr 0.9fr 1.2fr 1.2fr 0.9fr 0.8fr 0.9fr;
		color: #dcefed;
		font-size: 12px;
	}

	.task-row strong {
		color: #fff;
		font-weight: 500;
	}

	.task-row span {
		color: #c7d9d8;
	}

	.task-table-head {
		grid-template-columns: 2fr 1fr 0.9fr 1fr 0.7fr;
	}

	.task-row-actions {
		grid-template-columns: 2fr 1.1fr 1fr 0.7fr auto;
		gap: 10px;
	}

	.task-actions {
		display: inline-flex;
		gap: 7px;
	}

	.task-actions button {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 8px;
		height: auto;
		font-size: 11px;
	}

	.identity-note {
		margin: 0 0 8px;
		color: #91a8a8;
		font-size: 12px;
	}

	.identity-candidate-row {
		display: grid;
		gap: 8px;
		padding: 10px 0;
		border-top: 1px solid rgba(102, 189, 180, 0.08);
	}

	.identity-candidate-row:first-child {
		border-top: none;
	}

	.identity-candidate-row strong {
		display: block;
		margin-bottom: 3px;
	}

	.identity-candidate-row p {
		margin: 0 0 4px;
		color: #dbe9e8;
	}

	.identity-candidate-row small {
		display: block;
		color: #91a8a8;
	}

	.identity-actions {
		display: inline-flex;
		gap: 7px;
		margin-top: 8px;
	}

	.identity-actions button {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 8px;
		height: auto;
		font-size: 11px;
	}

	.inline-copy {
		margin: 0;
		padding: 12px 16px;
		color: #91a8a8;
		font-size: 12px;
	}

	.task-row em,
	.task-row b {
		justify-self: start;
	}

	.task-group {
		color: #dcefed;
		font-size: 13px;
		font-weight: 500;
		padding: 18px 16px 8px;
	}

	.task-group em {
		border-radius: 999px;
		background: rgba(142, 174, 174, 0.16);
		color: #d5e7e5;
		font-size: 10px;
		font-style: normal;
		padding: 2px 7px;
	}

	.chart-panel .donut {
		width: 114px;
		height: 114px;
		margin: 6px auto;
	}

	.donut strong {
		color: #fff;
		font-size: 24px;
		font-weight: 500;
	}

	.donut span {
		margin-top: -28px;
		font-size: 11px;
	}

	.donut.small {
		width: 70px;
		height: 70px;
	}

	.bar-row {
		display: grid;
		grid-template-columns: 1fr 120px;
		align-items: center;
		gap: 8px;
	}

	.bar-row span {
		color: #dcefed;
		font-size: 12px;
	}

	.bar-row div {
		height: 5px;
		border-radius: 999px;
		background: rgba(66, 130, 126, 0.22);
		overflow: hidden;
	}

	.bar-row i {
		display: block;
		width: 72%;
		height: 100%;
		background: #2df0ce;
	}

	.deadline {
		grid-template-columns: 1fr auto;
	}

	.week-board {
		display: grid;
		grid-template-rows: 52px 1fr auto;
		min-height: 710px;
	}

	.week-header {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		border-bottom: 1px solid rgba(102, 189, 180, 0.12);
	}

	.week-header strong {
		display: grid;
		place-items: center;
		color: #dcefed;
		font-size: 12px;
		font-weight: 500;
	}

	.time-grid {
		position: relative;
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		height: 580px;
		background:
			linear-gradient(rgba(111, 205, 195, 0.09) 1px, transparent 1px),
			linear-gradient(90deg, rgba(111, 205, 195, 0.09) 1px, transparent 1px);
		background-size: calc(100% / 7) 46px;
	}

	.event-block {
		position: absolute;
		left: calc((100% / 7) * var(--day) + 6px);
		top: var(--top);
		width: calc((100% / 7) - 12px);
		height: var(--height);
		border: 1px solid rgba(45, 240, 206, 0.26);
		border-radius: 6px;
		background: rgba(13, 96, 90, 0.38);
		padding: 9px;
	}

	.event-block.blue {
		border-color: rgba(36, 148, 255, 0.54);
		background: rgba(20, 86, 136, 0.38);
	}

	.event-block.purple {
		border-color: rgba(181, 99, 231, 0.5);
		background: rgba(89, 53, 119, 0.38);
	}

	.event-block.amber {
		border-color: rgba(236, 183, 70, 0.72);
		background: rgba(93, 78, 25, 0.4);
	}

	.event-block strong,
	.event-block span {
		display: block;
	}

	.event-block strong {
		color: #fff;
		font-size: 12px;
		font-weight: 560;
	}

	.event-block span {
		margin-top: 4px;
		color: #b8cbc9;
		font-size: 10px;
	}

	.now-line {
		position: absolute;
		right: 0;
		left: 0;
		top: 230px;
		height: 1px;
		background: #ef3140;
	}

	.now-line span {
		position: absolute;
		left: 0;
		top: -10px;
		border-radius: 4px;
		background: #ef3140;
		color: #fff;
		font-size: 10px;
		padding: 2px 5px;
	}

	.week-board footer {
		display: flex;
		gap: 18px;
		border-top: 1px solid rgba(102, 189, 180, 0.12);
		color: #a6bbbb;
		font-size: 11px;
		padding: 12px 14px;
	}

	.source-strip {
		display: grid;
		grid-template-columns: repeat(5, minmax(0, 1fr));
		gap: 10px;
		margin-bottom: 12px;
	}

	.source-card {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 58px;
		border-radius: 8px;
		color: #eefefb;
		padding: 0 14px;
	}

	.source-card.add {
		justify-content: center;
		color: #2df0ce;
	}

	.docs-layout {
		grid-template-columns: 220px minmax(640px, 1fr) 310px;
	}

	.docs-table header {
		padding: 16px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.1);
	}

	.category-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 10px;
		padding: 12px;
	}

	.category-grid article {
		display: grid;
		grid-template-columns: 24px 1fr;
		align-items: center;
		gap: 8px;
		min-height: 56px;
		border: 1px solid rgba(111, 205, 195, 0.1);
		border-radius: 8px;
		background: rgba(8, 36, 39, 0.48);
		padding: 10px;
		color: #dffffa;
		font-size: 12px;
	}

	.table-head.docs {
		grid-template-columns: 2fr 1fr 1fr 0.7fr 1fr 0.7fr;
	}

	.doc-row {
		grid-template-columns: 24px 2fr 1fr 1fr 0.7fr 1fr 0.7fr;
		color: #dcefed;
		font-size: 12px;
	}

	.doc-row strong {
		color: #fff;
		font-size: 12px;
		font-weight: 500;
	}

	.doc-mini {
		grid-template-columns: 26px 1fr;
		gap: 10px;
		min-height: 38px;
	}

	.collection-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.08);
		padding-bottom: 8px;
		font-size: 12px;
	}

	.notes-layout {
		grid-template-columns: 250px minmax(620px, 1fr) 310px;
	}

	.notes-list {
		padding: 12px;
	}

	.notes-list h3 {
		color: #dcefed;
		font-size: 12px;
		font-weight: 500;
		padding: 8px 0;
	}

	.notes-list article {
		display: grid;
		grid-template-columns: 28px minmax(0, 1fr) 86px;
		align-items: start;
		gap: 12px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.08);
		padding: 12px 0;
	}

	.notes-list article > div {
		min-width: 0;
	}

	.notes-list strong {
		display: block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.notes-list p {
		margin-top: 4px;
		font-size: 12px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.notes-list small {
		display: block;
		margin-top: 6px;
		font-size: 11px;
	}

	.notes-list em {
		justify-self: end;
		max-width: 86px;
		overflow: hidden;
		text-align: center;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.graph-filter-tabs {
		margin-top: 2px;
	}

	.graph-filter-tabs em {
		border-radius: 999px;
		background: rgba(142, 174, 174, 0.16);
		color: #d5e7e5;
		font-size: 10px;
		font-style: normal;
		padding: 2px 7px;
	}

	.knowledge-layout {
		grid-template-columns: minmax(760px, 1fr) 310px;
		min-height: 760px;
	}

	.graph-workbench {
		display: grid;
		grid-template-rows: auto auto minmax(0, 1fr) auto;
		overflow: hidden;
	}

	.graph-toolbar {
		display: flex;
		gap: 8px;
		padding: 12px;
	}

	.graph-search-form {
		display: grid;
		grid-template-columns: auto minmax(260px, 1fr) auto;
		gap: 10px;
		align-items: center;
		flex: 1;
		min-height: 38px;
		border: 1px solid rgba(111, 205, 195, 0.14);
		border-radius: 8px;
		background: rgba(4, 21, 24, 0.72);
		padding: 0 8px 0 12px;
		color: #9fb8b6;
	}

	.graph-search-form input {
		width: 100%;
		border: 0;
		outline: 0;
		background: transparent;
		color: #edf8f6;
		font-size: 13px;
	}

	.graph-search-form input:focus-visible {
		border-radius: 4px;
		outline: 1px solid rgba(45, 240, 206, 0.62);
		outline-offset: 2px;
	}

	.graph-toolbar button,
	.graph-search-form button,
	.graph-strip-message button,
	.graph-state-card button {
		min-height: 34px;
		border: 1px solid rgba(111, 205, 195, 0.14);
		border-radius: 7px;
		background: rgba(4, 21, 24, 0.72);
		color: #dcefed;
		padding: 0 12px;
		transition:
			border-color 160ms ease,
			background 160ms ease,
			color 160ms ease,
			transform 160ms ease;
	}

	.graph-search-form button:not(:disabled):hover,
	.graph-strip-message button:not(:disabled):hover,
	.graph-state-card button:not(:disabled):hover {
		border-color: rgba(45, 240, 206, 0.38);
		background: rgba(25, 154, 132, 0.18);
		color: #2df0ce;
		transform: translateY(-1px);
	}

	.graph-search-strip {
		min-height: 54px;
		border-top: 1px solid rgba(82, 204, 190, 0.08);
		border-bottom: 1px solid rgba(82, 204, 190, 0.08);
		padding: 9px 12px;
	}

	.graph-picker {
		display: grid;
		gap: 7px;
	}

	.graph-picker-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		color: #91a8a8;
		font-size: 11px;
	}

	.graph-picker-head span {
		color: #dcefed;
		font-weight: 560;
	}

	.graph-picker-head em {
		border: 1px solid rgba(45, 240, 206, 0.14);
		border-radius: 999px;
		background: rgba(45, 240, 206, 0.08);
		color: #9ee8df;
		font-style: normal;
		padding: 1px 7px;
	}

	.graph-result-row {
		display: flex;
		gap: 8px;
		overflow-x: auto;
		padding-bottom: 2px;
	}

	.graph-result-row button {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		flex: 0 0 auto;
		max-width: 240px;
		min-height: 34px;
		border: 1px solid rgba(111, 205, 195, 0.14);
		border-radius: 8px;
		background: rgba(7, 29, 33, 0.76);
		color: #dcefed;
		padding: 0 10px;
		transition:
			border-color 160ms ease,
			background 160ms ease,
			color 160ms ease;
	}

	.graph-result-row button.active,
	.graph-result-row button:hover {
		border-color: rgba(45, 240, 206, 0.42);
		background: rgba(25, 154, 132, 0.2);
		color: #2df0ce;
	}

	.graph-result-row span {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.graph-result-row em {
		color: #8eaead;
		font-size: 10px;
		font-style: normal;
		white-space: nowrap;
	}

	.graph-strip-message {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		min-height: 34px;
		color: #a7bbba;
		font-size: 12px;
	}

	.graph-strip-message.error {
		color: #ffabab;
	}

	.knowledge-canvas {
		position: relative;
		min-height: 610px;
		overflow: hidden;
		background:
			radial-gradient(circle at center, rgba(21, 132, 126, 0.18), transparent 35%),
			linear-gradient(rgba(45, 240, 206, 0.035) 1px, transparent 1px),
			linear-gradient(90deg, rgba(45, 240, 206, 0.025) 1px, transparent 1px);
		background-size: auto, 30px 30px, 30px 30px;
	}

	.graph-edge-layer {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
	}

	.graph-edge-layer line {
		stroke: rgba(45, 240, 206, 0.2);
		stroke-width: 0.14;
		vector-effect: non-scaling-stroke;
		transition: stroke 180ms ease;
	}

	.graph-edge-layer line.reviewed {
		stroke: rgba(45, 240, 206, 0.42);
	}

	.graph-edge-label {
		position: absolute;
		z-index: 2;
		display: block;
		max-width: 120px;
		border: 1px solid rgba(45, 240, 206, 0.12);
		border-radius: 999px;
		background: rgba(5, 22, 25, 0.82);
		color: #8fece1;
		font-size: 10px;
		overflow: hidden;
		padding: 3px 7px;
		text-overflow: ellipsis;
		transform: translate(-50%, -50%);
		white-space: nowrap;
		pointer-events: none;
	}

	.graph-node {
		position: absolute;
		z-index: 3;
		display: grid;
		place-items: center;
		gap: 4px;
		width: 118px;
		min-height: 74px;
		border: 1px solid rgba(45, 240, 206, 0.18);
		border-radius: 8px;
		background: rgba(6, 30, 34, 0.9);
		color: #dcefed;
		padding: 9px;
		transform: translate(-50%, -50%);
		transition:
			border-color 180ms ease,
			background 180ms ease,
			box-shadow 180ms ease,
			transform 180ms ease;
	}

	.graph-node.kind-person {
		border-color: rgba(43, 235, 175, 0.28);
	}

	.graph-node.kind-email_address,
	.graph-node.kind-message {
		border-color: rgba(44, 174, 255, 0.28);
	}

	.graph-node.kind-document {
		border-color: rgba(142, 98, 255, 0.28);
	}

	.graph-node:hover {
		border-color: rgba(45, 240, 206, 0.44);
		background: rgba(8, 44, 48, 0.94);
		transform: translate(-50%, -50%) scale(1.02);
	}

	.graph-node.selected {
		width: 138px;
		min-height: 92px;
		border-color: rgba(45, 240, 206, 0.72);
		background: rgba(7, 50, 51, 0.94);
		box-shadow:
			0 0 0 1px rgba(45, 240, 206, 0.2),
			0 0 34px rgba(45, 240, 206, 0.2);
	}

	.graph-node strong {
		max-width: 100%;
		overflow: hidden;
		text-align: center;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.graph-node span {
		color: #8eaead;
		font-size: 10px;
	}

	.graph-state-card {
		position: absolute;
		top: 50%;
		left: 50%;
		display: grid;
		justify-items: center;
		gap: 10px;
		width: min(420px, 72%);
		border: 1px solid rgba(82, 204, 190, 0.14);
		border-radius: 8px;
		background: rgba(5, 22, 25, 0.78);
		color: #dcefed;
		padding: 28px;
		text-align: center;
		transform: translate(-50%, -50%);
	}

	.graph-state-card.error {
		border-color: rgba(255, 110, 110, 0.3);
		background: rgba(128, 32, 40, 0.22);
	}

	.graph-state-card img {
		width: 56px;
		height: 56px;
		object-fit: contain;
	}

	.graph-state-card h2 {
		margin: 0;
		font-size: 18px;
	}

	.graph-state-card p {
		margin: 0;
		color: #9fb8b6;
		line-height: 1.5;
	}

	.graph-loading-overlay {
		position: absolute;
		right: 18px;
		bottom: 18px;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		border: 1px solid rgba(45, 240, 206, 0.18);
		border-radius: 999px;
		background: rgba(5, 22, 25, 0.86);
		color: #2df0ce;
		padding: 8px 12px;
	}

	.graph-status-bar {
		display: flex;
		flex-wrap: wrap;
		gap: 6px 12px;
		align-items: center;
		align-content: center;
		min-height: 46px;
		border-top: 1px solid rgba(82, 204, 190, 0.08);
		padding: 8px 14px;
		color: #a6bbbb;
		font-size: 12px;
		line-height: 1.4;
	}

	.evidence-row {
		border-bottom: 1px solid rgba(102, 189, 180, 0.08);
		padding: 10px 0;
	}

	.evidence-row strong {
		color: #2df0ce;
		font-size: 12px;
	}

	.evidence-row p {
		margin: 5px 0 0;
		color: #a7bbba;
		line-height: 1.4;
	}

	.timeline-slider {
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: 14px;
		align-items: center;
		padding: 14px;
		color: #a6bbbb;
		font-size: 11px;
	}

	.timeline-slider div {
		height: 4px;
		border-radius: 999px;
		background: rgba(66, 130, 126, 0.22);
	}

	.timeline-slider i {
		display: block;
		width: 64%;
		height: 100%;
		margin-left: 28%;
		background: #2df0ce;
	}

	.agent-metrics {
		grid-template-columns: repeat(6, 1fr);
		margin-bottom: 12px;
	}

	.agents-layout {
		grid-template-columns: minmax(760px, 1fr) 310px;
	}

	.agent-grid {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 10px;
	}

	.agent-card {
		display: grid;
		grid-template-columns: 44px 1fr;
		gap: 12px;
		min-height: 128px;
		padding: 12px;
		text-align: left;
	}

	.agent-card.active {
		border-color: rgba(45, 240, 206, 0.38);
	}

	.agent-card footer {
		grid-column: 1 / -1;
		display: flex;
		justify-content: space-between;
		border-top: 1px solid rgba(102, 189, 180, 0.08);
		padding-top: 10px;
		font-size: 11px;
	}

	.agent-detail {
		margin-top: 12px;
		padding: 14px;
	}

	.agent-detail header {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.agent-detail h2 {
		color: #fff;
		font-size: 20px;
	}

	.agent-detail-grid {
		display: grid;
		grid-template-columns: 1fr 300px 240px;
		gap: 22px;
		padding: 14px 8px 0;
	}

	.agent-detail-grid p,
	.agent-detail-grid li {
		color: #c7d9d8;
		font-size: 13px;
		line-height: 1.5;
	}

	.agent-detail-grid ul {
		display: grid;
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.agent-detail-grid li {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.ai-workflow-grid {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 10px;
		margin-top: 16px;
	}

	.ai-workflow-block {
		display: grid;
		gap: 10px;
		min-height: 190px;
		border: 1px solid rgba(111, 205, 195, 0.1);
		border-radius: 8px;
		background: rgba(5, 22, 25, 0.54);
		padding: 12px;
	}

	.ai-workflow-block label {
		display: grid;
		gap: 8px;
	}

	.ai-workflow-block span {
		color: #dcefed;
		font-size: 12px;
		font-weight: 650;
	}

	.ai-workflow-block textarea {
		width: 100%;
		min-height: 92px;
		max-height: 130px;
		resize: vertical;
		border: 1px solid rgba(111, 205, 195, 0.16);
		border-radius: 8px;
		background: rgba(2, 9, 11, 0.7);
		color: #eefefb;
		font-size: 12px;
		line-height: 1.45;
		padding: 9px 10px;
	}

	.ai-workflow-block button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 7px;
		min-height: 34px;
		border-radius: 8px;
		background: #2df0ce;
		color: #032522;
		font-size: 12px;
		font-weight: 760;
	}

	.ai-workflow-block button:disabled {
		background: rgba(111, 205, 195, 0.16);
		color: #789b98;
	}

	.ai-result-block {
		display: grid;
		gap: 10px;
		margin-top: 14px;
		border-top: 1px solid rgba(102, 189, 180, 0.1);
		padding-top: 14px;
	}

	.ai-result-block h3 {
		margin: 0;
		color: #fff;
		font-size: 15px;
	}

	.ai-result-block > p {
		margin: 0;
		color: #dcefed;
		font-size: 13px;
		line-height: 1.55;
	}

	.citation-list {
		display: grid;
		gap: 8px;
	}

	.citation-row {
		display: grid;
		gap: 4px;
		border-left: 2px solid rgba(45, 240, 206, 0.42);
		background: rgba(45, 240, 206, 0.045);
		padding: 8px 10px;
	}

	.citation-row strong,
	.citation-row span,
	.citation-row p {
		overflow-wrap: anywhere;
	}

	.citation-row strong {
		color: #eefefb;
		font-size: 12px;
	}

	.citation-row span {
		color: #7ea4a0;
		font-size: 10px;
	}

	.citation-row p {
		margin: 0;
		color: #bcd3d1;
		font-size: 12px;
		line-height: 1.45;
	}

	.spark-chart {
		height: 150px;
		border: 1px solid rgba(111, 205, 195, 0.1);
		border-radius: 8px;
		background:
			linear-gradient(160deg, transparent 42%, rgba(45, 240, 206, 0.9) 43%, transparent 44%),
			linear-gradient(rgba(45, 240, 206, 0.035) 1px, transparent 1px);
		background-size: auto, 28px 28px;
	}

	.large-timeline {
		padding: 0 0 12px;
	}

	.timeline-event-row {
		display: grid;
		grid-template-columns: 64px 18px 40px 1fr;
		gap: 10px;
		min-height: 72px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.08);
		padding: 12px 18px;
	}

	.timeline-event-row time {
		color: #2df0ce;
		font-size: 12px;
	}

	.rail-dot {
		width: 8px;
		height: 8px;
		margin-top: 8px;
		border-radius: 50%;
		background: #2df0ce;
		box-shadow: 0 0 12px rgba(45, 240, 206, 0.85);
	}

	.timeline-event-row strong {
		color: #fff;
		font-size: 13px;
	}

	.timeline-event-row p {
		margin-top: 5px;
		color: #91a8a8;
		font-size: 12px;
	}

	.drawer-backdrop {
		position: fixed;
		inset: 0;
		z-index: 20;
		background: rgba(0, 0, 0, 0.48);
	}

	.account-drawer {
		position: fixed;
		top: 18px;
		right: 18px;
		bottom: 18px;
		z-index: 21;
		display: grid;
		grid-template-rows: auto auto auto 1fr;
		gap: 16px;
		width: min(560px, calc(100vw - 36px));
		overflow: auto;
		border: 1px solid rgba(45, 240, 206, 0.24);
		border-radius: 14px;
		background:
			linear-gradient(180deg, rgba(8, 31, 35, 0.98), rgba(4, 18, 21, 0.98)),
			#041215;
		box-shadow: 0 24px 80px rgba(0, 0, 0, 0.55);
		padding: 18px;
	}

	.account-drawer > header {
		display: flex;
		justify-content: space-between;
		gap: 18px;
		align-items: start;
	}

	.account-drawer p {
		color: #37e8c9;
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
	}

	.account-drawer h2 {
		margin-top: 6px;
		color: #ffffff;
		font-size: 22px;
		font-weight: 500;
	}

	.provider-tabs {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px;
		border: 1px solid rgba(111, 205, 195, 0.14);
		border-radius: 8px;
		background: rgba(4, 21, 24, 0.72);
	}

	.provider-tabs button {
		flex: 1;
		height: 34px;
		border-radius: 6px;
		background: transparent;
		color: #9bb1b0;
	}

	.provider-tabs button.active {
		background: rgba(36, 207, 178, 0.16);
		color: #39f0d0;
	}

	.setup-form {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 12px;
		align-content: start;
	}

	.setup-form label {
		display: grid;
		gap: 6px;
		min-width: 0;
	}

	.setup-form label.wide,
	.form-actions.wide {
		grid-column: 1 / -1;
	}

	.setup-form span {
		color: #91a8a8;
		font-size: 11px;
		font-weight: 600;
	}

	.setup-form input,
	.setup-form select,
	.setup-form textarea {
		height: 38px;
		border: 1px solid rgba(111, 205, 195, 0.18);
		border-radius: 7px;
		background: rgba(4, 21, 24, 0.76);
		padding: 0 10px;
	}

	.setup-form textarea {
		min-height: 74px;
		padding: 9px 10px;
		resize: vertical;
	}

	.setup-form select {
		appearance: none;
		color: #dffcf7;
	}

	.compact-form {
		margin-top: 12px;
	}

	.telegram-grid,
	.whatsapp-grid {
		align-items: start;
	}

	.telegram-chat-pane,
	.whatsapp-chat-pane {
		min-height: 720px;
	}

	.telegram-inline-form {
		display: grid;
		grid-template-columns: minmax(120px, 0.8fr) minmax(120px, 0.8fr) minmax(180px, 1.8fr) auto;
		gap: 8px;
		padding: 12px;
		border-top: 1px solid rgba(111, 205, 195, 0.12);
		background: rgba(2, 12, 16, 0.68);
	}

	.telegram-inline-form input {
		min-width: 0;
		height: 38px;
		border: 1px solid rgba(111, 205, 195, 0.18);
		border-radius: 7px;
		background: rgba(4, 21, 24, 0.76);
		padding: 0 10px;
	}

	.telegram-inline-form button {
		height: 38px;
		border-radius: 7px;
		background: #25d8bd;
		color: #02201f;
		font-weight: 700;
		padding: 0 14px;
	}

	.collection-row.as-button {
		width: 100%;
		border: 0;
		text-align: left;
	}

	.collection-row.as-button.active {
		background: rgba(38, 216, 189, 0.12);
		color: #e9fffb;
	}

	.checkbox-row {
		display: flex !important;
		align-items: center;
		gap: 8px !important;
		padding-top: 18px;
	}

	.form-actions {
		display: flex;
		align-items: end;
	}

	.form-actions button,
	.oauth-box button {
		height: 38px;
		border-radius: 7px;
		background: #25d8bd;
		color: #02201f;
		font-weight: 700;
		padding: 0 15px;
	}

	.oauth-box {
		display: grid;
		gap: 12px;
		border: 1px solid rgba(45, 240, 206, 0.18);
		border-radius: 9px;
		background: rgba(10, 44, 47, 0.58);
		padding: 12px;
	}

	.oauth-box a {
		color: #41f3d3;
	}

	.settings-metrics {
		grid-template-columns: repeat(6, 1fr);
		margin-bottom: 12px;
	}

	.settings-tabs {
		margin-top: 12px;
	}

	.settings-layout {
		grid-template-columns: minmax(760px, 1fr) 330px;
		align-items: start;
	}

	.settings-list-panel {
		min-height: 640px;
	}

	.settings-category-list {
		display: grid;
		gap: 14px;
		padding: 14px;
	}

	.settings-category {
		display: grid;
		gap: 8px;
	}

	.settings-category > header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		min-height: 34px;
		border-bottom: 1px solid rgba(102, 189, 180, 0.1);
		color: #dcefed;
	}

	.settings-category h3 {
		color: #f6fffe;
		font-size: 14px;
		font-weight: 560;
	}

	.settings-category header span {
		border: 1px solid rgba(45, 240, 206, 0.14);
		border-radius: 999px;
		background: rgba(45, 240, 206, 0.08);
		color: #9ee8df;
		font-size: 11px;
		padding: 2px 8px;
	}

	.setting-row {
		display: grid;
		grid-template-columns: minmax(280px, 1fr) minmax(360px, 0.92fr);
		gap: 14px;
		align-items: center;
		min-height: 92px;
		border: 1px solid rgba(111, 205, 195, 0.1);
		border-radius: 8px;
		background: rgba(4, 21, 24, 0.44);
		padding: 12px;
	}

	.setting-copy {
		display: grid;
		gap: 5px;
		min-width: 0;
	}

	.setting-copy strong {
		color: #eefefb;
		font-size: 13px;
		font-weight: 650;
	}

	.setting-copy p {
		color: #91a8a8;
		font-size: 12px;
		line-height: 1.45;
	}

	.setting-meta-row {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		align-items: center;
		min-width: 0;
	}

	.setting-copy code,
	.account-card code {
		overflow: hidden;
		border: 1px solid rgba(111, 205, 195, 0.1);
		border-radius: 999px;
		background: rgba(2, 9, 11, 0.56);
		color: #8fece1;
		font-size: 11px;
		padding: 3px 8px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.setting-meta-row em {
		max-width: 210px;
		overflow: hidden;
		border: 1px solid rgba(45, 240, 206, 0.12);
		border-radius: 999px;
		background: rgba(45, 240, 206, 0.07);
		color: #9ee8df;
		font-size: 10px;
		font-style: normal;
		padding: 3px 8px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.setting-control {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 82px;
		gap: 8px;
		align-items: center;
		min-width: 0;
	}

	.setting-control input,
	.setting-control select,
	.setting-control textarea {
		width: 100%;
		min-width: 0;
		border: 1px solid rgba(111, 205, 195, 0.18);
		border-radius: 7px;
		background: rgba(2, 12, 16, 0.72);
		color: #eefefb;
		padding: 0 10px;
	}

	.setting-control input,
	.setting-control select {
		height: 38px;
	}

	.setting-control textarea {
		min-height: 86px;
		padding: 9px 10px;
		resize: vertical;
	}

	.setting-control button {
		height: 38px;
		border-radius: 7px;
		background: #2df0ce;
		color: #032522;
		font-size: 12px;
		font-weight: 760;
	}

	.setting-control button:disabled {
		background: rgba(111, 205, 195, 0.14);
		color: #789b98;
	}

	.setting-toggle {
		display: inline-flex;
		align-items: center;
		gap: 9px;
		min-height: 38px;
		border: 1px solid rgba(111, 205, 195, 0.18);
		border-radius: 7px;
		background: rgba(2, 12, 16, 0.72);
		color: #dcefed;
		padding: 0 10px;
	}

	.setting-toggle input {
		width: 16px;
		height: 16px;
		accent-color: #2df0ce;
	}

	.setting-toggle span {
		font-size: 12px;
	}

	.settings-rail {
		display: grid;
		gap: 12px;
		align-content: start;
	}

	.settings-rail .detail-list li {
		grid-template-columns: minmax(0, 1fr) auto;
	}

	.settings-account-layout {
		display: grid;
		gap: 12px;
	}

	.account-section {
		min-height: 188px;
	}

	.account-card-grid {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 10px;
		padding: 14px;
	}

	.account-card {
		display: grid;
		grid-template-columns: 38px minmax(0, 1fr);
		gap: 10px;
		min-height: 118px;
		border: 1px solid rgba(111, 205, 195, 0.1);
		border-radius: 8px;
		background: rgba(4, 21, 24, 0.5);
		padding: 12px;
	}

	.account-card div {
		min-width: 0;
	}

	.account-card strong {
		display: block;
		overflow: hidden;
		color: #eefefb;
		font-size: 13px;
		font-weight: 650;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.account-card p,
	.account-card small {
		display: block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.account-card p {
		margin-top: 4px;
		color: #c7d9d8;
		font-size: 12px;
	}

	.account-card small {
		margin-top: 6px;
		color: #91a8a8;
		font-size: 11px;
	}

	.account-card code {
		grid-column: 1 / -1;
	}

	.setup-state {
		border-radius: 8px;
		font-size: 12px;
		padding: 10px 12px;
	}

	.setup-state.success {
		border: 1px solid rgba(45, 240, 206, 0.25);
		background: rgba(37, 216, 189, 0.12);
		color: #51f7d9;
	}

	.setup-state.error {
		border: 1px solid rgba(255, 110, 110, 0.3);
		background: rgba(128, 32, 40, 0.26);
		color: #ffabab;
	}

	@media (max-width: 1360px) {
		.desktop-shell {
			grid-template-columns: 210px minmax(980px, 1fr);
			gap: 12px;
		}

		.hero-row,
		.dashboard-grid,
		.three-pane,
		.contacts-layout,
		.docs-layout,
		.notes-layout,
		.project-dashboard-grid,
		.tasks-layout,
		.calendar-layout,
		.knowledge-layout,
		.agents-layout,
		.settings-layout,
		.timeline-layout {
			transform-origin: top left;
		}

		.metric-grid {
			gap: 8px;
		}

		.agent-grid {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}

		.account-card-grid {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}
</style>
