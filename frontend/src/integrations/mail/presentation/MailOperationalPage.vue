<script setup lang="ts">
import type {
	MailCompositionModel,
	MailDraftEditorPatch,
	MailSignatureEditorPatch,
	MailTemplateEditorPatch,
} from './mailCompositionModel'
import MailCompositionPanel from './MailCompositionPanel.vue'
import type { MailDeliveryModel } from './mailDeliveryModel'
import MailDeliveryPanel from './MailDeliveryPanel.vue'
import MailOperationalReadPanel from './MailOperationalReadPanel.vue'
import type { MailMessageFlagModel } from './mailMessageFlagModel'
import type { MailOperationalReadModel } from './mailOperationalReadModel'
import MailSyncHealthPanel from './MailSyncHealthPanel.vue'
import type { MailSyncHealthModel } from './mailSyncHealthModel'
import type { MailSyncModel } from './mailSyncModel'
import MailSyncPanel from './MailSyncPanel.vue'
import './mailOperationalPage.css'

defineProps<{
	compositionModel: MailCompositionModel
	deliveryModel: MailDeliveryModel
	flagModel: MailMessageFlagModel
	readModel: MailOperationalReadModel
	syncHealthModel: MailSyncHealthModel
	syncModel: MailSyncModel
}>()

const emit = defineEmits<{
	compositionApplyTemplate: []
	compositionNewDraft: []
	compositionNewSignature: []
	compositionNewTemplate: []
	compositionRefresh: []
	compositionRemoveDraft: []
	compositionRemoveSignature: []
	compositionRemoveTemplate: []
	compositionSaveDraft: []
	compositionSaveSignature: []
	compositionSaveTemplate: []
	compositionSelectConnection: [connectionId: string]
	compositionSelectDraft: [draftId: string]
	compositionSelectSignature: [signatureId: string]
	compositionSelectTemplate: [templateId: string]
	compositionUpdateDraft: [patch: MailDraftEditorPatch]
	compositionUpdateSignature: [patch: MailSignatureEditorPatch]
	compositionUpdateTemplate: [patch: MailTemplateEditorPatch]
	compositionUseSignature: [signatureId: string]
	deliver: []
	flagRefreshStatus: []
	flagSetRead: [targetValue: boolean]
	flagSetStarred: [targetValue: boolean]
	loadMoreFolders: []
	loadMoreMessages: []
	loadMoreThreads: []
	readRefresh: []
	refreshStatus: []
	selectConnection: [connectionId: string]
	selectFolder: [folderId: string]
	selectMessage: [providerMessageId: string]
	selectThread: [providerThreadId: string]
	sync: []
	syncHealthLoadMore: []
	syncHealthRefresh: []
	selectSyncHealthConnection: [connectionId: string]
	updateOperationId: [value: string]
}>()
</script>

<template>
	<section class="mail-operational-page">
		<header class="mail-operational-page__header">
			<div>
				<span>Provider operations</span>
				<h1>Mail</h1>
				<p>Mail-owned read, composition, sync and asynchronous delivery contracts.</p>
			</div>
			<MailSyncPanel :model="syncModel" @sync="emit('sync')" />
		</header>

		<MailSyncHealthPanel
			:model="syncHealthModel"
			@load-more="emit('syncHealthLoadMore')"
			@refresh="emit('syncHealthRefresh')"
			@select-connection="emit('selectSyncHealthConnection', $event)"
		/>

		<MailOperationalReadPanel
			:flag-model="flagModel"
			:model="readModel"
			@flag-refresh-status="emit('flagRefreshStatus')"
			@flag-set-read="emit('flagSetRead', $event)"
			@flag-set-starred="emit('flagSetStarred', $event)"
			@load-more-folders="emit('loadMoreFolders')"
			@load-more-messages="emit('loadMoreMessages')"
			@load-more-threads="emit('loadMoreThreads')"
			@refresh="emit('readRefresh')"
			@select-connection="emit('selectConnection', $event)"
			@select-folder="emit('selectFolder', $event)"
			@select-message="emit('selectMessage', $event)"
			@select-thread="emit('selectThread', $event)"
		/>

		<MailCompositionPanel
			:model="compositionModel"
			:can-deliver="deliveryModel.canDeliver"
			:delivery-busy="deliveryModel.busy"
			@apply-template="emit('compositionApplyTemplate')"
			@deliver="emit('deliver')"
			@new-draft="emit('compositionNewDraft')"
			@new-signature="emit('compositionNewSignature')"
			@new-template="emit('compositionNewTemplate')"
			@refresh="emit('compositionRefresh')"
			@remove-draft="emit('compositionRemoveDraft')"
			@remove-signature="emit('compositionRemoveSignature')"
			@remove-template="emit('compositionRemoveTemplate')"
			@save-draft="emit('compositionSaveDraft')"
			@save-signature="emit('compositionSaveSignature')"
			@save-template="emit('compositionSaveTemplate')"
			@select-connection="emit('compositionSelectConnection', $event)"
			@select-draft="emit('compositionSelectDraft', $event)"
			@select-signature="emit('compositionSelectSignature', $event)"
			@select-template="emit('compositionSelectTemplate', $event)"
			@update-draft="emit('compositionUpdateDraft', $event)"
			@update-signature="emit('compositionUpdateSignature', $event)"
			@update-template="emit('compositionUpdateTemplate', $event)"
			@use-signature="emit('compositionUseSignature', $event)"
		/>

		<div class="mail-delivery-grid">
			<MailDeliveryPanel
				:model="deliveryModel"
				@refresh-status="emit('refreshStatus')"
				@update-operation-id="emit('updateOperationId', $event)"
			/>
			<section class="mail-operational-card mail-delivery-boundary">
				<div>
					<span>Boundary</span>
					<h2>Delivery stays separate</h2>
					<p>
						Saving a draft never sends it. Sending produces a receipt; terminal provider
						outcome remains queryable by operation ID.
					</p>
				</div>
			</section>
		</div>
	</section>
</template>
