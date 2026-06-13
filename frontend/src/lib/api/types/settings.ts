export type SettingValueKind = 'boolean' | 'integer' | 'string' | 'json';

export type ApplicationSetting = {
	setting_key: string;
	category: string;
	value_kind: SettingValueKind;
	value: boolean | number | string | Record<string, unknown> | unknown[];
	label: string;
	description: string;
	metadata: Record<string, unknown>;
	is_editable: boolean;
	updated_by_actor_id: string | null;
	created_at: string;
	updated_at: string;
};

export type ApplicationSettingsResponse = {
	items: ApplicationSetting[];
};

export const FRONTEND_LAYOUT_SETTING_KEY = 'frontend.layout';
export const FRONTEND_SIDEBAR_SETTING_KEY = 'frontend.sidebar';
export const FRONTEND_LOCALE_SETTING_KEY = 'frontend.locale';
export const FRONTEND_THEME_SETTING_KEY = 'frontend.theme';
export const FRONTEND_UI_STATE_SETTING_KEY = 'frontend.ui_state';
