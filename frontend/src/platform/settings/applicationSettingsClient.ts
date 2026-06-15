import { ApiClient } from '../api/ApiClient'

export type SettingValueKind = 'boolean' | 'integer' | 'string' | 'json'

export type ApplicationSettingValue = boolean | number | string | Record<string, unknown> | unknown[]

export interface ApplicationSetting {
	setting_key: string
	category: string
	value_kind: SettingValueKind
	value: ApplicationSettingValue
	label: string
	description: string
	metadata: Record<string, unknown>
	is_editable: boolean
	updated_by_actor_id: string | null
	created_at: string
	updated_at: string
}

export interface ApplicationSettingsResponse {
	items: ApplicationSetting[]
}

export const FRONTEND_LAYOUT_SETTING_KEY = 'frontend.layout'
export const FRONTEND_SIDEBAR_SETTING_KEY = 'frontend.sidebar'
export const FRONTEND_LOCALE_SETTING_KEY = 'frontend.locale'
export const FRONTEND_THEME_SETTING_KEY = 'frontend.theme'
export const FRONTEND_UI_STATE_SETTING_KEY = 'frontend.ui_state'

export async function fetchApplicationSettings(): Promise<ApplicationSettingsResponse> {
	return ApiClient.instance.get<ApplicationSettingsResponse>(
		'/api/v1/settings',
		'Settings request failed'
	)
}

export async function saveApplicationSetting(
	settingKey: string,
	value: ApplicationSetting['value']
): Promise<ApplicationSetting> {
	return ApiClient.instance.put<ApplicationSetting>(
		`/api/v1/settings/${encodeURIComponent(settingKey)}`,
		{ value },
		'Setting update failed'
	)
}
