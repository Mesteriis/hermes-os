import { describe, expect, it } from 'vitest';
import {
	defaultFrontendThemeSettings,
	parseFrontendThemeSettings,
	shellBackgroundClass,
	shellBrightnessClass,
	shellAccentClass,
	shellPanelBlurClass,
	shellPanelOpacityClass
} from './theme-settings';

describe('frontend theme settings parser', () => {
	it('returns defaults for missing or invalid values', () => {
		expect(parseFrontendThemeSettings(null)).toEqual(defaultFrontendThemeSettings());
		expect(parseFrontendThemeSettings('bad')).toEqual(defaultFrontendThemeSettings());
		expect(parseFrontendThemeSettings({ schemaVersion: 99 })).toEqual(
			defaultFrontendThemeSettings()
		);
	});

	it('keeps valid shell background, brightness and accent settings', () => {
		expect(
			parseFrontendThemeSettings({
				schemaVersion: 1,
				shellBackground: 'knowledge-map',
				backgroundBrightness: 80,
				accentColor: 'violet',
				panelOpacity: 60,
				panelBlur: 16
			})
		).toEqual({
			schemaVersion: 1,
			shellBackground: 'knowledge-map',
			backgroundBrightness: 80,
			accentColor: 'violet',
			panelOpacity: 60,
			panelBlur: 16
		});
	});

	it('clamps unsupported values to safe defaults', () => {
		expect(
			parseFrontendThemeSettings({
				schemaVersion: 1,
				shellBackground: 'unknown',
				backgroundBrightness: 73,
				accentColor: 'hotpink',
				panelOpacity: 13,
				panelBlur: 99
			})
		).toEqual({
			schemaVersion: 1,
			shellBackground: 'network-mesh',
			backgroundBrightness: 70,
			accentColor: 'teal',
			panelOpacity: 70,
			panelBlur: 12
		});
	});

	it('returns only allowlisted css classes', () => {
		const settings = parseFrontendThemeSettings({
			schemaVersion: 1,
			shellBackground: 'rune-teal',
			backgroundBrightness: 90,
			accentColor: 'cyan',
			panelOpacity: 50,
			panelBlur: 20
		});

		expect(shellBackgroundClass(settings)).toBe('shell-bg-rune-teal');
		expect(shellBrightnessClass(settings)).toBe('shell-bg-brightness-90');
		expect(shellAccentClass(settings)).toBe('theme-accent-cyan');
		expect(shellPanelOpacityClass(settings)).toBe('panel-opacity-50');
		expect(shellPanelBlurClass(settings)).toBe('panel-blur-20');
	});
});
