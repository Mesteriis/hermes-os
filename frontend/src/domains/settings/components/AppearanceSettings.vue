<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useThemeStore } from '../../../shared/stores/theme'
import { useSettingsStore } from '../stores/settings'

const { t } = useI18n()
const theme = useThemeStore()
const settingsStore = useSettingsStore()

const backgroundOptions = [
  { id: 'none', label: 'None' },
  { id: 'rune-teal', label: 'Rune Teal' },
  { id: 'rune-gold', label: 'Rune Gold' },
  { id: 'forest-network', label: 'Forest Network' },
  { id: 'forest-stream', label: 'Forest Stream' },
  { id: 'knowledge-map', label: 'Knowledge Map' },
  { id: 'eclipse-grid', label: 'Eclipse Grid' },
  { id: 'network-mesh', label: 'Network Mesh' },
  { id: 'dna-blueprint', label: 'DNA Blueprint' },
  { id: 'data-stream', label: 'Data Stream' }
] as const

const accentOptions = [
  { id: 'teal', label: 'Teal' },
  { id: 'cyan', label: 'Cyan' },
  { id: 'blue', label: 'Blue' },
  { id: 'violet', label: 'Violet' },
  { id: 'amber', label: 'Amber' },
  { id: 'rose', label: 'Rose' }
] as const

const brightnessValues = [30, 40, 50, 60, 70, 80, 90, 100] as const
const panelOpacityValues = [40, 50, 60, 70, 80, 90, 100] as const
const panelBlurValues = [0, 2, 4, 6, 8, 12, 24] as const

/** Map numeric brightness levels to theme brightness enum */
const brightnessToLevel: Record<number, 'dark' | 'darker' | 'darkest'> = {
  30: 'darkest',
  40: 'darkest',
  50: 'darkest',
  60: 'darker',
  70: 'darker',
  80: 'darker',
  90: 'dark',
  100: 'dark'
}

const levelToBrightness: Record<string, number> = {
  darkest: 40,
  darker: 70,
  dark: 90
}

const effectiveBrightnessValue = computed(() => {
  return levelToBrightness[theme.effectiveThemeSettings.shellBrightness] ?? 70
})

function selectBackground(id: string) {
  theme.updateThemeDraft({ shellBackground: id as any })
}

function selectAccent(id: string) {
  theme.updateThemeDraft({ shellAccentColor: id as any })
}

function updateBrightnessFromValue(val: number) {
  const level = brightnessToLevel[val]
  if (level) {
    theme.updateThemeDraft({ shellBrightness: level })
  }
}

function updatePanelOpacity(event: Event) {
  const val = parseInt((event.target as HTMLInputElement).value, 10)
  theme.updateThemeDraft({ panelOpacity: val as any })
}

function updatePanelBlur(event: Event) {
  const val = parseInt((event.target as HTMLInputElement).value, 10)
  theme.updateThemeDraft({ panelBlur: val as any })
}

