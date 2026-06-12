import { derived, get, writable } from 'svelte/store';
import {
	FRONTEND_LOCALE_SETTING_KEY,
	FRONTEND_THEME_SETTING_KEY,
	FRONTEND_UI_STATE_SETTING_KEY,
	saveFrontendLocaleSetting,
	type ApplicationSetting,
	type CalendarAccount,
	type ProviderAccount
} from '$lib/api';
import {
	type SidebarItemId,
	type SidebarNavGroup,
	type SidebarRootItemId
} from '$lib/layout';
import { accountProviderIcon, accountProviderLabel, accountUpdatedLabel } from '$lib/services/accounts';
import { formatDateTime } from '$lib/services/formatting';
import { buildIntegrationViewModels } from '$lib/services/integrations';
import { setLocale, type Locale } from '$lib/i18n';
import * as settingsService from '$lib/services/settings';
import {
	isLayoutSettingsSaving,
	layoutDraft,
	layoutError,
	layoutSettings,
	setLayoutSettings
} from './layoutEditor';
import {
	addSidebarGroup,
	cancelSidebarSettingsEditing,
	effectiveSidebarSettings,
	hasSidebarChanges,
	isSidebarSettingsSaving,
	moveSidebarGroup,
	moveSidebarItem,
	moveSidebarItemToGroup,
	moveSidebarRootItem,
	newSidebarGroupLabel,
	removeSidebarGroup,
	resetSidebarSettingsToDefault,
	setSidebarSettings,
	sidebarConfigItem,
	sidebarDraft,
	sidebarError,
	sidebarGroupHasSeparatorBefore,
	sidebarGroupIdFromLabel,
	sidebarGroupLabel,
	sidebarHiddenNavItems,
	sidebarItemLabel,
	sidebarRootEntries,
	sidebarRootIndexForGroup,
	sidebarSettings,
	toggleSidebarGroupSeparator,
	toggleSidebarItemHidden,
	updateSidebarGroupLabel
} from './sidebar';
import { setThemeSettings, themeDraft, themeError, themeSettings } from './theme';

export type SettingsSection = 'appearance' | 'application' | 'sidebar' | 'integrations' | 'language' | 'ai';

export const applicationSettings = writable<ApplicationSetting[]>([]);
export const providerAccounts = writable<ProviderAccount[]>([]);
export const calendarAccounts = writable<CalendarAccount[]>([]);
export const settingDrafts = writable<Record<string, string>>({});
export const settingsError = writable('');
export const settingsActionMessage = writable('');
export const isSettingsLoading = writable(false);
export const savingSettingKey = writable<string | null>(null);
export const selectedSettingsSection = writable<SettingsSection>('appearance');

export const settingsByCategory = derived(applicationSettings, ($applicationSettings) =>
	groupSettingsByCategory(
		$applicationSettings.filter(
			(setting) =>
				setting.setting_key !== 'frontend.sidebar' &&
				setting.setting_key !== FRONTEND_THEME_SETTING_KEY &&
				setting.setting_key !== FRONTEND_UI_STATE_SETTING_KEY &&
				!setting.setting_key.startsWith('ai.')
		)
	)
);

export const emailProviderAccounts = derived(providerAccounts, ($providerAccounts) =>
	$providerAccounts.filter((account) => ['gmail', 'icloud', 'imap'].includes(account.provider_kind))
);

export const telegramProviderAccounts = derived(providerAccounts, ($providerAccounts) =>
	$providerAccounts.filter((account) =>
		['telegram_user', 'telegram_bot'].includes(account.provider_kind)
	)
);

export const whatsappProviderAccounts = derived(providerAccounts, ($providerAccounts) =>
	$providerAccounts.filter((account) => account.provider_kind === 'whatsapp_web')
);

export const contactsProviderAccounts = derived(providerAccounts, ($providerAccounts) =>
	$providerAccounts.filter(
		(account) =>
			Array.isArray(account.config.connected_services) &&
			account.config.connected_services.includes('contacts')
	)
);

export const integrationViewModels = derived(
	[providerAccounts, calendarAccounts],
	([$providerAccounts, $calendarAccounts]) =>
		buildIntegrationViewModels($providerAccounts, $calendarAccounts)
);

export {
	accountProviderIcon,
	accountProviderLabel,
	accountUpdatedLabel,
	addSidebarGroup,
	cancelSidebarSettingsEditing,
	effectiveSidebarSettings,
	formatDateTime,
	hasSidebarChanges,
	isLayoutSettingsSaving,
	isSidebarSettingsSaving,
	layoutError,
	moveSidebarGroup,
	moveSidebarItem,
	moveSidebarItemToGroup,
	moveSidebarRootItem,
	newSidebarGroupLabel,
	removeSidebarGroup,
	resetSidebarSettingsToDefault,
	sidebarConfigItem,
	sidebarDraft,
	sidebarError,
	sidebarGroupHasSeparatorBefore,
	sidebarGroupIdFromLabel,
	sidebarGroupLabel,
	sidebarHiddenNavItems,
	sidebarItemLabel,
	sidebarRootEntries,
	sidebarRootIndexForGroup,
	sidebarSettings,
	toggleSidebarGroupSeparator,
	toggleSidebarItemHidden,
	updateSidebarGroupLabel
};

