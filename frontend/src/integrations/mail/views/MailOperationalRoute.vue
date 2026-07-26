<script setup lang="ts">
import MailOperationalPage from '../presentation/MailOperationalPage.vue'
import { useMailOperationalPage } from '../queries/useMailOperationalPage'

const props = defineProps<{
	canDeliver: boolean
	canSync: boolean
}>()
const surface = useMailOperationalPage({
	canDeliver: () => props.canDeliver,
	canSync: () => props.canSync,
})
</script>

<template>
	<MailOperationalPage
		:model="surface.model.value"
		@deliver="surface.deliver"
		@refresh-status="surface.refreshStatus"
		@sync="surface.sync"
		@update-operation-id="surface.updateOperationId"
		@update-provider-conversation-id="surface.updateProviderConversationId"
		@update-recipients="surface.updateRecipients"
		@update-subject="surface.updateSubject"
		@update-text-body="surface.updateTextBody"
	/>
</template>
