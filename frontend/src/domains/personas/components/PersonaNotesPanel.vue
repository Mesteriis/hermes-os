<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import { Icon, SkeletonPanel } from '../../../shared/ui'
import type { PersonaPanelProfile } from '../types/persona'

defineProps<{
  selectedPersona: PersonaPanelProfile
}>()

const { t } = useI18n()
</script>

<template>
  <section class="personas-tab-panel">
    <section class="personas-dashboard-panel personas-notes-panel">
      <header>
        <Icon icon="tabler:notes" />
        <h3>{{ t('Notes') }}</h3>
      </header>
      <p>{{ selectedPersona.notes || t('No notes yet') }}</p>
      <SkeletonPanel
        :title="t('Notes history and evidence review are not connected to this story surface yet.')"
        :description="t('Current notes come from the persona record; historical notes need a dedicated query.')"
        :rows="4"
      />
    </section>
  </section>
</template>
