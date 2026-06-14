<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useApplicationSettingsQuery, groupSettingsByCategory } from '../queries/useSettingsQuery'
import { useSettingsStore } from '../stores/settings'
import type { ApplicationSetting } from '../types/settings'

const { t } = useI18n()
const store = useSettingsStore()
const { data: appSettingsData, isLoading } = useApplicationSettingsQuery()

const applicationSettings = computed(() => appSettingsData.value?.items ?? [])
const settingsByCategory = computed(() => groupSettingsByCategory(applicationSettings.value))

function settingDraftValue(setting: ApplicationSetting): string {
  const draft = store.settingDrafts[setting.setting_key]
  if (draft !== undefined) return draft
  return String(setting.value ?? '')
}

function settingHasChanged(setting: ApplicationSetting): boolean {
  const draft = store.settingDrafts[setting.setting_key]
  if (draft === undefined) return false
  return draft !== String(setting.value)
}

function settingControlType(setting: ApplicationSetting): string {
  const allowedValues = setting.metadata?.allowed_values
  if (Array.isArray(allowedValues) && allowedValues.length > 0) return 'select'
  if (setting.value_kind === 'boolean') return 'checkbox'
  if (setting.value_kind === 'integer') return 'number'
  return 'text'
}

function settingAllowedValues(setting: ApplicationSetting): string[] {
  const vals = setting.metadata?.allowed_values
  return Array.isArray(vals) ? vals.map(String) : []
}

function settingMetadataFlag(setting: ApplicationSetting, key: string): boolean {
  return Boolean(setting.metadata?.[key])
}

function settingMetadataText(setting: ApplicationSetting, key: string): string {
  const val = setting.metadata?.[key]
  return typeof val === 'string' ? val : ''
}

function categoryLabel(category: string): string {
  const labels: Record<string, string> = {
    general: 'General',
    frontend: 'Interface',
    ai: 'AI',
    privacy: 'Privacy',
    notifications: 'Notifications'
  }
  return labels[category] ?? category
}

async function handleSave(setting: ApplicationSetting) {
  await store.saveSetting(setting)
}

function handleInput(setting: ApplicationSetting, event: Event) {
  const target = event.target as HTMLInputElement | HTMLSelectElement
  store.updateSettingDraft(setting.setting_key, target.value)
}
</script>

<template>
  <div class="settings-page">
    <section class="panel settings-list-panel settings-primary-pane">
      <header class="panel-title-row">
        <div>
          <h2>{{ t('Application Settings') }}</h2>
          <p>{{ t('All non-secret settings except database connectivity.') }}</p>
        </div>
      </header>

      <!-- Action messages -->
      <div v-if="store.actionMessage" class="setup-state success">{{ store.actionMessage }}</div>
      <div v-if="store.errorMessage" class="inline-error">{{ store.errorMessage }}</div>

      <!-- Loading -->
      <div v-if="isLoading && applicationSettings.length === 0" class="empty-panel fill">
        {{ t('Loading settings...') }}
      </div>

      <!-- Empty -->
      <div v-else-if="Object.keys(settingsByCategory).length === 0" class="empty-panel fill">
        {{ t('No application settings are declared yet.') }}
      </div>

      <!-- Settings list -->
      <div v-else class="settings-category-list">
        <section
          v-for="(settings, category) in settingsByCategory"
          :key="category"
          class="settings-category"
        >
          <header>
            <h3>{{ categoryLabel(category) }}</h3>
            <span>{{ settings.length }}</span>
          </header>

          <div
            v-for="setting in settings"
            :key="setting.setting_key"
            class="setting-row"
          >
            <div class="setting-copy">
              <strong>{{ setting.label }}</strong>
              <p>{{ setting.description }}</p>
              <div class="setting-meta-row">
                <code>{{ setting.setting_key }}</code>
                <em v-if="settingMetadataFlag(setting, 'bootstrap')">Bootstrap</em>
                <em v-if="settingMetadataFlag(setting, 'restart_required')">Restart</em>
                <em v-if="settingMetadataText(setting, 'env_var')">
                  {{ settingMetadataText(setting, 'env_var') }}
                </em>
              </div>
            </div>

            <div class="setting-control">
              <!-- Select control -->
              <select
                v-if="settingControlType(setting) === 'select'"
                class="hermes-select-control"
                :value="settingDraftValue(setting)"
                :disabled="!setting.is_editable"
                @change="(e) => handleInput(setting, e)"
              >
                <option
                  v-for="val in settingAllowedValues(setting)"
                  :key="val"
                  :value="val"
                >
                  {{ val }}
                </option>
              </select>

              <!-- Boolean checkbox -->
              <input
                v-else-if="settingControlType(setting) === 'checkbox'"
                type="checkbox"
                :checked="setting.value === true"
                :disabled="!setting.is_editable"
                @change="(e) => {
                  const checked = (e.target as HTMLInputElement).checked
                  store.updateSettingDraft(setting.setting_key, checked ? 'true' : 'false')
                }"
              />

              <!-- Number input -->
              <input
                v-else-if="settingControlType(setting) === 'number'"
                type="number"
                class="hermes-input-control"
                :value="settingDraftValue(setting)"
                :disabled="!setting.is_editable"
                @input="(e) => handleInput(setting, e)"
              />

              <!-- Text input -->
              <input
                v-else
                type="text"
                class="hermes-input-control"
                :value="settingDraftValue(setting)"
                :disabled="!setting.is_editable"
                @input="(e) => handleInput(setting, e)"
              />

              <!-- Save button -->
              <button
                v-if="settingHasChanged(setting)"
                type="button"
                class="hermes-btn hermes-btn--primary"
                :disabled="store.savingSettingKey === setting.setting_key"
                @click="handleSave(setting)"
              >
                {{ store.savingSettingKey === setting.setting_key ? t('Saving...') : t('Save') }}
              </button>
            </div>
          </div>
        </section>
      </div>
    </section>
  </div>
