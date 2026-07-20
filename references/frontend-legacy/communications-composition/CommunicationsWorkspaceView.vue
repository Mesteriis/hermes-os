<!-- Historical pre-clean-room workspace composition. Not part of the active client graph. -->
<script setup lang="ts">
import { computed } from 'vue'
import type { TelegramConversationRuntimeActionRunner } from '../../../shared/communications/types/telegramRuntimeActions'
import CommunicationsPage from '../presentation/CommunicationsPage.vue'
import type { CommunicationsPageActions, CommunicationsPageModel } from '../presentation/communicationsPageModel'
import { useCommunicationsWorkspaceViewSurface } from '../queries/useCommunicationsWorkspaceViewSurface'

const props = defineProps<{
	selectedRouteId?: string
	telegramRuntimeActionRunner?: TelegramConversationRuntimeActionRunner
}>()

// Temporary legacy adapter. The presentation page contains no query, store, or
// transport dependency and can later receive a generated Gateway adapter.
const surface = useCommunicationsWorkspaceViewSurface(
	() => props.selectedRouteId,
	() => props.telegramRuntimeActionRunner,
)

const model = computed<CommunicationsPageModel>(() => {
	if (surface.activeChannelId.value === 'mail') {
		return {
			channel: 'mail',
			items: surface.mailItems.value,
			conversation: surface.conversation.value,
			hasMoreItems: surface.pageSurface.hasVisibleNextPage.value,
			inspector: surface.mailInspector.value,
			isImporting: surface.isMailImporting.value,
			composeError: surface.pageSurface.store.composeSendError,
			composeAccountOptions: surface.pageSurface.mailComposeAccountOptions.value,
			composeForm: surface.pageSurface.store.composeForm,
			composeOpen: surface.pageSurface.store.isComposeOpen,
			composeStatus: surface.pageSurface.store.composeStatusMessage,
			isActionRunning: surface.isMailActionRunning.value,
			isLoadingMore: surface.pageSurface.isFetchingVisibleNextPage.value,
			isSending: surface.pageSurface.store.isSendingMessage,
			searchQuery: surface.pageSurface.store.messageSearchQuery,
			syncStatus: surface.mailSyncStatus.value,
		}
	}
	if (surface.activeChannelId.value === 'telegram') {
		return {
			channel: 'telegram',
			items: surface.telegramMessengerItems.value,
			conversation: surface.telegramMessengerConversation.value,
			inspector: surface.telegramMessengerInspector.value,
			isActionRunning: surface.isTelegramActionRunning.value,
			isListLoading: surface.isTelegramListLoading.value,
			isListRefreshing: surface.isTelegramListRefreshing.value,
			isLoadingOlder: surface.isTelegramLoadingOlder.value,
			listError: surface.telegramListError.value,
			selectedMessageId: surface.selectedTelegramMessage.value?.message_id,
			telegramChat: surface.selectedTelegramChat.value,
			telegramMessage: surface.selectedTelegramMessage.value,
			runtimeActionRunner: props.telegramRuntimeActionRunner,
		}
	}
	return {
		channel: 'whatsapp',
		items: surface.whatsappMessengerItems.value,
		conversation: surface.whatsappMessengerConversation.value,
		inspector: surface.whatsappMessengerInspector.value,
	}
})

const actions: CommunicationsPageActions = {
	closeCompose: surface.pageSurface.store.closeCompose,
	importMailFile: surface.importMailFile,
	attachComposeFiles: surface.pageSurface.handleComposeFiles,
	loadMoreMail: surface.pageSurface.handleLoadMoreMessages,
	newMailMessage: surface.pageSurface.handleNewMessage,
	refreshMail: surface.refreshMail,
	removeComposeAttachment: surface.pageSurface.handleRemoveComposeAttachment,
	saveCompose: surface.pageSurface.handleSaveComposeDraft,
	selectMailAction: surface.selectMailAction,
	selectMailMessage: surface.selectMailMessage,
	sendCompose: surface.pageSurface.handleSendCompose,
	updateMailSearch: surface.pageSurface.handleSearchQueryUpdate,
	updateCompose: surface.pageSurface.store.updateComposeForm,
	setVisibleMailItemIds: surface.handleVisibleMailItemIdsChange,
	runTelegramAction: (action, file, caption) => surface.runTelegramConversationAction(
		action,
		props.telegramRuntimeActionRunner,
		file,
		caption,
	),
	refreshTelegram: surface.refreshTelegramConversations,
	selectTelegramConversation: surface.selectTelegramConversation,
	selectTelegramMessage: surface.selectTelegramMessage,
	submitTelegram: surface.submitTelegramMessage,
	downloadTelegramAttachment: (attachment) => surface.downloadTelegramAttachment(
		attachment,
		props.telegramRuntimeActionRunner,
	),
	loadOlderTelegram: surface.loadOlderTelegramMessages,
	markTelegramMessagesVisible: surface.markTelegramMessagesVisible,
	selectWhatsappConversation: surface.selectWhatsappConversation,
}
</script>

<template>
	<CommunicationsPage :model="model" :actions="actions" />
</template>
