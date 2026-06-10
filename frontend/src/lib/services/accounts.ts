import {
	startGmailOAuthSetup,
	completeGmailOAuthSetup,
	setupImapAccount,
	createCalendarAccount,
	type ProviderAccount,
	type GmailOAuthStartRequest,
	type GmailOAuthStartResponse,
	type TelegramProviderKind
} from '$lib/api';
import { formatDateTime } from './formatting';

type Provider = 'gmail' | 'icloud' | 'imap';
type AccountWizardKind = 'mail' | 'calendar' | 'telegram' | 'whatsapp';
type AccountWizardTarget = AccountWizardKind | Provider;
type MailService = Provider | 'microsoft' | 'yahoo' | 'aol';
type MailWizardStep = 'provider' | 'details';
type CalendarProvider = 'local' | 'google' | 'microsoft' | 'apple' | 'caldav' | 'ics';
type CalendarWizardStep = 'provider' | 'details';
type TelegramWizardStep = 'account' | 'auth' | 'details';

export const GOOGLE_WORKSPACE_OAUTH_SCOPES = [
	'https://www.googleapis.com/auth/gmail.readonly',
	'https://www.googleapis.com/auth/calendar.readonly',
	'https://www.googleapis.com/auth/contacts.readonly'
];

export type TelegramAccountDraft = {
	account_id: string;
	provider_kind: TelegramProviderKind;
	display_name: string;
	external_account_id: string;
	api_id: string;
	api_hash: string;
	bot_token: string;
	session_encryption_key: string;
	tdlib_data_path: string;
	transcription_enabled: boolean;
};

export function openAccountDrawer(target: AccountWizardTarget): {
	wizardKind: AccountWizardKind;
	mailWizardStep: MailWizardStep;
	calendarWizardStep: CalendarWizardStep;
	telegramWizardStep: TelegramWizardStep;
	mailService: MailService | null;
} {
	const accountWizardKind: AccountWizardKind =
		target === 'gmail' || target === 'icloud' || target === 'imap' ? 'mail' : target;
	let mailService: MailService | null = null;
	if (target === 'gmail' || target === 'icloud' || target === 'imap') {
		mailService = target;
	}
	const result: {
		wizardKind: AccountWizardKind;
		mailWizardStep: MailWizardStep;
		calendarWizardStep: CalendarWizardStep;
		telegramWizardStep: TelegramWizardStep;
		mailService: MailService | null;
	} = {
		wizardKind: accountWizardKind,
		mailWizardStep: 'provider',
		calendarWizardStep: 'provider',
		telegramWizardStep: 'account',
		mailService
	};
	return result;
}

export function closeAccountDrawer(): boolean {
	return false;
}

function defaultTelegramDraftSeed() {
	return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

export function safeAccountIdSegment(value: string) {
	return value
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9_-]+/g, '-')
		.replace(/^-+|-+$/g, '')
		.slice(0, 48);
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

export function createGoogleWorkspaceOAuthStartRequest(gmailForm: {
	account_id: string;
	display_name: string;
	external_account_id: string;
	client_id: string;
	client_secret: string;
	redirect_uri: string;
}): GmailOAuthStartRequest {
	const request: GmailOAuthStartRequest = {
		account_id: optionalTrimmed(gmailForm.account_id) ?? 'gmail-primary',
		display_name: optionalTrimmed(gmailForm.display_name) ?? 'Google Workspace',
		redirect_uri: gmailForm.redirect_uri.trim(),
		scopes: [...GOOGLE_WORKSPACE_OAUTH_SCOPES]
	};
	const externalAccountId = optionalTrimmed(gmailForm.external_account_id);
	const clientId = optionalTrimmed(gmailForm.client_id);
	const clientSecret = optionalTrimmed(gmailForm.client_secret);

	if (externalAccountId) {
		request.external_account_id = externalAccountId;
	}
	if (clientId) {
		request.client_id = clientId;
	}
	if (clientSecret) {
		request.client_secret = clientSecret;
	}

	return request;
}

