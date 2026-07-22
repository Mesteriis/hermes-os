import { describe, expect, it } from 'vitest'
import {
	accountSetupDefaultAccountId,
	accountSetupFormDefaults,
	accountSetupFormSchema,
	accountSetupFormToGmailOAuthStart,
	accountSetupFormToImapRequest
} from './accountSetupForm'

describe('account setup form', () => {
	it('normalizes iCloud values into the protected IMAP setup payload', () => {
		const values = accountSetupFormSchema.parse({
			...accountSetupFormDefaults('icloud'),
			display_name: '  Personal iCloud  ',
			email: ' User@iCloud.com ',
			password: 'icloud-app-password'
		})

		expect(accountSetupFormToImapRequest(values)).toEqual({
			account_id: 'mail-icloud-user-icloud-com',
			provider_kind: 'icloud',
			display_name: 'Personal iCloud',
			external_account_id: 'user@icloud.com',
			host: 'imap.mail.me.com',
			port: 993,
			tls: true,
			mailbox: 'INBOX',
			username: 'user@icloud.com',
			password: 'icloud-app-password',
			secret_kind: 'app_password'
		})
	})

	it('normalizes generic IMAP values with SMTP settings', () => {
		const values = accountSetupFormSchema.parse({
			...accountSetupFormDefaults('imap'),
			email: 'sender@example.com',
			password: 'mailbox-password',
			imap_host: ' imap.example.com ',
			imap_port: 143,
			imap_tls: false,
			username: ' sender@example.com ',
			smtp_host: ' smtp.example.com ',
			smtp_port: 587,
			smtp_tls: false,
			smtp_starttls: true,
			smtp_username: ' sender@example.com '
		})

		expect(accountSetupFormToImapRequest(values)).toEqual({
			account_id: 'mail-imap-sender-example-com',
			provider_kind: 'imap',
			display_name: 'sender@example.com',
			external_account_id: 'sender@example.com',
			host: 'imap.example.com',
			port: 143,
			tls: false,
			mailbox: 'INBOX',
			username: 'sender@example.com',
			password: 'mailbox-password',
			secret_kind: 'imap_password_file',
			smtp_host: 'smtp.example.com',
			smtp_port: 587,
			smtp_tls: false,
			smtp_starttls: true,
			smtp_username: 'sender@example.com'
		})
	})

	it('rejects provider-specific missing credential and host fields', () => {
		const result = accountSetupFormSchema.safeParse({
			...accountSetupFormDefaults('imap'),
			email: 'sender@example.com',
			password: '',
			imap_host: ''
		})

		expect(result.success).toBe(false)
		if (!result.success) {
			expect(result.error.issues.map((issue) => issue.path.join('.'))).toEqual([
				'password',
				'imap_host'
			])
		}
	})

	it('builds Gmail OAuth start requests without password fields', () => {
		const values = accountSetupFormSchema.parse({
			...accountSetupFormDefaults('gmail'),
			display_name: '',
			email: ' User@Gmail.com ',
			password: 'ignored-password'
		})

		expect(accountSetupFormToGmailOAuthStart(values, 'http://127.0.0.1:8080/')).toEqual({
			account_id: 'mail-gmail-user-gmail-com',
			display_name: 'user@gmail.com',
			external_account_id: 'user@gmail.com',
			redirect_uri: 'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback',
			app_return_url: 'http://127.0.0.1:5174/?hermes_route=settings&hermes_oauth=gmail_connected'
		})
	})

	it('generates stable safe account ids from provider and email', () => {
		expect(accountSetupDefaultAccountId('imap', 'Team.Mail+Archive@example.org')).toBe(
			'mail-imap-team-mail-archive-example-org'
		)
	})
})
