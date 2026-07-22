<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { useApplicationSettingsSurface } from '../queries/useApplicationSettingsSurface'
import { useApplicationSettingsPanelController } from '../queries/useApplicationSettingsPanelController'

type ApplicationSettingsSurface = ReturnType<typeof useApplicationSettingsSurface>

const props = defineProps<{
  surface: ApplicationSettingsSurface
}>()

const { t } = useI18n()
const {
  categoryLabel,
  isLoading,
  handleSaveSetting,
  settingAllowedValues,
  settingsByCategory,
  settingControlType,
  settingDraftValue,
  settingHasChanged,
  savingSettingKey,
  handleSettingBooleanInput,
  handleSettingInput,
} = useApplicationSettingsPanelController({ surface: props.surface })
</script>

<template>
  <section class="settings-section">
    <header class="settings-section-toolbar">
      <div>
        <h3>{{ t('Application Settings') }}</h3>
        <p>{{ t('Declared backend settings registry, grouped by category.') }}</p>
      </div>
    </header>

    <div v-if="isLoading" class="settings-empty-state">
      <Icon icon="tabler:loader-2" />
      <strong>{{ t('Loading settings') }}</strong>
    </div>

    <template v-else>
      <section
        v-for="(settings, category) in settingsByCategory"
        :key="category"
        class="settings-registry-group"
      >
        <h4>{{ t(categoryLabel(String(category))) }}</h4>
        <article v-for="setting in settings" :key="setting.setting_key" class="settings-registry-row">
          <div>
            <strong>{{ setting.label }}</strong>
            <small>{{ setting.description }}</small>
            <code>{{ setting.setting_key }}</code>
          </div>

          <select
            v-if="settingControlType(setting) === 'select'"
            :value="settingDraftValue(setting)"
            :disabled="!setting.is_editable"
            @change="handleSettingInput(setting, $event)"
          >
            <option
              v-for="value in settingAllowedValues(setting)"
              :key="value"
              :value="value"
            >
              {{ value }}
            </option>
          </select>

          <label v-else-if="settingControlType(setting) === 'checkbox'" class="settings-switch">
            <input
              type="checkbox"
              :checked="settingDraftValue(setting) === 'true'"
              :disabled="!setting.is_editable"
              @change="handleSettingBooleanInput(setting, $event)"
            >
            <span />
            <strong>{{ settingDraftValue(setting) }}</strong>
          </label>

          <input
            v-else-if="settingControlType(setting) === 'number'"
            type="number"
            :value="settingDraftValue(setting)"
            :disabled="!setting.is_editable"
            @input="handleSettingInput(setting, $event)"
          >

          <textarea
            v-else-if="setting.value_kind === 'json'"
            :value="settingDraftValue(setting)"
            :disabled="!setting.is_editable"
            rows="3"
            @input="handleSettingInput(setting, $event)"
          />

          <input
            v-else
            type="text"
            :value="settingDraftValue(setting)"
            :disabled="!setting.is_editable"
            @input="handleSettingInput(setting, $event)"
          >

          <button
            type="button"
            class="secondary-button"
            :disabled="!setting.is_editable || !settingHasChanged(setting) || savingSettingKey === setting.setting_key"
            @click="handleSaveSetting(setting)"
          >
            <Icon icon="tabler:device-floppy" />
            {{ t('Save') }}
          </button>
        </article>
      </section>
    </template>
  </section>
</template>
