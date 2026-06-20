import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

function scriptSetupSource(source: string): string {
	const match = source.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)
	return match?.[1] ?? ''
}

describe('CommunicationsPage folder management integration', () => {
	it('keeps page orchestration in a view-level controller instead of the Vue component', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)
		const scriptSource = scriptSetupSource(source)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)
		const resourceOverviewSource = readFileSync(
			new URL('./useMailResourceOverview.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useCommunicationsPageController')
		expect(source).toContain("../../../integrations/mail/components/AccountSetupModal.vue")
		expect(source).toContain('@generate-ai-reply="handleGenerateAiReply"')
		expect(source).toContain('@apply-ai-reply="handleApplyAiReply"')
		expect(source).toContain('@review-security="handleReviewSecurity"')
		expect(source).toContain('@review-recipients="handleReviewRecipients"')
		expect(source).toContain('@reply-all="handleReplyAll"')
		expect(source).toContain('@forward-message="handleForwardMessage"')
		expect(source).toContain('@redirect-message="handleRedirectMessage"')
		expect(source).toContain('@mark-message-read="handleMarkMessageRead"')
		expect(source).toContain('@mark-message-unread="handleMarkMessageUnread"')
		expect(source).toContain('@delete-from-provider="handleDeleteFromProvider"')
		expect(scriptSource).toContain('handleGenerateAiReply,')
		expect(scriptSource).toContain('handleApplyAiReply,')
		expect(scriptSource).toContain('handleReviewSecurity,')
		expect(scriptSource).toContain('handleReviewRecipients,')
		expect(scriptSource).toContain('handleReplyAll,')
		expect(scriptSource).toContain('handleForwardMessage,')
		expect(scriptSource).toContain('handleRedirectMessage,')
		expect(scriptSource).toContain('handleMarkMessageRead,')
		expect(scriptSource).toContain('handleMarkMessageUnread,')
		expect(scriptSource).toContain('handleDeleteFromProvider,')
		expect(source).not.toContain('useMailListQuery')
		expect(source).not.toContain('useBulkMessageActionMutation')
		expect(source).not.toContain('watch(')
		expect(source).not.toContain('onMounted')
		expect(controllerSource).toContain('useMailListQuery')
		expect(controllerSource).toContain('useFolderMailList')
		expect(controllerSource).toContain('useThreadReplyActions')
		expect(controllerSource).toContain('useMailSyncActions')
		expect(controllerSource).toContain('useSelectedMessageActions')
		expect(controllerSource).toContain('handleBulkAction')
		expect(controllerSource).toContain('useMailResourceOverview')
		expect(resourceOverviewSource).toContain('useSubscriptionsQuery')
		expect(resourceOverviewSource).toContain('useTopSendersQuery')
		expect(resourceOverviewSource).toContain('useMailBlockersQuery')
		expect(resourceOverviewSource).toContain('handleLoadMoreSubscriptions')
		expect(resourceOverviewSource).toContain('handleLoadMoreTopSenders')
		expect(controllerSource).toContain('handleGenerateAiReply')
		expect(controllerSource).toContain('handleApplyAiReply')
		expect(controllerSource).toContain('handleReviewSecurity')
		expect(controllerSource).toContain('handleReviewRecipients')
		expect(controllerSource).toContain('handleReplyAll')
		expect(controllerSource).toContain('handleForwardMessage')
		expect(controllerSource).toContain('handleRedirectMessage')
		expect(controllerSource).toContain('handleMarkMessageRead')
		expect(controllerSource).toContain('handleMarkMessageUnread')
		expect(controllerSource).toContain('handleDeleteFromProvider')
		expect(controllerSource).not.toContain("from '../components/")
		expect(controllerSource).not.toContain('fetch(')
		expect(controllerSource).not.toContain('ApiClient')
	})

	it('keeps selected-message side-effect orchestration in a focused controller helper', () => {
		const source = readFileSync(
			new URL('./useSelectedMessageActions.ts', import.meta.url),
			'utf8'
		)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useGenerateAiReplyMutation')
		expect(source).toContain('useReviewMessageSecurityMutation')
		expect(source).toContain('useReviewMessageRecipientsMutation')
		expect(source).toContain('useRedirectMessageMutation')
		expect(source).toContain('runSelectedMessageAction')
		expect(source).toContain('handleGenerateAiReply')
		expect(source).toContain('handleApplyAiReply')
		expect(source).toContain('handleReviewSecurity')
		expect(source).toContain('handleReviewRecipients')
		expect(source).toContain('handleRedirectMessage')
		expect(source).toContain('handleExportMessage')
		expect(source).toContain('handleAddLabel')
		expect(source).toContain('handleSnoozeMessage')
		expect(source).not.toContain("from '../components/")
		expect(source).not.toContain('fetch(')
		expect(source).not.toContain('ApiClient')
		expect(controllerSource).not.toContain('useGenerateAiReplyMutation')
		expect(controllerSource).not.toContain('useReviewMessageSecurityMutation')
		expect(controllerSource).not.toContain('useReviewMessageRecipientsMutation')
		expect(controllerSource).not.toContain('useRedirectMessageMutation')
	})

	it('keeps thread reply send/draft orchestration in a focused controller helper', () => {
		const source = readFileSync(
			new URL('./useThreadReplyActions.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useSaveDraftMutation')
		expect(source).toContain('useSendMailMutation')
		expect(source).toContain('buildComposeDraftPayload')
		expect(source).toContain('composeFormToSendRequest')
		expect(source).toContain('threadReplyComposeForm')
		expect(source).toContain('handleReplyToThreadMessage')
		expect(source).toContain('handleSaveThreadReplyDraft')
		expect(source).toContain('handleSendThreadReply')
		expect(source).not.toContain("from '../components/")
		expect(source).not.toContain('fetch(')
		expect(source).not.toContain('ApiClient')
	})

	it('keeps mail sync side-effect orchestration in a focused controller helper', () => {
		const source = readFileSync(
			new URL('./useMailSyncActions.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('useRunMailSyncNowMutation')
		expect(source).toContain('handleSyncNow')
		expect(source).toContain('useMailSyncSettingsQuery')
		expect(source).toContain('useUpdateMailSyncSettingsMutation')
		expect(source).toContain('handleUpdateSyncSettings')
		expect(source).toContain('clearSyncStatus')
		expect(source).toContain('loadInitialData')
		expect(source).not.toContain("from '../components/")
		expect(source).not.toContain('fetch(')
		expect(source).not.toContain('ApiClient')
	})

	it('renders the custom folder management strip with the selected account context', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('MailFolderStrip')
		expect(source).toContain('savedSearchChannelKind')
		expect(source).toContain('AttachmentSearchPanel')
		expect(source).toContain(':account-id="store.selectedMailAccountId || null"')
		expect(source).toContain('activeFolderId')
		expect(source).toContain(':active-id="activeFolderId"')
		expect(source).toContain('@select="handleFolderSelect"')
		expect(source).toContain(':current-channel-kind="savedSearchChannelKind || \'email\'"')
		expect(source).toContain(':is-folder-mode="Boolean(activeFolderId)"')
		expect(source).toContain(':messages="visibleMailList"')
		expect(source).toContain(':account-id="store.selectedMailAccountId"')
		expect(controllerSource).toContain('const savedSearchChannelKind = ref<string>()')
		expect(controllerSource).toContain('savedSearchChannelKind,')
	})

	it('routes mail list keyboard selection commands into the communications store', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain('@select-visible="store.selectVisibleMessages"')
		expect(source).toContain('@clear-selection="store.clearMessageSelection"')
	})

	it('wires server-backed thread pagination into the navigator list', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain('threads="store.threads"')
		expect(source).toContain(':has-thread-next-page="hasThreadNextPage"')
		expect(source).toContain(':is-fetching-thread-next-page="isFetchingThreadNextPage"')
		expect(source).toContain(':selected-thread-id="store.selectedThreadId"')
		expect(source).toContain('@select-thread="handleSelectThread"')
		expect(source).toContain('@load-more-threads="handleLoadMoreThreads"')
	})

	it('loads selected thread messages into the detail conversation view', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)

		expect(controllerSource).toContain('useThreadMessagesQuery')
		expect(source).toContain('selectedThreadMessages')
		expect(source).toContain(':selected-thread="store.selectedThread"')
		expect(source).toContain(':thread-messages="selectedThreadMessages"')
		expect(source).toContain('@open-thread-message="handleOpenThreadMessage"')
		expect(source).toContain('@reply-to-thread-message="handleReplyToThreadMessage"')
		expect(source).toContain('@save-thread-reply-draft="handleSaveThreadReplyDraft"')
		expect(source).toContain('@send-thread-reply="handleSendThreadReply"')
		expect(source).toContain(':is-thread-reply-sending="isThreadReplySending"')
	})

	it('surfaces outbox delivery status through query-backed strip wiring', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('OutboxStatusStrip')
		expect(controllerSource).toContain('useOutboxStatusStrip')
		expect(source).toContain(':items="outboxItems"')
		expect(source).toContain(':error-message="outboxErrorMessage"')
		expect(source).toContain(':has-more="hasMoreOutboxItems"')
		expect(source).toContain('@load-more="loadMoreOutboxItems"')
		expect(source).toContain('@prefetch-more="prefetchMoreOutboxItems"')
		expect(source).toContain('@undo="undoOutbox"')
	})

  it('routes bilingual reply send actions from message detail into compose', () => {
    const source = readFileSync(
      new URL('./CommunicationsPage.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('@send-bilingual-reply="handleBilingualReplySend"')
    expect(source).toContain('handleBilingualReplySend')
  })

	it('wires format-aware message export into the detail pane and action bar', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('@export-message="handleExportMessage"')
		expect(source).toContain(':last-message-export="store.lastMessageExport"')
		expect(controllerSource).toContain('handleExportMessage')
		expect(controllerSource).toContain('useSelectedMessageActions')
		expect(controllerSource).not.toContain('handleExportMd')
	})

	it('wires single-message label and snooze actions through the detail pane', () => {
		const source = readFileSync(
			new URL('./CommunicationsPage.vue', import.meta.url),
			'utf8'
		)
		const controllerSource = readFileSync(
			new URL('./useCommunicationsPageController.ts', import.meta.url),
			'utf8'
		)

		expect(source).toContain('@add-label="handleAddLabel"')
		expect(source).toContain('@remove-label="handleRemoveLabel"')
		expect(source).toContain('@snooze-message="handleSnoozeMessage"')
		expect(controllerSource).toContain('handleAddLabel')
		expect(controllerSource).toContain('handleRemoveLabel')
		expect(controllerSource).toContain('handleSnoozeMessage')
		expect(controllerSource).toContain('useSelectedMessageActions')
	})
})
