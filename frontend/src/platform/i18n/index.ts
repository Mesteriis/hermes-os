import { ref } from 'vue'
import type { Locale, TranslationFunction, Dictionary } from './types'
import ru from './ru.json'
import en from './en.json'

/** Reactive current locale (persisted to localStorage). */
const currentLocale = ref<Locale>(loadLocale())
const dictionaries: Record<Locale, Dictionary> = { ru, en }

function loadLocale(): Locale {
	try {
		const stored = localStorage.getItem('hh-locale')
		if (stored === 'ru' || stored === 'en') return stored
	} catch {
		// localStorage unavailable
	}
	// Default to Russian
	return 'ru'
}

function persistLocale(locale: Locale) {
	try {
		localStorage.setItem('hh-locale', locale)
	} catch {
		// ignore
	}
}

/**
 * Set the active locale and persist to localStorage.
 */
export function setLocale(locale: Locale) {
	currentLocale.value = locale
	persistLocale(locale)
}

/**
 * Vue composable: returns `t(key)` function that looks up the key
 * in the current locale dictionary.
 *
 * - If the key exists in the active dictionary, returns the translated value.
 * - If the key does not exist, returns the key itself as fallback (identity).
 * - Supports `{param}` interpolation: `t("Hello {name}", { name: "Alex" })`.
 */
export function useI18n(): {
	t: TranslationFunction
	locale: typeof currentLocale
	setLocale: typeof setLocale
} {
	const t: TranslationFunction = (key, params) => {
		const dict = dictionaries[currentLocale.value as Locale]
		let value = dict?.[key] ?? key
		if (params) {
			for (const [k, v] of Object.entries(params)) {
				value = value.replace(`{${k}}`, String(v))
			}
		}
		return value
	}

	return { t, locale: currentLocale, setLocale }
}
