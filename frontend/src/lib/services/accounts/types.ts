import type { TelegramProviderKind } from '$lib/api';

export type Provider = 'gmail' | 'icloud' | 'imap';
export type AccountWizardKind = 'mail' | 'calendar' | 'telegram' | 'whatsapp';
export type AccountWizardTarget = AccountWizardKind | Provider;
export type MailService =
	| Provider
	| 'microsoft'
	| 'exchange'
	| 'fastmail'
	| 'mailru'
	| 'yandex'
	| 'proton'
	| 'yahoo'
	| 'aol';
export type MailWizardStep = 'provider' | 'details';
export type CalendarProvider = 'local' | 'google' | 'microsoft' | 'apple' | 'caldav' | 'ics';
export type CalendarWizardStep = 'provider' | 'details';
export type TelegramSetupMode = 'fixture' | 'live';
export type TelegramAuthMethod = 'fixture' | 'phone' | 'qr' | 'bot_token';
export type TelegramWizardStep = 'account' | 'auth' | 'details';

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
