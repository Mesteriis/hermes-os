import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiClient } from '../client';
import {
	deleteEmailAccount,
	exportEmailAccount,
	fetchEmailAccount,
	fetchEmailAccounts,
	importEmailAccount,
	logoutEmailAccount
} from './accounts';

const account = {
	account_id: 'fastmail-primary',
	provider_kind: 'imap',
	display_name: 'Fastmail',
	external_account_id: 'alex@example.com',
	config: {
		host: 'imap.fastmail.com',
		port: 993,
		smtp_host: 'smtp.fastmail.com',
		smtp_port: 587
	},
	created_at: '2026-06-13T10:00:00Z',
	updated_at: '2026-06-13T10:00:00Z'
};

const capabilities = {
	read: true,
	sync: true,
	send: true,
	oauth: false,
	imap: true,
	smtp: true,
	mutate_flags: true,
	mutate_mailboxes: false,
	server_delete: false,
	provider_folders: false,
	local_trash: true
};

describe('mail account API endpoints', () => {
	beforeEach(() => {
		ApiClient.init('http://127.0.0.1:8080', 'local-secret');
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string, init?: RequestInit) => {
				if (String(url).endsWith('/api/v1/email-accounts')) {
					return jsonResponse({ items: [{ account, capabilities }] });
				}
				if (String(url).endsWith('/export')) {
					return jsonResponse({
						exported_at: '2026-06-13T10:00:00Z',
						account,
						capabilities,
						sync_settings: {
							account_id: account.account_id,
							sync_enabled: true,
							batch_size: 25,
							poll_interval_seconds: 900,
							updated_at: '2026-06-13T10:00:00Z'
						}
					});
				}
				if (String(url).endsWith('/logout')) {
					return jsonResponse({
						account: { ...account, config: { ...account.config, auth_state: 'logged_out' } },
						capabilities: { ...capabilities, read: false, sync: false, send: false },
						sync_settings: {
							account_id: account.account_id,
							sync_enabled: false,
							batch_size: 25,
							poll_interval_seconds: 900,
							updated_at: '2026-06-13T10:00:00Z'
						}
					});
				}
				if (init?.method === 'POST' && String(url).endsWith('/import')) {
					return jsonResponse({
						account,
						capabilities,
						sync_settings: {
							account_id: account.account_id,
							sync_enabled: true,
							batch_size: 25,
							poll_interval_seconds: 900,
							updated_at: '2026-06-13T10:00:00Z'
						}
					});
				}
				if (init?.method === 'DELETE') {
					return jsonResponse({
						account_id: account.account_id,
						deleted: true,
						unbound_secret_refs: []
					});
				}
				return jsonResponse({ account, capabilities });
			})
		);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('lists and reads mail accounts through dedicated account routes', async () => {
		await fetchEmailAccounts();
		await fetchEmailAccount('fastmail primary');

		const fetchMock = vi.mocked(fetch);
		expect(fetchMock.mock.calls[0][0]).toBe('http://127.0.0.1:8080/api/v1/email-accounts');
		expect(fetchMock.mock.calls[0][1]?.headers).toEqual({ 'X-Hermes-Secret': 'local-secret' });
		expect(fetchMock.mock.calls[1][0]).toBe(
			'http://127.0.0.1:8080/api/v1/email-accounts/fastmail%20primary'
		);
	});

	it('exports, logs out, imports and deletes accounts with explicit methods', async () => {
		await exportEmailAccount('fastmail-primary');
		await logoutEmailAccount('fastmail-primary');
		await importEmailAccount({
			account: {
				account_id: 'fastmail-primary',
				provider_kind: 'imap',
				display_name: 'Fastmail',
				external_account_id: 'alex@example.com',
				config: { provider_preset: 'fastmail' }
			},
			sync_settings: {
				sync_enabled: true,
				batch_size: 25,
				poll_interval_seconds: 900
			}
		});
		await deleteEmailAccount('fastmail-primary');

		const fetchMock = vi.mocked(fetch);
		expect(fetchMock.mock.calls[0][0]).toBe(
			'http://127.0.0.1:8080/api/v1/email-accounts/fastmail-primary/export'
		);
		expect(fetchMock.mock.calls[1][1]?.method).toBe('POST');
		expect(fetchMock.mock.calls[1][0]).toBe(
			'http://127.0.0.1:8080/api/v1/email-accounts/fastmail-primary/logout'
		);
		expect(fetchMock.mock.calls[2][0]).toBe(
			'http://127.0.0.1:8080/api/v1/email-accounts/import'
		);
		expect(fetchMock.mock.calls[2][1]?.method).toBe('POST');
		expect(JSON.parse(String(fetchMock.mock.calls[2][1]?.body))).toMatchObject({
			account: { account_id: 'fastmail-primary', provider_kind: 'imap' }
		});
		expect(fetchMock.mock.calls[3][1]?.method).toBe('DELETE');
	});
});

function jsonResponse(body: unknown): Response {
	return new Response(JSON.stringify(body), {
		status: 200,
		headers: { 'Content-Type': 'application/json' }
	});
}
