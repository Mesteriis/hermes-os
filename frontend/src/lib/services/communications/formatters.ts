import type { CommunicationSectionId } from '$lib/layout';
import type {
	CommunicationMessageDetailItem,
	CommunicationMessageSummary,
	MailboxHealth,
	WorkflowState
} from '$lib/api';
import { formatDateTime } from '../formatting';

export function messageTime(message: CommunicationMessageSummary | CommunicationMessageDetailItem) {
	return formatDateTime(message.occurred_at ?? message.projected_at);
}

export function communicationChannelIcon(channelKind: string) {
	if (channelKind === 'telegram_user' || channelKind === 'telegram_bot') {
		return 'tabler:brand-telegram';
	}
	if (channelKind === 'whatsapp_web') {
		return 'tabler:brand-whatsapp';
	}
	return 'tabler:mail';
}

export function communicationChannelLabel(channelKind: string) {
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

export function attachmentIcon(contentType: string) {
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

export function communicationSectionBadge(sectionId: CommunicationSectionId, mailboxHealth: MailboxHealth | null) {
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

export function communicationSectionWorkflowState(sectionId: CommunicationSectionId): WorkflowState | '' | null {
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

export function selectCommunicationSection(
	sectionId: CommunicationSectionId
): { viewId: 'communications'; sectionId: CommunicationSectionId; workflowState: WorkflowState | '' } | null {
	const workflowState = communicationSectionWorkflowState(sectionId);
	if (workflowState !== null) {
		return { viewId: 'communications', sectionId, workflowState };
	}
	return null;
}
