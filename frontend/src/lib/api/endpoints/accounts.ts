import { ApiClient } from '../client';
import type {
	GmailOAuthStartRequest,
	GmailOAuthStartResponse,
	GmailOAuthCompleteRequest,
	EmailAccountSetupResponse,
	ImapAccountSetupRequest
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
