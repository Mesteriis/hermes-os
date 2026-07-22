<script setup lang="ts">
import MailWorkspace from '../components/mail/MailWorkspace.vue'
import MessengerWorkspace from '../components/messengers/MessengerWorkspace.vue'
import type { CommunicationsPageActions, CommunicationsPageModel } from './communicationsPageModel'
import type { MailListItemModel } from '../components/mail/mailElements'
import type {
	MessengerAttachmentModel,
	MessengerListItemModel,
} from '../components/messengers/messengerElements'

const props = defineProps<{
	model: CommunicationsPageModel
	actions: CommunicationsPageActions
}>()

function handleUploadMessengerMedia(file: File, caption: string): void {
	void props.actions.runMessengerAction('upload_media', file, caption)
}

function handleAttachComposeFiles(files: File[]): void {
	props.actions.attachComposeFiles(files)
}

function handleCloseCompose(): void {
	props.actions.closeCompose()
}

function handleImportMailFile(file: File): void {
	void props.actions.importMailFile(file)
}

function handleLoadMoreMail(): void {
	void props.actions.loadMoreMail()
}

function handleNewMailMessage(): void {
	props.actions.newMailMessage()
}

function handleRefreshMail(): void {
	void props.actions.refreshMail()
}

function handleRemoveComposeAttachment(attachmentId: string): void {
	props.actions.removeComposeAttachment(attachmentId)
}

function handleSaveCompose(): void {
	void props.actions.saveCompose()
}

function handleSelectMailAction(actionId: string): void {
	void props.actions.selectMailAction(actionId)
}

function handleSelectMailMessage(item: MailListItemModel): void {
	props.actions.selectMailMessage(item)
}

function handleSendCompose(): void {
	void props.actions.sendCompose()
}

function handleUpdateMailSearch(query: string): void {
	props.actions.updateMailSearch(query)
}

function handleUpdateCompose(partial: Partial<Parameters<CommunicationsPageActions['updateCompose']>[0]>): void {
	props.actions.updateCompose(partial)
}

function handleSetVisibleMailItemIds(itemIds: string[]): void {
	props.actions.setVisibleMailItemIds(itemIds)
}

function handleConversationAction(action: Parameters<CommunicationsPageActions['runMessengerAction']>[0]): void {
	void props.actions.runMessengerAction(action)
}

function handleRefreshMessengerConversation(): void {
	void props.actions.refreshMessengerConversation()
}

function handleSelectMessengerConversation(item: MessengerListItemModel): void {
	props.actions.selectMessengerConversation(item)
}

function handleSelectMessengerMessage(messageId: string): void {
	props.actions.selectMessengerMessage(messageId)
}

function handleSubmitMessengerMessage(value: string): void {
	void props.actions.submitMessengerMessage(value)
}

function handleDownloadMessengerAttachment(attachment: MessengerAttachmentModel): void {
	void props.actions.downloadMessengerAttachment(attachment)
}

function handleLoadOlderMessengerMessages(): void {
	void props.actions.loadOlderMessengerMessages()
}

function handleMarkMessengerMessagesVisible(): void {
	void props.actions.markMessengerMessagesVisible()
}

</script>

<template>
	<section class="communications-workspace-view" aria-label="Communications workspace">
		<MailWorkspace
			v-if="props.model.channel === 'mail'"
			:items="props.model.items"
			:conversation="props.model.conversation"
			:has-more-items="props.model.hasMoreItems"
			:inspector="props.model.inspector"
			:is-importing="props.model.isImporting"
			:compose-error="props.model.composeError"
			:compose-account-options="props.model.composeAccountOptions"
			:compose-form="props.model.composeForm"
			:compose-open="props.model.composeOpen"
			:compose-status="props.model.composeStatus"
			:is-action-running="props.model.isActionRunning"
			:is-loading-more="props.model.isLoadingMore"
			:is-sending="props.model.isSending"
			:search-query="props.model.searchQuery"
			:sync-status="props.model.syncStatus"
			@attach-compose-files="handleAttachComposeFiles"
			@close-compose="handleCloseCompose"
			@import-mail-file="handleImportMailFile"
			@load-more="handleLoadMoreMail"
			@new-message="handleNewMailMessage"
			@refresh="handleRefreshMail"
			@remove-compose-attachment="handleRemoveComposeAttachment"
			@save-compose="handleSaveCompose"
			@select-action="handleSelectMailAction"
			@select-message="handleSelectMailMessage"
			@send-compose="handleSendCompose"
			@update-search-query="handleUpdateMailSearch"
			@update-compose="handleUpdateCompose"
			@visible-items-change="handleSetVisibleMailItemIds"
		/>

		<MessengerWorkspace
			v-else-if="props.model.channel === 'telegram' || props.model.channel === 'whatsapp'"
			:items="props.model.items"
			:conversation="props.model.conversation"
			:inspector="props.model.inspector"
			:is-action-running="props.model.isActionRunning"
			:is-list-loading="props.model.isListLoading"
			:is-list-refreshing="props.model.isListRefreshing"
			:is-loading-older="props.model.isLoadingOlder"
			:list-error="props.model.listError"
			:selected-message-id="props.model.selectedMessageId"
			:runtime-action-runner="props.model.runtimeActionRunner"
			@conversation-action="handleConversationAction"
			@refresh="handleRefreshMessengerConversation"
			@select-conversation="handleSelectMessengerConversation"
			@select-message="handleSelectMessengerMessage"
			@submit="handleSubmitMessengerMessage"
			@upload-file="handleUploadMessengerMedia"
			@download-attachment="handleDownloadMessengerAttachment"
			@load-older="handleLoadOlderMessengerMessages"
			@messages-visible="handleMarkMessengerMessagesVisible"
		/>
	</section>
</template>