</template>

<style scoped>
.setting-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid var(--hh-border);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-copy {
  flex: 1;
  min-width: 0;
}

.setting-copy strong {
  font-size: 13px;
  font-weight: 620;
  color: var(--hh-text-primary);
}

.setting-copy p {
  margin: 2px 0 0;
  font-size: 11px;
  color: var(--hh-text-muted);
  line-height: 1.4;
}

.setting-meta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
  align-items: center;
}

.setting-meta-row code {
  font-size: 10px;
  color: var(--hh-text-muted);
  background: var(--hh-hover-bg);
  padding: 1px 4px;
  border-radius: 3px;
}

.setting-meta-row em {
  font-size: 10px;
  font-weight: 620;
  color: var(--hh-accent);
  font-style: normal;
  background: color-mix(in srgb, var(--hh-accent) 12%, transparent);
  padding: 1px 5px;
  border-radius: 3px;
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.hermes-select-control,
.hermes-input-control {
  min-width: 160px;
  height: 2.125rem;
  padding: 0 0.625rem;
  background: var(--hh-surface-deep);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-text-primary);
  font-size: 0.8125rem;
  font-family: inherit;
  outline: none;
}

.hermes-select-control:focus-visible,
.hermes-input-control:focus-visible {
  box-shadow: 0 0 0 2px var(--hh-focus-ring);
  border-color: var(--hh-accent);
}

.hermes-input-control[type="checkbox"] {
  min-width: auto;
  width: 1rem;
  height: 1rem;
}

/* Section headers */
.settings-category {
  padding: 0;
}

.settings-category header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0 4px;
  border-bottom: 1px solid var(--hh-border);
  margin-bottom: 4px;
}

.settings-category header h3 {
  margin: 0;
  font-size: 12px;
  font-weight: 720;
  color: var(--hh-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.settings-category header span {
  font-size: 11px;
  color: var(--hh-text-muted);
}

.setup-state.success {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-success, #22c55e) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-success) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-success, #22c55e);
  font-size: 12px;
  margin-bottom: 8px;
}

.inline-error {
  padding: 8px 12px;
  background: color-mix(in srgb, var(--hh-status-danger, #ef4444) 15%, transparent);
  border: 1px solid color-mix(in srgb, var(--hh-status-danger) 30%, transparent);
  border-radius: var(--hh-radius-sm);
  color: var(--hh-status-danger, #ef4444);
  font-size: 12px;
  margin-bottom: 8px;
}
</style>