export async function loadSettingsWorkspace(): Promise<void> {
	isSettingsLoading.set(true);
	const result = await settingsService.loadSettingsWorkspace();
	applicationSettings.set(result.applicationSettings);
	providerAccounts.set(result.providerAccounts);
	calendarAccounts.set(result.calendarAccounts);
	settingDrafts.set(result.settingDrafts);
	setLayoutSettings(result.layoutSettings);
	setSidebarSettings(result.sidebarSettings);
	setThemeSettings(result.themeSettings);
	layoutError.set(result.layoutError);
	sidebarError.set(result.sidebarError);
	themeError.set(result.themeError);
	settingsError.set(result.settingsError);
	isSettingsLoading.set(result.isLoading);
}

export async function saveLocaleSetting(locale: Locale): Promise<void> {
	setLocale(locale);
	savingSettingKey.set(FRONTEND_LOCALE_SETTING_KEY);
	settingsError.set('');
	settingsActionMessage.set('');
	try {
		const updated = await saveFrontendLocaleSetting(locale);
		applicationSettings.update((settings) =>
			settings.some((setting) => setting.setting_key === updated.setting_key)
				? settings.map((setting) =>
						setting.setting_key === updated.setting_key ? updated : setting
					)
				: [...settings, updated]
		);
		settingDrafts.update((drafts) => ({
			...drafts,
			[updated.setting_key]: settingDraftValue(updated)
		}));
		settingsActionMessage.set('Language saved');
	} catch (error) {
		settingsError.set(error instanceof Error ? error.message : 'Unknown locale update error');
	} finally {
		savingSettingKey.set(null);
	}
}

export async function saveSetting(setting: ApplicationSetting): Promise<void> {
	const draft = get(settingDrafts)[setting.setting_key] ?? '';
	savingSettingKey.set(setting.setting_key);
	const result = await settingsService.saveSetting(
		setting,
		draft,
		get(applicationSettings),
		get(layoutSettings),
		get(sidebarSettings),
		get(themeSettings)
	);
	applicationSettings.set(result.applicationSettings);
	setLayoutSettings(result.layoutSettings);
	setSidebarSettings(result.sidebarSettings);
	setThemeSettings(result.themeSettings);
	settingDrafts.update((drafts) => ({ ...drafts, ...result.settingDrafts }));
	settingsActionMessage.set(result.actionMessage);
	settingsError.set(result.error);
	if (!result.error) {
		layoutDraft.set(null);
		sidebarDraft.set(null);
		themeDraft.set(null);
		savingSettingKey.set(null);
	}
}

export async function saveSidebarSettings(): Promise<void> {
	isSidebarSettingsSaving.set(true);
	savingSettingKey.set('frontend.sidebar');
	const result = await settingsService.saveSidebarSettings(
		get(sidebarDraft),
		get(sidebarSettings),
		get(applicationSettings),
		get(settingDrafts)
	);
	applicationSettings.set(result.applicationSettings);
	settingDrafts.set(result.settingDrafts);
	if (result.error) {
		sidebarError.set(result.error);
		settingsError.set(result.error);
	} else {
		setSidebarSettings(result.sidebarSettings);
		settingsError.set('');
		settingsActionMessage.set('Sidebar navigation saved');
	}
	isSidebarSettingsSaving.set(false);
	savingSettingKey.set(null);
}

export function updateSettingDraft(settingKey: string, value: string): void {
	settingDrafts.update((drafts) => ({
		...drafts,
		[settingKey]: value
	}));
	settingsActionMessage.set('');
}

export function updateNewSidebarGroupLabel(label: string): void {
	newSidebarGroupLabel.set(label);
}

export function settingDraftValue(setting: ApplicationSetting): string {
	if (setting.value_kind === 'json') {
		return JSON.stringify(setting.value, null, 2);
	}
	return String(setting.value);
}

export function settingHasChanged(setting: ApplicationSetting): boolean {
	return (get(settingDrafts)[setting.setting_key] ?? settingDraftValue(setting)) !== settingDraftValue(setting);
}

export function settingAllowedValues(setting: ApplicationSetting): string[] {
	const values = setting.metadata.allowed_values;
	if (!Array.isArray(values)) {
		return [];
	}
	return values.filter((value): value is string => typeof value === 'string');
}

export function settingControl(setting: ApplicationSetting): string {
	const control = setting.metadata.ui_control;
	return typeof control === 'string' ? control : '';
}

export function settingValueText(settingKey: string): string {
	const setting = get(applicationSettings).find((item) => item.setting_key === settingKey);
	if (!setting || setting.value === null || setting.value === undefined) {
		return 'not set';
	}
	if (typeof setting.value === 'object') {
		return JSON.stringify(setting.value);
	}
	return String(setting.value);
}

export function settingMetadataFlag(setting: ApplicationSetting, key: string): boolean {
	return setting.metadata[key] === true;
}

export function settingMetadataText(setting: ApplicationSetting, key: string): string {
	const value = setting.metadata[key];
	return typeof value === 'string' && value.trim() ? value.trim() : '';
}

export function settingsCategoryLabel(category: string): string {
	return category
		.split('_')
		.flatMap((part) => part.split('-'))
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

export function inputEventValue(event: Event): string {
	return (event.currentTarget as HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement).value;
}

export function checkboxEventValue(event: Event): string {
	return (event.currentTarget as HTMLInputElement).checked ? 'true' : 'false';
}

function groupSettingsByCategory(settings: ApplicationSetting[]): Record<string, ApplicationSetting[]> {
	return settings.reduce<Record<string, ApplicationSetting[]>>((groups, setting) => {
		groups[setting.category] = [...(groups[setting.category] ?? []), setting];
		return groups;
	}, {});
}

export type {
	SidebarItemId,
	SidebarNavGroup,
	SidebarRootItemId
};
