import { ApiClient } from '../../../platform/api/ApiClient'

export type EmailAccountSetupSecretKind = 'app_password' | 'password' | 'oauth_token'
export type EmailAccountSetupStoreKind =
	| 'host_vault'
	| 'os_keychain'
	| 'encrypted_vault'
	| 'database_encrypted_vault'
	| 'external_vault'
	| string

export type GmailOAuthStartRequest = {
	account_id: string
	display_name: string
	external_account_id?: string
	redirect_uri: string
	app_return_url?: string
	scopes?: string[]
}

export type GmailOAuthStartResponse = {
	setup_id: string
	authorization_url: string
	state: string
	redirect_uri: string
}

export type ImapEmailAccountSetupRequest = {
	account_id: string
	provider_kind: 'icloud' | 'imap'
	display_name: string
	external_account_id: string
	host: string
	port: number
	tls: boolean
	mailbox: string
	username: string
	password: string
	secret_kind: 'app_password' | 'password'
	smtp_host?: string
	smtp_port?: number
	smtp_tls?: boolean
	smtp_starttls?: boolean
	smtp_username?: string
}

export type EmailAccountSetupResponse = {
	account_id: string
	secret_ref: string
	secret_kind: EmailAccountSetupSecretKind
	store_kind: EmailAccountSetupStoreKind
}

export async function startGmailOAuthSetup(
	request: GmailOAuthStartRequest
): Promise<GmailOAuthStartResponse> {
	return ApiClient.instance.post<GmailOAuthStartResponse>(
		'/api/v1/integrations/mail/accounts/gmail/oauth/start',
		request,
		'Gmail OAuth setup start failed'
	)
}

export async function setupImapEmailAccount(
	request: ImapEmailAccountSetupRequest
): Promise<EmailAccountSetupResponse> {
	return ApiClient.instance.post<EmailAccountSetupResponse>(
		'/api/v1/integrations/mail/accounts/imap',
		request,
		'IMAP account setup failed'
	)
}
