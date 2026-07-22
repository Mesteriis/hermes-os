import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import {
  backgroundBrightnessValues,
  panelBlurValues,
  panelOpacityValues,
  type ThemeSettings
} from '../../../platform/theme/settings'
import { useThemeStore } from '../../../shared/stores/theme'
import { pickAllowedThemeNumber } from './appearanceSettingsPredicates'

export function useAppearanceSettingsSurface() {
  const { t } = useI18n()
  const theme = useThemeStore()

  const currentBackgroundLabel = computed(() =>
    t(theme.shellBackgroundLabel(theme.effectiveThemeSettings.shellBackground))
  )
  const currentAccentLabel = computed(() =>
    t(theme.shellAccentLabel(theme.effectiveThemeSettings.accentColor))
  )
  const saveStateLabel = computed(() => t(theme.themePersistenceLabel))

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
    const backgroundBrightness = pickAllowedThemeNumber(value, backgroundBrightnessValues)
    if (backgroundBrightness !== null) {
      saveThemePatch({ backgroundBrightness })
    }
  }

  function updatePanelOpacity(value: number) {
    const panelOpacity = pickAllowedThemeNumber(value, panelOpacityValues)
    if (panelOpacity !== null) {
      previewThemePatch({ panelOpacity })
    }
  }

  function updatePanelBlur(value: number) {
    const panelBlur = pickAllowedThemeNumber(value, panelBlurValues)
    if (panelBlur !== null) {
      previewThemePatch({ panelBlur })
    }
  }

  function resetTheme() {
    theme.resetThemeSettings()
    void theme.saveThemeSettings()
  }

  return {
    backgroundBrightnessValues,
    panelBlurValues,
    panelOpacityValues,
    theme,
    currentBackgroundLabel,
    currentAccentLabel,
    saveStateLabel,
    saveThemePatch,
    previewThemePatch,
    commitThemeSettings,
    updateBackgroundBrightness,
    updatePanelOpacity,
    updatePanelBlur,
    resetTheme
  }
}
