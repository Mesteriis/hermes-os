<script setup lang="ts">
import PersonasWorkspace from '../components/PersonasWorkspace.vue'
import type { PersonasPageActions, PersonasPageModel } from './personasPageModel'

defineProps<{
	model: PersonasPageModel
	actions: PersonasPageActions
}>()
</script>

<template>
	<PersonasWorkspace
		:owner-persona="model.ownerPersona"
		:selected-persona="model.selectedPersona"
		:filtered-personas="model.filteredPersonas"
		:directory-count="model.directoryCount"
		:pending-review-count="model.pendingReviewCount"
		:directory-filter="model.directoryFilter"
		:active-section="model.activeSection"
		:search-query="model.searchQuery"
		:suggested-identity-candidates="model.suggestedIdentityCandidates"
		:identity-traces="model.identityTraces"
		:selected-persona-relationships="model.selectedPersonaRelationships"
		:is-loading="model.isLoading"
		:is-refreshing="model.isRefreshing"
		:action-error="model.actionError"
		:setting-owner-persona-id="model.settingOwnerPersonaId"
		:reviewing-candidate-id="model.reviewingCandidateId"
		:assigning-trace-id="model.assigningTraceId"
		@refresh="actions.refresh"
		@select-persona="actions.selectPersona"
		@set-owner="actions.setOwner"
		@review-candidate="actions.reviewCandidate"
		@assign-trace-to-owner="actions.assignTraceToOwner"
		@assign-trace-to-selected-persona="actions.assignTraceToSelectedPersona"
		@toggle-address-book="actions.toggleAddressBook"
		@update:directory-filter="actions.setDirectoryFilter"
		@update:active-section="actions.setActiveSection"
		@update:search-query="actions.setSearchQuery"
	/>
</template>
