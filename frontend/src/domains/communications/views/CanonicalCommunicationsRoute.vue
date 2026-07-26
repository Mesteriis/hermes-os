<script setup lang="ts">
import { onMounted } from 'vue'

import CanonicalCommunicationsPage from '../presentation/CanonicalCommunicationsPage.vue'
import { useCanonicalCommunicationsPage } from '../queries/useCanonicalCommunicationsPage'

const surface = useCanonicalCommunicationsPage()

onMounted(() => {
	void surface.load()
})
</script>

<template>
	<CanonicalCommunicationsPage
		:model="surface.model.value"
		@retry="surface.load"
		@search="surface.search"
		@select-account="surface.selectAccount"
		@select-conversation="surface.selectConversation"
		@update-search-text="surface.updateSearchText"
	/>
</template>
