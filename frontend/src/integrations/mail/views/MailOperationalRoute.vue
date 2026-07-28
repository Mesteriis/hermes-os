<script setup lang="ts">
import { watch } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import MailOperationalPage from '../presentation/MailOperationalPage.vue'
import {
	mailCompositionConnectionFingerprint,
} from '../queries/mailCompositionConnections'
import {
	mailOperationalConnectionFingerprint,
} from '../queries/mailOperationalConnections'
import {
	mailSyncHealthConnectionFingerprint,
} from '../queries/mailSyncHealthConnections'
import { useMailComposition } from '../queries/useMailComposition'
import { useMailDelivery } from '../queries/useMailDelivery'
import { useMailOperationalRead } from '../queries/useMailOperationalRead'
import { useMailMessageFlags } from '../queries/useMailMessageFlags'
import { useMailMessageLocation } from '../queries/useMailMessageLocation'
import { useMailSync } from '../queries/useMailSync'
import { useMailSyncHealth } from '../queries/useMailSyncHealth'

const props = defineProps<{
	canCompose: boolean
	canComposeQuery: boolean
	canDeliver: boolean
	canMutateFlags: boolean
	canQuery: boolean
	canQueryFlagStatus: boolean
	canMutateLocation: boolean
	canQueryLocationStatus: boolean
	canSync: boolean
	canSyncHealth: boolean
	modules: readonly ClientModuleBootstrapV1[]
}>()

const composition = useMailComposition({
	canMutate: () => props.canCompose,
	canQuery: () => props.canComposeQuery,
	modules: () => props.modules,
})
const delivery = useMailDelivery({ canDeliver: () => props.canDeliver })
const sync = useMailSync({ canSync: () => props.canSync })
const read = useMailOperationalRead({
	canQuery: () => props.canQuery,
	modules: () => props.modules,
})
const messageFlags = useMailMessageFlags({
	canMutate: () => props.canMutateFlags,
	canQueryStatus: () => props.canQueryFlagStatus,
	selection: () => {
		const detail = read.model.value.detail
		const connectionId = read.model.value.selectedConnectionId
		if (!detail || !connectionId) return null
		return {
			connectionId,
			messageId: detail.id,
			isRead: detail.isRead,
			isStarred: detail.isStarred,
		}
	},
	refreshProjection: read.refresh,
})
const messageLocation = useMailMessageLocation({
	canMutate: () => props.canMutateLocation,
	canQueryStatus: () => props.canQueryLocationStatus,
	selection: () => {
		const detail = read.model.value.detail
		const connectionId = read.model.value.selectedConnectionId
		if (!detail || !connectionId) return null
		return {
			connectionId,
			messageId: detail.id,
			isTrashed: detail.isTrashed,
			folders: read.model.value.folders,
		}
	},
	refreshProjection: read.refresh,
})
const syncHealth = useMailSyncHealth({
	canQuery: () => props.canSyncHealth,
	modules: () => props.modules,
})

watch(
	() => `${props.canComposeQuery}:${mailCompositionConnectionFingerprint(props.modules)}`,
	() => { void composition.reconcile() },
	{ immediate: true },
)

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
		:composition-model="composition.model.value"
		:delivery-model="delivery.model.value"
		:flag-model="messageFlags.model.value"
		:location-model="messageLocation.model.value"
		:read-model="read.model.value"
		:sync-health-model="syncHealth.model.value"
		:sync-model="sync.model.value"
		@composition-apply-template="composition.applyTemplate"
		@composition-new-draft="composition.newDraft"
		@composition-new-signature="composition.newSignature"
		@composition-new-template="composition.newTemplate"
		@composition-refresh="composition.refresh"
		@composition-remove-draft="composition.removeDraft"
		@composition-remove-signature="composition.removeSignature"
		@composition-remove-template="composition.removeTemplate"
		@composition-save-draft="composition.saveDraft"
		@composition-save-signature="composition.saveSignature"
		@composition-save-template="composition.saveTemplate"
		@composition-select-connection="composition.selectConnection"
		@composition-select-draft="composition.selectDraft"
		@composition-select-signature="composition.selectSignature"
		@composition-select-template="composition.selectTemplate"
		@composition-update-draft="composition.updateDraft"
		@composition-update-signature="composition.updateSignature"
		@composition-update-template="composition.updateTemplate"
		@composition-use-signature="composition.useSignature"
		@deliver="delivery.deliver(composition.deliveryInput.value)"
		@load-more-folders="read.loadMoreFolders"
		@load-more-messages="read.loadMoreMessages"
		@load-more-threads="read.loadMoreThreads"
		@read-refresh="read.refresh"
		@flag-refresh-status="messageFlags.refreshStatus"
		@flag-set-read="messageFlags.setRead"
		@flag-set-starred="messageFlags.setStarred"
		@location-archive="messageLocation.archive"
		@location-move="messageLocation.move"
		@location-refresh-status="messageLocation.refreshStatus"
		@location-restore="messageLocation.restore"
		@location-select-target-folder="messageLocation.selectTargetFolder"
		@location-trash="messageLocation.trash"
		@refresh-status="delivery.refreshStatus"
		@select-connection="read.selectConnection"
		@select-folder="read.selectFolder"
		@select-message="read.selectMessage"
		@select-thread="read.selectThread"
		@sync="sync.sync"
		@sync-health-load-more="syncHealth.loadMore"
		@sync-health-refresh="syncHealth.refresh"
		@select-sync-health-connection="syncHealth.selectConnection"
		@update-operation-id="delivery.updateOperationId"
	/>
</template>
