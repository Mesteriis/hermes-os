import { describe, expect, it } from 'vitest';

import {
	GOOGLE_WORKSPACE_OAUTH_SCOPES,
	deleteMailAccount,
	emailAccountExportFilename,
	exportMailAccountSettings,
	importMailAccountSettings,
	inferMailService,
	loadEmailAccountViews,
	logoutMailAccount,
	mailServiceDisplayName,
	mailServicePreset,
	parseEmailAccountImportJson,
	saveImapAccount,
	selectMailService,
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

describe('Mail account management service helpers', () => {
	const accountView = {
		account: {
			account_id: 'fastmail-primary',
			provider_kind: 'imap',
			display_name: 'Fastmail',
			external_account_id: 'alex@example.com',
			config: {},
			created_at: '2026-06-13T10:00:00Z',
			updated_at: '2026-06-13T10:00:00Z'
		},
		capabilities: {
			read: true,
			sync: true,
			send: true,
			oauth: false,
			imap: true,
			smtp: true,
			mutate_flags: true,
			mutate_mailboxes: false,
			server_delete: false,
			provider_folders: false,
			local_trash: true
		}
	};

	it('loads account management views without exposing HTTP errors to callers', async () => {
		const result = await loadEmailAccountViews(async () => ({ items: [accountView] }));

		expect(result.accounts).toEqual([accountView]);
		expect(result.error).toBe('');
	});

	it('guards destructive and import/export actions with explicit account ids', async () => {
		await expect(exportMailAccountSettings('', async () => {
			throw new Error('should not be called');
		})).resolves.toMatchObject({ result: null, error: 'Account id is required' });

		await expect(logoutMailAccount(' ', async () => {
			throw new Error('should not be called');
		})).resolves.toMatchObject({ result: null, error: 'Account id is required' });

		await expect(deleteMailAccount('', async () => {
			throw new Error('should not be called');
		})).resolves.toMatchObject({ result: null, error: 'Account id is required' });
	});

	it('runs account import/export/logout/delete through injected API functions', async () => {
		const exportResult = await exportMailAccountSettings('fastmail-primary', async (accountId) => ({
			exported_at: '2026-06-13T10:00:00Z',
			account: { ...accountView.account, account_id: accountId },
			capabilities: accountView.capabilities,
			sync_settings: {
				account_id: accountId,
				sync_enabled: true,
				batch_size: 25,
				poll_interval_seconds: 900,
				updated_at: '2026-06-13T10:00:00Z'
			}
		}));
		expect(exportResult.result?.account.account_id).toBe('fastmail-primary');

		const importResult = await importMailAccountSettings(
			{
				account: {
					account_id: 'fastmail-primary',
					provider_kind: 'imap',
					display_name: 'Fastmail',
					external_account_id: 'alex@example.com'
				}
			},
			async () => ({
				account: accountView.account,
				capabilities: accountView.capabilities,
				sync_settings: {
					account_id: 'fastmail-primary',
					sync_enabled: true,
					batch_size: 100,
					poll_interval_seconds: 300,
					updated_at: '2026-06-13T10:00:00Z'
				}
			})
		);
		expect(importResult.result?.account.account_id).toBe('fastmail-primary');

		const logoutResult = await logoutMailAccount('fastmail-primary', async () => ({
			account: accountView.account,
			capabilities: { ...accountView.capabilities, sync: false },
			sync_settings: {
				account_id: 'fastmail-primary',
				sync_enabled: false,
				batch_size: 100,
				poll_interval_seconds: 300,
				updated_at: '2026-06-13T10:00:00Z'
			}
		}));
		expect(logoutResult.result?.sync_settings.sync_enabled).toBe(false);

		const deleteResult = await deleteMailAccount('fastmail-primary', async (accountId) => ({
			account_id: accountId,
			deleted: true,
			unbound_secret_refs: []
		}));
		expect(deleteResult.result?.deleted).toBe(true);
	});

	it('parses import JSON and rejects malformed account settings payloads', () => {
		const parsed = parseEmailAccountImportJson(
			JSON.stringify({
				account: {
					account_id: 'fastmail-primary',
					provider_kind: 'imap',
					display_name: 'Fastmail',
					external_account_id: 'owner@fastmail.com',
					config: { host: 'imap.fastmail.com' }
				},
				sync_settings: { sync_enabled: false }
			})
		);

		expect(parsed.account.account_id).toBe('fastmail-primary');
		expect(parsed.sync_settings?.sync_enabled).toBe(false);

		expect(() => parseEmailAccountImportJson('{')).toThrow('Import settings must be valid JSON');
		expect(() => parseEmailAccountImportJson(JSON.stringify({ account: null }))).toThrow(
			'Import settings must contain an account object'
		);
		expect(() =>
			parseEmailAccountImportJson(
				JSON.stringify({
					account: {
						account_id: 'x',
						provider_kind: 'pop3',
						display_name: 'POP3',
						external_account_id: 'x@example.com'
					}
				})
			)
		).toThrow('Import settings provider_kind must be gmail, icloud or imap');
	});

	it('builds filesystem-safe export filenames', () => {
		expect(emailAccountExportFilename('Fast Mail Primary', '2026-06-13T12:34:56Z')).toBe(
			'hermes-mail-account-fast-mail-primary-2026-06-13T12-34-56Z.json'
		);
		expect(emailAccountExportFilename(' @@@ ')).toBe('hermes-mail-account-account.json');
	});
});

describe('Mail provider presets', () => {
	it('infers common requested providers from email domains', () => {
		expect(inferMailService('owner@fastmail.com')).toBe('fastmail');
		expect(inferMailService('owner@mail.ru')).toBe('mailru');
		expect(inferMailService('owner@bk.ru')).toBe('mailru');
		expect(inferMailService('owner@yandex.ru')).toBe('yandex');
		expect(inferMailService('owner@ya.ru')).toBe('yandex');
		expect(inferMailService('owner@proton.me')).toBe('proton');
		expect(inferMailService('owner@protonmail.com')).toBe('proton');
		expect(inferMailService('owner@outlook.com')).toBe('microsoft');
	});

	it('returns IMAP and SMTP settings for fixed provider presets', () => {
		expect(mailServicePreset('fastmail', {})).toMatchObject({
			provider: 'imap',
			accountId: 'fastmail-primary',
			displayName: 'Fastmail',
			host: 'imap.fastmail.com',
			port: 993,
			smtpHost: 'smtp.fastmail.com',
			smtpPort: 587,
			smtpStarttls: true,
			secretKind: 'app_password'
		});
		expect(mailServicePreset('mailru', {})).toMatchObject({
			host: 'imap.mail.ru',
			smtpHost: 'smtp.mail.ru'
		});
		expect(mailServicePreset('yandex', {})).toMatchObject({
			host: 'imap.yandex.com',
			smtpHost: 'smtp.yandex.com'
		});
		expect(mailServicePreset('proton', {})).toMatchObject({
			host: '127.0.0.1',
			port: 1143,
			smtpHost: '127.0.0.1',
			smtpPort: 1025,
			displayName: 'Proton Bridge'
		});
	});

	it('applies SMTP preset fields when selecting and saving an account', async () => {
		const selected = selectMailService('fastmail', {}, {});
		expect(selected.imapForm).toMatchObject({
			account_id: 'fastmail-primary',
			display_name: 'Fastmail',
			host: 'imap.fastmail.com',
			smtp_host: 'smtp.fastmail.com',
			smtp_port: 587,
			smtp_starttls: true
		});

		const setupCalls: unknown[] = [];
		const result = await saveImapAccount(
			{
				selectedProvider: 'imap',
				selectedMailService: 'fastmail',
				imapForm: {
					account_id: 'fastmail-primary',
					display_name: 'Fastmail',
					external_account_id: '',
					host: 'imap.fastmail.com',
					port: 993,
					tls: true,
					mailbox: 'INBOX',
					username: 'owner@fastmail.com',
					password: ['app', 'password'].join('-'),
					secret_kind: 'app_password',
					smtp_host: 'smtp.fastmail.com',
					smtp_port: 587,
					smtp_tls: true,
					smtp_starttls: true,
					smtp_username: 'owner@fastmail.com'
				}
			},
			async (request) => {
				setupCalls.push(request);
				return {
					account_id: request.account_id,
					secret_ref: 'secret:provider-account:fastmail-primary:imap_password',
					secret_kind: 'app_password',
					store_kind: 'host_vault'
				};
			}
		);

		expect(result.success).toBe(true);
		expect(setupCalls[0]).toMatchObject({
			account_id: 'fastmail-owner-fastmail-com',
			provider_kind: 'imap',
			host: 'imap.fastmail.com',
			smtp_host: 'smtp.fastmail.com',
			smtp_port: 587,
			smtp_starttls: true,
			smtp_username: 'owner@fastmail.com'
		});
	});

	it('labels Microsoft and Proton without overstating native support', () => {
		expect(mailServiceDisplayName('microsoft')).toBe('Microsoft 365 / Exchange Online');
		expect(mailServiceDisplayName('proton')).toBe('Proton Bridge');
	});
});
