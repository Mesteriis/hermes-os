import {
	panelBlurValues,
	panelOpacityValues,
	type PanelBlur,
	type PanelOpacity
} from './types';

export const FRONTEND_THEME_SCHEMA_VERSION = 1;

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
] as const;

export type ShellBackgroundId = (typeof shellBackgroundIds)[number];

export const shellBackgroundOptions: Array<{ id: ShellBackgroundId; label: string }> = [
	{ id: 'none', label: 'Default' },
	{ id: 'network-mesh', label: 'Network Mesh' },
	{ id: 'data-stream', label: 'Data Stream' },
	{ id: 'node-frame', label: 'Node Frame' },
	{ id: 'eclipse-grid', label: 'Eclipse Grid' },
	{ id: 'dna-blueprint', label: 'DNA Blueprint' },
	{ id: 'forest-network', label: 'Forest Network' },
	{ id: 'forest-stream', label: 'Forest Stream' },
	{ id: 'knowledge-map', label: 'Knowledge Map' },
	{ id: 'rune-gold', label: 'Rune Gold' },
	{ id: 'rune-teal', label: 'Rune Teal' }
];

export const shellBackgroundBrightnessValues = [30, 40, 50, 60, 70, 80, 90, 100] as const;

export type ShellBackgroundBrightness = (typeof shellBackgroundBrightnessValues)[number];

export const shellAccentColorIds = ['teal', 'cyan', 'blue', 'violet', 'amber', 'rose'] as const;

export type ShellAccentColorId = (typeof shellAccentColorIds)[number];

export const shellAccentColorOptions: Array<{ id: ShellAccentColorId; label: string }> = [
	{ id: 'teal', label: 'Teal' },
	{ id: 'cyan', label: 'Cyan' },
	{ id: 'blue', label: 'Blue' },
	{ id: 'violet', label: 'Violet' },
	{ id: 'amber', label: 'Amber' },
	{ id: 'rose', label: 'Rose' }
];

export type FrontendThemeSettings = {
	schemaVersion: typeof FRONTEND_THEME_SCHEMA_VERSION;
	shellBackground: ShellBackgroundId;
	backgroundBrightness: ShellBackgroundBrightness;
	accentColor: ShellAccentColorId;
	panelOpacity: PanelOpacity;
	panelBlur: PanelBlur;
};

const shellBackgroundIdSet = new Set<string>(shellBackgroundIds);
const shellBackgroundBrightnessSet = new Set<number>(shellBackgroundBrightnessValues);
const shellAccentColorIdSet = new Set<string>(shellAccentColorIds);
const panelOpacitySet = new Set<number>(panelOpacityValues);
const panelBlurSet = new Set<number>(panelBlurValues);

export function defaultFrontendThemeSettings(): FrontendThemeSettings {
	return {
		schemaVersion: FRONTEND_THEME_SCHEMA_VERSION,
		shellBackground: 'network-mesh',
		backgroundBrightness: 70,
		accentColor: 'teal',
		panelOpacity: 70,
		panelBlur: 12
	};
}

export function parseFrontendThemeSettings(value: unknown): FrontendThemeSettings {
	if (
		!isRecord(value) ||
		value.schemaVersion !== FRONTEND_THEME_SCHEMA_VERSION
	) {
		return defaultFrontendThemeSettings();
	}

	const defaults = defaultFrontendThemeSettings();
	return {
		schemaVersion: FRONTEND_THEME_SCHEMA_VERSION,
		shellBackground:
			typeof value.shellBackground === 'string' && isShellBackgroundId(value.shellBackground)
				? value.shellBackground
				: defaults.shellBackground,
		backgroundBrightness:
			typeof value.backgroundBrightness === 'number' &&
			Number.isInteger(value.backgroundBrightness) &&
			isShellBackgroundBrightness(value.backgroundBrightness)
				? value.backgroundBrightness
				: defaults.backgroundBrightness,
		accentColor:
			typeof value.accentColor === 'string' && isShellAccentColorId(value.accentColor)
				? value.accentColor
				: defaults.accentColor,
		panelOpacity:
			typeof value.panelOpacity === 'number' &&
			Number.isInteger(value.panelOpacity) &&
			isPanelOpacity(value.panelOpacity)
				? value.panelOpacity
				: defaults.panelOpacity,
		panelBlur:
			typeof value.panelBlur === 'number' &&
			Number.isInteger(value.panelBlur) &&
			isPanelBlur(value.panelBlur)
				? value.panelBlur
				: defaults.panelBlur
	};
}

export function shellBackgroundClass(settings: FrontendThemeSettings): string {
	return `shell-bg-${settings.shellBackground}`;
}

export function shellBrightnessClass(settings: FrontendThemeSettings): string {
	return `shell-bg-brightness-${settings.backgroundBrightness}`;
}

export function shellAccentClass(settings: FrontendThemeSettings): string {
	return `theme-accent-${settings.accentColor}`;
}

export function shellPanelOpacityClass(settings: FrontendThemeSettings): string {
	return `panel-opacity-${settings.panelOpacity}`;
}

export function shellPanelBlurClass(settings: FrontendThemeSettings): string {
	return `panel-blur-${settings.panelBlur}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isShellBackgroundId(value: string): value is ShellBackgroundId {
	return shellBackgroundIdSet.has(value);
}

function isShellBackgroundBrightness(value: number): value is ShellBackgroundBrightness {
	return shellBackgroundBrightnessSet.has(value);
}

function isShellAccentColorId(value: string): value is ShellAccentColorId {
	return shellAccentColorIdSet.has(value);
}

function isPanelOpacity(value: number): value is PanelOpacity {
	return panelOpacitySet.has(value);
}

function isPanelBlur(value: number): value is PanelBlur {
	return panelBlurSet.has(value);
}
