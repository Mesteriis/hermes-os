import { get } from 'svelte/store';
import type { EmailDraft } from '$lib/api';
import { emailProviderAccounts } from '../settings';
import * as commsService from '$lib/services/communications';
import {
	composeForm,
	composeSendError,
	composeStatusMessage,
	drafts,
	emptyComposeForm,
	isComposeOpen,
	isSendReviewOpen,
	isSendingMessage,
	lastSendResponse
} from './state';
import {
	loadDrafts,
	loadMailboxHealth,
	loadMailResources
} from './loaders';
import {
	composeHasDraftContent,
	currentComposeDraftId,
	defaultComposeAccountId,
	selectedMessageForCompose
} from './selectors';

export async function handleSaveDraft(): Promise<void> {
	const result = await commsService.handleSaveDraft(get(composeForm));
	if (!result.success) {
		composeSendError.set(result.error);
		return;
	}
	composeForm.set({ ...emptyComposeForm });
	isComposeOpen.set(false);
	isSendReviewOpen.set(false);
	composeSendError.set('');
	composeStatusMessage.set('Draft saved');
	await loadDrafts();
}

export async function handleDeleteDraft(draftId: string): Promise<void> {
	const normalizedDraftId = draftId.trim();
	if (!normalizedDraftId) return;
	composeSendError.set('');
	composeStatusMessage.set('');
	const result = await commsService.handleDeleteDraft(normalizedDraftId);
	if (!result.success) {
		composeSendError.set(result.error);
		return;
	}
	drafts.update((items) => items.filter((draft) => draft.draft_id !== normalizedDraftId));
	if (currentComposeDraftId() === normalizedDraftId) {
		composeForm.set({ ...emptyComposeForm });
		isComposeOpen.set(false);
		isSendReviewOpen.set(false);
	}
	composeStatusMessage.set(result.deleted ? 'Draft deleted' : 'Draft was already deleted');
	await loadDrafts();
}

export async function autoSaveOpenComposeDraft(): Promise<void> {
	if (!get(isComposeOpen)) return;
	const form = get(composeForm);
	if (!form.account_id || !form.draft_id || !composeHasDraftContent(form)) return;
	const result = await commsService.handleSaveDraft(form);
	if (!result.success) {
		composeSendError.set(result.error);
		return;
	}
	composeSendError.set('');
}

export async function restoreComposeDraftById(draftId: string | null | undefined): Promise<boolean> {
	if (!draftId?.trim()) {
		isComposeOpen.set(false);
		return false;
	}
	if (!get(drafts).some((draft) => draft.draft_id === draftId)) {
		await loadDrafts();
	}
	const draft = get(drafts).find((candidate) => candidate.draft_id === draftId);
	if (!draft) {
		isComposeOpen.set(false);
		return false;
	}
	openComposeForDraft(draft);
	return true;
}

export function openComposeForDraft(draft: EmailDraft): void {
	composeForm.set({
		draft_id: draft.draft_id,
		account_id: draft.account_id,
		to_text: draft.to_recipients.join(', '),
		cc_text: draft.cc_recipients.join(', '),
		bcc_text: draft.bcc_recipients.join(', '),
		subject: draft.subject,
		body: draft.body_text,
		mode: 'compose',
		in_reply_to: draft.in_reply_to,
		references: draft.references
	});
	composeSendError.set('');
	composeStatusMessage.set('');
	isComposeOpen.set(true);
}

export function openNewMessage(accountId?: string): void {
	const selectedAccountId = accountId || defaultComposeAccountId();
	composeForm.set(commsService.newComposeForm(selectedAccountId));
	composeSendError.set('');
	composeStatusMessage.set('');
	isSendReviewOpen.set(false);
	isComposeOpen.set(true);
}

export function openReplyToSelected(): void {
	const message = selectedMessageForCompose();
	if (!message) return;
	composeForm.set(commsService.buildReplyComposeForm(message, defaultComposeAccountId(message.account_id)));
	composeSendError.set('');
	composeStatusMessage.set('');
	isSendReviewOpen.set(false);
	isComposeOpen.set(true);
}

export function openForwardSelected(): void {
	const message = selectedMessageForCompose();
	if (!message) return;
	composeForm.set(commsService.buildForwardComposeForm(message, defaultComposeAccountId(message.account_id)));
	composeSendError.set('');
	composeStatusMessage.set('');
	isSendReviewOpen.set(false);
	isComposeOpen.set(true);
}

export function openSendReview(): void {
	composeSendError.set('');
	isSendReviewOpen.set(true);
}

export function closeSendReview(): void {
	isSendReviewOpen.set(false);
}

export async function confirmSendMessage(): Promise<void> {
	const form = get(composeForm);
	const account = get(emailProviderAccounts).find((candidate) => candidate.account_id === form.account_id);
	const capability = commsService.sendCapabilityForAccount(account);
	if (!capability.canSend) {
		composeSendError.set(capability.reason ?? 'Sending is unavailable for this account');
		isSendReviewOpen.set(false);
		return;
	}
	isSendingMessage.set(true);
	composeSendError.set('');
	const result = await commsService.handleSendMessage(form);
	isSendingMessage.set(false);
	if (!result.success) {
		composeSendError.set(result.error);
		return;
	}
	lastSendResponse.set(result.result);
	composeStatusMessage.set(`Sent via ${result.result?.transport ?? 'provider'}`);
	isSendReviewOpen.set(false);
	isComposeOpen.set(false);
	composeForm.set({ ...emptyComposeForm });
	await Promise.all([loadDrafts(), loadMailboxHealth(), loadMailResources()]);
}
