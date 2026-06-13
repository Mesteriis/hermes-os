import { derived, writable } from 'svelte/store';
import type {
	CommunicationMessageDetail,
	CommunicationMessageSummary,
	EmailDraft,
	EmailThread,
	LocalMessageState,
	MailMessageInsight,
	MailResourceSnapshot,
	MailResourceSummary,
	MailSyncRunResponse,
	MailSyncSettings,
	MailSyncStatus,
	MailboxHealth,
	MessageAnalyzeResponse,
	MessageExportResponse,
	SendEmailResponse,
	SenderStats,
	WorkflowState,
	WorkflowStateCountItem
} from '$lib/api';
import type { ComposeFormModel } from '$lib/services/communications';
import { emailProviderAccounts } from '../settings';
import * as commsService from '$lib/services/communications';

export type ComposeForm = ComposeFormModel;

export type ProjectItem = {
	name: string;
	kind: string;
	progress: number;
	icon: string;
	tone: string;
};

export type TaskItem = {
	title: string;
	due: string;
};

export type CommunicationsNavigatorMode = 'threads' | 'contacts';
export type CommunicationsInspectorMode = 'context' | 'contact' | 'organization' | null;
export type MessageContextTab = 'message' | 'attachments' | 'headers' | 'related' | 'timeline';

export const emptyComposeForm: ComposeForm = {
	draft_id: '',
	account_id: '',
	to_text: '',
	cc_text: '',
	bcc_text: '',
	subject: '',
	body: '',
	mode: 'compose',
	in_reply_to: null,
	references: []
};

export const communicationMessages = writable<CommunicationMessageSummary[]>([]);
export const selectedCommunicationDetail = writable<CommunicationMessageDetail | null>(null);
export const communicationsError = writable('');
export const isCommunicationsLoading = writable(false);
export const selectedConversationIndex = writable(0);
export const selectedCommunicationMessageId = writable<string | null>(null);
export const mailStateFilter = writable<WorkflowState | ''>('');
export const mailLocalStateFilter = writable<LocalMessageState>('active');
export const mailStateCounts = writable<WorkflowStateCountItem[]>([]);
export const isMailStateTransitioning = writable(false);
export const isAiAnswerSubmitting = writable(false);
export const aiAnalysisResult = writable<MessageAnalyzeResponse | null>(null);
export const drafts = writable<EmailDraft[]>([]);
export const mailboxHealth = writable<MailboxHealth | null>(null);
export const topSenders = writable<SenderStats[]>([]);
export const threads = writable<EmailThread[]>([]);
export const mailResources = writable<MailResourceSnapshot>({ ...commsService.emptyMailResourceSnapshot });
export const mailResourceSummary = writable<MailResourceSummary>(
	commsService.summarizeMailResourceSnapshot(commsService.emptyMailResourceSnapshot)
);
export const mailMessageInsight = writable<MailMessageInsight | null>(null);
export const isMailActionRunning = writable(false);
export const mailActionStatus = writable('');
export const mailActionError = writable('');
export const lastMessageExport = writable<MessageExportResponse | null>(null);
export const mailSyncStatuses = writable<MailSyncStatus[]>([]);
export const selectedMailSyncSettings = writable<MailSyncSettings | null>(null);
export const lastMailSyncRuns = writable<MailSyncRunResponse[]>([]);
export const isMailSyncBusy = writable(false);
export const mailSyncStatusMessage = writable('');
export const mailSyncError = writable('');
export const isComposeOpen = writable(false);
export const composeForm = writable<ComposeForm>({ ...emptyComposeForm });
export const selectedMailAccountId = writable('');
export const messageSearchQuery = writable('');
export const communicationsNavigatorMode = writable<CommunicationsNavigatorMode>('threads');
export const expandedCommunicationContactKey = writable<string | null>(null);
export const communicationsInspectorMode = writable<CommunicationsInspectorMode>(null);
export const activeMessageContextTab = writable<MessageContextTab>('message');
export const isSendReviewOpen = writable(false);
export const isSendingMessage = writable(false);
export const composeSendError = writable('');
export const composeStatusMessage = writable('');
export const lastSendResponse = writable<SendEmailResponse | null>(null);
export const communicationProjects = writable<ProjectItem[]>([]);
export const communicationTasks = writable<TaskItem[]>([]);

export const mailAccountOptions = derived(emailProviderAccounts, ($emailProviderAccounts) =>
	commsService.buildMailAccountOptions($emailProviderAccounts)
);

export const selectedMailAccountOption = derived(
	[mailAccountOptions, selectedMailAccountId],
	([$mailAccountOptions, $selectedMailAccountId]) =>
		$mailAccountOptions.find((account) => account.accountId === $selectedMailAccountId) ??
		$mailAccountOptions[0] ??
		null
);

export const selectedCommunication = derived(
	[communicationMessages, selectedCommunicationMessageId, selectedConversationIndex],
	([$communicationMessages, $selectedCommunicationMessageId, $selectedConversationIndex]) =>
		($selectedCommunicationMessageId
			? $communicationMessages.find((message) => message.message_id === $selectedCommunicationMessageId)
			: null) ??
		$communicationMessages[$selectedConversationIndex] ??
		null
);

export const conversations = derived(communicationMessages, ($communicationMessages) => {
	const channelLabels: Record<string, string> = {
		email: 'Email',
		gmail: 'Gmail',
		icloud: 'iCloud',
		imap: 'IMAP',
		telegram_user: 'Telegram',
		telegram_bot: 'Telegram',
		whatsapp_web: 'WhatsApp'
	};
	return $communicationMessages.map((msg) => ({
		name: msg.sender_display_name || msg.sender || 'Unknown',
		role: msg.sender || '',
		project: msg.subject || msg.body_text_preview,
		channel: channelLabels[msg.channel_kind] || msg.channel_kind,
		time: msg.occurred_at || msg.projected_at,
		preview: msg.body_text_preview
	}));
});
