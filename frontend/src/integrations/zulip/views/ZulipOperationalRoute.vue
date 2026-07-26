<script setup lang="ts">
import ZulipOperationalPage from '../presentation/ZulipOperationalPage.vue'
import { useZulipOperationalPage } from '../queries/useZulipOperationalPage'

const props = defineProps<{ canCommand: boolean }>()
const surface = useZulipOperationalPage(() => props.canCommand)
</script>

<template>
	<ZulipOperationalPage
		:model="surface.model.value"
		@refresh-status="surface.refreshStatus"
		@send="surface.send"
		@update-destination="surface.updateDestination"
		@update-account-id="surface.updateAccountId"
		@update-stream="surface.updateStream"
		@update-topic="surface.updateTopic"
		@update-recipients="surface.updateRecipients"
		@update-content="surface.updateContent"
		@update-operation-id="surface.updateOperationId"
	/>
</template>