export function selectMailService(
	service: MailService,
	gmailForm: Record<string, string>,
	imapForm: Record<string, unknown>
): {
	selectedProvider: Provider;
	selectedMailService: MailService;
	gmailForm: Record<string, string>;
	imapForm: Record<string, unknown>;
} {
	if (service === 'gmail') {
		return {
			selectedProvider: 'gmail',
			selectedMailService: service,
			gmailForm: {
				...gmailForm,
				account_id: gmailForm.account_id || 'gmail-primary',
				display_name: gmailForm.display_name || 'Primary Gmail'
			},
			imapForm
		};
	}

	const preset = mailServicePreset(service, imapForm);
	return {
		selectedProvider: preset.provider,
		selectedMailService: service,
		gmailForm,
		imapForm: {
			...imapForm,
			account_id: preset.accountId,
			display_name: preset.displayName,
			host: preset.host,
			port: preset.port,
			tls: true,
			mailbox: (imapForm.mailbox as string) || 'INBOX',
			secret_kind: preset.secretKind
		}
	};
}

export function continueMailWizard(
	email: string,
	gmailForm: Record<string, string>,
	imapForm: Record<string, unknown>
): {
	mailWizardStep: MailWizardStep;
	gmailForm: Record<string, string>;
	imapForm: Record<string, unknown>;
} {
	if (email) {
		const inferred = inferMailService(email);
		const nextGmailForm = {
			...gmailForm,
			external_account_id: email,
			account_id: gmailForm.account_id || accountIdFromEmail(email, 'gmail'),
			display_name: gmailForm.display_name || email
		};
		const nextImapForm = {
			...imapForm,
			external_account_id: email,
			username: (imapForm.username as string) || email,
			account_id: imapForm.account_id || accountIdFromEmail(email, 'mail'),
			display_name: imapForm.display_name || email
		};
		return { mailWizardStep: 'details', gmailForm: nextGmailForm, imapForm: nextImapForm };
	}
	return { mailWizardStep: 'details', gmailForm, imapForm };
}

