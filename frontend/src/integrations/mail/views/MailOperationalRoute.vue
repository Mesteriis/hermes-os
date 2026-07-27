<script setup lang="ts">
import { watch } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import MailOperationalPage from '../presentation/MailOperationalPage.vue'
import {
	mailOperationalConnectionFingerprint,
} from '../queries/mailOperationalConnections'
import { useMailOperationalRead } from '../queries/useMailOperationalRead'
import { useMailOperationalPage } from '../queries/useMailOperationalPage'

const props = defineProps<{
	canDeliver: boolean
	canQuery: boolean
	canSync: boolean
	modules: readonly ClientModuleBootstrapV1[]
}>()
const surface = useMailOperationalPage({
	canDeliver: () => props.canDeliver,
	canSync: () => props.canSync,
})
const read = useMailOperationalRead({
	canQuery: () => props.canQuery,
	modules: () => props.modules,
})

watch(
	() => `${props.canQuery}:${mailOperationalConnectionFingerprint(props.modules)}`,
	() => { void read.reconcile() },
	{ immediate: true },
)
</script>

<template>
	<MailOperationalPage
		:model="surface.model.value"
		:read-model="read.model.value"
		@deliver="surface.deliver"
		@load-more-folders="read.loadMoreFolders"
		@load-more-messages="read.loadMoreMessages"
		@load-more-threads="read.loadMoreThreads"
		@read-refresh="read.refresh"
		@refresh-status="surface.refreshStatus"
		@select-connection="read.selectConnection"
		@select-folder="read.selectFolder"
		@select-message="read.selectMessage"
		@select-thread="read.selectThread"
		@sync="surface.sync"
		@update-operation-id="surface.updateOperationId"
		@update-provider-conversation-id="surface.updateProviderConversationId"
		@update-recipients="surface.updateRecipients"
		@update-subject="surface.updateSubject"
		@update-text-body="surface.updateTextBody"
	/>
</template>
