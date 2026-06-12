import { writable, derived, get } from 'svelte/store';
import {
	defaultFrontendThemeSettings,
	parseFrontendThemeSettings,
	shellBackgroundClass,
	shellBrightnessClass,
	shellAccentClass,
	shellPanelOpacityClass,
	shellPanelBlurClass,
	shellBackgroundOptions,
	shellAccentColorOptions,
	type FrontendThemeSettings,
	type ShellBackgroundId,
	type ShellAccentColorId,
	type ShellBackgroundBrightness
} from '$lib/layout/theme-settings';
import type { PanelOpacity, PanelBlur } from '$lib/layout/types';
import { saveFrontendThemeSetting } from '$lib/api';

const THEME_AUTOSAVE_DELAY_MS = 400;

export const themeSettings = writable<FrontendThemeSettings>(defaultFrontendThemeSettings());
export const themeDraft = writable<FrontendThemeSettings | null>(null);
export const isThemeSettingsSaving = writable(false);
export const themeError = writable('');

let themeAutosaveTimer: ReturnType<typeof setTimeout> | null = null;
let themeSaveVersion = 0;

export const effectiveThemeSettings = derived([themeSettings, themeDraft], ([$themeSettings, $themeDraft]) =>
	$themeDraft ?? $themeSettings
);

export const shellThemeClass = derived(effectiveThemeSettings, ($eff) =>
	`${shellBackgroundClass($eff)} ${shellBrightnessClass($eff)} ${shellAccentClass($eff)} ${shellPanelOpacityClass($eff)} ${shellPanelBlurClass($eff)}`
);

export const hasThemeChanges = derived([themeSettings, themeDraft], ([$themeSettings, $themeDraft]) =>
	$themeDraft !== null && JSON.stringify($themeDraft) !== JSON.stringify($themeSettings)
);

export function setThemeSettings(settings: FrontendThemeSettings): void {
	themeSettings.set(settings);
}

export function resetThemeDraft(): void {
	themeDraft.set(null);
	themeError.set('');
}

export function cancelThemeEditing(): void {
	if (themeAutosaveTimer) {
		clearTimeout(themeAutosaveTimer);
		themeAutosaveTimer = null;
	}
	themeSaveVersion += 1;
	themeDraft.set(null);
	themeError.set('');
}

export function updateThemeDraft(patch: Partial<FrontendThemeSettings>): void {
	themeDraft.update((draft) => {
		const base = draft ?? get(themeSettings);
		return { ...base, ...patch };
	});
}

export function selectShellBackground(shellBackground: ShellBackgroundId): void {
	applyThemePatch({ shellBackground });
}

export function updateShellBrightness(event: Event): void {
	const raw = (event.target as HTMLInputElement | null)?.value;
	const value = Number(raw);
	if (Number.isNaN(value)) return;
	applyThemePatch({ backgroundBrightness: value as ShellBackgroundBrightness });
}

export function updateGlobalPanelOpacity(event: Event): void {
	const raw = (event.target as HTMLInputElement | null)?.value;
	const value = Number(raw);
	if (Number.isNaN(value)) return;
	applyThemePatch({ panelOpacity: value as PanelOpacity });
}

export function updateGlobalPanelBlur(event: Event): void {
	const raw = (event.target as HTMLInputElement | null)?.value;
	const value = Number(raw);
	if (Number.isNaN(value)) return;
	applyThemePatch({ panelBlur: value as PanelBlur });
}

export function selectShellAccent(accentColor: ShellAccentColorId): void {
	applyThemePatch({ accentColor });
}

export function resetThemeSettingsToDefault(): void {
	applyThemeSettings(defaultFrontendThemeSettings());
}

export async function saveThemeSettings(): Promise<void> {
	let draft: FrontendThemeSettings | null = null;
	themeDraft.subscribe((value) => { draft = value; })();
	if (!draft) return;

	if (themeAutosaveTimer) {
		clearTimeout(themeAutosaveTimer);
		themeAutosaveTimer = null;
	}
	const version = ++themeSaveVersion;
	await persistThemeSettings(draft, version);
}

function applyThemePatch(patch: Partial<FrontendThemeSettings>): void {
	const base = get(themeDraft) ?? get(themeSettings);
	applyThemeSettings({ ...base, ...patch });
}

function applyThemeSettings(settings: FrontendThemeSettings): void {
	themeDraft.set(settings);
	themeError.set('');
	scheduleThemeAutosave(settings);
}

function scheduleThemeAutosave(settings: FrontendThemeSettings): void {
	if (themeAutosaveTimer) {
		clearTimeout(themeAutosaveTimer);
	}
	const version = ++themeSaveVersion;
	themeAutosaveTimer = setTimeout(() => {
		themeAutosaveTimer = null;
		void persistThemeSettings(settings, version);
	}, THEME_AUTOSAVE_DELAY_MS);
}

async function persistThemeSettings(settings: FrontendThemeSettings, version: number): Promise<void> {
	isThemeSettingsSaving.set(true);
	themeError.set('');
	try {
		const updated = await saveFrontendThemeSetting(settings);
		if (version !== themeSaveVersion) return;
		const persisted = parseFrontendThemeSettings(updated.value);
		themeSettings.set(persisted);
		themeDraft.set(null);
	} catch (e) {
		if (version === themeSaveVersion) {
			themeError.set(e instanceof Error ? e.message : 'Failed to save theme settings');
		}
	} finally {
		if (version === themeSaveVersion) {
			isThemeSettingsSaving.set(false);
		}
	}
}

// Label helpers moved from god file
export function shellBackgroundLabel(id: ShellBackgroundId): string {
	const option = shellBackgroundOptions.find((o) => o.id === id);
	return option?.label ?? id;
}

export function shellBackgroundPreviewClass(id: ShellBackgroundId): string {
	return `background-preview bg-preview-${id}`;
}

export function shellAccentSwatchClass(id: ShellAccentColorId): string {
	return `accent-swatch accent-swatch-${id}`;
}

export function shellAccentLabel(id: ShellAccentColorId): string {
	const option = shellAccentColorOptions.find((o) => o.id === id);
	return option?.label ?? id;
}
