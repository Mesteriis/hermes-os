import type { TelegramProviderKind } from '$lib/api';
import { safeAccountIdSegment } from './shared';
import type {
	TelegramAccountDraft,
	TelegramWizardStep
} from './types';

function defaultTelegramDraftSeed() {
	return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

export function createTelegramAccountDraft(
	providerKind: TelegramProviderKind = 'telegram_user',
	seed = defaultTelegramDraftSeed()
): TelegramAccountDraft {
	const suffix = safeAccountIdSegment(seed) || 'draft';
	const prefix = providerKind === 'telegram_bot' ? 'telegram-bot' : 'telegram-user';
	const accountId = `${prefix}-${suffix}`;

	return {
		account_id: accountId,
		provider_kind: providerKind,
		display_name: providerKind === 'telegram_bot' ? 'Telegram Bot' : 'Telegram Account',
		external_account_id: '',
		api_id: '',
		api_hash: '',
		bot_token: '',
		session_encryption_key: '',
		tdlib_data_path: `docker/data/telegram/${accountId}`,
		transcription_enabled: false
	};
}

export function continueTelegramWizard(nextStep: TelegramWizardStep): TelegramWizardStep {
	return nextStep;
}

export function selectTelegramAuthMethod(
	method: string,
	accountForm: Record<string, unknown>
): {
	authMethod: string;
	setupMode: string;
	accountForm: Record<string, unknown>;
} {
	let nextForm = accountForm;
	if (method === 'qr' && accountForm.external_account_id === '@telegram_fixture') {
		nextForm = { ...nextForm, external_account_id: '' };
	} else if (method === 'bot_token') {
		nextForm = { ...nextForm, provider_kind: 'telegram_bot' };
	}
	return {
		authMethod: method,
		setupMode: method === 'fixture' ? 'fixture' : 'live',
		accountForm: nextForm
	};
}

export function telegramWizardExternalAccountId(
	accountForm: { account_id: string; external_account_id: string },
	authMethod: string
): string {
	return (
		accountForm.external_account_id.trim() ||
		(authMethod === 'qr'
			? `qr-login:${accountForm.account_id}`
			: accountForm.account_id)
	);
}

export function telegramQrStatusLabel(status: string) {
	switch (status) {
		case 'waiting_qr_scan':
			return 'Waiting for QR scan';
		case 'waiting_password':
			return 'Telegram password required';
		case 'ready':
			return 'Telegram authorized';
		case 'expired':
			return 'QR code expired';
		case 'failed':
			return 'QR login failed';
		case 'runtime_unavailable':
			return 'TDLib runtime unavailable';
		default:
			return 'QR login status';
	}
}

export function applyTelegramQrLoginResult(
	result: { status: string; suggested_account_id?: string; suggested_display_name?: string; suggested_external_account_id?: string },
	accountForm: Record<string, string>
): Record<string, string> {
	if (result.status !== 'ready') {
		return accountForm;
	}
	return {
		...accountForm,
		account_id: result.suggested_account_id ?? accountForm.account_id,
		display_name: result.suggested_display_name ?? accountForm.display_name,
		external_account_id: result.suggested_external_account_id ?? accountForm.external_account_id
	};
}

export function telegramWizardNote(
	authMethod: string,
	capabilities: { tdjson_runtime_available?: boolean; telegram_app_credentials_configured?: boolean } | null,
	qrNeedsAppCredentials: boolean,
	qrLogin: { status: string } | null
): string {
	if (authMethod === 'fixture') {
		return 'Fixture mode creates local Telegram records for UI and policy testing.';
	}
	if (authMethod === 'qr') {
		if (capabilities !== null && !capabilities.tdjson_runtime_available) {
			return 'TDLib JSON runtime is not available in the running backend.';
		}
		if (qrNeedsAppCredentials) {
			return 'Enter Telegram API ID and API hash to start QR login in this dev session.';
		}
		if (qrLogin?.status === 'waiting_password') {
			return 'Enter the Telegram 2-step verification password to finish local TDLib authorization.';
		}
		return 'Telegram app credentials are configured in the backend environment. QR login is ready.';
	}
	return 'Live credentials are stored in the encrypted database vault. Telegram live runtime remains blocked until the adapter is implemented.';
}
