import type { FrontendThemeSettings, LayoutSettings, SidebarSettings } from '$lib/layout';
import { ApiClient } from '../client';
import {
	FRONTEND_LAYOUT_SETTING_KEY,
	FRONTEND_SIDEBAR_SETTING_KEY,
	FRONTEND_LOCALE_SETTING_KEY,
	FRONTEND_THEME_SETTING_KEY,
	FRONTEND_UI_STATE_SETTING_KEY,
	type ApplicationSetting,
	type ApplicationSettingsResponse,
	type ProviderAccountListResponse
} from '../types';

export {
	FRONTEND_LAYOUT_SETTING_KEY,
	FRONTEND_SIDEBAR_SETTING_KEY,
	FRONTEND_LOCALE_SETTING_KEY,
	FRONTEND_THEME_SETTING_KEY,
	FRONTEND_UI_STATE_SETTING_KEY
};

export async function fetchApplicationSettings(): Promise<ApplicationSettingsResponse> {
	return ApiClient.instance.get<ApplicationSettingsResponse>('/api/v1/settings', 'Settings request failed');
}

export async function saveApplicationSetting(
	settingKey: string,
	value: ApplicationSetting['value']
): Promise<ApplicationSetting> {
	return ApiClient.instance.put<ApplicationSetting>(
		`/api/v1/settings/${encodeURIComponent(settingKey)}`,
		{ value },
		'Setting update failed'
	);
}

export function findFrontendLayoutSetting(settings: ApplicationSetting[]): ApplicationSetting | null {
	return settings.find((setting) => setting.setting_key === FRONTEND_LAYOUT_SETTING_KEY) ?? null;
}

export function findFrontendSidebarSetting(settings: ApplicationSetting[]): ApplicationSetting | null {
	return settings.find((setting) => setting.setting_key === FRONTEND_SIDEBAR_SETTING_KEY) ?? null;
}

export function findFrontendLocaleSetting(settings: ApplicationSetting[]): ApplicationSetting | null {
	return settings.find((setting) => setting.setting_key === FRONTEND_LOCALE_SETTING_KEY) ?? null;
}

export function findFrontendThemeSetting(settings: ApplicationSetting[]): ApplicationSetting | null {
	return settings.find((setting) => setting.setting_key === FRONTEND_THEME_SETTING_KEY) ?? null;
}

export function findFrontendUiStateSetting(settings: ApplicationSetting[]): ApplicationSetting | null {
	return settings.find((setting) => setting.setting_key === FRONTEND_UI_STATE_SETTING_KEY) ?? null;
}

export async function saveFrontendLayoutSetting(value: LayoutSettings): Promise<ApplicationSetting> {
	return saveApplicationSetting(FRONTEND_LAYOUT_SETTING_KEY, value);
}

export async function saveFrontendSidebarSetting(value: SidebarSettings): Promise<ApplicationSetting> {
	return saveApplicationSetting(FRONTEND_SIDEBAR_SETTING_KEY, value);
}

export async function saveFrontendLocaleSetting(value: string): Promise<ApplicationSetting> {
	return saveApplicationSetting(FRONTEND_LOCALE_SETTING_KEY, value);
}

export async function saveFrontendThemeSetting(value: FrontendThemeSettings): Promise<ApplicationSetting> {
	return saveApplicationSetting(FRONTEND_THEME_SETTING_KEY, value);
}

export async function saveFrontendUiStateSetting(value: Record<string, unknown>): Promise<ApplicationSetting> {
	return saveApplicationSetting(FRONTEND_UI_STATE_SETTING_KEY, value);
}

export async function fetchProviderAccounts(): Promise<ProviderAccountListResponse> {
	return ApiClient.instance.get<ProviderAccountListResponse>(
		'/api/v1/settings/accounts',
		'Provider accounts request failed'
	);
}
