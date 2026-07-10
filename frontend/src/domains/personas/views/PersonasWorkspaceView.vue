<script setup lang="ts">
import PersonasWorkspace from '../components/PersonasWorkspace.vue'
import { usePersonasPageSurface } from '../queries/usePersonasPageSurface'

const surface = usePersonasPageSurface()
</script>

<template>
  <PersonasWorkspace
    :owner-persona="surface.ownerPersona.value"
    :selected-persona="surface.selectedPersona.value"
    :filtered-personas="surface.filteredPersonas.value"
    :directory-count="surface.directoryCount.value"
    :pending-review-count="surface.pendingReviewCount.value"
    :directory-filter="surface.directoryFilter.value"
    :active-section="surface.activeSection.value"
    :search-query="surface.personaSearchQuery.value"
    :suggested-identity-candidates="surface.suggestedIdentityCandidates.value"
    :identity-traces="surface.identityTraces.value"
    :selected-persona-relationships="surface.selectedPersonaRelationships.value"
    :is-loading="surface.isLoading.value"
    :is-refreshing="surface.isRefreshing.value"
    :action-error="surface.actionError.value"
    :setting-owner-persona-id="surface.settingOwnerPersonaId.value"
    :reviewing-candidate-id="surface.reviewingCandidateId.value"
    :assigning-trace-id="surface.assigningTraceId.value"
    @refresh="surface.refresh"
    @select-persona="surface.selectPersona"
    @set-owner="surface.setOwnerPersona"
    @review-candidate="surface.setIdentityCandidateReview"
    @assign-trace-to-owner="surface.assignTraceToOwner"
    @assign-trace-to-selected-persona="surface.assignTraceToSelectedPersona"
    @toggle-address-book="surface.toggleAddressBookMembership"
    @update:directory-filter="surface.directoryFilter.value = $event"
    @update:active-section="surface.activeSection.value = $event"
    @update:search-query="surface.personaSearchQuery.value = $event"
  />
</template>
