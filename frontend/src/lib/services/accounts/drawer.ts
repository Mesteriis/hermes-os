import type {
	AccountWizardKind,
	AccountWizardTarget,
	CalendarWizardStep,
	MailService,
	MailWizardStep,
	TelegramWizardStep
} from './types';

export function openAccountDrawer(target: AccountWizardTarget): {
	wizardKind: AccountWizardKind;
	mailWizardStep: MailWizardStep;
	calendarWizardStep: CalendarWizardStep;
	telegramWizardStep: TelegramWizardStep;
	mailService: MailService | null;
} {
	const accountWizardKind: AccountWizardKind =
		target === 'gmail' || target === 'icloud' || target === 'imap' ? 'mail' : target;
	let mailService: MailService | null = null;
	if (target === 'gmail' || target === 'icloud' || target === 'imap') {
		mailService = target;
	}
	return {
		wizardKind: accountWizardKind,
		mailWizardStep: 'provider',
		calendarWizardStep: 'provider',
		telegramWizardStep: 'account',
		mailService
	};
}

export function closeAccountDrawer(): boolean {
	return false;
}
