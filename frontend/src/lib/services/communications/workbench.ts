import type { CommunicationMessageSummary, ProviderAccount, WorkflowState } from '$lib/api';
import type { MailAccountOption, SendCapability } from './types';

export function senderLabel(sender: string) {
	const match = sender.match(/^"?([^"<]+)"?\s*</);
	return (match?.[1] ?? senderEmail(sender) ?? sender).trim();
}

export function senderEmail(sender: string) {
	const angleMatch = sender.match(/<([^>]+)>/);
	if (angleMatch?.[1]) {
		return angleMatch[1].trim();
	}
	const emailMatch = sender.match(/[^\s<>]+@[^\s<>]+/);
	return emailMatch?.[0]?.trim() ?? sender.trim();
}

export function sendCapabilityForAccount(account: ProviderAccount | null | undefined): SendCapability {
	if (!account) {
		return { canSend: false, transport: null, reason: 'Select a sending account' };
	}
	if (account.provider_kind === 'gmail') {
		return {
			canSend: false,
			transport: null,
			reason: 'Gmail send is unavailable until OAuth send scopes are configured'
		};
	}
	if (!['icloud', 'imap'].includes(account.provider_kind)) {
		return { canSend: false, transport: null, reason: 'This provider cannot send email' };
	}
	if (
		typeof account.config.smtp_host !== 'string' ||
		account.config.smtp_host.trim() === '' ||
		typeof account.config.smtp_port !== 'number'
	) {
		return {
			canSend: false,
			transport: null,
			reason: 'Reconnect this account to enable SMTP send'
		};
	}
	return { canSend: true, transport: 'smtp', reason: null };
}

export function nextReadWorkflowState(currentState: WorkflowState | null | undefined): WorkflowState {
	return currentState === 'reviewed' ? 'new' : 'reviewed';
}

export function buildMailAccountOptions(accounts: ProviderAccount[]): MailAccountOption[] {
	return accounts.map((account) => {
		const capability = sendCapabilityForAccount(account);
		return {
			accountId: account.account_id,
			providerKind: account.provider_kind,
			label: account.display_name || account.external_account_id || account.account_id,
			email: account.external_account_id,
			canSend: capability.canSend,
			transport: capability.transport,
			sendUnavailableReason: capability.reason
		};
	});
}

export function filterMessagesForWorkbench(
	messages: CommunicationMessageSummary[],
	accountId: string,
	query: string
): CommunicationMessageSummary[] {
	const trimmedAccountId = accountId.trim();
	const terms = query
		.toLowerCase()
		.trim()
		.split(/\s+/)
		.filter(Boolean);
	return messages.filter((message) => {
		if (trimmedAccountId && message.account_id !== trimmedAccountId) {
			return false;
		}
		if (terms.length === 0) {
			return true;
		}
		const haystack = [
			message.subject,
			message.sender,
			message.sender_display_name ?? '',
			message.body_text_preview,
			message.provider_record_id
		]
			.join(' ')
			.toLowerCase();
		return terms.every((term) => haystack.includes(term));
	});
}
