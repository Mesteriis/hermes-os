import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type ShellBackgroundId =
  | 'eclipse-grid'
  | 'data-stream'
  | 'network-mesh'
  | 'forest-network'
  | 'knowledge-map'
  | 'forest-stream'
  | 'dna-blueprint'
  | 'node-frame'
  | 'rune-teal'
  | 'rune-gold'

export type ShellBrightness = 'dark' | 'darker' | 'darkest'
export type ShellAccentColorId =
  | 'teal'
  | 'cyan'
  | 'blue'
  | 'violet'
  | 'amber'
  | 'rose'

export type ShellPanelOpacity = 40 | 50 | 60 | 70 | 80 | 90 | 100
export type ShellPanelBlur = 0 | 4 | 8 | 12 | 16 | 20 | 24

export type FrontendThemeSettings = {
  shellBackground: ShellBackgroundId
  shellBrightness: ShellBrightness
  shellAccentColor: ShellAccentColorId
  panelOpacity: ShellPanelOpacity
  panelBlur: ShellPanelBlur
}

const defaultTheme: FrontendThemeSettings = {
  shellBackground: 'eclipse-grid',
  shellBrightness: 'darker',
  shellAccentColor: 'teal',
  panelOpacity: 60,
  panelBlur: 12
}

function loadThemeSettings(): FrontendThemeSettings {
  try {
    const stored = localStorage.getItem('hermes-theme-settings')
    if (stored) {
      return JSON.parse(stored) as FrontendThemeSettings
    }
  } catch {
    // ignore parse errors
  }
  return { ...defaultTheme }
}

export const useThemeStore = defineStore('theme', () => {
  const themeSettings = ref<FrontendThemeSettings>(loadThemeSettings())
  const themeDraft = ref<FrontendThemeSettings | null>(null)

  const effectiveThemeSettings = computed<FrontendThemeSettings>(() => {
    return themeDraft.value ?? themeSettings.value
  })

  const shellBackgroundClass = computed<string>(() => {
    return `shell-bg-${effectiveThemeSettings.value.shellBackground}`
  })

  const shellBrightnessClass = computed<string>(() => {
    return `shell-brightness-${effectiveThemeSettings.value.shellBrightness}`
  })

  const shellAccentClass = computed<string>(() => {
    return `accent-${effectiveThemeSettings.value.shellAccentColor}`
  })

  const shellPanelOpacityClass = computed<string>(() => {
    return `panel-opacity-${effectiveThemeSettings.value.panelOpacity}`
  })

  const shellPanelBlurClass = computed<string>(() => {
    return `panel-blur-${effectiveThemeSettings.value.panelBlur}`
  })

  const shellThemeClass = computed<string>(() => {
    return [
      shellBackgroundClass.value,
      shellBrightnessClass.value,
      shellAccentClass.value,
      shellPanelOpacityClass.value,
      shellPanelBlurClass.value
    ].join(' ')
  })

  function startThemeEditing(): void {
    themeDraft.value = { ...themeSettings.value }
  }

  function updateThemeDraft(patch: Partial<FrontendThemeSettings>): void {
    if (!themeDraft.value) {
      themeDraft.value = { ...themeSettings.value }
    }
    Object.assign(themeDraft.value, patch)
  }

  function cancelThemeEditing(): void {
    themeDraft.value = null
  }

  function saveThemeSettings(): void {
    if (themeDraft.value) {
      themeSettings.value = { ...themeDraft.value }
      themeDraft.value = null
    }
    persistThemeSettings()
  }

  function persistThemeSettings(): void {
    try {
      localStorage.setItem('hermes-theme-settings', JSON.stringify(themeSettings.value))
    } catch {
      // ignore storage errors
    }
  }

  function shellBackgroundLabel(id: ShellBackgroundId): string {
    const labels: Record<ShellBackgroundId, string> = {
      'eclipse-grid': 'Eclipse Grid',
      'data-stream': 'Data Stream',
      'network-mesh': 'Network Mesh',
      'forest-network': 'Forest Network',
      'knowledge-map': 'Knowledge Map',
      'forest-stream': 'Forest Stream',
      'dna-blueprint': 'DNA Blueprint',
      'node-frame': 'Node Frame',
      'rune-teal': 'Rune Teal',
      'rune-gold': 'Rune Gold'
    }
    return labels[id] ?? id
  }

  function shellAccentLabel(id: ShellAccentColorId): string {
    const labels: Record<ShellAccentColorId, string> = {
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
    effectiveThemeSettings,
    shellBackgroundClass,
    shellBrightnessClass,
    shellAccentClass,
    shellPanelOpacityClass,
    shellPanelBlurClass,
    shellThemeClass,
    startThemeEditing,
    updateThemeDraft,
    cancelThemeEditing,
    saveThemeSettings,
    shellBackgroundLabel,
    shellAccentLabel
  }
})
