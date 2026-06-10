import { describe, expect, it } from 'vitest';
import type { CalendarAccount, ProviderAccount } from '$lib/api';
import { buildIntegrationViewModels, serviceStateLabel } from './integrations';

function providerAccount(overrides: Partial<ProviderAccount>): ProviderAccount {
	return {
		account_id: 'account-primary',
		provider_kind: 'gmail',
		display_name: 'Provider Account',
		external_account_id: 'provider@example.com',
		config: {},
		created_at: '2026-06-10T00:00:00Z',
		updated_at: '2026-06-10T10:00:00Z',
		...overrides
	};
}

function calendarAccount(overrides: Partial<CalendarAccount>): CalendarAccount {
	return {
		account_id: 'google-calendar:gmail-primary',
		provider: 'google',
		account_name: 'Google Workspace',
		email: 'gmail-primary',
		credentials_reference: 'secret:provider-account:gmail-primary:oauth_token',
		sync_status: 'idle',
		capabilities: { mail_account_id: 'gmail-primary', connected_services: ['calendar'] },
		created_at: '2026-06-10T00:00:00Z',
		updated_at: '2026-06-10T10:30:00Z',
		...overrides
	};
}

describe('integration view models', () => {
	it('derives Google mail, calendar and people service states from existing metadata', () => {
		const linkedCalendarAccount = calendarAccount({ account_id: 'google-calendar:gmail-primary' });
		const integrations = buildIntegrationViewModels(
			[
				providerAccount({
					account_id: 'gmail-primary',
					provider_kind: 'gmail',
					display_name: 'Google Workspace',
					external_account_id: 'gmail-primary',
					config: { connected_services: ['mail', 'calendar', 'contacts'] }
				})
			],
			[linkedCalendarAccount]
		);

		expect(integrations).toHaveLength(2);
		expect(integrations[0]).toMatchObject({
			integrationId: 'gmail:gmail-primary',
			providerKind: 'gmail',
			title: 'Google Workspace',
			subtitle: 'gmail-primary',
			status: 'connected'
		});
		expect(integrations[0].services.map((service) => [service.id, service.state])).toEqual([
			['mail', 'ready'],
			['calendar', 'ready'],
			['people', 'ready'],
			['messages', 'not_applicable']
		]);
		expect(integrations[0].calendarAccounts).toEqual([linkedCalendarAccount]);
	});

	it('retains linked iCloud calendar metadata and people service state', () => {
		const linkedCalendarAccount = calendarAccount({
			account_id: 'icloud-calendar:icloud-primary',
			provider: 'apple',
			account_name: 'iCloud Calendar',
			email: 'user@icloud.com',
			credentials_reference: 'secret:provider-account:icloud-primary:app_password',
			capabilities: { mail_account_id: 'icloud-primary', connected_services: ['calendar'] }
		});

		const [integration] = buildIntegrationViewModels(
			[
				providerAccount({
					account_id: 'icloud-primary',
					provider_kind: 'icloud',
					display_name: 'Primary iCloud',
					external_account_id: 'user@icloud.com',
					config: { connected_services: ['mail', 'calendar', 'people'] }
				})
			],
			[linkedCalendarAccount]
		);

		expect(integration).toMatchObject({
			integrationId: 'icloud:icloud-primary',
			providerKind: 'icloud',
			title: 'Primary iCloud',
			subtitle: 'user@icloud.com',
			status: 'connected'
		});
		expect(integration.services.map((service) => [service.id, service.state])).toEqual([
			['mail', 'ready'],
			['calendar', 'ready'],
			['people', 'ready'],
			['messages', 'not_applicable']
		]);
		expect(integration.calendarAccounts).toEqual([linkedCalendarAccount]);
	});

	it('marks requested calendar service as unknown when provider metadata exists but calendar row is missing', () => {
		const [integration] = buildIntegrationViewModels(
			[
				providerAccount({
					account_id: 'icloud-primary',
					provider_kind: 'icloud',
					display_name: 'Primary iCloud',
					external_account_id: 'user@icloud.com',
					config: { connected_services: ['mail', 'calendar', 'contacts'] }
				})
			],
			[]
		);

		expect(integration.status).toBe('partial');
		expect(integration.services.map((service) => [service.id, service.state])).toEqual([
			['mail', 'ready'],
			['calendar', 'unknown'],
			['people', 'ready'],
			['messages', 'not_applicable']
		]);
		expect(integration.calendarAccounts).toEqual([]);
	});

	it('groups Telegram accounts into one messaging integration row', () => {
		const integrations = buildIntegrationViewModels(
			[
				providerAccount({
					account_id: 'telegram-fixture-one',
					provider_kind: 'telegram_user',
					display_name: '@telegram_fixture_one',
					external_account_id: 'telegram:100000001'
				}),
				providerAccount({
					account_id: 'telegram-fixture-two',
					provider_kind: 'telegram_user',
					display_name: '@telegram_fixture_two',
					external_account_id: 'telegram:100000002'
				})
			],
			[]
		);

		const telegram = integrations.find((integration) => integration.integrationId === 'telegram');
		expect(telegram).toMatchObject({
			providerKind: 'telegram',
			title: 'Telegram',
			subtitle: '@telegram_fixture_one, @telegram_fixture_two',
			status: 'connected'
		});
		expect(telegram?.services.map((service) => [service.id, service.state])).toEqual([
			['mail', 'not_applicable'],
			['calendar', 'not_applicable'],
			['people', 'not_applicable'],
			['messages', 'ready']
		]);
		expect(telegram?.accounts).toHaveLength(2);
		expect(telegram?.calendarAccounts).toEqual([]);
	});

	it('groups configured WhatsApp accounts into one messaging integration row', () => {
		const whatsappAccount = providerAccount({
			account_id: 'whatsapp-fixture-primary',
			provider_kind: 'whatsapp_web',
			display_name: 'WhatsApp Fixture',
			external_account_id: 'whatsapp:100000001'
		});

		const integrations = buildIntegrationViewModels([whatsappAccount], []);

		expect(integrations).toEqual([
			expect.objectContaining({
				integrationId: 'whatsapp',
				providerKind: 'whatsapp_web',
				title: 'WhatsApp',
				subtitle: 'WhatsApp Fixture',
				status: 'connected',
				accounts: [whatsappAccount],
				calendarAccounts: []
			})
		]);
		expect(integrations[0].services.map((service) => [service.id, service.state])).toEqual([
			['mail', 'not_applicable'],
			['calendar', 'not_applicable'],
			['people', 'not_applicable'],
			['messages', 'ready']
		]);
	});

	it('adds an empty WhatsApp row when no WhatsApp account exists', () => {
		const integrations = buildIntegrationViewModels([], []);

		expect(integrations).toEqual([
			expect.objectContaining({
				integrationId: 'whatsapp',
				providerKind: 'whatsapp_web',
				title: 'WhatsApp',
				subtitle: 'No account configured',
				status: 'empty',
				calendarAccounts: []
			})
		]);
	});

	it('maps service state labels for table cells', () => {
		expect(serviceStateLabel('ready')).toBe('Ready');
		expect(serviceStateLabel('unknown')).toBe('Auth');
		expect(serviceStateLabel('disabled')).toBe('Disabled');
		expect(serviceStateLabel('not_applicable')).toBe('-');
	});
});
