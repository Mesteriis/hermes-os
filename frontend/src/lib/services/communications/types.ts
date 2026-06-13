import type {
	CommunicationMessageSummary,
	CommunicationMessageSummaryV2,
	MailMessageInsight,
	MailResourceSnapshot,
	MailResourceSummary,
	ProviderAccount
} from '$lib/api';

export type ComposeMode = 'compose' | 'reply' | 'forward';

export type ComposeFormModel = {
	draft_id: string;
	account_id: string;
	to_text: string;
	cc_text: string;
	bcc_text: string;
	subject: string;
	body: string;
	mode: ComposeMode;
	in_reply_to: string | null;
	references: string[];
};

export type MailAccountOption = {
	accountId: string;
	providerKind: string;
	label: string;
	email: string;
	canSend: boolean;
	transport: 'smtp' | null;
	sendUnavailableReason: string | null;
};

export type SendCapability = {
	canSend: boolean;
	transport: 'smtp' | null;
	reason: string | null;
};

export type CommunicationListMessage = CommunicationMessageSummary | CommunicationMessageSummaryV2;

export type RelatedCommunicationMessage = CommunicationListMessage & {
	relation: 'same_contact' | 'same_conversation';
};

export type RenderedMessageContent = {
	html: string;
	mode: 'html' | 'text';
};

export type OriginalMailSrcdocOptions = {
	messageId?: string | null;
	apiBaseUrl?: string | null;
};

export type {
	MailMessageInsight,
	MailResourceSnapshot,
	MailResourceSummary,
	ProviderAccount
};
