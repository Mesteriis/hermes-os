import type { CalendarAccount, ProviderAccount } from '$lib/api';
import { accountProviderIcon, accountProviderLabel, accountUpdatedLabel } from './accounts';

export type IntegrationServiceId = 'mail' | 'calendar' | 'people' | 'messages';
export type IntegrationServiceState = 'ready' | 'unknown' | 'disabled' | 'not_applicable';
export type IntegrationStatus = 'connected' | 'partial' | 'empty';

export type IntegrationService = {
	id: IntegrationServiceId;
	label: string;
	state: IntegrationServiceState;
	description: string;
};

export type IntegrationViewModel = {
	integrationId: string;
	providerKind: string;
	title: string;
	subtitle: string;
	status: IntegrationStatus;
	icon: string;
	updatedAt: string | null;
	updatedLabel: string;
	services: IntegrationService[];
	accounts: ProviderAccount[];
	calendarAccounts: CalendarAccount[];
	metadata: Record<string, string>;
};

const SERVICE_IDS: IntegrationServiceId[] = ['mail', 'calendar', 'people', 'messages'];
const MAIL_PROVIDER_ORDER = ['gmail', 'icloud', 'imap'] as const;

type MailProviderKind = (typeof MAIL_PROVIDER_ORDER)[number];

export function buildIntegrationViewModels(
	providerAccounts: ProviderAccount[],
	calendarAccounts: CalendarAccount[]
): IntegrationViewModel[] {
	const integrations: IntegrationViewModel[] = [];

	for (const providerKind of MAIL_PROVIDER_ORDER) {
		for (const account of providerAccounts.filter((item) => item.provider_kind === providerKind)) {
			integrations.push(buildMailIntegration(account, providerKind, calendarAccounts));
		}
	}

	const telegramAccounts = providerAccounts.filter(isTelegramAccount);
	if (telegramAccounts.length > 0) {
		integrations.push(buildMessagingIntegration('telegram', 'telegram', 'Telegram', telegramAccounts));
	}

	const whatsappAccounts = providerAccounts.filter((account) => account.provider_kind === 'whatsapp_web');
	integrations.push(buildWhatsappIntegration(whatsappAccounts));

	return integrations;
}

export function serviceStateLabel(state: IntegrationServiceState): string {
	switch (state) {
		case 'ready':
			return 'Ready';
		case 'unknown':
			return 'Auth';
		case 'disabled':
			return 'Disabled';
		case 'not_applicable':
			return '-';
	}
}

export function integrationStatusLabel(status: IntegrationStatus): string {
	switch (status) {
		case 'connected':
			return 'Connected';
		case 'partial':
			return 'Needs attention';
		case 'empty':
			return 'Not configured';
	}
}

function buildMailIntegration(
	account: ProviderAccount,
	providerKind: MailProviderKind,
	calendarAccounts: CalendarAccount[]
): IntegrationViewModel {
	const connectedServices = accountConnectedServices(account);
	const linkedCalendarAccounts = calendarAccounts.filter((calendarAccount) =>
		isLinkedCalendarAccount(account, calendarAccount)
	);
	const calendarRequested = connectedServices.has('calendar');
	const calendarState: IntegrationServiceState = calendarRequested
		? linkedCalendarAccounts.length > 0
			? 'ready'
			: 'unknown'
		: 'not_applicable';
	const peopleState: IntegrationServiceState =
		connectedServices.has('contacts') || connectedServices.has('people') ? 'ready' : 'not_applicable';
	const services = servicesFor({
		mail: {
			state: 'ready',
			description: 'Mail account metadata is available for this provider.'
		},
		calendar: {
			state: calendarState,
			description:
				calendarState === 'ready'
					? 'Calendar account metadata is linked to this provider.'
					: calendarState === 'unknown'
						? 'Calendar access was requested, but no calendar account record is linked.'
						: 'Calendar is not configured for this provider.'
		},
		people: {
			state: peopleState,
			description:
				peopleState === 'ready'
					? 'Contacts capability is available from this provider.'
					: 'Contacts are not configured for this provider.'
		},
		messages: {
			state: 'not_applicable',
			description: 'Messages are not provided by this integration.'
		}
	});

	return {
		integrationId: `${providerKind}:${account.account_id}`,
		providerKind,
		title: account.display_name || accountProviderLabel(account.provider_kind),
		subtitle: account.external_account_id || account.account_id,
		status: services.some((service) => service.state === 'unknown') ? 'partial' : 'connected',
		icon: accountProviderIcon(account.provider_kind),
		updatedAt: latestTimestamp([
			account.updated_at,
			...linkedCalendarAccounts.map((calendarAccount) => calendarAccount.updated_at)
		]),
		updatedLabel: accountUpdatedLabel(account),
		services,
		accounts: [account],
		calendarAccounts: linkedCalendarAccounts,
		metadata: {
			'Provider': accountProviderLabel(account.provider_kind),
			'Account ID': account.account_id,
			'External ID': account.external_account_id || account.account_id
		}
	};
}

