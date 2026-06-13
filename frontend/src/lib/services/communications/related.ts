import type { CommunicationListMessage, RelatedCommunicationMessage } from './types';
import { senderEmail } from './workbench';

export function relatedMessagesForSelection(
	messages: CommunicationListMessage[],
	selected: CommunicationListMessage | null,
	limit = 8
): RelatedCommunicationMessage[] {
	if (!selected) return [];
	const selectedSender = senderEmail(selected.sender).toLowerCase();
	const selectedConversationId = selected.conversation_id?.trim() ?? '';
	return messages
		.filter((message) => message.message_id !== selected.message_id)
		.map((message) => {
			const sameConversation =
				selectedConversationId !== '' && message.conversation_id === selectedConversationId;
			const sameContact = senderEmail(message.sender).toLowerCase() === selectedSender;
			if (!sameConversation && !sameContact) return null;
			return {
				...message,
				relation: sameConversation ? 'same_conversation' : 'same_contact'
			} satisfies RelatedCommunicationMessage;
		})
		.filter((message): message is RelatedCommunicationMessage => message !== null)
		.sort((a, b) => messageSortTimestamp(b) - messageSortTimestamp(a))
		.slice(0, limit);
}

function messageSortTimestamp(message: CommunicationListMessage): number {
	const value = message.occurred_at ?? message.projected_at;
	const timestamp = Date.parse(value);
	return Number.isFinite(timestamp) ? timestamp : 0;
}
