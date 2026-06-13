import {
	completeGmailOAuthSetup,
	setupImapAccount,
	startGmailOAuthSetup,
	type GmailOAuthStartRequest,
	type GmailOAuthStartResponse
} from '$lib/api';
import { optionalTrimmed } from './shared';
import { GOOGLE_WORKSPACE_OAUTH_SCOPES, type MailService, type Provider } from './types';
import {
	accountIdFromEmail,
	hasFixedMailServerPreset,
	mailServiceAccountPrefix,
	mailServiceDisplayName
} from './mailWizard';

type ImapAccountSetup = typeof setupImapAccount;

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

export async function saveImapAccount(
	params: {
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
			smtp_host?: string;
			smtp_port?: number;
			smtp_tls?: boolean;
			smtp_starttls?: boolean;
			smtp_username?: string;
		};
	},
	setupAccount: ImapAccountSetup = setupImapAccount
): Promise<{
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

		const result = await setupAccount({
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
			secret_kind: imapForm.secret_kind,
			smtp_host: imapForm.smtp_host?.trim(),
			smtp_port: imapForm.smtp_port,
			smtp_tls: imapForm.smtp_tls,
			smtp_starttls: imapForm.smtp_starttls,
			smtp_username: imapForm.smtp_username?.trim() || username
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
