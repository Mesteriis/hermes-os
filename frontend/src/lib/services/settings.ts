import {
	fetchApplicationSettings,
	fetchCalendarAccounts,
	fetchProviderAccounts,
	saveApplicationSetting,
	saveFrontendLayoutSetting,
	saveFrontendSidebarSetting,
	findFrontendLayoutSetting,
	findFrontendSidebarSetting,
	findFrontendLocaleSetting,
	findFrontendThemeSetting,
	FRONTEND_THEME_SETTING_KEY,
	type ApplicationSetting,
	type CalendarAccount,
	type ProviderAccount
} from '$lib/api';
import {
	defaultLayoutSettings,
	defaultSidebarSettings,
	defaultFrontendThemeSettings,
	parseLayoutSettings,
	parseSidebarSettings,
	parseFrontendThemeSettings,
	type LayoutSettings,
	type SidebarSettings,
	type FrontendThemeSettings
} from '$lib/layout';
import { setLocale, type Locale } from '$lib/i18n';

export type SettingsWorkspaceResult = {
	applicationSettings: ApplicationSetting[];
	layoutSettings: LayoutSettings;
	sidebarSettings: SidebarSettings;
	themeSettings: FrontendThemeSettings;
	providerAccounts: ProviderAccount[];
	calendarAccounts: CalendarAccount[];
	settingDrafts: Record<string, string>;
	locale: Locale | null;
	layoutError: string;
	sidebarError: string;
	themeError: string;
	settingsError: string;
	isLoading: boolean;
};

export async function loadSettingsWorkspace(): Promise<SettingsWorkspaceResult> {
	try {
		const [settingsResponse, accountsResponse, calendarAccountsResponse] = await Promise.all([
			fetchApplicationSettings(),
			fetchProviderAccounts(),
			fetchCalendarAccounts()
		]);
		const frontendLayoutSetting = findFrontendLayoutSetting(settingsResponse.items);
		const frontendSidebarSetting = findFrontendSidebarSetting(settingsResponse.items);
		const frontendThemeSetting = findFrontendThemeSetting(settingsResponse.items);
		const layoutSettings = parseLayoutSettings(frontendLayoutSetting?.value ?? null);
		const sidebarSettings = parseSidebarSettings(frontendSidebarSetting?.value ?? null);
		const themeSettings = parseFrontendThemeSettings(frontendThemeSetting?.value ?? null);
		const frontendLocaleSetting = findFrontendLocaleSetting(settingsResponse.items);
		const locale: Locale | null =
			frontendLocaleSetting?.value === 'ru' || frontendLocaleSetting?.value === 'en'
				? frontendLocaleSetting.value
				: null;
		if (locale) {
			setLocale(locale);
		}
		const settingDrafts = Object.fromEntries(
			settingsResponse.items.map((setting) => [setting.setting_key, settingDraftValue(setting)])
		);
		return {
			applicationSettings: settingsResponse.items,
			layoutSettings,
			sidebarSettings,
			themeSettings,
			providerAccounts: accountsResponse.items,
			calendarAccounts: calendarAccountsResponse.items,
			settingDrafts,
			locale,
			layoutError: '',
			sidebarError: '',
			themeError: '',
			settingsError: '',
			isLoading: false
		};
	} catch (error) {
		return {
			applicationSettings: [],
			layoutSettings: defaultLayoutSettings(),
			sidebarSettings: defaultSidebarSettings(),
			themeSettings: defaultFrontendThemeSettings(),
			providerAccounts: [],
			calendarAccounts: [],
			settingDrafts: {},
			locale: null,
			layoutError: error instanceof Error ? error.message : 'Unknown layout settings error',
			sidebarError: error instanceof Error ? error.message : 'Unknown sidebar settings error',
			themeError: error instanceof Error ? error.message : 'Unknown appearance settings error',
			settingsError: error instanceof Error ? error.message : 'Unknown settings error',
			isLoading: false
		};
	}
}

function settingDraftValue(setting: ApplicationSetting) {
	if (setting.value_kind === 'json') {
		return JSON.stringify(setting.value, null, 2);
	}
	return String(setting.value);
}

function settingDraftToValue(setting: ApplicationSetting, draft: string): ApplicationSetting['value'] {
	const value = draft.trim();
	if (setting.value_kind === 'integer') {
		const numberValue = Number(value);
		if (!Number.isInteger(numberValue)) {
			throw new Error(`${setting.label} must be an integer`);
		}
		return numberValue;
	}
	if (setting.value_kind === 'boolean') {
		return value === 'true';
	}
	if (setting.value_kind === 'json') {
		return JSON.parse(value);
	}
	return value;
}

