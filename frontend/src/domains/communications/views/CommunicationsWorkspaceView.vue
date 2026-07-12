<script setup lang="ts">
import MailWorkspace from '../components/mail/MailWorkspace.vue'
import MessengerWorkspace from '../components/messengers/MessengerWorkspace.vue'
import { useCommunicationsWorkspaceViewSurface } from '../queries/useCommunicationsWorkspaceViewSurface'
import type { TelegramConversationRuntimeActionRunner } from '../../../shared/communications/types/telegramRuntimeActions'

const props = defineProps<{
	selectedRouteId?: string
	telegramRuntimeActionRunner?: TelegramConversationRuntimeActionRunner
}>()

const surface = useCommunicationsWorkspaceViewSurface(
	() => props.selectedRouteId,
	() => props.telegramRuntimeActionRunner
)
</script>

<template>
	<section class="communications-workspace-view" aria-label="Communications workspace">
		<MailWorkspace
			v-if="surface.activeChannelId.value === 'mail'"
			:items="surface.mailItems.value"
			:conversation="surface.conversation.value"
			:has-more-items="surface.pageSurface.hasVisibleNextPage.value"
			:inspector="surface.mailInspector.value"
			:is-importing="surface.isMailImporting.value"
			:compose-error="surface.pageSurface.store.composeSendError"
			:compose-account-options="surface.pageSurface.mailComposeAccountOptions.value"
			:compose-form="surface.pageSurface.store.composeForm"
			:compose-open="surface.pageSurface.store.isComposeOpen"
			:compose-status="surface.pageSurface.store.composeStatusMessage"
			:is-action-running="surface.isMailActionRunning.value"
			:is-loading-more="surface.pageSurface.isFetchingVisibleNextPage.value"
			:is-sending="surface.pageSurface.store.isSendingMessage"
			:search-query="surface.pageSurface.store.messageSearchQuery"
			:sync-status="surface.mailSyncStatus.value"
			@attach-compose-files="surface.pageSurface.handleComposeFiles"
			@close-compose="surface.pageSurface.store.closeCompose"
			@import-mail-file="surface.importMailFile"
			@load-more="surface.pageSurface.handleLoadMoreMessages"
			@new-message="surface.pageSurface.handleNewMessage"
			@refresh="surface.refreshMail"
			@remove-compose-attachment="surface.pageSurface.handleRemoveComposeAttachment"
			@save-compose="surface.pageSurface.handleSaveComposeDraft"
			@select-action="surface.selectMailAction"
			@select-message="surface.selectMailMessage"
			@send-compose="surface.pageSurface.handleSendCompose"
			@update-search-query="surface.pageSurface.handleSearchQueryUpdate"
			@update-compose="surface.pageSurface.store.updateComposeForm"
			@visible-items-change="surface.handleVisibleMailItemIdsChange"
		/>

		<MessengerWorkspace
			v-else-if="surface.activeChannelId.value === 'telegram'"
			:items="surface.telegramMessengerItems.value"
			:conversation="surface.telegramMessengerConversation.value"
			:inspector="surface.telegramMessengerInspector.value"
			:is-action-running="surface.isTelegramActionRunning.value"
			:is-list-loading="surface.isTelegramListLoading.value"
			:is-list-refreshing="surface.isTelegramListRefreshing.value"
			:is-loading-older="surface.isTelegramLoadingOlder.value"
			:list-error="surface.telegramListError.value"
			:selected-message-id="surface.selectedTelegramMessage.value?.message_id"
			:telegram-chat="surface.selectedTelegramChat.value"
			:telegram-message="surface.selectedTelegramMessage.value"
			:runtime-action-runner="props.telegramRuntimeActionRunner"
			@conversation-action="surface.runTelegramConversationAction($event, props.telegramRuntimeActionRunner)"
			@refresh="surface.refreshTelegramConversations"
			@select-conversation="surface.selectTelegramConversation"
			@select-message="surface.selectTelegramMessage"
			@submit="surface.submitTelegramMessage"
			@upload-file="(file, caption) => surface.runTelegramConversationAction('upload_media', props.telegramRuntimeActionRunner, file, caption)"
			@download-attachment="surface.downloadTelegramAttachment($event, props.telegramRuntimeActionRunner)"
			@load-older="surface.loadOlderTelegramMessages"
			@messages-visible="surface.markTelegramMessagesVisible"
		/>

		<MessengerWorkspace
			v-else
			:items="surface.whatsappMessengerItems.value"
			:conversation="surface.whatsappMessengerConversation.value"
			:inspector="surface.whatsappMessengerInspector.value"
			@select-conversation="surface.selectWhatsappConversation"
		/>
	</section>
</template>
