export const THEME_SCHEMA_VERSION = 1

export const shellBackgroundIds = [
	'none',
	'network-mesh',
	'data-stream',
	'node-frame',
	'eclipse-grid',
	'dna-blueprint',
	'forest-network',
	'forest-stream',
	'knowledge-map',
	'rune-gold',
	'rune-teal'
] as const

export const backgroundBrightnessValues = [30, 40, 50, 60, 70, 80, 90, 100] as const
export const accentColorIds = ['teal', 'cyan', 'blue', 'violet', 'amber', 'rose'] as const
export const panelOpacityValues = [40, 50, 60, 70, 80, 90, 100] as const
export const panelBlurValues = [0, 4, 8, 12, 16, 20, 24] as const
export const spacingDensityIds = ['compact', 'normal', 'comfortable'] as const

export type ShellBackgroundId = (typeof shellBackgroundIds)[number]
export type BackgroundBrightness = (typeof backgroundBrightnessValues)[number]
export type AccentColorId = (typeof accentColorIds)[number]
export type PanelOpacity = (typeof panelOpacityValues)[number]
export type PanelBlur = (typeof panelBlurValues)[number]
export type SpacingDensity = (typeof spacingDensityIds)[number]

export type ThemeSettings = {
	schemaVersion: typeof THEME_SCHEMA_VERSION
	shellBackground: ShellBackgroundId
	backgroundBrightness: BackgroundBrightness
	accentColor: AccentColorId
	panelOpacity: PanelOpacity
	panelBlur: PanelBlur
	spacingDensity: SpacingDensity
}

export function defaultThemeSettings(): ThemeSettings {
	return {
		schemaVersion: THEME_SCHEMA_VERSION,
		shellBackground: 'network-mesh',
		backgroundBrightness: 70,
		accentColor: 'teal',
		panelOpacity: 70,
		panelBlur: 12,
		spacingDensity: 'normal'
	}
}

export function parseThemeSettings(value: unknown): ThemeSettings {
	if (!isRecord(value) || value.schemaVersion !== THEME_SCHEMA_VERSION) {
		return defaultThemeSettings()
	}

	const defaults = defaultThemeSettings()
	return {
		schemaVersion: THEME_SCHEMA_VERSION,
		shellBackground: pick(value.shellBackground, shellBackgroundIds, defaults.shellBackground),
		backgroundBrightness: pick(
			value.backgroundBrightness,
			backgroundBrightnessValues,
			defaults.backgroundBrightness
		),
		accentColor: pick(value.accentColor, accentColorIds, defaults.accentColor),
		panelOpacity: pick(value.panelOpacity, panelOpacityValues, defaults.panelOpacity),
		panelBlur: pick(value.panelBlur, panelBlurValues, defaults.panelBlur),
		spacingDensity: pick(value.spacingDensity, spacingDensityIds, defaults.spacingDensity)
	}
}

export function shellBackgroundClass(settings: ThemeSettings): string {
	return `shell-bg-${settings.shellBackground}`
}

export function shellBrightnessClass(settings: ThemeSettings): string {
	return `shell-bg-brightness-${settings.backgroundBrightness}`
}

export function shellAccentClass(settings: ThemeSettings): string {
	return `theme-accent-${settings.accentColor}`
}

export function shellPanelOpacityClass(settings: ThemeSettings): string {
	return `panel-opacity-${settings.panelOpacity}`
}

export function shellPanelBlurClass(settings: ThemeSettings): string {
	return `panel-blur-${settings.panelBlur}`
}

export function shellSpacingDensityClass(settings: ThemeSettings): string {
	return `spacing-density-${settings.spacingDensity}`
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function pick<const T extends readonly (string | number)[]>(
	value: unknown,
	allowed: T,
	fallback: T[number]
): T[number] {
	return allowed.includes(value as T[number]) ? (value as T[number]) : fallback
}
