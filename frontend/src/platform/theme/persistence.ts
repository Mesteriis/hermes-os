import {
	FRONTEND_THEME_SETTING_KEY,
	fetchApplicationSettings,
	saveApplicationSetting
} from '../settings/applicationSettingsClient'
import { defaultThemeSettings, parseThemeSettings, type ThemeSettings } from './settings'

const LOCAL_STORAGE_KEY = 'hermes-theme-settings'

export type ThemePersistenceSource = 'application_settings' | 'local_storage'

export interface PersistedThemeSettings {
	settings: ThemeSettings
	source: ThemePersistenceSource
	errorMessage: string
}

const LOAD_FALLBACK_MESSAGE = 'Theme settings backend unavailable; using local browser settings.'
const SAVE_FALLBACK_MESSAGE = 'Theme saved locally only. Application settings backend is unavailable.'

export async function loadPersistedThemeSettings(): Promise<PersistedThemeSettings> {
	try {
		const response = await fetchApplicationSettings()
		const setting = response.items.find((item) => item.setting_key === FRONTEND_THEME_SETTING_KEY)
		if (setting) {
			const parsed = parseThemeSettings(setting.value)
			saveLocalThemeSettings(parsed)
			return {
				settings: parsed,
				source: 'application_settings',
				errorMessage: ''
			}
		}
	} catch {
		return {
			settings: loadLocalThemeSettings(),
			source: 'local_storage',
			errorMessage: LOAD_FALLBACK_MESSAGE
		}
	}

	return {
		settings: loadLocalThemeSettings(),
		source: 'local_storage',
		errorMessage: ''
	}
}

export async function savePersistedThemeSettings(settings: ThemeSettings): Promise<PersistedThemeSettings> {
	try {
		const saved = await saveApplicationSetting(FRONTEND_THEME_SETTING_KEY, settings)
		const parsed = parseThemeSettings(saved.value)
		saveLocalThemeSettings(parsed)
		return {
			settings: parsed,
			source: 'application_settings',
			errorMessage: ''
		}
	} catch {
		saveLocalThemeSettings(settings)
		return {
			settings,
			source: 'local_storage',
			errorMessage: SAVE_FALLBACK_MESSAGE
		}
	}
}

export function loadLocalThemeSettings(): ThemeSettings {
	try {
		const raw = localStorage.getItem(LOCAL_STORAGE_KEY)
		return raw ? parseThemeSettings(JSON.parse(raw)) : defaultThemeSettings()
	} catch {
		return defaultThemeSettings()
	}
}

function saveLocalThemeSettings(settings: ThemeSettings): void {
	try {
		localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(settings))
	} catch {
		// localStorage may be unavailable; runtime theme still applies in memory.
	}
}
