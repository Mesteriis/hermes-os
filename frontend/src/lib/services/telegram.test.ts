import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({
	fetchTelegramCapabilities: vi.fn(),
	fetchTelegramChats: vi.fn(),
	fetchTelegramMessages: vi.fn(),
	fetchAutomationTemplates: vi.fn(),
	fetchAutomationPolicies: vi.fn(),
	fetchTelegramCalls: vi.fn(),
	fetchCallTranscript: vi.fn(),
	fetchTelegramQrLoginStatus: vi.fn(),
	ingestTelegramFixtureMessage: vi.fn(),
	saveAutomationPolicy: vi.fn(),
	saveAutomationTemplate: vi.fn(),
	dryRunTelegramSend: vi.fn(),
	saveTelegramCall: vi.fn(),
	saveCallTranscriptFixture: vi.fn(),
	setupTelegramAccount: vi.fn(),
	setupTelegramFixtureAccount: vi.fn(),
	startTelegramQrLogin: vi.fn(),
	submitTelegramQrLoginPassword: vi.fn()
}));

import { setupTelegramAccount, startTelegramQrLogin } from '$lib/api';
import {
	saveTelegramAccountFromWizard,
	shouldPollTelegramQrLoginStatus,
	startTelegramQrLoginFromWizard
} from './telegram';

const qrReadyCapabilities = {
	version: 'v4',
	runtime_mode: 'tdlib_qr',
	telegram_app_credentials_configured: true,
	tdjson_runtime_available: true,
	qr_login_ready: true,
	capabilities: [],
	unsupported_features: []
};

const accountForm = {
	account_id: 'telegram-primary',
	display_name: 'Primary Telegram',
	api_id: '12345',
	api_hash: 'hash-from-old-form',
	session_encryption_key: 'session-key-from-old-form',
	tdlib_data_path: 'docker/data/telegram/telegram-primary'
};

describe('Telegram service QR login', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('starts QR login through backend-configured app credentials without posting form secrets', async () => {
		vi.mocked(startTelegramQrLogin).mockResolvedValue({
			setup_id: 'setup-1',
			account_id: 'telegram-primary',
			status: 'waiting_qr_scan',
			qr_link: 'tg://login?token=fixture',
			qr_svg: '<svg />',
			telegram_user_id: null,
			telegram_username: null,
			suggested_account_id: null,
			suggested_display_name: null,
			suggested_external_account_id: null,
			expires_at: null,
			poll_after_ms: 2500,
			message: 'Scan this QR code'
		});

		const result = await startTelegramQrLoginFromWizard({
			accountForm,
			capabilities: qrReadyCapabilities,
			externalAccountId: 'qr-login:telegram-primary'
		});

		expect(result.error).toBe('');
		expect(result.qrLogin?.status).toBe('waiting_qr_scan');

		const request = vi.mocked(startTelegramQrLogin).mock.calls[0][0] as Record<string, unknown>;
		expect(request).toMatchObject({
			account_id: 'telegram-primary',
			display_name: 'Primary Telegram',
			external_account_id: 'qr-login:telegram-primary',
			tdlib_data_path: 'docker/data/telegram/telegram-primary',
			transcription_enabled: false
		});
		expect('api_id' in request).toBe(false);
		expect('api_hash' in request).toBe(false);
		expect('session_encryption_key' in request).toBe(false);
	});

	it('saves QR-ready accounts through the QR-authorized metadata path', async () => {
		vi.mocked(setupTelegramAccount).mockResolvedValue({
			account_id: 'telegram-user-second',
			provider_kind: 'telegram_user',
			runtime: 'tdlib_qr_authorized',
			transcription_enabled: false,
			credential_bindings: []
		});

		const result = await saveTelegramAccountFromWizard({
			accountForm: {
				account_id: 'telegram-user-second',
				provider_kind: 'telegram_user',
				display_name: '@second',
				external_account_id: 'telegram:100200300',
				api_id: '12345',
				api_hash: 'hash-from-old-form',
				bot_token: '',
				session_encryption_key: 'session-key-from-old-form',
				tdlib_data_path: 'docker/data/telegram/telegram-user-second',
				transcription_enabled: true
			},
			authMethod: 'qr',
			qrLogin: {
				setup_id: 'setup-2',
				account_id: 'telegram-user-second',
				status: 'ready',
				qr_link: null,
				qr_svg: null,
				telegram_user_id: '100200300',
				telegram_username: 'second',
				suggested_account_id: 'telegram-user-second',
				suggested_display_name: '@second',
				suggested_external_account_id: 'telegram:100200300',
				expires_at: null,
				poll_after_ms: 2500,
				message: null
			},
			isFixtureSetup: false
		});

		expect(result.error).toBe('');
		expect(result.accountId).toBe('telegram-user-second');

		const request = vi.mocked(setupTelegramAccount).mock.calls[0][0] as Record<string, unknown>;
		expect(request).toMatchObject({
			account_id: 'telegram-user-second',
			provider_kind: 'telegram_user',
			display_name: '@second',
			external_account_id: 'telegram:100200300',
			tdlib_data_path: 'docker/data/telegram/telegram-user-second',
			transcription_enabled: false,
			qr_authorized: true
		});
		expect('api_id' in request).toBe(false);
		expect('api_hash' in request).toBe(false);
		expect('session_encryption_key' in request).toBe(false);
	});

	it('does not start QR login when backend Telegram app credentials are missing', async () => {
		const result = await startTelegramQrLoginFromWizard({
			accountForm,
			capabilities: {
				...qrReadyCapabilities,
				telegram_app_credentials_configured: false,
				qr_login_ready: false
			},
			externalAccountId: 'qr-login:telegram-primary'
		});

		expect(result.qrLogin).toBeNull();
		expect(result.error).toBe('Telegram app credentials are not configured in the backend environment');
		expect(startTelegramQrLogin).not.toHaveBeenCalled();
	});

	it('keeps polling while Telegram is checking the 2-step verification password', () => {
		expect(shouldPollTelegramQrLoginStatus('waiting_qr_scan')).toBe(true);
		expect(shouldPollTelegramQrLoginStatus('waiting_password')).toBe(true);
		expect(shouldPollTelegramQrLoginStatus('ready')).toBe(false);
		expect(shouldPollTelegramQrLoginStatus('failed')).toBe(false);
	});
});
