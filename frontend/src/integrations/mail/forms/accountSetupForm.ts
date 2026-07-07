import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type {
	GmailOAuthStartRequest,
	ImapEmailAccountSetupRequest
} from '../api/accountSetup'

const providerKinds = ['gmail', 'icloud', 'imap'] as const
const DEFAULT_GMAIL_APP_RETURN_ORIGIN = 'http://127.0.0.1:5174'

export type MailAccountSetupProvider = typeof providerKinds[number]
export type AccountSetupFormValues = z.infer<typeof accountSetupFormSchema>

const emailSchema = z
	.string()
	.trim()
	.email('Valid email address is required')
	.transform((value) => value.toLowerCase())

export const accountSetupFormSchema = z.object({
	provider_kind: z.enum(providerKinds),
	display_name: z.string().trim().max(120, 'Account name is too long'),
	email: emailSchema,
	password: z.string(),
	imap_host: z.string().trim(),
	imap_port: z.coerce.number().int().min(1, 'IMAP port is required').max(65535, 'IMAP port is invalid'),
	imap_tls: z.boolean(),
	mailbox: z.string().trim().min(1, 'Mailbox is required'),
	username: z.string().trim(),
	smtp_host: z.string().trim(),
	smtp_port: z.coerce.number().int().min(1, 'SMTP port is required').max(65535, 'SMTP port is invalid'),
	smtp_tls: z.boolean(),
	smtp_starttls: z.boolean(),
	smtp_username: z.string().trim()
}).superRefine((values, context) => {
	if (values.provider_kind === 'gmail') return

	if (values.password.trim().length === 0) {
		context.addIssue({
			code: z.ZodIssueCode.custom,
			message: values.provider_kind === 'icloud'
				? 'App password is required'
				: 'Password is required',
			path: ['password']
		})
	}
	if (values.imap_host.trim().length === 0) {
		context.addIssue({
			code: z.ZodIssueCode.custom,
			message: 'IMAP host is required',
			path: ['imap_host']
		})
	}
})

export const accountSetupVeeValidationSchema = toTypedSchema(accountSetupFormSchema)

export function accountSetupFormDefaults(
	provider: MailAccountSetupProvider
): AccountSetupFormValues {
	return {
		provider_kind: provider,
		display_name: '',
		email: '',
		password: '',
		imap_host: provider === 'icloud' ? 'imap.mail.me.com' : '',
		imap_port: 993,
		imap_tls: true,
		mailbox: 'INBOX',
		username: '',
		smtp_host: '',
		smtp_port: 587,
		smtp_tls: true,
		smtp_starttls: true,
		smtp_username: ''
	}
}

export function accountSetupDefaultAccountId(
	provider: MailAccountSetupProvider,
	email: string
): string {
	const slug = email
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '')
	return `mail-${provider}-${slug || 'account'}`
}

export function accountSetupFormToImapRequest(
	values: AccountSetupFormValues
): ImapEmailAccountSetupRequest {
	const parsed = accountSetupFormSchema.parse(values)
	if (parsed.provider_kind === 'gmail') {
		throw new Error('Gmail account setup must use OAuth')
	}

	const request: ImapEmailAccountSetupRequest = {
		account_id: accountSetupDefaultAccountId(parsed.provider_kind, parsed.email),
		provider_kind: parsed.provider_kind,
		display_name: parsed.display_name || parsed.email,
		external_account_id: parsed.email,
		host: parsed.imap_host,
		port: parsed.imap_port,
		tls: parsed.imap_tls,
		mailbox: parsed.mailbox,
		username: parsed.username || parsed.email,
		password: parsed.password,
		secret_kind: parsed.provider_kind === 'icloud' ? 'app_password' : 'password'
	}

	if (parsed.smtp_host || parsed.smtp_username) {
		request.smtp_host = parsed.smtp_host || undefined
		request.smtp_port = parsed.smtp_port
		request.smtp_tls = parsed.smtp_tls
		request.smtp_starttls = parsed.smtp_starttls
		request.smtp_username = parsed.smtp_username || undefined
	}

	return request
}

export function accountSetupFormToGmailOAuthStart(
	values: AccountSetupFormValues,
	apiBaseUrl: string
): GmailOAuthStartRequest {
	const parsed = accountSetupFormSchema.parse(values)
	return {
		account_id: accountSetupDefaultAccountId('gmail', parsed.email),
		display_name: parsed.display_name || parsed.email,
		external_account_id: parsed.email,
		redirect_uri: gmailOAuthRedirectUri(apiBaseUrl),
		app_return_url: gmailOAuthAppReturnUrl()
	}
}

function gmailOAuthRedirectUri(apiBaseUrl: string): string {
	return `${apiBaseUrl.replace(/\/+$/, '')}/api/v1/integrations/mail/accounts/gmail/oauth/callback`
}

function gmailOAuthAppReturnUrl(): string {
	const origin =
		typeof window === 'undefined'
			? DEFAULT_GMAIL_APP_RETURN_ORIGIN
			: window.location.origin
	const returnUrl = new URL('/', origin)
	returnUrl.searchParams.set('hermes_route', 'settings')
	returnUrl.searchParams.set('hermes_oauth', 'gmail_connected')
	return returnUrl.toString()
}
