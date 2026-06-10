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
			[calendarAccount({ account_id: 'google-calendar:gmail-primary' })]
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
	});

	it('groups Telegram accounts into one messaging integration row', () => {
		const integrations = buildIntegrationViewModels(
			[
				providerAccount({
					account_id: '682703602_account_alexm36',
					provider_kind: 'telegram_user',
					display_name: '@AlexM36',
					external_account_id: 'telegram:682703602'
				}),
				providerAccount({
					account_id: '5499503231_account_viki_avm',
					provider_kind: 'telegram_user',
					display_name: '@viki_avm',
					external_account_id: 'telegram:5499503231'
				})
			],
			[]
		);

		const telegram = integrations.find((integration) => integration.integrationId === 'telegram');
		expect(telegram).toMatchObject({
			providerKind: 'telegram',
			title: 'Telegram',
			subtitle: '@AlexM36, @viki_avm',
			status: 'connected'
		});
		expect(telegram?.services.map((service) => [service.id, service.state])).toEqual([
			['mail', 'not_applicable'],
			['calendar', 'not_applicable'],
			['people', 'not_applicable'],
			['messages', 'ready']
		]);
		expect(telegram?.accounts).toHaveLength(2);
	});

	it('adds an empty WhatsApp row when no WhatsApp account exists', () => {
		const integrations = buildIntegrationViewModels([], []);

		expect(integrations).toEqual([
			expect.objectContaining({
				integrationId: 'whatsapp',
				providerKind: 'whatsapp_web',
				title: 'WhatsApp',
				subtitle: 'No account configured',
				status: 'empty'
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
