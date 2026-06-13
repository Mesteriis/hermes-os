import { createCalendarAccount } from '$lib/api';
import type {
	CalendarProvider,
	CalendarWizardStep
} from './types';

export function continueCalendarWizard(
	provider: CalendarProvider | undefined,
	calendarForm: Record<string, string>
): {
	calendarWizardStep: CalendarWizardStep;
	calendarForm: Record<string, string>;
} {
	const nextForm = provider
		? { ...calendarForm, provider, account_name: calendarProviderDefaultName(provider) }
		: calendarForm;
	return { calendarWizardStep: 'details', calendarForm: nextForm };
}

export function calendarProviderDefaultName(provider: CalendarProvider) {
	switch (provider) {
		case 'google':
			return 'Google Calendar';
		case 'microsoft':
			return 'Microsoft Calendar';
		case 'apple':
			return 'Apple Calendar';
		case 'caldav':
			return 'CalDAV Calendar';
		case 'ics':
			return 'ICS Feed';
		default:
			return 'Local Calendar';
	}
}

export async function saveCalendarAccount(params: {
	provider: CalendarProvider;
	account_name: string;
	email: string;
}): Promise<{
	message: string;
	error: string;
}> {
	try {
		const result = await createCalendarAccount({
			provider: params.provider,
			account_name: params.account_name,
			email: params.email.trim() || undefined
		});
		return { message: `Calendar account ${result.account_name} saved`, error: '' };
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Calendar account setup failed'
		};
	}
}
