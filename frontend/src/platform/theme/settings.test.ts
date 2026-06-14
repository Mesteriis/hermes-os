import { describe, expect, it } from 'vitest'
import {
	defaultThemeSettings,
	parseThemeSettings,
	shellAccentClass,
	shellBackgroundClass,
	shellBrightnessClass,
	shellPanelBlurClass,
	shellPanelOpacityClass,
	shellSpacingDensityClass
} from './settings'

describe('theme settings', () => {
	it('returns defaults for invalid values', () => {
		expect(parseThemeSettings(null)).toEqual(defaultThemeSettings())
		expect(parseThemeSettings({ schemaVersion: 99 })).toEqual(defaultThemeSettings())
	})

	it('keeps allowlisted values', () => {
		expect(
			parseThemeSettings({
				schemaVersion: 1,
				shellBackground: 'rune-teal',
				backgroundBrightness: 90,
				accentColor: 'cyan',
				panelOpacity: 50,
				panelBlur: 20,
				spacingDensity: 'compact'
			})
		).toEqual({
			schemaVersion: 1,
			shellBackground: 'rune-teal',
			backgroundBrightness: 90,
			accentColor: 'cyan',
			panelOpacity: 50,
			panelBlur: 20,
			spacingDensity: 'compact'
		})
	})

	it('returns allowlisted CSS classes', () => {
		const settings = parseThemeSettings({
			schemaVersion: 1,
			shellBackground: 'network-mesh',
			backgroundBrightness: 70,
			accentColor: 'violet',
			panelOpacity: 80,
			panelBlur: 12,
			spacingDensity: 'comfortable'
		})

		expect(shellBackgroundClass(settings)).toBe('shell-bg-network-mesh')
		expect(shellBrightnessClass(settings)).toBe('shell-bg-brightness-70')
		expect(shellAccentClass(settings)).toBe('theme-accent-violet')
		expect(shellPanelOpacityClass(settings)).toBe('panel-opacity-80')
		expect(shellPanelBlurClass(settings)).toBe('panel-blur-12')
		expect(shellSpacingDensityClass(settings)).toBe('spacing-density-comfortable')
	})
})
