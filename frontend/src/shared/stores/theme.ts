import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
	accentColorIds,
	defaultThemeSettings,
	parseThemeSettings,
	shellAccentClass as themeAccentClass,
	shellBackgroundClass as themeBackgroundClass,
	shellBackgroundIds,
	shellBrightnessClass as themeBrightnessClass,
	shellPanelBlurClass as themePanelBlurClass,
	shellPanelOpacityClass as themePanelOpacityClass,
	shellSpacingDensityClass as themeSpacingDensityClass,
	type AccentColorId,
	type ShellBackgroundId,
	type ThemeSettings
} from '../../platform/theme/settings'
import {
	loadLocalThemeSettings,
	loadPersistedThemeSettings,
	savePersistedThemeSettings,
	type ThemePersistenceSource
} from '../../platform/theme/persistence'

export type {
	AccentColorId as ShellAccentColorId,
	BackgroundBrightness,
	PanelBlur as ShellPanelBlur,
	PanelOpacity as ShellPanelOpacity,
	ShellBackgroundId,
	SpacingDensity,
	ThemeSettings as FrontendThemeSettings
} from '../../platform/theme/settings'

export const useThemeStore = defineStore('theme', () => {
	const themeSettings = ref<ThemeSettings>(loadLocalThemeSettings())
	const themeDraft = ref<ThemeSettings | null>(null)
	const isHydratingTheme = ref(false)
	const isSavingTheme = ref(false)
	const themePersistenceSource = ref<ThemePersistenceSource>('local_storage')
	const themePersistenceError = ref('')

	const effectiveThemeSettings = computed<ThemeSettings>(() => {
		return themeDraft.value ?? themeSettings.value
	})

	const shellBackgroundClass = computed<string>(() => {
		return themeBackgroundClass(effectiveThemeSettings.value)
	})

	const shellBrightnessClass = computed<string>(() => {
		return themeBrightnessClass(effectiveThemeSettings.value)
	})

	const shellAccentClass = computed<string>(() => {
		return themeAccentClass(effectiveThemeSettings.value)
	})

	const shellPanelOpacityClass = computed<string>(() => {
		return themePanelOpacityClass(effectiveThemeSettings.value)
	})

	const shellPanelBlurClass = computed<string>(() => {
		return themePanelBlurClass(effectiveThemeSettings.value)
	})

	const shellSpacingDensityClass = computed<string>(() => {
		return themeSpacingDensityClass(effectiveThemeSettings.value)
	})

		const shellThemeClass = computed<string>(() => {
			return [
				shellBackgroundClass.value,
			shellBrightnessClass.value,
			shellAccentClass.value,
			shellPanelOpacityClass.value,
			shellPanelBlurClass.value,
			shellSpacingDensityClass.value
			].join(' ')
		})

		const themePersistenceLabel = computed<string>(() => {
			if (isSavingTheme.value) return 'Saving'
			if (themePersistenceError.value) return 'Local fallback'
			return themePersistenceSource.value === 'application_settings' ? 'Auto-save' : 'Local settings'
		})

	function startThemeEditing(): void {
		themeDraft.value = { ...themeSettings.value }
	}

	function updateThemeDraft(patch: Partial<ThemeSettings>): void {
		const current = themeDraft.value ?? themeSettings.value
		themeDraft.value = parseThemeSettings({
			...current,
			...patch,
			schemaVersion: current.schemaVersion
		})
	}

	function cancelThemeEditing(): void {
		themeDraft.value = null
	}

	async function hydrateThemeSettings(): Promise<void> {
			isHydratingTheme.value = true
			themePersistenceError.value = ''
			try {
				const result = await loadPersistedThemeSettings()
				themeSettings.value = result.settings
				themePersistenceSource.value = result.source
				themePersistenceError.value = result.errorMessage
				themeDraft.value = null
			} catch (error) {
				themePersistenceError.value = error instanceof Error ? error.message : 'Failed to load theme'
		} finally {
			isHydratingTheme.value = false
		}
	}

	async function saveThemeSettings(): Promise<void> {
		isSavingTheme.value = true
			themePersistenceError.value = ''
			try {
				const next = themeDraft.value ?? themeSettings.value
				const result = await savePersistedThemeSettings(next)
				themeSettings.value = result.settings
				themePersistenceSource.value = result.source
				themePersistenceError.value = result.errorMessage
				themeDraft.value = null
			} catch (error) {
				themePersistenceError.value = error instanceof Error ? error.message : 'Failed to save theme'
		} finally {
			isSavingTheme.value = false
		}
	}

	function resetThemeSettings(): void {
		themeDraft.value = defaultThemeSettings()
	}

	function shellBackgroundLabel(id: ShellBackgroundId): string {
			const labels: Record<ShellBackgroundId, string> = {
				none: 'No background',
				'eclipse-grid': 'Dark grid',
				'data-stream': 'Data flow',
				'network-mesh': 'Digital network',
				'forest-network': 'Green network',
				'knowledge-map': 'Knowledge map',
				'forest-stream': 'Green flow',
				'dna-blueprint': 'Connection blueprint',
				'node-frame': 'Node grid',
				'rune-teal': 'Teal accent',
				'rune-gold': 'Warm accent'
			}
		return labels[id] ?? id
	}

	function shellAccentLabel(id: AccentColorId): string {
		const labels: Record<AccentColorId, string> = {
			teal: 'Teal',
			cyan: 'Cyan',
			blue: 'Blue',
			violet: 'Violet',
			amber: 'Amber',
			rose: 'Rose'
		}
		return labels[id] ?? id
	}

	return {
		themeSettings,
		themeDraft,
			isHydratingTheme,
			isSavingTheme,
			themePersistenceSource,
			themePersistenceError,
			themePersistenceLabel,
			backgroundOptions: shellBackgroundIds,
		accentOptions: accentColorIds,
		effectiveThemeSettings,
		shellBackgroundClass,
		shellBrightnessClass,
		shellAccentClass,
		shellPanelOpacityClass,
		shellPanelBlurClass,
		shellSpacingDensityClass,
		shellThemeClass,
		startThemeEditing,
		updateThemeDraft,
		cancelThemeEditing,
		hydrateThemeSettings,
		saveThemeSettings,
		resetThemeSettings,
		shellBackgroundLabel,
		shellAccentLabel
	}
})
