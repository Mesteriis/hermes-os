import { describe, expect, it } from 'vitest';

import {
	GOOGLE_WORKSPACE_OAUTH_SCOPES,
	createGoogleWorkspaceOAuthStartRequest,
	createTelegramAccountDraft,
	safeAccountIdSegment
} from './accounts';

describe('Google Workspace OAuth setup', () => {
	it('requests mail, calendar, and contacts read scopes', () => {
		expect(GOOGLE_WORKSPACE_OAUTH_SCOPES).toEqual([
			'https://www.googleapis.com/auth/gmail.readonly',
			'https://www.googleapis.com/auth/calendar.readonly',
			'https://www.googleapis.com/auth/contacts.readonly'
		]);
	});

	it('builds a start request without requiring the Gmail address first', () => {
		const request = createGoogleWorkspaceOAuthStartRequest({
			account_id: '',
			display_name: '',
			external_account_id: '',
			client_id: '',
			client_secret: '',
			redirect_uri: 'http://127.0.0.1:8080/api/v1/email-accounts/gmail/oauth/callback'
		});

		expect(request).toMatchObject({
			account_id: 'gmail-primary',
			display_name: 'Google Workspace',
			redirect_uri: 'http://127.0.0.1:8080/api/v1/email-accounts/gmail/oauth/callback',
			scopes: GOOGLE_WORKSPACE_OAUTH_SCOPES
		});
		expect(request).not.toHaveProperty('external_account_id');
		expect(request).not.toHaveProperty('client_id');
		expect(request).not.toHaveProperty('client_secret');
	});
});

describe('Telegram account drafts', () => {
	it('creates account-scoped Telegram user draft identities', () => {
		const draft = createTelegramAccountDraft('telegram_user', 'Second Telegram Account');

		expect(draft).toMatchObject({
			account_id: 'telegram-user-second-telegram-account',
			provider_kind: 'telegram_user',
			display_name: 'Telegram Account',
			external_account_id: '',
			tdlib_data_path: 'docker/data/telegram/telegram-user-second-telegram-account'
		});
		expect(draft.api_hash).toBe('');
		expect(draft.session_encryption_key).toBe('');
	});

	it('creates separate bot draft identities', () => {
		const draft = createTelegramAccountDraft('telegram_bot', 'Ops Bot');

		expect(draft.account_id).toBe('telegram-bot-ops-bot');
		expect(draft.provider_kind).toBe('telegram_bot');
		expect(draft.tdlib_data_path).toBe('docker/data/telegram/telegram-bot-ops-bot');
	});

	it('normalizes unsafe account id seed text', () => {
		expect(safeAccountIdSegment(' @Alex M#36 / main ')).toBe('alex-m-36-main');
	});
});
