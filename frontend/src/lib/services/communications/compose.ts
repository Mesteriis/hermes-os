import {
	createDraft,
	deleteDraft,
	sendEmail,
	type CommunicationMessageDetailItem,
	type CommunicationMessageSummary,
	type SendEmailResponse
} from '$lib/api';
import type { ComposeFormModel } from './types';
import { senderEmail } from './workbench';

type DraftCreator = typeof createDraft;
type DraftDeleter = typeof deleteDraft;

export async function handleSaveDraft(
	draft: ComposeFormModel,
	draftCreator: DraftCreator = createDraft
): Promise<{ success: boolean; error: string }> {
	if (!draft.draft_id || !draft.account_id) {
		return { success: false, error: 'Select a sending account before saving a draft' };
	}
	try {
		await draftCreator(buildComposeDraftPayload(draft));
		return { success: true, error: '' };
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Draft save failed'
		};
	}
}

export async function handleDeleteDraft(
	draftId: string,
	draftDeleter: DraftDeleter = deleteDraft
): Promise<{ success: boolean; error: string; deleted: boolean }> {
	const normalizedDraftId = draftId.trim();
	if (!normalizedDraftId) {
		return { success: false, error: 'Draft id is required', deleted: false };
	}
	try {
		const result = await draftDeleter(normalizedDraftId);
		return { success: true, error: '', deleted: Boolean(result.deleted) };
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Draft deletion failed',
			deleted: false
		};
	}
}

export function splitRecipients(value: string): string[] {
	return value
		.split(',')
		.map((recipient) => recipient.trim())
		.filter(Boolean);
}

export function buildComposeDraftPayload(draft: ComposeFormModel): Record<string, unknown> {
	return {
		draft_id: draft.draft_id,
		account_id: draft.account_id,
		to_recipients: splitRecipients(draft.to_text),
		cc_recipients: splitRecipients(draft.cc_text),
		bcc_recipients: splitRecipients(draft.bcc_text),
		subject: draft.subject,
		body_text: draft.body,
		in_reply_to: draft.in_reply_to,
		references: draft.references,
		status: 'draft',
		metadata: { compose_mode: draft.mode }
	};
}

export async function handleSendMessage(
	draft: ComposeFormModel
): Promise<{ success: boolean; error: string; result: SendEmailResponse | null }> {
	try {
		const result = await sendEmail({
			account_id: draft.account_id,
			to: splitRecipients(draft.to_text),
			cc: splitRecipients(draft.cc_text),
			bcc: splitRecipients(draft.bcc_text),
			subject: draft.subject,
			body_text: draft.body,
			in_reply_to: draft.in_reply_to,
			references: draft.references,
			confirmed_provider_write: true
		});
		return { success: true, error: '', result };
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Email send failed',
			result: null
		};
	}
}

export function newComposeForm(accountId: string): ComposeFormModel {
	return {
		draft_id: `draft-${Date.now()}`,
		account_id: accountId,
		to_text: '',
		cc_text: '',
		bcc_text: '',
		subject: '',
		body: '',
		mode: 'compose',
		in_reply_to: null,
		references: []
	};
}

export function buildReplyComposeForm(
	message: CommunicationMessageSummary | CommunicationMessageDetailItem,
	accountId: string
): ComposeFormModel {
	const providerRecordId = message.provider_record_id || message.message_id;
	return {
		...newComposeForm(accountId),
		to_text: senderEmail(message.sender),
		subject: subjectWithPrefix(message.subject, 'Re:'),
		mode: 'reply',
		in_reply_to: providerRecordId,
		references: [providerRecordId]
	};
}

export function buildForwardComposeForm(
	message: CommunicationMessageSummary | CommunicationMessageDetailItem,
	accountId: string
): ComposeFormModel {
	return {
		...newComposeForm(accountId),
		subject: subjectWithPrefix(message.subject, 'Fwd:'),
		body: `\n\nForwarded message:\nFrom: ${message.sender}\nSubject: ${message.subject}\n\n${
			'body_text' in message ? message.body_text : message.body_text_preview
		}`,
		mode: 'forward'
	};
}

function subjectWithPrefix(subject: string, prefix: 'Re:' | 'Fwd:'): string {
	return subject.toLowerCase().startsWith(prefix.toLowerCase()) ? subject : `${prefix} ${subject}`;
}
