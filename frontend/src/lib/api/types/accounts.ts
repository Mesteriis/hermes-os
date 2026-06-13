import type { MailSyncSettings } from './mail';

export type ProviderAccount = {
	account_id: string;
	provider_kind:
		| 'gmail'
		| 'icloud'
		| 'imap'
		| 'telegram_user'
		| 'telegram_bot'
		| 'whatsapp_web'
		| string;
	display_name: string;
	external_account_id: string;
	config: Record<string, unknown>;
	created_at: string;
	updated_at: string;
};

export type ProviderAccountListResponse = {
	items: ProviderAccount[];
};

export type EmailAccountCapabilities = {
	read: boolean;
	sync: boolean;
	send: boolean;
	oauth: boolean;
	imap: boolean;
	smtp: boolean;
	mutate_flags: boolean;
	mutate_mailboxes: boolean;
	server_delete: boolean;
	provider_folders: boolean;
	local_trash: boolean;
};

export type EmailAccountView = {
	account: ProviderAccount;
	capabilities: EmailAccountCapabilities;
};

export type EmailAccountListResponse = {
	items: EmailAccountView[];
};

export type EmailAccountExportResponse = {
	exported_at: string;
	account: ProviderAccount;
	capabilities: EmailAccountCapabilities;
	sync_settings: MailSyncSettings;
};

export type EmailAccountLogoutResponse = {
	account: ProviderAccount;
	capabilities: EmailAccountCapabilities;
	sync_settings: MailSyncSettings;
};

export type EmailAccountDeleteResponse = {
	account_id: string;
	deleted: boolean;
	unbound_secret_refs: string[];
};

export type EmailAccountImportRequest = {
	account: {
		account_id: string;
		provider_kind: 'gmail' | 'icloud' | 'imap';
		display_name: string;
		external_account_id: string;
		config?: Record<string, unknown>;
	};
	sync_settings?: {
		sync_enabled?: boolean;
		batch_size?: number;
		poll_interval_seconds?: number;
	};
};

export type GmailOAuthStartRequest = {
	account_id: string;
	display_name: string;
	external_account_id?: string;
	client_id?: string;
	client_secret?: string;
	redirect_uri: string;
	app_return_url?: string;
	scopes?: string[];
};

export type GmailOAuthStartResponse = {
	setup_id: string;
	authorization_url: string;
	state: string;
	redirect_uri: string;
};

export type GmailOAuthCompleteRequest = {
	setup_id: string;
	state: string;
	authorization_code: string;
	external_account_id?: string;
};

export type EmailAccountSetupResponse = {
	account_id: string;
	secret_ref: string;
	secret_kind: 'oauth_token' | 'app_password' | 'password';
	store_kind: 'encrypted_vault' | 'database_encrypted_vault' | 'host_vault';
};

export type ImapAccountSetupRequest = {
	account_id: string;
	provider_kind: 'icloud' | 'imap';
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
