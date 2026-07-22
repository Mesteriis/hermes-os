import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
	setupImapEmailAccount,
	startGmailOAuthSetup
} from './accountSetup'

describe('account setup API', () => {
	beforeEach(() => {
		ApiClient.resetForTests()
		ApiClient.init('http://127.0.0.1:8080', 'test-secret')
	})

	afterEach(() => {
		vi.unstubAllGlobals()
		ApiClient.resetForTests()
	})

	it('starts Gmail OAuth through the protected email account setup endpoint', async () => {
		const fetchMock = vi.fn().mockResolvedValue(
			new Response(
				JSON.stringify({
					setup_id: 'setup-1',
					authorization_url: 'https://accounts.google.com/o/oauth2/v2/auth?state=oauth-state',
					state: 'oauth-state',
					redirect_uri: 'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback'
				}),
				{ status: 200, headers: { 'Content-Type': 'application/json' } }
			)
		)
		vi.stubGlobal('fetch', fetchMock)

		const response = await startGmailOAuthSetup({
			account_id: 'mail-gmail-user-gmail-com',
			display_name: 'user@gmail.com',
			external_account_id: 'user@gmail.com',
			redirect_uri: 'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback'
		})

		expect(response.setup_id).toBe('setup-1')
		expect(fetchMock).toHaveBeenCalledOnce()
		const [url, init] = fetchMock.mock.calls[0]
		expect(url).toBe('http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/start')
		expect(init.method).toBe('POST')
		expect(init.headers['X-Hermes-Secret']).toBe('test-secret')
		expect(JSON.parse(init.body as string)).toEqual({
			account_id: 'mail-gmail-user-gmail-com',
			display_name: 'user@gmail.com',
			external_account_id: 'user@gmail.com',
			redirect_uri: 'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback'
		})
	})

	it('creates IMAP-backed accounts through the protected setup endpoint', async () => {
		const fetchMock = vi.fn().mockResolvedValue(
			new Response(
				JSON.stringify({
					account_id: 'mail-imap-user-example-com',
					secret_ref: 'secret://mail/mail-imap-user-example-com',
					secret_kind: 'imap_password_file',
					store_kind: 'runtime_secret_store'
				}),
				{ status: 200, headers: { 'Content-Type': 'application/json' } }
			)
		)
		vi.stubGlobal('fetch', fetchMock)

		const response = await setupImapEmailAccount({
			account_id: 'mail-imap-user-example-com',
			provider_kind: 'imap',
			display_name: 'User',
			external_account_id: 'user@example.com',
			host: 'imap.example.com',
			port: 993,
			tls: true,
			mailbox: 'INBOX',
			username: 'user@example.com',
			password: 'mailbox-password',
			secret_kind: 'imap_password_file'
		})

		expect(response.account_id).toBe('mail-imap-user-example-com')
		expect(fetchMock).toHaveBeenCalledOnce()
		const [url, init] = fetchMock.mock.calls[0]
		expect(url).toBe('http://127.0.0.1:8080/api/v1/integrations/mail/accounts/imap')
		expect(init.method).toBe('POST')
		expect(JSON.parse(init.body as string)).toMatchObject({
			account_id: 'mail-imap-user-example-com',
			provider_kind: 'imap',
			password: 'mailbox-password',
			secret_kind: 'imap_password_file'
		})
	})
})
