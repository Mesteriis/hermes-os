<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { PersonaWorkspaceSection } from '../types/persona'
import { PERSONA_WORKSPACE_SECTIONS, sectionLabel } from './personaWorkspaceElements'

const props = defineProps<{
  activeSection: PersonaWorkspaceSection
}>()

const emit = defineEmits<{
  'update:activeSection': [section: PersonaWorkspaceSection]
}>()

const { t } = useI18n()
</script>

<template>
  <nav class="personas-profile-tabs" :aria-label="t('Persona sections')">
    <button
      v-for="section in PERSONA_WORKSPACE_SECTIONS"
      :key="section"
      type="button"
      :class="{ 'is-active': props.activeSection === section }"
      :aria-selected="props.activeSection === section"
      @click="emit('update:activeSection', section)"
    >
      {{ sectionLabel(section, t) }}
    </button>
  </nav>
</template>