function resetTheme() {
  theme.cancelThemeEditing()
}
</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Interface Appearance') }}</h2>
          <p>{{ t('Choose shell background, brightness and application accent color.') }}</p>
        </div>
        <div class="appearance-settings-actions">
          <button type="button" class="hermes-btn hermes-btn--outline" @click="resetTheme">
            {{ t('Default') }}
          </button>
          <span class="appearance-save-state">{{ t('Auto-save') }}</span>
        </div>
      </header>

      <!-- Shell Background -->
      <section class="appearance-section">
        <header>
          <div>
            <h3>{{ t('Shell Background') }}</h3>
            <p>{{ t('Background image for the desktop shell.') }}</p>
          </div>
        </header>
        <div class="background-option-grid">
          <button
            v-for="opt in backgroundOptions"
            :key="opt.id"
            type="button"
            class="background-option-btn"
            :class="{ active: theme.effectiveThemeSettings.shellBackground === opt.id }"
            :aria-pressed="theme.effectiveThemeSettings.shellBackground === opt.id"
            @click="selectBackground(opt.id)"
          >
            <span class="shell-bg-preview" :class="`shell-bg--${opt.id}`" />
            <span>{{ t(opt.label) }}</span>
          </button>
        </div>
      </section>

      <!-- Shell Brightness -->
      <section class="appearance-section">
        <header>
          <div>
            <h3>{{ t('Shell Brightness') }}</h3>
            <p>{{ t('Controls shell brightness level.') }}</p>
          </div>
        </header>
        <div class="brightness-options">
          <button
            v-for="v in brightnessValues"
            :key="v"
            type="button"
            class="brightness-option-btn"
            :class="{ active: effectiveBrightnessValue === v }"
            @click="updateBrightnessFromValue(v)"
          >
            {{ v }}%
          </button>
        </div>
      </section>

      <!-- Accent Color -->
      <section class="appearance-section">
        <header>
          <div>
            <h3>{{ t('Accent Color') }}</h3>
            <p>{{ t('Application accent color used for highlights and active elements.') }}</p>
          </div>
        </header>
        <div class="accent-option-grid">
          <button
            v-for="opt in accentOptions"
            :key="opt.id"
            type="button"
            class="accent-option-btn"
            :class="{ active: theme.effectiveThemeSettings.shellAccentColor === opt.id }"
            :aria-pressed="theme.effectiveThemeSettings.shellAccentColor === opt.id"
            @click="selectAccent(opt.id)"
          >
            <span class="accent-swatch" :class="`accent-swatch--${opt.id}`" />
            <span>{{ t(opt.label) }}</span>
          </button>
        </div>
      </section>

      <!-- Panel Opacity -->
      <section class="appearance-section">
        <header>
          <div>
            <h3>{{ t('Panel Opacity') }}</h3>
            <p>{{ t('Controls the opacity of panels and cards.') }}</p>
          </div>
          <strong>{{ theme.effectiveThemeSettings.panelOpacity }}%</strong>
        </header>
        <div class="brightness-control">
          <input
            type="range"
            min="40"
            max="100"
            step="10"
            :value="theme.effectiveThemeSettings.panelOpacity"
            list="panel-opacity-values"
            aria-label="Panel Opacity"
            @input="updatePanelOpacity"
          />
          <datalist id="panel-opacity-values">
            <option v-for="v in panelOpacityValues" :key="v" :value="v" />
          </datalist>
        </div>
      </section>

      <!-- Panel Blur -->
      <section class="appearance-section">
        <header>
          <div>
            <h3>{{ t('Panel Blur') }}</h3>
            <p>{{ t('Controls background blur behind panels.') }}</p>
          </div>
          <strong>{{ theme.effectiveThemeSettings.panelBlur }}px</strong>
        </header>
        <div class="brightness-control">
          <input
            type="range"
            min="0"
            max="24"
            step="2"
            :value="theme.effectiveThemeSettings.panelBlur"
            list="panel-blur-values"
            aria-label="Panel Blur"
            @input="updatePanelBlur"
          />
          <datalist id="panel-blur-values">
            <option v-for="v in panelBlurValues" :key="v" :value="v" />
          </datalist>
        </div>
      </section>
    </section>
  </div>
</template>

<style scoped>
.appearance-settings-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
}

.appearance-save-state {
  display: inline-flex;
  align-items: center;
  min-height: 34px;
  border: 1px solid rgba(111, 205, 195, 0.12);
  border-radius: var(--hh-radius-control, 6px);
  background: rgba(2, 12, 16, 0.48);
  color: var(--hh-text-muted);
  font-size: 12px;
  font-weight: 720;
  padding: 0 12px;
}

.appearance-section {
  display: grid;
  gap: 12px;
  padding: 14px;
  border-top: 1px solid var(--hh-border);
}

.appearance-section header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
}

.appearance-section header h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 680;
  color: var(--hh-text-primary);
}

.appearance-section header p {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--hh-text-muted);
}

