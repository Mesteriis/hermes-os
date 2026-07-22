<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { Icon } from '../../../shared/ui'
import './personaWorkspace.css'
import type {
  EnrichedPersona,
  PersonaDirectoryFilter,
  PersonaIdentity,
  PersonaIdentityCandidate,
  PersonaIdentityReviewState,
  PersonaPanelProfile,
  PersonaWorkspaceSection,
  Relationship
} from '../types/persona'
import PersonaDirectoryPanel from './PersonaDirectoryPanel.vue'
import PersonaNotesPanel from './PersonaNotesPanel.vue'
import PersonaOverviewPanel from './PersonaOverviewPanel.vue'
import PersonaProfileHero from './PersonaProfileHero.vue'
import PersonaRelationshipsPanel from './PersonaRelationshipsPanel.vue'
import PersonaSectionTabs from './PersonaSectionTabs.vue'
import PersonaUnavailablePanel from './PersonaUnavailablePanel.vue'
import { buildPersonaEntityLabels } from './personaWorkspacePresentation'

const props = withDefaults(defineProps<{
  ownerPersona: PersonaPanelProfile | null
  selectedPersona: PersonaPanelProfile | null
  filteredPersonas: readonly EnrichedPersona[]
  directoryCount: number
  pendingReviewCount: number
  directoryFilter?: PersonaDirectoryFilter
  activeSection?: PersonaWorkspaceSection
  searchQuery: string
  suggestedIdentityCandidates: readonly PersonaIdentityCandidate[]
  identityTraces: readonly PersonaIdentity[]
  selectedPersonaRelationships: readonly Relationship[]
  isLoading?: boolean
  isRefreshing?: boolean
  actionError?: string
  settingOwnerPersonaId?: string | null
  reviewingCandidateId?: string | null
  assigningTraceId?: string | null
}>(), {
  directoryFilter: 'all',
  activeSection: 'overview',
  isLoading: false,
  isRefreshing: false,
  actionError: '',
  settingOwnerPersonaId: null,
  reviewingCandidateId: null,
  assigningTraceId: null
})

const emit = defineEmits<{
  refresh: []
  selectPersona: [index: number]
  setOwner: [persona: EnrichedPersona]
  reviewCandidate: [candidate: PersonaIdentityCandidate, state: PersonaIdentityReviewState]
  assignTraceToOwner: [trace: PersonaIdentity]
  assignTraceToSelectedPersona: [trace: PersonaIdentity]
  toggleAddressBook: [persona: PersonaPanelProfile, value: boolean]
  'update:directoryFilter': [value: PersonaDirectoryFilter]
  'update:activeSection': [section: PersonaWorkspaceSection]
  'update:searchQuery': [value: string]
}>()

const { t } = useI18n()

const entityLabels = computed(() => buildPersonaEntityLabels(
  props.filteredPersonas,
  props.ownerPersona,
  props.selectedPersona,
))

function handleSetOwner(persona: EnrichedPersona): void {
  emit('setOwner', persona)
}

function handleReviewCandidate(
  candidate: PersonaIdentityCandidate,
  state: PersonaIdentityReviewState
): void {
  emit('reviewCandidate', candidate, state)
}

function handleAssignTraceToOwner(trace: PersonaIdentity): void {
  emit('assignTraceToOwner', trace)
}

function handleAssignTraceToSelectedPersona(trace: PersonaIdentity): void {
  emit('assignTraceToSelectedPersona', trace)
}

function handleToggleAddressBook(persona: PersonaPanelProfile, value: boolean): void {
  emit('toggleAddressBook', persona, value)
}
</script>

<template>
  <section class="personas-workspace-view" :aria-label="t('Personas')">
    <p v-if="actionError" class="personas-inline-error">
      <Icon icon="tabler:alert-triangle" />
      {{ actionError }}
    </p>

    <div class="personas-workbench">
      <PersonaDirectoryPanel
        :owner-persona="ownerPersona"
        :selected-persona="selectedPersona"
        :filtered-personas="filteredPersonas"
        :directory-count="directoryCount"
        :search-query="searchQuery"
        :directory-filter="directoryFilter"
        :is-loading="isLoading"
        @select-persona="emit('selectPersona', $event)"
        @update:directory-filter="emit('update:directoryFilter', $event)"
        @update:search-query="emit('update:searchQuery', $event)"
      />

      <main class="personas-main-panel">
        <template v-if="selectedPersona">
          <PersonaProfileHero
            :selected-persona="selectedPersona"
            :relationship-count="selectedPersonaRelationships.length"
            :is-refreshing="isRefreshing"
            :setting-owner-persona-id="settingOwnerPersonaId"
            @refresh="emit('refresh')"
            @set-owner="handleSetOwner"
          />

          <PersonaSectionTabs
            :active-section="activeSection"
            @update:active-section="emit('update:activeSection', $event)"
          />

          <PersonaOverviewPanel
            v-if="activeSection === 'overview'"
            :owner-persona="ownerPersona"
            :selected-persona="selectedPersona"
            :entity-labels="entityLabels"
            :selected-persona-relationships="selectedPersonaRelationships"
            :pending-review-count="pendingReviewCount"
            :suggested-identity-candidates="suggestedIdentityCandidates"
            :identity-traces="identityTraces"
            :reviewing-candidate-id="reviewingCandidateId"
            :assigning-trace-id="assigningTraceId"
            @review-candidate="handleReviewCandidate"
            @assign-trace-to-owner="handleAssignTraceToOwner"
            @assign-trace-to-selected-persona="handleAssignTraceToSelectedPersona"
            @toggle-address-book="handleToggleAddressBook"
          />

          <PersonaRelationshipsPanel
            v-else-if="activeSection === 'relationships'"
            :selected-persona="selectedPersona"
            :entity-labels="entityLabels"
            :selected-persona-relationships="selectedPersonaRelationships"
          />

          <PersonaNotesPanel
            v-else-if="activeSection === 'notes'"
            :selected-persona="selectedPersona"
          />

          <PersonaUnavailablePanel
            v-else
            :active-section="activeSection"
          />
        </template>

        <div v-else class="personas-empty-large">
          <Icon icon="tabler:user-search" />
          <strong>{{ t('No persona selected') }}</strong>
        </div>
      </main>
    </div>
  </section>
</template>