function buildMessagingIntegration(
	integrationId: string,
	providerKind: string,
	title: string,
	accounts: ProviderAccount[]
): IntegrationViewModel {
	return {
		integrationId,
		providerKind,
		title,
		subtitle: accounts.map(accountSubtitle).join(', '),
		status: 'connected',
		icon: accountProviderIcon(accounts[0]?.provider_kind ?? providerKind),
		updatedAt: latestTimestamp(accounts.map((account) => account.updated_at)),
		updatedLabel: mostRecentAccountUpdatedLabel(accounts),
		services: servicesFor({
			mail: {
				state: 'not_applicable',
				description: 'Mail is not provided by this integration.'
			},
			calendar: {
				state: 'not_applicable',
				description: 'Calendar is not provided by this integration.'
			},
			people: {
				state: 'not_applicable',
				description: 'Contacts are not provided by this integration.'
			},
			messages: {
				state: 'ready',
				description: 'Messaging account metadata is available.'
			}
		}),
		accounts,
		calendarAccounts: [],
		metadata: {
			'Provider': title,
			'Accounts': String(accounts.length)
		}
	};
}

function buildWhatsappIntegration(accounts: ProviderAccount[]): IntegrationViewModel {
	if (accounts.length === 0) {
		return {
			integrationId: 'whatsapp',
			providerKind: 'whatsapp_web',
			title: 'WhatsApp',
			subtitle: 'No account configured',
			status: 'empty',
			icon: accountProviderIcon('whatsapp_web'),
			updatedAt: null,
			updatedLabel: 'Never',
			services: servicesFor({
				mail: {
					state: 'not_applicable',
					description: 'Mail is not provided by WhatsApp.'
				},
				calendar: {
					state: 'not_applicable',
					description: 'Calendar is not provided by WhatsApp.'
				},
				people: {
					state: 'not_applicable',
					description: 'Contacts are not provided by WhatsApp.'
				},
				messages: {
					state: 'disabled',
					description: 'No WhatsApp account is configured.'
				}
			}),
			accounts: [],
			calendarAccounts: [],
			metadata: {
				'Provider': 'WhatsApp Web',
				'Accounts': '0'
			}
		};
	}

	return buildMessagingIntegration('whatsapp', 'whatsapp_web', 'WhatsApp', accounts);
}

function servicesFor(
	states: Record<IntegrationServiceId, { state: IntegrationServiceState; description: string }>
): IntegrationService[] {
	return SERVICE_IDS.map((id) => ({
		id,
		label: serviceLabel(id),
		state: states[id].state,
		description: states[id].description
	}));
}

function serviceLabel(id: IntegrationServiceId): string {
	switch (id) {
		case 'mail':
			return 'Mail';
		case 'calendar':
			return 'Calendar';
		case 'people':
			return 'People';
		case 'messages':
			return 'Messages';
	}
}

function isTelegramAccount(account: ProviderAccount): boolean {
	return account.provider_kind === 'telegram_user' || account.provider_kind === 'telegram_bot';
}

function accountConnectedServices(account: ProviderAccount): Set<string> {
	const connectedServices = account.config.connected_services;
	if (!Array.isArray(connectedServices)) {
		return new Set();
	}

	return new Set(
		connectedServices.filter((service): service is string => typeof service === 'string').map((service) => service.toLowerCase())
	);
}

function isLinkedCalendarAccount(account: ProviderAccount, calendarAccount: CalendarAccount): boolean {
	const mailAccountId = calendarAccount.capabilities.mail_account_id;
	if (typeof mailAccountId === 'string' && mailAccountId === account.account_id) {
		return true;
	}

	return calendarAccount.account_id.includes(account.account_id) || calendarAccount.email === account.external_account_id;
}

function accountSubtitle(account: ProviderAccount): string {
	return account.display_name || account.external_account_id || account.account_id;
}

function mostRecentAccountUpdatedLabel(accounts: ProviderAccount[]): string {
	const newestAccount = accounts.reduce<ProviderAccount | null>((newest, account) => {
		if (newest === null || account.updated_at > newest.updated_at) {
			return account;
		}
		return newest;
	}, null);

	return newestAccount ? accountUpdatedLabel(newestAccount) : 'Never';
}

function latestTimestamp(values: Array<string | null | undefined>): string | null {
	const timestamps = values.filter((value): value is string => typeof value === 'string' && value.length > 0);
	if (timestamps.length === 0) {
		return null;
	}
	return timestamps.sort().at(-1) ?? null;
}
