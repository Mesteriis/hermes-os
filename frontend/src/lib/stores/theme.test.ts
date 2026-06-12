import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';
import {
	cancelThemeEditing,
	setThemeSettings,
	shellAccentSwatchClass,
	shellBackgroundPreviewClass,
	shellThemeClass,
	themeDraft,
	updateThemeDraft
} from './theme';

describe('theme store', () => {
	beforeEach(() => {
		cancelThemeEditing();
		setThemeSettings({
			schemaVersion: 1,
			shellBackground: 'network-mesh',
			backgroundBrightness: 70,
			accentColor: 'teal',
			panelOpacity: 70,
			panelBlur: 12
		});
	});

	it('starts theme drafts from the persisted shell theme, not defaults', () => {
		setThemeSettings({
			schemaVersion: 1,
			shellBackground: 'rune-teal',
			backgroundBrightness: 90,
			accentColor: 'violet',
			panelOpacity: 50,
			panelBlur: 20
		});

		updateThemeDraft({ accentColor: 'cyan' });

		expect(get(themeDraft)).toEqual({
			schemaVersion: 1,
			shellBackground: 'rune-teal',
			backgroundBrightness: 90,
			accentColor: 'cyan',
			panelOpacity: 50,
			panelBlur: 20
		});
		expect(get(shellThemeClass)).toContain('shell-bg-rune-teal');
		expect(get(shellThemeClass)).toContain('theme-accent-cyan');
	});

	it('keeps appearance preview classes explicit for CSS thumbnails', () => {
		expect(shellBackgroundPreviewClass('forest-stream')).toBe(
			'background-preview bg-preview-forest-stream'
		);
		expect(shellAccentSwatchClass('violet')).toBe('accent-swatch accent-swatch-violet');
	});
});
