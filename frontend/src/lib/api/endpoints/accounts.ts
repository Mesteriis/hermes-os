import { ApiClient } from '../client';
import type {
	GmailOAuthStartRequest,
	GmailOAuthStartResponse,
	GmailOAuthCompleteRequest,
	EmailAccountSetupResponse,
	ImapAccountSetupRequest,
	EmailAccountListResponse,
	EmailAccountView,
	EmailAccountExportResponse,
	EmailAccountLogoutResponse,
	EmailAccountDeleteResponse,
	EmailAccountImportRequest
} from '../types';

export async function startGmailOAuthSetup(
	request: GmailOAuthStartRequest
): Promise<GmailOAuthStartResponse> {
	return ApiClient.instance.post<GmailOAuthStartResponse>(
		'/api/v1/email-accounts/gmail/oauth/start',
		request
	);
}

export async function completeGmailOAuthSetup(
	request: GmailOAuthCompleteRequest
): Promise<EmailAccountSetupResponse> {
	return ApiClient.instance.post<EmailAccountSetupResponse>(
		'/api/v1/email-accounts/gmail/oauth/complete',
		request
	);
}

export async function setupImapAccount(
	request: ImapAccountSetupRequest
): Promise<EmailAccountSetupResponse> {
	return ApiClient.instance.post<EmailAccountSetupResponse>('/api/v1/email-accounts/imap', request);
}

export async function fetchEmailAccounts(): Promise<EmailAccountListResponse> {
	return ApiClient.instance.get<EmailAccountListResponse>(
		'/api/v1/email-accounts',
		'Email accounts request failed'
	);
}

export async function fetchEmailAccount(accountId: string): Promise<EmailAccountView> {
	return ApiClient.instance.get<EmailAccountView>(
		`/api/v1/email-accounts/${encodeURIComponent(accountId)}`,
		'Email account request failed'
	);
}

export async function exportEmailAccount(accountId: string): Promise<EmailAccountExportResponse> {
	return ApiClient.instance.get<EmailAccountExportResponse>(
		`/api/v1/email-accounts/${encodeURIComponent(accountId)}/export`,
		'Email account export failed'
	);
}

export async function importEmailAccount(
	request: EmailAccountImportRequest
): Promise<EmailAccountLogoutResponse> {
	return ApiClient.instance.post<EmailAccountLogoutResponse>(
		'/api/v1/email-accounts/import',
		request,
		'Email account import failed'
	);
}

export async function logoutEmailAccount(accountId: string): Promise<EmailAccountLogoutResponse> {
	return ApiClient.instance.post<EmailAccountLogoutResponse>(
		`/api/v1/email-accounts/${encodeURIComponent(accountId)}/logout`,
		{},
		'Email account logout failed'
	);
}

export async function deleteEmailAccount(accountId: string): Promise<EmailAccountDeleteResponse> {
	return ApiClient.instance.delete<EmailAccountDeleteResponse>(
		`/api/v1/email-accounts/${encodeURIComponent(accountId)}`,
		'Email account delete failed'
	);
}
