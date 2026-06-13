import type {
	MailService,
	MailWizardStep,
	Provider
} from './types';

export function selectMailService(
	service: MailService,
	gmailForm: Record<string, string>,
	imapForm: Record<string, unknown>
): {
	selectedProvider: Provider;
	selectedMailService: MailService;
	gmailForm: Record<string, string>;
	imapForm: Record<string, unknown>;
} {
	if (service === 'gmail') {
		return {
			selectedProvider: 'gmail',
			selectedMailService: service,
			gmailForm: {
				...gmailForm,
				account_id: gmailForm.account_id || 'gmail-primary',
				display_name: gmailForm.display_name || 'Primary Gmail'
			},
			imapForm
		};
	}

	const preset = mailServicePreset(service, imapForm);
	return {
		selectedProvider: preset.provider,
		selectedMailService: service,
		gmailForm,
		imapForm: {
			...imapForm,
			account_id: preset.accountId,
			display_name: preset.displayName,
			host: preset.host,
			port: preset.port,
			tls: true,
			mailbox: (imapForm.mailbox as string) || 'INBOX',
			secret_kind: preset.secretKind,
			smtp_host: preset.smtpHost,
			smtp_port: preset.smtpPort,
			smtp_tls: preset.smtpTls,
			smtp_starttls: preset.smtpStarttls
		}
	};
}

export function continueMailWizard(
	email: string,
	gmailForm: Record<string, string>,
	imapForm: Record<string, unknown>
): {
	mailWizardStep: MailWizardStep;
	gmailForm: Record<string, string>;
	imapForm: Record<string, unknown>;
} {
	if (email) {
		const nextGmailForm = {
			...gmailForm,
			external_account_id: email,
			account_id: gmailForm.account_id || accountIdFromEmail(email, 'gmail'),
			display_name: gmailForm.display_name || email
		};
		const nextImapForm = {
			...imapForm,
			external_account_id: email,
			username: (imapForm.username as string) || email,
			account_id: imapForm.account_id || accountIdFromEmail(email, 'mail'),
			display_name: imapForm.display_name || email
		};
		return { mailWizardStep: 'details', gmailForm: nextGmailForm, imapForm: nextImapForm };
	}
	return { mailWizardStep: 'details', gmailForm, imapForm };
}

export function selectProvider(
	provider: Provider,
	imapForm: Record<string, unknown>
): {
	selectedProvider: Provider;
	selectedMailService: MailService;
	imapForm: Record<string, unknown>;
} {
	let nextImapForm = imapForm;
	if (provider === 'icloud') {
		nextImapForm = {
			...nextImapForm,
			account_id: (nextImapForm.account_id as string) || 'icloud-primary',
			display_name: (nextImapForm.display_name as string) || 'Primary iCloud',
			host: 'imap.mail.me.com',
			port: 993,
			tls: true,
			mailbox: (nextImapForm.mailbox as string) || 'INBOX',
			secret_kind: 'app_password'
		};
	}
	if (provider === 'imap') {
		nextImapForm = {
			...nextImapForm,
			account_id: nextImapForm.account_id === 'icloud-primary' ? 'imap-primary' : nextImapForm.account_id,
			display_name: nextImapForm.display_name === 'Primary iCloud' ? 'Primary IMAP' : nextImapForm.display_name,
			host: nextImapForm.host === 'imap.mail.me.com' ? '' : nextImapForm.host,
			secret_kind: 'password'
		};
	}
	return {
		selectedProvider: provider,
		selectedMailService: provider,
		imapForm: nextImapForm
	};
}

