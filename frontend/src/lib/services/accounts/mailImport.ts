import {
	deleteEmailAccount,
	exportEmailAccount,
	fetchEmailAccounts,
	importEmailAccount,
	logoutEmailAccount,
	type EmailAccountDeleteResponse,
	type EmailAccountExportResponse,
	type EmailAccountImportRequest,
	type EmailAccountLogoutResponse,
	type EmailAccountView
} from '$lib/api';
import { safeAccountIdSegment } from './shared';

type EmailAccountListFetcher = typeof fetchEmailAccounts;
type EmailAccountExporter = typeof exportEmailAccount;
type EmailAccountImporter = typeof importEmailAccount;
type EmailAccountLogout = typeof logoutEmailAccount;
type EmailAccountDelete = typeof deleteEmailAccount;
type EmailImportProviderKind = EmailAccountImportRequest['account']['provider_kind'];

const EMAIL_IMPORT_PROVIDER_KINDS = new Set<EmailImportProviderKind>(['gmail', 'icloud', 'imap']);

export async function loadEmailAccountViews(
	fetcher: EmailAccountListFetcher = fetchEmailAccounts
): Promise<{ accounts: EmailAccountView[]; error: string }> {
	try {
		const response = await fetcher();
		return { accounts: response.items ?? [], error: '' };
	} catch (error) {
		return {
			accounts: [],
			error: error instanceof Error ? error.message : 'Mail account list failed'
		};
	}
}

export async function exportMailAccountSettings(
	accountId: string,
	exporter: EmailAccountExporter = exportEmailAccount
): Promise<{ result: EmailAccountExportResponse | null; error: string }> {
	const normalizedAccountId = accountId.trim();
	if (!normalizedAccountId) {
		return { result: null, error: 'Account id is required' };
	}
	try {
		return { result: await exporter(normalizedAccountId), error: '' };
	} catch (error) {
		return {
			result: null,
			error: error instanceof Error ? error.message : 'Mail account export failed'
		};
	}
}

export async function importMailAccountSettings(
	request: EmailAccountImportRequest,
	importer: EmailAccountImporter = importEmailAccount
): Promise<{ result: EmailAccountLogoutResponse | null; error: string }> {
	if (!request.account.account_id.trim()) {
		return { result: null, error: 'Account id is required' };
	}
	try {
		return { result: await importer(request), error: '' };
	} catch (error) {
		return {
			result: null,
			error: error instanceof Error ? error.message : 'Mail account import failed'
		};
	}
}

export function parseEmailAccountImportJson(payload: string): EmailAccountImportRequest {
	let parsed: unknown;
	try {
		parsed = JSON.parse(payload);
	} catch {
		throw new Error('Import settings must be valid JSON');
	}

	if (!isRecord(parsed) || !isRecord(parsed.account)) {
		throw new Error('Import settings must contain an account object');
	}

	const account = parsed.account;
	const providerKind = requiredImportString(account, 'provider_kind');
	if (!isEmailImportProviderKind(providerKind)) {
		throw new Error('Import settings provider_kind must be gmail, icloud or imap');
	}

	const request: EmailAccountImportRequest = {
		account: {
			account_id: requiredImportString(account, 'account_id'),
			provider_kind: providerKind,
			display_name: requiredImportString(account, 'display_name'),
			external_account_id: requiredImportString(account, 'external_account_id')
		}
	};

	if (account.config !== undefined) {
		if (!isRecord(account.config)) {
			throw new Error('Import settings account.config must be an object');
		}
		request.account.config = account.config;
	}

	if (parsed.sync_settings !== undefined) {
		if (!isRecord(parsed.sync_settings)) {
			throw new Error('Import settings sync_settings must be an object');
		}
		request.sync_settings = {};
		if (typeof parsed.sync_settings.sync_enabled === 'boolean') {
			request.sync_settings.sync_enabled = parsed.sync_settings.sync_enabled;
		}
		if (typeof parsed.sync_settings.batch_size === 'number') {
			request.sync_settings.batch_size = parsed.sync_settings.batch_size;
		}
		if (typeof parsed.sync_settings.poll_interval_seconds === 'number') {
			request.sync_settings.poll_interval_seconds = parsed.sync_settings.poll_interval_seconds;
		}
	}

	return request;
}

export function emailAccountExportFilename(accountId: string, exportedAt?: string): string {
	const accountSegment = safeAccountIdSegment(accountId) || 'account';
	const timestampSegment = exportedAt?.trim().replace(/[:.]/g, '-');
	return timestampSegment
		? `hermes-mail-account-${accountSegment}-${timestampSegment}.json`
		: `hermes-mail-account-${accountSegment}.json`;
}

export async function logoutMailAccount(
	accountId: string,
	logout: EmailAccountLogout = logoutEmailAccount
): Promise<{ result: EmailAccountLogoutResponse | null; error: string }> {
	const normalizedAccountId = accountId.trim();
	if (!normalizedAccountId) {
		return { result: null, error: 'Account id is required' };
	}
	try {
		return { result: await logout(normalizedAccountId), error: '' };
	} catch (error) {
		return {
			result: null,
			error: error instanceof Error ? error.message : 'Mail account logout failed'
		};
	}
}

export async function deleteMailAccount(
	accountId: string,
	deleteAccount: EmailAccountDelete = deleteEmailAccount
): Promise<{ result: EmailAccountDeleteResponse | null; error: string }> {
	const normalizedAccountId = accountId.trim();
	if (!normalizedAccountId) {
		return { result: null, error: 'Account id is required' };
	}
	try {
		return { result: await deleteAccount(normalizedAccountId), error: '' };
	} catch (error) {
		return {
			result: null,
			error: error instanceof Error ? error.message : 'Mail account delete failed'
		};
	}
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isEmailImportProviderKind(value: string): value is EmailImportProviderKind {
	return EMAIL_IMPORT_PROVIDER_KINDS.has(value as EmailImportProviderKind);
}

function requiredImportString(record: Record<string, unknown>, field: string): string {
	const value = record[field];
	if (typeof value !== 'string' || value.trim() === '') {
		throw new Error(`Import settings account.${field} must be a non-empty string`);
	}
	return value.trim();
}
