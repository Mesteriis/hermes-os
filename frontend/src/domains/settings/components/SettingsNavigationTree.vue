<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { SettingsTreeGroup } from '../queries/settingsPagePresentation'
import type { SettingsSection } from '../stores/settings'

defineProps<{
  groups: SettingsTreeGroup[]
  selectedSection: SettingsSection
}>()

const emit = defineEmits<{
  selectSection: [section: SettingsSection]
}>()

const { t } = useI18n()

function handleSectionSelect(section: SettingsSection): void {
  emit('selectSection', section)
}
</script>

<template>
  <nav class="settings-tree" :aria-label="t('Settings sections')">
    <header class="settings-tree-header">
      <span>{{ t('Settings') }}</span>
      <strong>{{ t('Control Center') }}</strong>
    </header>

    <section
      v-for="group in groups"
      :key="group.label"
      class="settings-tree-group"
    >
      <h2>{{ t(group.label) }}</h2>
      <button
        v-for="item in group.items"
        :key="item.id"
        type="button"
        :class="{ active: selectedSection === item.id }"
        @click="handleSectionSelect(item.id)"
      >
        <Icon class="tree-icon" :icon="item.icon" />
        <span class="settings-tree-copy">
          <strong>{{ t(item.label) }}</strong>
          <small>{{ t(item.description) }}</small>
        </span>
        <em v-if="item.meta">{{ item.meta }}</em>
      </button>
    </section>
  </nav>
</template>
