<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useLanguageSettingsPanelController } from '../queries/useLanguageSettingsPanelController'
import type { useLanguageSettingsSurface } from '../queries/useLanguageSettingsSurface'

type LanguageSettingsSurface = ReturnType<typeof useLanguageSettingsSurface>

const props = defineProps<{
  surface: LanguageSettingsSurface
}>()

const { t } = useI18n()
const {
  localeOptions,
  currentLocale,
  isBusy,
  handleLocaleSelection,
} = useLanguageSettingsPanelController({
  surface: props.surface,
})
</script>

<template>
  <section class="settings-section">
    <header class="settings-section-toolbar">
      <div>
        <h3>{{ t('Language') }}</h3>
        <p>{{ t('Locale preference is stored through the settings registry.') }}</p>
      </div>
    </header>
    <div class="settings-form-grid">
      <button
        v-for="localeOption in localeOptions"
        :key="localeOption.value"
        type="button"
        class="settings-choice"
        :class="{ active: currentLocale.value === localeOption.value }"
        :disabled="isBusy.value"
        @click="handleLocaleSelection(localeOption.value)"
      >
        <Icon icon="tabler:language" />
        <strong>{{ localeOption.label }}</strong>
      </button>
    </div>
  </section>
</template>
