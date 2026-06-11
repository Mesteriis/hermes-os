import { beforeEach, describe, expect, it } from 'vitest';
import {
	composeForm,
	isComposeOpen,
	selectedCommunicationMessageId,
	selectedMailAccountId
} from './communications';
import { currentUiStateSnapshot, resetUiStatePersistenceForTests } from './uiState';

describe('ui state store snapshot', () => {
	beforeEach(() => {
		resetUiStatePersistenceForTests();
		isComposeOpen.set(false);
		selectedMailAccountId.set('');
		selectedCommunicationMessageId.set(null);
		composeForm.set({
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
		});
	});

	it('captures compose restore metadata without serializing private draft body', () => {
		selectedMailAccountId.set('imap-primary');
		selectedCommunicationMessageId.set('msg-1');
		composeForm.set({
			draft_id: 'draft-1',
			account_id: 'imap-primary',
			to_text: 'alice@example.com',
			cc_text: '',
			bcc_text: '',
			subject: 'Hello',
			body: 'private draft body',
			mode: 'reply',
			in_reply_to: 'provider-msg-1',
			references: ['provider-msg-1']
		});
		isComposeOpen.set(true);

		const snapshot = currentUiStateSnapshot();

		expect(snapshot.communications?.compose).toEqual({
			isOpen: true,
			mode: 'reply',
			draftId: 'draft-1',
			accountId: 'imap-primary',
			sourceMessageId: 'msg-1'
		});
		expect(JSON.stringify(snapshot)).not.toContain('private draft body');
		expect(JSON.stringify(snapshot)).not.toContain('body');
	});
});