export async function saveSetting(
	setting: ApplicationSetting,
	draft: string,
	applicationSettings: ApplicationSetting[],
	layoutSettings: LayoutSettings,
	sidebarSettings: SidebarSettings,
	themeSettings: FrontendThemeSettings
): Promise<{
	applicationSettings: ApplicationSetting[];
	layoutSettings: LayoutSettings;
	sidebarSettings: SidebarSettings;
	themeSettings: FrontendThemeSettings;
	settingDrafts: Record<string, string>;
	actionMessage: string;
	error: string;
}> {
	let nextValue: ApplicationSetting['value'];
	try {
		nextValue = settingDraftToValue(setting, draft);
	} catch (error) {
		return {
			applicationSettings,
			layoutSettings,
			sidebarSettings,
			themeSettings,
			settingDrafts: {},
			actionMessage: '',
			error: error instanceof Error ? error.message : 'Invalid setting value'
		};
	}

	try {
		const updated = await saveApplicationSetting(
			setting.setting_key,
			nextValue
		);
		const nextApplicationSettings = applicationSettings.map((item) =>
			item.setting_key === updated.setting_key ? updated : item
		);
		const settingDrafts: Record<string, string> = {
			[updated.setting_key]: settingDraftValue(updated)
		};
		let nextLayoutSettings = layoutSettings;
		let nextSidebarSettings = sidebarSettings;
		let nextThemeSettings = themeSettings;
		if (updated.setting_key === 'frontend.layout') {
			nextLayoutSettings = parseLayoutSettings(updated.value);
		}
		if (updated.setting_key === 'frontend.sidebar') {
			nextSidebarSettings = parseSidebarSettings(updated.value);
		}
		if (updated.setting_key === FRONTEND_THEME_SETTING_KEY) {
			nextThemeSettings = parseFrontendThemeSettings(updated.value);
		}
		return {
			applicationSettings: nextApplicationSettings,
			layoutSettings: nextLayoutSettings,
			sidebarSettings: nextSidebarSettings,
			themeSettings: nextThemeSettings,
			settingDrafts,
			actionMessage: `${updated.label} saved`,
			error: ''
		};
	} catch (error) {
		return {
			applicationSettings,
			layoutSettings,
			sidebarSettings,
			themeSettings,
			settingDrafts: {},
			actionMessage: '',
			error: error instanceof Error ? error.message : 'Unknown setting update error'
		};
	}
}

export async function saveLayoutSettings(
	layoutDraft: LayoutSettings | null,
	layoutSettings: LayoutSettings,
	applicationSettings: ApplicationSetting[],
	settingDrafts: Record<string, string>
): Promise<{
	layoutSettings: LayoutSettings;
	applicationSettings: ApplicationSetting[];
	settingDrafts: Record<string, string>;
	error: string;
	actionMessage?: string;
}> {
	const nextSettings = parseLayoutSettings(layoutDraft ?? layoutSettings);
	const savingSettingKey = 'frontend.layout';
	try {
		const updated = await saveFrontendLayoutSetting(nextSettings);
		const nextApplicationSettings = applicationSettings.some((item) => item.setting_key === updated.setting_key)
			? applicationSettings.map((item) =>
					item.setting_key === updated.setting_key ? updated : item
				)
			: [...applicationSettings, updated];
		const nextLayoutSettings = parseLayoutSettings(updated.value);
		const nextSettingDrafts = {
			...settingDrafts,
			[updated.setting_key]: settingDraftValue(updated)
		};
		return {
			layoutSettings: nextLayoutSettings,
			applicationSettings: nextApplicationSettings,
			settingDrafts: nextSettingDrafts,
			error: ''
		};
	} catch (error) {
		return {
			layoutSettings,
			applicationSettings,
			settingDrafts,
			error: error instanceof Error ? error.message : 'Unknown layout settings update error'
		};
	}
}

export async function saveSidebarSettings(
	sidebarDraft: SidebarSettings | null,
	sidebarSettings: SidebarSettings,
	applicationSettings: ApplicationSetting[],
	settingDrafts: Record<string, string>
): Promise<{
	sidebarSettings: SidebarSettings;
	applicationSettings: ApplicationSetting[];
	settingDrafts: Record<string, string>;
	error: string;
	actionMessage?: string;
}> {
	const nextSettings = parseSidebarSettings(sidebarDraft ?? sidebarSettings);
	const savingSettingKey = 'frontend.sidebar';
	try {
		const updated = await saveFrontendSidebarSetting(nextSettings);
		const nextApplicationSettings = applicationSettings.some((item) => item.setting_key === updated.setting_key)
			? applicationSettings.map((item) =>
					item.setting_key === updated.setting_key ? updated : item
				)
			: [...applicationSettings, updated];
		const nextSidebarSettings = parseSidebarSettings(updated.value);
		const nextSettingDrafts = {
			...settingDrafts,
			[updated.setting_key]: settingDraftValue(updated)
		};
		return {
			sidebarSettings: nextSidebarSettings,
			applicationSettings: nextApplicationSettings,
			settingDrafts: nextSettingDrafts,
			error: ''
		};
	} catch (error) {
		return {
			sidebarSettings,
			applicationSettings,
			settingDrafts,
			error: error instanceof Error ? error.message : 'Unknown sidebar settings update error'
		};
	}
}
