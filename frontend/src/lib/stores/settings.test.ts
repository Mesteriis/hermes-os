import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { ApplicationSetting, CalendarAccount, ProviderAccount } from '$lib/api';
import { layoutSettings } from './layoutEditor';
import { sidebarSettings } from './sidebar';
import { themeSettings } from './theme';

const workspaceResult = vi.hoisted(() => {
	const frontendLocaleSetting: ApplicationSetting = {
		setting_key: 'frontend.locale',
		category: 'frontend',
		value_kind: 'string',
		value: 'en',
		label: 'Locale',
		description: 'Frontend locale',
		metadata: {},
		is_editable: true,
		updated_by_actor_id: null,
		created_at: '2026-06-10T00:00:00Z',
		updated_at: '2026-06-10T00:00:00Z'
	};
	const telegramAccount: ProviderAccount = {
		account_id: 'telegram-primary',
		provider_kind: 'telegram_user',
		display_name: 'Primary Telegram',
		external_account_id: '@telegram',
		config: {},
		created_at: '2026-06-10T00:00:00Z',
		updated_at: '2026-06-10T00:00:00Z'
	};
	const gmailAccount: ProviderAccount = {
		account_id: 'gmail-primary',
		provider_kind: 'gmail',
		display_name: 'Google Workspace',
		external_account_id: 'gmail-primary',
		config: {
			connected_services: ['mail', 'calendar', 'contacts']
		},
		created_at: '2026-06-10T00:00:00Z',
		updated_at: '2026-06-10T00:00:00Z'
	};
	const icloudAccount: ProviderAccount = {
		account_id: 'icloud-primary',
		provider_kind: 'icloud',
		display_name: 'Primary iCloud',
		external_account_id: 'user@icloud.com',
		config: {
			connected_services: ['mail', 'calendar', 'contacts']
		},
		created_at: '2026-06-10T00:00:00Z',
		updated_at: '2026-06-10T00:00:00Z'
	};
	const calendarAccount: CalendarAccount = {
		account_id: 'google-calendar:gmail-primary',
		provider: 'google',
		account_name: 'Google Workspace',
		email: 'gmail-primary',
		credentials_reference: 'secret:provider-account:gmail-primary:oauth_token',
		sync_status: 'idle',
		capabilities: {
			mail_account_id: 'gmail-primary',
			connected_services: ['calendar']
		},
		created_at: '2026-06-10T00:00:00Z',
		updated_at: '2026-06-10T00:00:00Z'
	};

	return {
		applicationSettings: [frontendLocaleSetting],
		layoutSettings: {
			schemaVersion: 2,
			views: {
				home: {
					presetId: 'home-default',
					presetVersion: 1,
					hiddenWidgetIds: ['home-priorities'],
					zoneOverrides: {},
					orderOverrides: {},
					gridOverrides: {}
				}
			}
		},
		sidebarSettings: {
			schemaVersion: 3,
			rootItemIds: ['home', 'group:communications', 'timeline'],
			groups: [
				{
					id: 'communications',
					label: 'Communications',
					icon: 'tabler:messages',
					itemIds: ['communications.mail', 'communications.telegram', 'communications.whatsapp'],
					separatorBeforeItemIds: []
				}
			],
			hiddenItemIds: ['tasks' as const]
		},
		themeSettings: {
			schemaVersion: 1,
			shellBackground: 'rune-teal',
			backgroundBrightness: 90,
			accentColor: 'violet',
			panelOpacity: 50,
			panelBlur: 20
		},
		providerAccounts: [gmailAccount, icloudAccount, telegramAccount],
		calendarAccounts: [calendarAccount],
		settingDrafts: { 'frontend.locale': 'en' },
		locale: 'en',
		layoutError: '',
		sidebarError: '',
		themeError: '',
		settingsError: '',
		isLoading: false
	};
});

vi.mock('$lib/services/settings', () => ({
	loadSettingsWorkspace: vi.fn(async () => workspaceResult)
}));

describe('settings store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('loads workspace settings and synchronizes shell stores', async () => {
		const settingsStore = await import('./settings');

		await settingsStore.loadSettingsWorkspace();

		expect(get(settingsStore.applicationSettings).map((setting) => setting.setting_key)).toEqual([
			'frontend.locale'
		]);
		expect(get(settingsStore.telegramProviderAccounts)).toHaveLength(1);
		expect(get(settingsStore.calendarAccounts)).toHaveLength(1);
		expect(get(settingsStore.contactsProviderAccounts)).toEqual([
			expect.objectContaining({ account_id: 'gmail-primary' }),
			expect.objectContaining({ account_id: 'icloud-primary' })
		]);
		expect(get(settingsStore.integrationViewModels).map((integration) => integration.integrationId)).toEqual([
			'gmail:gmail-primary',
			'icloud:icloud-primary',
			'telegram',
			'whatsapp'
		]);
		expect(get(settingsStore.integrationViewModels)[0].services.map((service) => service.state)).toEqual([
			'ready',
			'ready',
			'ready',
			'not_applicable'
		]);
		expect(get(layoutSettings).views.home?.hiddenWidgetIds).toEqual(['home-priorities']);
		expect(get(sidebarSettings).hiddenItemIds).toEqual(['tasks']);
		expect(get(themeSettings).shellBackground).toBe('rune-teal');
		expect(get(settingsStore.settingsError)).toBe('');
	});
});
