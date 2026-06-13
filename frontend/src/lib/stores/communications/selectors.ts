import { get } from 'svelte/store';
import type {
	CommunicationMessageDetailItem,
	CommunicationMessageSummary
} from '$lib/api';
import type { ComposeForm } from './state';
import {
	composeForm,
	mailAccountOptions,
	selectedCommunication,
	selectedCommunicationDetail,
	selectedMailAccountId
} from './state';

export function defaultComposeAccountId(fallbackAccountId = ''): string {
	const selected = get(selectedMailAccountId);
	if (selected) return selected;
	const options = get(mailAccountOptions);
	return (
		options.find((account) => account.canSend)?.accountId ??
		options[0]?.accountId ??
		fallbackAccountId
	);
}

export function selectedMessageForCompose(): CommunicationMessageSummary | CommunicationMessageDetailItem | null {
	return get(selectedCommunicationDetail)?.message ?? get(selectedCommunication);
}

export function composeHasDraftContent(form: ComposeForm): boolean {
	return Boolean(
		form.to_text.trim() ||
		form.cc_text.trim() ||
		form.bcc_text.trim() ||
		form.subject.trim() ||
		form.body.trim()
	);
}

export function selectedMessageId(): string | null {
	return get(selectedCommunicationDetail)?.message.message_id ?? get(selectedCommunication)?.message_id ?? null;
}

export function currentComposeDraftId(): string {
	return get(composeForm).draft_id;
}
