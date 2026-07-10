<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import { Icon, SkeletonPanel } from '../../../shared/ui'
import type { PersonaWorkspaceSection } from '../types/persona'
import { sectionLabel, sectionUnavailableMessage } from './personaWorkspaceElements'

defineProps<{
  activeSection: PersonaWorkspaceSection
}>()

const { t } = useI18n()
</script>

<template>
  <section class="personas-tab-panel">
    <section class="personas-dashboard-panel personas-skeleton-panel">
      <header>
        <Icon icon="tabler:progress" />
        <h3>{{ sectionLabel(activeSection, t) }}</h3>
      </header>
      <SkeletonPanel
        class="personas-skeleton-panel-body"
        :title="sectionUnavailableMessage(activeSection, t)"
        :description="t('The container is intentionally reserved at full size until the backend projection is connected.')"
        :rows="7"
      />
    </section>
  </section>
</template>
