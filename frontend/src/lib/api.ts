export type V1Status = {
	version: string;
	surfaces: {
		messages: boolean;
		contacts: boolean;
		search: boolean;
		documents: boolean;
		account_setup: boolean;
	};
};

export type GmailOAuthStartRequest = {
	account_id: string;
	display_name: string;
	external_account_id: string;
	client_id: string;
	client_secret?: string;
	redirect_uri: string;
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
};

export type EmailAccountSetupResponse = {
	account_id: string;
	secret_ref: string;
	secret_kind: 'oauth_token' | 'app_password' | 'password';
	store_kind: 'encrypted_vault';
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
};

export async function fetchV1Status(
	baseUrl: string,
	token: string,
	actorId: string
): Promise<V1Status> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}/api/v1/status`, {
		headers: {
			Authorization: `Bearer ${token}`,
			'X-Hermes-Actor-Id': actorId
		}
	});

	if (!response.ok) {
		throw new Error(`V1 status request failed: ${response.status}`);
	}

	return (await response.json()) as V1Status;
}

export async function startGmailOAuthSetup(
	baseUrl: string,
	token: string,
	actorId: string,
	request: GmailOAuthStartRequest
): Promise<GmailOAuthStartResponse> {
	return postJson(baseUrl, token, actorId, '/api/v1/email-accounts/gmail/oauth/start', request);
}

export async function completeGmailOAuthSetup(
	baseUrl: string,
	token: string,
	actorId: string,
	request: GmailOAuthCompleteRequest
): Promise<EmailAccountSetupResponse> {
	return postJson(baseUrl, token, actorId, '/api/v1/email-accounts/gmail/oauth/complete', request);
}

export async function setupImapAccount(
	baseUrl: string,
	token: string,
	actorId: string,
	request: ImapAccountSetupRequest
): Promise<EmailAccountSetupResponse> {
	return postJson(baseUrl, token, actorId, '/api/v1/email-accounts/imap', request);
}

async function postJson<TResponse>(
	baseUrl: string,
	token: string,
	actorId: string,
	path: string,
	body: unknown
): Promise<TResponse> {
	const normalizedBaseUrl = baseUrl.replace(/\/+$/, '');
	const response = await fetch(`${normalizedBaseUrl}${path}`, {
		method: 'POST',
		headers: {
			Authorization: `Bearer ${token}`,
			'Content-Type': 'application/json',
			'X-Hermes-Actor-Id': actorId
		},
		body: JSON.stringify(body)
	});

	if (!response.ok) {
		const error = (await response.json().catch(() => null)) as
			| { message?: string }
			| null;
		throw new Error(error?.message ?? `Account setup request failed: ${response.status}`);
	}

	return (await response.json()) as TResponse;
}
