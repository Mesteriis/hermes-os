import { writable } from 'svelte/store';
import en from './en.json';
import ru from './ru.json';

export type Locale = 'en' | 'ru';

const dictionaries: Record<Locale, Record<string, string>> = { en, ru };

export const currentLocale = writable<Locale>('en');

export function setLocale(locale: Locale): void {
	currentLocale.set(locale);
}

/** Pure translation function. Pass $currentLocale from a Svelte component. */
export function t(locale: Locale, key: string): string {
	return dictionaries[locale]?.[key] ?? key;
}
