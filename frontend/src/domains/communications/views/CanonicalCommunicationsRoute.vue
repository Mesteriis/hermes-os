<script setup lang="ts">
import { onMounted } from 'vue'

import CanonicalCommunicationContent from '../presentation/CanonicalCommunicationContent.vue'
import CanonicalCommunicationDetail from '../presentation/CanonicalCommunicationDetail.vue'
import CanonicalCommunicationsPage from '../presentation/CanonicalCommunicationsPage.vue'
import CanonicalSavedSearchPanel from '../presentation/CanonicalSavedSearchPanel.vue'
import { useCanonicalCommunicationContent } from '../queries/useCanonicalCommunicationContent'
import { useCanonicalCommunicationDetail } from '../queries/useCanonicalCommunicationDetail'
import { useCanonicalCommunicationsPage } from '../queries/useCanonicalCommunicationsPage'
import { useCanonicalCommunicationsSavedSearches } from '../queries/useCanonicalCommunicationsSavedSearches'

const props = defineProps<{ canManageSavedSearches: boolean }>()
const surface = useCanonicalCommunicationsPage()
const detail = useCanonicalCommunicationDetail()
const content = useCanonicalCommunicationContent()
const savedSearches = useCanonicalCommunicationsSavedSearches(
	() => props.canManageSavedSearches,
	surface.currentSearchDraft,
)

onMounted(() => {
	void surface.load()
	void savedSearches.load()
})

function openMessage(messageKey: string): void {
	const messageId = surface.selectMessage(messageKey)
	if (messageId) {
		void detail.open(messageId)
		void content.open(messageId)
	}
}

function openSavedSearchMessage(messageKey: string): void {
	const messageId = savedSearches.selectMessage(messageKey)
	if (messageId) {
		void detail.open(messageId)
		void content.open(messageId)
	}
}

function closeMessage(): void {
	detail.close()
	content.close()
	surface.clearSelectedMessage()
	savedSearches.clearSelectedMessage()
}
</script>

<template>
	<div>
		<CanonicalCommunicationsPage
			:model="surface.model.value"
			@load-more-accounts="surface.loadMoreAccounts"
			@load-more-conversations="surface.loadMoreConversations"
			@load-more-messages="surface.loadMoreMessages"
			@load-more-search-results="surface.loadMoreSearchResults"
			@retry="surface.load"
			@search="surface.search"
			@select-account="surface.selectAccount"
			@select-conversation="surface.selectConversation"
			@select-message="openMessage"
			@update-search-text="surface.updateSearchText"
		/>
		<CanonicalSavedSearchPanel
			:model="savedSearches.model.value"
			@create="savedSearches.create"
			@execute="savedSearches.execute"
			@load-more-items="savedSearches.loadMoreItems"
			@load-more-results="savedSearches.loadMoreResults"
			@remove="savedSearches.remove"
			@replace="savedSearches.replace"
			@select-message="openSavedSearchMessage"
			@update-description="savedSearches.updateDescription"
			@update-name="savedSearches.updateName"
			@update-scope-current-account="savedSearches.updateScopeCurrentAccount"
		/>
		<CanonicalCommunicationDetail
			:model="detail.model.value"
			@close="closeMessage"
			@load-more-attachments="detail.loadMoreAttachments"
			@load-more-evidence="detail.loadMoreEvidence"
			@load-more-participants="detail.loadMoreParticipants"
			@load-more-references="detail.loadMoreReferences"
		/>
		<CanonicalCommunicationContent :model="content.model.value" />
	</div>
</template>
