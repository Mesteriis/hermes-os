import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';
import type { CommunicationMessageSummary } from '$lib/api';
import {
	communicationMessages,
	selectedCommunication,
	selectedCommunicationMessageId,
	selectedConversationIndex
} from './communications';

function message(messageId: string, subject: string): CommunicationMessageSummary {
	return {
		message_id: messageId,
		raw_record_id: `raw-${messageId}`,
		account_id: 'imap-primary',
		provider_record_id: `provider-${messageId}`,
		subject,
		sender: 'Alice <alice@example.com>',
		recipients: ['user@example.com'],
		body_text_preview: 'Preview',
		occurred_at: '2026-06-11T12:00:00Z',
		projected_at: '2026-06-11T12:00:01Z',
		channel_kind: 'email',
		conversation_id: null,
		sender_display_name: 'Alice',
		delivery_state: 'delivered',
		message_metadata: {},
		attachment_count: 0,
		local_state: 'active',
		local_state_changed_at: null
	};
}

describe('communications store selection', () => {
	beforeEach(() => {
		communicationMessages.set([]);
		selectedConversationIndex.set(0);
		selectedCommunicationMessageId.set(null);
	});

	it('prefers stable selected message id over list index and falls back to index when missing', () => {
		communicationMessages.set([
			message('msg-1', 'First'),
			message('msg-2', 'Second')
		]);
		selectedConversationIndex.set(0);
		selectedCommunicationMessageId.set('msg-2');

		expect(get(selectedCommunication)?.message_id).toBe('msg-2');

		selectedCommunicationMessageId.set('missing');
		expect(get(selectedCommunication)?.message_id).toBe('msg-1');
	});
});
