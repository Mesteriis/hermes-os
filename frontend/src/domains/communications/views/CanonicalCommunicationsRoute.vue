<script setup lang="ts">
import { onMounted } from 'vue'

import CanonicalCommunicationDetail from '../presentation/CanonicalCommunicationDetail.vue'
import CanonicalCommunicationsPage from '../presentation/CanonicalCommunicationsPage.vue'
import { useCanonicalCommunicationDetail } from '../queries/useCanonicalCommunicationDetail'
import { useCanonicalCommunicationsPage } from '../queries/useCanonicalCommunicationsPage'

const surface = useCanonicalCommunicationsPage()
const detail = useCanonicalCommunicationDetail()

onMounted(() => {
	void surface.load()
})

function openMessage(messageKey: string): void {
	const messageId = surface.selectMessage(messageKey)
	if (messageId) void detail.open(messageId)
}

function closeMessage(): void {
	detail.close()
	surface.clearSelectedMessage()
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
		<CanonicalCommunicationDetail
			:model="detail.model.value"
			@close="closeMessage"
			@load-more-attachments="detail.loadMoreAttachments"
			@load-more-evidence="detail.loadMoreEvidence"
			@load-more-participants="detail.loadMoreParticipants"
			@load-more-references="detail.loadMoreReferences"
		/>
	</div>
</template>
