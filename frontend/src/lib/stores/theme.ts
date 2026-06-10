import { writable, derived, get } from 'svelte/store';
import {
	defaultFrontendThemeSettings,
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

export const themeSettings = writable<FrontendThemeSettings>(defaultFrontendThemeSettings());
export const themeDraft = writable<FrontendThemeSettings | null>(null);
export const isThemeSettingsSaving = writable(false);
export const themeError = writable('');

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
	themeDraft.update((draft) => {
		const base = draft ?? get(themeSettings);
		return { ...base, shellBackground };
	});
}

export function updateShellBrightness(event: Event): void {
	const raw = (event.target as HTMLInputElement | null)?.value;
	const value = Number(raw);
	if (Number.isNaN(value)) return;
	themeDraft.update((draft) => {
		const base = draft ?? get(themeSettings);
		return { ...base, backgroundBrightness: value as ShellBackgroundBrightness };
	});
}

export function updateGlobalPanelOpacity(event: Event): void {
	const raw = (event.target as HTMLInputElement | null)?.value;
	const value = Number(raw);
	if (Number.isNaN(value)) return;
	themeDraft.update((draft) => {
		const base = draft ?? get(themeSettings);
		return { ...base, panelOpacity: value as PanelOpacity };
	});
}

export function updateGlobalPanelBlur(event: Event): void {
	const raw = (event.target as HTMLInputElement | null)?.value;
	const value = Number(raw);
	if (Number.isNaN(value)) return;
	themeDraft.update((draft) => {
		const base = draft ?? get(themeSettings);
		return { ...base, panelBlur: value as PanelBlur };
	});
}

export function selectShellAccent(accentColor: ShellAccentColorId): void {
	themeDraft.update((draft) => {
		const base = draft ?? get(themeSettings);
		return { ...base, accentColor };
	});
}

export function resetThemeSettingsToDefault(): void {
	themeDraft.set(defaultFrontendThemeSettings());
}

export async function saveThemeSettings(): Promise<void> {
	let draft: FrontendThemeSettings | null = null;
	themeDraft.subscribe((value) => { draft = value; })();
	if (!draft) return;

	isThemeSettingsSaving.set(true);
	themeError.set('');
	try {
		await saveFrontendThemeSetting(draft);
		themeSettings.set(draft);
		themeDraft.set(null);
	} catch (e) {
		themeError.set(e instanceof Error ? e.message : 'Failed to save theme settings');
	} finally {
		isThemeSettingsSaving.set(false);
	}
}

// Label helpers moved from god file
export function shellBackgroundLabel(id: ShellBackgroundId): string {
	const option = shellBackgroundOptions.find((o) => o.id === id);
	return option?.label ?? id;
}

export function shellBackgroundPreviewClass(id: ShellBackgroundId): string {
	return `bg-preview-${id}`;
}

export function shellAccentSwatchClass(id: ShellAccentColorId): string {
	return `accent-swatch accent-swatch-${id}`;
}

export function shellAccentLabel(id: ShellAccentColorId): string {
	const option = shellAccentColorOptions.find((o) => o.id === id);
	return option?.label ?? id;
}