export function continueCalendarWizard(
	provider: CalendarProvider | undefined,
	calendarForm: Record<string, string>
): {
	calendarWizardStep: CalendarWizardStep;
	calendarForm: Record<string, string>;
} {
	const nextForm = provider
		? { ...calendarForm, provider, account_name: calendarProviderDefaultName(provider) }
		: calendarForm;
	return { calendarWizardStep: 'details', calendarForm: nextForm };
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

export function selectProvider(
	provider: Provider,
	imapForm: Record<string, unknown>
): {
	selectedProvider: Provider;
	selectedMailService: MailService;
	imapForm: Record<string, unknown>;
} {
	let nextImapForm = imapForm;
	if (provider === 'icloud') {
		nextImapForm = {
			...nextImapForm,
			account_id: (nextImapForm.account_id as string) || 'icloud-primary',
			display_name: (nextImapForm.display_name as string) || 'Primary iCloud',
			host: 'imap.mail.me.com',
			port: 993,
			tls: true,
			mailbox: (nextImapForm.mailbox as string) || 'INBOX',
			secret_kind: 'app_password'
		};
	}
	if (provider === 'imap') {
		nextImapForm = {
			...nextImapForm,
			account_id: nextImapForm.account_id === 'icloud-primary' ? 'imap-primary' : nextImapForm.account_id,
			display_name: nextImapForm.display_name === 'Primary iCloud' ? 'Primary IMAP' : nextImapForm.display_name,
			host: nextImapForm.host === 'imap.mail.me.com' ? '' : nextImapForm.host,
			secret_kind: 'password'
		};
	}
	return {
		selectedProvider: provider,
		selectedMailService: provider,
		imapForm: nextImapForm
	};
}

export function mailServicePreset(service: MailService, imapForm: Record<string, unknown>): {
	provider: Provider;
	accountId: string;
	displayName: string;
	host: string;
	port: number;
	secretKind: 'app_password' | 'password';
} {
	switch (service) {
		case 'icloud':
			return {
				provider: 'icloud',
				accountId: 'icloud-primary',
				displayName: 'Primary iCloud',
				host: 'imap.mail.me.com',
				port: 993,
				secretKind: 'app_password'
			};
		case 'microsoft':
			return {
				provider: 'imap',
				accountId: 'microsoft-primary',
				displayName: 'Microsoft Mail',
				host: 'outlook.office365.com',
				port: 993,
				secretKind: 'password'
			};
		case 'yahoo':
			return {
				provider: 'imap',
				accountId: 'yahoo-primary',
				displayName: 'Yahoo Mail',
				host: 'imap.mail.yahoo.com',
				port: 993,
				secretKind: 'app_password'
			};
		case 'aol':
			return {
				provider: 'imap',
				accountId: 'aol-primary',
				displayName: 'AOL Mail',
				host: 'imap.aol.com',
				port: 993,
				secretKind: 'app_password'
			};
		default:
			return {
				provider: 'imap',
				accountId: 'imap-primary',
				displayName: 'IMAP Mail',
				host: (imapForm.host as string) === 'imap.mail.me.com' ? '' : (imapForm.host as string),
				port: Number(imapForm.port) || 993,
				secretKind: 'password'
			};
	}
}

export function hasFixedMailServerPreset(service: MailService) {
	return service !== 'imap';
}

export function mailServiceDisplayName(service: MailService) {
	switch (service) {
		case 'gmail':
			return 'Google';
		case 'icloud':
			return 'iCloud';
		case 'microsoft':
			return 'Microsoft Exchange';
		case 'yahoo':
			return 'Yahoo';
		case 'aol':
			return 'AOL';
		default:
			return 'Other Mail Account';
	}
}

export function mailServiceIcon(service: MailService) {
	switch (service) {
		case 'gmail':
			return 'tabler:brand-gmail';
		case 'icloud':
			return 'tabler:cloud';
		case 'microsoft':
			return 'tabler:brand-office';
		case 'yahoo':
			return 'tabler:mail';
		case 'aol':
			return 'tabler:mail-bolt';
		default:
			return 'tabler:server';
	}
}

export function mailServiceAccountPrefix(service: MailService) {
	switch (service) {
		case 'icloud':
			return 'icloud';
		case 'microsoft':
			return 'microsoft';
		case 'yahoo':
			return 'yahoo';
		case 'aol':
			return 'aol';
		case 'gmail':
			return 'gmail';
		default:
			return 'imap';
	}
}

export function inferMailService(email: string): MailService | null {
	const domain = email.split('@')[1]?.trim().toLowerCase() ?? '';
	if (['gmail.com', 'googlemail.com'].includes(domain)) return 'gmail';
	if (['icloud.com', 'me.com', 'mac.com'].includes(domain)) return 'icloud';
	if (['outlook.com', 'hotmail.com', 'live.com', 'office365.com'].includes(domain)) return 'microsoft';
	if (domain.endsWith('yahoo.com')) return 'yahoo';
	if (domain === 'aol.com') return 'aol';
	return null;
}

export function accountIdFromEmail(email: string, fallback: string) {
	const normalized = email
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '');
	return normalized ? `${fallback}-${normalized}` : `${fallback}-primary`;
}

export function calendarProviderDefaultName(provider: CalendarProvider) {
	switch (provider) {
		case 'google':
			return 'Google Calendar';
		case 'microsoft':
			return 'Microsoft Calendar';
		case 'apple':
			return 'Apple Calendar';
		case 'caldav':
			return 'CalDAV Calendar';
		case 'ics':
			return 'ICS Feed';
		default:
			return 'Local Calendar';
	}
}

export async function startGmailSetup(gmailForm: {
	account_id: string;
	display_name: string;
	external_account_id: string;
	client_id: string;
	client_secret: string;
	redirect_uri: string;
}): Promise<{
	pending: GmailOAuthStartResponse | null;
	message: string;
	error: string;
}> {
	try {
		const pending = await startGmailOAuthSetup(createGoogleWorkspaceOAuthStartRequest(gmailForm));
		return { pending, message: 'Gmail OAuth grant started', error: '' };
	} catch (error) {
		return {
			pending: null,
			message: '',
			error: error instanceof Error ? error.message : 'Gmail setup failed'
		};
	}
}

