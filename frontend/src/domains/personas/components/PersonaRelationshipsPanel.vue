<script setup lang="ts">
import type { PersonaPanelProfile, Relationship } from '../types/persona'
import PersonaRelationshipGraph from './PersonaRelationshipGraph.vue'

defineProps<{
  selectedPersona: PersonaPanelProfile
  entityLabels?: Readonly<Record<string, string>>
  selectedPersonaRelationships: readonly Relationship[]
}>()
</script>

<template>
  <section class="personas-tab-panel personas-relationships-tab">
    <section class="personas-dashboard-panel personas-graph-panel">
      <PersonaRelationshipGraph
        :selected-persona="selectedPersona"
        :entity-labels="entityLabels"
        :relationships="selectedPersonaRelationships"
      />
    </section>
  </section>
</template>
