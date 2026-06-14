<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import { saveApplicationSetting } from '../api/settings'
import { FRONTEND_LOCALE_SETTING_KEY } from '../types/settings'
import type { Locale } from '../../../platform/i18n/types'

const { t, locale, setLocale } = useI18n()

const localeOptions = [
  { value: 'en', label: 'English' },
  { value: 'ru', label: 'Русский' }
]

async function updateLocale(value: string) {
  if (value !== 'en' && value !== 'ru') return
  setLocale(value as Locale)
  try {
    await saveApplicationSetting(FRONTEND_LOCALE_SETTING_KEY, value)
  } catch (err) {
    // Revert on failure
    const revert = value === 'en' ? 'ru' : 'en'
    setLocale(revert as Locale)
    console.error('Failed to save locale setting:', err)
  }
}
</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Interface Language') }}</h2>
          <p>{{ t('Choose the display language for the Hermes Hub interface.') }}</p>
        </div>
      </header>
      <div class="settings-category-list">
        <div class="setting-row">
          <span>{{ t('Language') }}</span>
          <div class="setting-control">
            <select
              class="hermes-select-control"
              :value="locale"
              @change="(e) => updateLocale((e.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in localeOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </select>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.hermes-select-control {
  min-width: 180px;
  height: 2.125rem;
  padding: 0 0.625rem;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 0.8125rem;
  font-family: inherit;
  cursor: pointer;
  outline: none;
}

.hermes-select-control:focus-visible {
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
  border-color: var(--hh-accent);
}
</style>
