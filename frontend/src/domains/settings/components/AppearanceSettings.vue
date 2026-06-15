<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import {
	backgroundBrightnessValues,
	panelBlurValues,
	panelOpacityValues,
	type BackgroundBrightness,
	type PanelBlur,
	type PanelOpacity,
	type ThemeSettings
} from '../../../platform/theme/settings'
import { useThemeStore } from '../../../shared/stores/theme'
import AccentPicker from './appearance/AccentPicker.vue'
import AppearanceHeader from './appearance/AppearanceHeader.vue'
import BackgroundPicker from './appearance/BackgroundPicker.vue'
import SpacingDensityControl from './appearance/SpacingDensityControl.vue'
import ThemeRangeControl from './appearance/ThemeRangeControl.vue'

const { t } = useI18n()
const theme = useThemeStore()

function saveThemePatch(patch: Partial<ThemeSettings>) {
	theme.updateThemeDraft(patch)
	void theme.saveThemeSettings()
}

function previewThemePatch(patch: Partial<ThemeSettings>) {
	theme.updateThemeDraft(patch)
}

function commitThemeSettings() {
	void theme.saveThemeSettings()
}

function updateBackgroundBrightness(value: number) {
	const backgroundBrightness = pickAllowedNumber(value, backgroundBrightnessValues)
	if (backgroundBrightness !== null) {
		saveThemePatch({ backgroundBrightness })
	}
}

function updatePanelOpacity(value: number) {
	const panelOpacity = pickAllowedNumber(value, panelOpacityValues)
	if (panelOpacity !== null) {
		previewThemePatch({ panelOpacity })
	}
}

function updatePanelBlur(value: number) {
	const panelBlur = pickAllowedNumber(value, panelBlurValues)
	if (panelBlur !== null) {
		previewThemePatch({ panelBlur })
	}
}

function resetTheme() {
	theme.resetThemeSettings()
	void theme.saveThemeSettings()
}

function pickAllowedNumber<T extends BackgroundBrightness | PanelOpacity | PanelBlur>(
	value: number,
	allowed: readonly T[]
): T | null {
	return allowed.includes(value as T) ? (value as T) : null
}
</script>

<template>
	<div class="settings-page">
		<section class="panel settings-list-panel settings-primary-pane">
				<AppearanceHeader
					:title="t('Interface Appearance')"
					:description="t('Choose shell background, brightness and application accent color.')"
					:is-saving="theme.isSavingTheme"
					:save-state-label="t(theme.themePersistenceLabel)"
					:persistence-error="theme.themePersistenceError ? t(theme.themePersistenceError) : ''"
					@reset="resetTheme"
				/>

			<BackgroundPicker
				:value="theme.effectiveThemeSettings.shellBackground"
				:title="t('Shell Background')"
				:description="t('Background image for the desktop shell.')"
				@change="saveThemePatch({ shellBackground: $event })"
			/>

			<ThemeRangeControl
				id="shell-brightness"
				:label="t('Shell Brightness')"
				:description="t('Controls shell brightness level.')"
				:value="theme.effectiveThemeSettings.backgroundBrightness"
				:min="30"
				:max="100"
				:step="10"
				unit="%"
				@preview="updateBackgroundBrightness"
				@commit="commitThemeSettings"
			/>

			<AccentPicker
				:value="theme.effectiveThemeSettings.accentColor"
				:title="t('Accent Color')"
				:description="t('Application accent color used for highlights and active elements.')"
				@change="saveThemePatch({ accentColor: $event })"
			/>

			<ThemeRangeControl
				id="panel-opacity"
				:label="t('Panel Opacity')"
				:description="t('Controls the opacity of panels and cards.')"
				:value="theme.effectiveThemeSettings.panelOpacity"
				:min="40"
				:max="100"
				:step="10"
				unit="%"
				@preview="updatePanelOpacity"
				@commit="commitThemeSettings"
			/>

			<ThemeRangeControl
				id="panel-blur"
				:label="t('Panel Blur')"
				:description="t('Controls background blur behind panels.')"
				:value="theme.effectiveThemeSettings.panelBlur"
				:min="0"
				:max="24"
				:step="4"
				unit="px"
				@preview="updatePanelBlur"
				@commit="commitThemeSettings"
			/>

			<SpacingDensityControl
				:value="theme.effectiveThemeSettings.spacingDensity"
				:title="t('Spacing Density')"
				:description="t('Controls interface padding density.')"
				@change="saveThemePatch({ spacingDensity: $event })"
			/>
		</section>
	</div>
</template>