export function mailServicePreset(service: MailService, imapForm: Record<string, unknown>): {
	provider: Provider;
	accountId: string;
	displayName: string;
	host: string;
	port: number;
	smtpHost: string;
	smtpPort: number;
	smtpTls: boolean;
	smtpStarttls: boolean;
	secretKind: 'app_password' | 'password';
} {
	switch (service) {
		case 'icloud':
			return {
				provider: 'icloud',
				accountId: 'icloud-primary',
				displayName: 'Primary iCloud',
				host: 'imap.mail.me.com',
				port: 993,
				smtpHost: 'smtp.mail.me.com',
				smtpPort: 587,
				smtpTls: true,
				smtpStarttls: true,
				secretKind: 'app_password'
			};
		case 'microsoft':
			return {
				provider: 'imap',
				accountId: 'microsoft-primary',
				displayName: 'Microsoft 365 / Exchange Online',
				host: 'outlook.office365.com',
				port: 993,
				smtpHost: 'smtp.office365.com',
				smtpPort: 587,
				smtpTls: true,
				smtpStarttls: true,
				secretKind: 'password'
			};
		case 'fastmail':
			return {
				provider: 'imap',
				accountId: 'fastmail-primary',
				displayName: 'Fastmail',
				host: 'imap.fastmail.com',
				port: 993,
				smtpHost: 'smtp.fastmail.com',
				smtpPort: 587,
				smtpTls: true,
				smtpStarttls: true,
				secretKind: 'app_password'
			};
		case 'mailru':
			return {
				provider: 'imap',
				accountId: 'mailru-primary',
				displayName: 'Mail.ru',
				host: 'imap.mail.ru',
				port: 993,
				smtpHost: 'smtp.mail.ru',
				smtpPort: 465,
				smtpTls: true,
				smtpStarttls: false,
				secretKind: 'app_password'
			};
		case 'yandex':
			return {
				provider: 'imap',
				accountId: 'yandex-primary',
				displayName: 'Yandex Mail',
				host: 'imap.yandex.com',
				port: 993,
				smtpHost: 'smtp.yandex.com',
				smtpPort: 465,
				smtpTls: true,
				smtpStarttls: false,
				secretKind: 'app_password'
			};
		case 'proton':
			return {
				provider: 'imap',
				accountId: 'proton-bridge',
				displayName: 'Proton Bridge',
				host: '127.0.0.1',
				port: 1143,
				smtpHost: '127.0.0.1',
				smtpPort: 1025,
				smtpTls: false,
				smtpStarttls: false,
				secretKind: 'password'
			};
		case 'yahoo':
			return {
				provider: 'imap',
				accountId: 'yahoo-primary',
				displayName: 'Yahoo Mail',
				host: 'imap.mail.yahoo.com',
				port: 993,
				smtpHost: 'smtp.mail.yahoo.com',
				smtpPort: 587,
				smtpTls: true,
				smtpStarttls: true,
				secretKind: 'app_password'
			};
		case 'aol':
			return {
				provider: 'imap',
				accountId: 'aol-primary',
				displayName: 'AOL Mail',
				host: 'imap.aol.com',
				port: 993,
				smtpHost: 'smtp.aol.com',
				smtpPort: 587,
				smtpTls: true,
				smtpStarttls: true,
				secretKind: 'app_password'
			};
		default:
			{
				const host =
					(imapForm.host as string) === 'imap.mail.me.com' ? '' : ((imapForm.host as string) ?? '');
				return {
					provider: 'imap',
					accountId: service === 'exchange' ? 'exchange-primary' : 'imap-primary',
					displayName: service === 'exchange' ? 'Exchange IMAP' : 'IMAP Mail',
					host,
					port: Number(imapForm.port) || 993,
					smtpHost: (imapForm.smtp_host as string) ?? '',
					smtpPort: Number(imapForm.smtp_port) || 587,
					smtpTls: typeof imapForm.smtp_tls === 'boolean' ? (imapForm.smtp_tls as boolean) : true,
					smtpStarttls:
						typeof imapForm.smtp_starttls === 'boolean'
							? (imapForm.smtp_starttls as boolean)
							: true,
					secretKind: 'password'
				};
			}
	}
}

export function hasFixedMailServerPreset(service: MailService) {
	return service !== 'imap' && service !== 'exchange';
}

export function mailServiceDisplayName(service: MailService) {
	switch (service) {
		case 'gmail':
			return 'Google';
		case 'icloud':
			return 'iCloud';
		case 'microsoft':
			return 'Microsoft 365 / Exchange Online';
		case 'exchange':
			return 'Exchange IMAP';
		case 'fastmail':
			return 'Fastmail';
		case 'mailru':
			return 'Mail.ru';
		case 'yandex':
			return 'Yandex Mail';
		case 'proton':
			return 'Proton Bridge';
		case 'yahoo':
			return 'Yahoo';
		case 'aol':
			return 'AOL';
		default:
			return 'Other Mail Account';
	}
}

export function mailServiceIcon(service: MailService) {
	switch (service) {
		case 'gmail':
			return 'tabler:brand-gmail';
		case 'icloud':
			return 'tabler:cloud';
		case 'microsoft':
			return 'tabler:brand-office';
		case 'exchange':
			return 'tabler:server-cog';
		case 'fastmail':
			return 'tabler:mail-fast';
		case 'mailru':
			return 'tabler:mail';
		case 'yandex':
			return 'tabler:letter-y';
		case 'proton':
			return 'tabler:shield-lock';
		case 'yahoo':
			return 'tabler:mail';
		case 'aol':
			return 'tabler:mail-bolt';
		default:
			return 'tabler:server';
	}
}

export function mailServiceAccountPrefix(service: MailService) {
	switch (service) {
		case 'icloud':
			return 'icloud';
		case 'microsoft':
			return 'microsoft';
		case 'exchange':
			return 'exchange';
		case 'fastmail':
			return 'fastmail';
		case 'mailru':
			return 'mailru';
		case 'yandex':
			return 'yandex';
		case 'proton':
			return 'proton';
		case 'yahoo':
			return 'yahoo';
		case 'aol':
			return 'aol';
		case 'gmail':
			return 'gmail';
		default:
			return 'imap';
	}
}

export function inferMailService(email: string): MailService | null {
	const domain = email.split('@')[1]?.trim().toLowerCase() ?? '';
	if (['gmail.com', 'googlemail.com'].includes(domain)) return 'gmail';
	if (['icloud.com', 'me.com', 'mac.com'].includes(domain)) return 'icloud';
	if (['outlook.com', 'hotmail.com', 'live.com', 'office365.com'].includes(domain)) return 'microsoft';
	if (['fastmail.com', 'fastmail.fm'].includes(domain)) return 'fastmail';
	if (['mail.ru', 'inbox.ru', 'list.ru', 'bk.ru'].includes(domain)) return 'mailru';
	if (['yandex.com', 'yandex.ru', 'ya.ru'].includes(domain)) return 'yandex';
	if (['proton.me', 'protonmail.com', 'pm.me'].includes(domain)) return 'proton';
	if (domain.endsWith('yahoo.com')) return 'yahoo';
	if (domain === 'aol.com') return 'aol';
	return null;
}

export function accountIdFromEmail(email: string, fallback: string) {
	const normalized = email
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '');
	return normalized ? `${fallback}-${normalized}` : `${fallback}-primary`;
}
