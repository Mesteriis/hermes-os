<script setup lang="ts">
import MailWorkspace from '../components/mail/MailWorkspace.vue'
import MessengerWorkspace from '../components/messengers/MessengerWorkspace.vue'
import { useCommunicationsWorkspaceViewSurface } from '../queries/useCommunicationsWorkspaceViewSurface'

const props = defineProps<{
	selectedRouteId?: string
}>()

const surface = useCommunicationsWorkspaceViewSurface(() => props.selectedRouteId)
</script>

<template>
	<section class="communications-workspace-view" aria-label="Communications workspace">
		<MailWorkspace
			v-if="surface.activeChannelId.value === 'mail'"
			:items="surface.mailItems.value"
			:conversation="surface.conversation.value"
			:has-more-items="surface.pageSurface.hasVisibleNextPage.value"
			:inspector="surface.mailInspector.value"
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
			@close-compose="surface.pageSurface.store.closeCompose"
			@load-more="surface.pageSurface.handleLoadMoreMessages"
			@new-message="surface.pageSurface.handleNewMessage"
			@refresh="surface.refreshMail"
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
			@select-conversation="surface.selectTelegramConversation"
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
