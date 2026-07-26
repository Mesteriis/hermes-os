<script setup lang="ts">
import WhatsAppOperationalPage from '../presentation/WhatsAppOperationalPage.vue'
import { useWhatsAppOperationalPage } from '../queries/useWhatsAppOperationalPage'

const props = defineProps<{ canSend: boolean }>()
const surface = useWhatsAppOperationalPage(() => props.canSend)
</script>

<template>
	<WhatsAppOperationalPage
		:model="surface.model.value"
		@refresh-status="surface.refreshStatus"
		@send="surface.send"
		@update-account-id="surface.updateAccountId"
		@update-provider-chat-id="surface.updateProviderChatId"
		@update-draft="surface.updateDraft"
		@update-operation-id="surface.updateOperationId"
	/>
</template>