.appearance-section header strong {
  font-size: 13px;
  font-weight: 720;
  color: var(--hh-accent);
  white-space: nowrap;
}

/* Background grid */
.background-option-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
  gap: 8px;
}

.background-option-btn {
  display: grid;
  gap: 6px;
  padding: 8px;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: var(--hh-surface-deep);
  cursor: pointer;
  text-align: center;
  font-size: 11px;
  color: var(--hh-text-secondary);
  transition: border-color 150ms ease, background 150ms ease;
}

.background-option-btn:hover {
  border-color: var(--hh-accent);
}

.background-option-btn.active {
  border-color: var(--hh-accent);
  background: var(--hh-hover-bg);
  color: var(--hh-accent);
}

/* Brightness button grid */
.brightness-options {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.brightness-option-btn {
  min-width: 48px;
  height: 34px;
  padding: 0 10px;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: transparent;
  color: var(--hh-text-secondary);
  font-size: 12px;
  font-weight: 620;
  cursor: pointer;
  transition: all 100ms ease;
}

.brightness-option-btn:hover {
  border-color: var(--hh-accent);
}

.brightness-option-btn.active {
  border-color: var(--hh-accent);
  background: color-mix(in srgb, var(--hh-accent) 15%, transparent);
  color: var(--hh-accent);
}

/* Accent grid */
.accent-option-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
  gap: 8px;
}

.accent-option-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  background: transparent;
  cursor: pointer;
  font-size: 11px;
  color: var(--hh-text-secondary);
  transition: border-color 150ms ease, background 150ms ease;
}

.accent-option-btn:hover {
  border-color: var(--hh-accent);
}

.accent-option-btn.active {
  border-color: var(--hh-accent);
  background: var(--hh-hover-bg);
  color: var(--hh-accent);
}

.accent-swatch {
  display: inline-block;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 1px solid var(--hh-border);
  flex-shrink: 0;
}

.accent-swatch--teal { background: #14b8a6; }
.accent-swatch--cyan { background: #06b6d4; }
.accent-swatch--blue { background: #3b82f6; }
.accent-swatch--violet { background: #8b5cf6; }
.accent-swatch--amber { background: #f59e0b; }
.accent-swatch--rose { background: #f43f5e; }

/* Background preview swatches */
.shell-bg-preview {
  display: block;
  width: 100%;
  height: 48px;
  border-radius: var(--hh-radius-xs);
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
}

.shell-bg--rune-teal {
  background: linear-gradient(135deg, #0f766e 0%, #042f2e 100%);
}
.shell-bg--rune-gold {
  background: linear-gradient(135deg, #b45309 0%, #451a03 100%);
}
.shell-bg--forest-network {
  background: linear-gradient(135deg, #065f46 0%, #022c22 100%);
}
.shell-bg--forest-stream {
  background: linear-gradient(135deg, #047857 0%, #064e3b 100%);
}
.shell-bg--knowledge-map {
  background: linear-gradient(135deg, #1e40af 0%, #1e1b4b 100%);
}
.shell-bg--eclipse-grid {
  background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
}
.shell-bg--network-mesh {
  background: linear-gradient(135deg, #334155 0%, #0f172a 100%);
}
.shell-bg--dna-blueprint {
  background: linear-gradient(135deg, #1e3a5f 0%, #0c1929 100%);
}
.shell-bg--data-stream {
  background: linear-gradient(135deg, #164e63 0%, #083344 100%);
}
.shell-bg--none {
  background: var(--hh-surface-deep);
}

/* Range input */
.brightness-control {
  display: flex;
  align-items: center;
}

.brightness-control input[type="range"] {
  width: 100%;
  height: 6px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--hh-hover-bg);
  border-radius: 3px;
  outline: none;
  cursor: pointer;
}

.brightness-control input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--hh-accent);
  border: 2px solid var(--hh-surface-panel);
  cursor: pointer;
  box-shadow: 0 1px 4px rgba(0,0,0,0.3);
}
</style>