export async function completeGmailSetup(
	pending: GmailOAuthStartResponse | null,
	authorizationCode: string,
	externalAccountId = ''
): Promise<{
	message: string;
	error: string;
	success: boolean;
}> {
	if (!pending) {
		return { message: '', error: 'Gmail OAuth grant has not been started', success: false };
	}

	try {
		const result = await completeGmailOAuthSetup({
			setup_id: pending.setup_id,
			state: pending.state,
			authorization_code: authorizationCode,
			external_account_id: optionalTrimmed(externalAccountId)
		});
		return { message: `Gmail account ${result.account_id} saved`, error: '', success: true };
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Gmail setup failed',
			success: false
		};
	}
}

function optionalTrimmed(value: string): string | undefined {
	const trimmed = value.trim();
	return trimmed ? trimmed : undefined;
}

export async function saveImapAccount(params: {
	selectedProvider: Provider;
	selectedMailService: MailService;
	imapForm: {
		account_id: string;
		display_name: string;
		external_account_id: string;
		host: string;
		port: number;
		tls: boolean;
		mailbox: string;
		username: string;
		password: string;
		secret_kind: 'app_password' | 'password';
	};
}): Promise<{
	message: string;
	error: string;
	success: boolean;
	imapFormPasswordCleared: boolean;
}> {
	const { selectedProvider, selectedMailService, imapForm } = params;
	try {
		const username = imapForm.username.trim();
		const fixedServerPreset = hasFixedMailServerPreset(selectedMailService);
		const externalAccountId = fixedServerPreset
			? username
			: imapForm.external_account_id.trim() || username;
		const accountId = fixedServerPreset
			? accountIdFromEmail(externalAccountId, mailServiceAccountPrefix(selectedMailService))
			: imapForm.account_id.trim() || accountIdFromEmail(externalAccountId, 'imap');
		const displayName = fixedServerPreset
			? externalAccountId || mailServiceDisplayName(selectedMailService)
			: imapForm.display_name.trim() || externalAccountId || mailServiceDisplayName(selectedMailService);
		const host = imapForm.host.trim();
		const mailbox = imapForm.mailbox.trim() || 'INBOX';

		const result = await setupImapAccount({
			account_id: accountId,
			provider_kind: selectedProvider === 'icloud' ? 'icloud' : 'imap',
			display_name: displayName,
			external_account_id: externalAccountId,
			host,
			port: Number(imapForm.port),
			tls: imapForm.tls,
			mailbox,
			username,
			password: imapForm.password,
			secret_kind: imapForm.secret_kind
		});
		return {
			message: `Mail account ${result.account_id} saved`,
			error: '',
			success: true,
			imapFormPasswordCleared: true
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Mail account setup failed',
			success: false,
			imapFormPasswordCleared: false
		};
	}
}

export async function saveCalendarAccount(params: {
	provider: CalendarProvider;
	account_name: string;
	email: string;
}): Promise<{
	message: string;
	error: string;
}> {
	try {
		const result = await createCalendarAccount({
			provider: params.provider,
			account_name: params.account_name,
			email: params.email.trim() || undefined
		});
		return { message: `Calendar account ${result.account_name} saved`, error: '' };
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Calendar account setup failed'
		};
	}
}

export function accountProviderIcon(providerKind: string) {
	if (providerKind === 'telegram_user' || providerKind === 'telegram_bot') {
		return 'tabler:brand-telegram';
	}
	if (providerKind === 'whatsapp_web') {
		return 'tabler:brand-whatsapp';
	}
	return 'tabler:mail';
}

export function accountProviderLabel(providerKind: string) {
	return providerKind
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

export function accountUpdatedLabel(account: ProviderAccount) {
	return formatDateTime(account.updated_at) || 'Never';
}

export function providerKindLabel(value: string) {
	return value
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

export function capabilityLabel(value: string) {
	return value
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}
