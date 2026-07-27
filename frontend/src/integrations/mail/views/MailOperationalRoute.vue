<script setup lang="ts">
import { watch } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import MailOperationalPage from '../presentation/MailOperationalPage.vue'
import {
	mailOperationalConnectionFingerprint,
} from '../queries/mailOperationalConnections'
import {
	mailSyncHealthConnectionFingerprint,
} from '../queries/mailSyncHealthConnections'
import { useMailOperationalRead } from '../queries/useMailOperationalRead'
import { useMailOperationalPage } from '../queries/useMailOperationalPage'
import { useMailSyncHealth } from '../queries/useMailSyncHealth'

const props = defineProps<{
	canDeliver: boolean
	canQuery: boolean
	canSync: boolean
	canSyncHealth: boolean
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
const syncHealth = useMailSyncHealth({
	canQuery: () => props.canSyncHealth,
	modules: () => props.modules,
})

watch(
	() => `${props.canQuery}:${mailOperationalConnectionFingerprint(props.modules)}`,
	() => { void read.reconcile() },
	{ immediate: true },
)

watch(
	() => `${props.canSyncHealth}:${mailSyncHealthConnectionFingerprint(props.modules)}`,
	() => { void syncHealth.reconcile() },
	{ immediate: true },
)
</script>

<template>
	<MailOperationalPage
		:model="surface.model.value"
		:read-model="read.model.value"
		:sync-health-model="syncHealth.model.value"
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
		@sync-health-load-more="syncHealth.loadMore"
		@sync-health-refresh="syncHealth.refresh"
		@select-sync-health-connection="syncHealth.selectConnection"
		@update-operation-id="surface.updateOperationId"
		@update-provider-conversation-id="surface.updateProviderConversationId"
		@update-recipients="surface.updateRecipients"
		@update-subject="surface.updateSubject"
		@update-text-body="surface.updateTextBody"
	/>
</template>
