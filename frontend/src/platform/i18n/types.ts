export type Locale = 'ru' | 'en'

export type TranslationFunction = (key: string, params?: Record<string, string | number>) => string

export type Dictionary = Record<string, string>
