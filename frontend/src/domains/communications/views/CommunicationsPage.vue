<script setup lang="ts">
import AttachmentSearchPanel from '../components/AttachmentSearchPanel.vue'
import BulkActionsBar from '../components/BulkActionsBar.vue'
import CommunicationsActionBar from '../components/CommunicationsActionBar.vue'
import CommunicationsDetailPane from '../components/CommunicationsDetailPane.vue'
import CommunicationsListPane from '../components/CommunicationsListPane.vue'
import CommunicationsRailPane from '../components/CommunicationsRailPane.vue'
import CommunicationsWorkbench from '../components/CommunicationsWorkbench.vue'
import ComposeDrawer from '../components/ComposeDrawer.vue'
import CommunicationFolderStrip from '../components/CommunicationFolderStrip.vue'
import OutboxStatusStrip from '../components/OutboxStatusStrip.vue'
import SavedSearchStrip from '../components/SavedSearchStrip.vue'
import TelegramCommunicationsPanel from '../providers/telegram/views/TelegramCommunicationsPanel.vue'
import WhatsAppCommunicationsPanel from '../providers/whatsapp/views/WhatsAppCommunicationsPanel.vue'
import { communicationSectionTabs } from '../constants/sectionTabs'
import AccountSetupModal from '../../../shared/mailSetup/AccountSetupModal.vue'
import { useNavigationStore } from '../../../shared/stores/navigation'
import { useCommunicationsPageController } from './useCommunicationsPageController'

const nav = useNavigationStore()

const {
  activeFolderId,
  activeSavedSearchId,
  activeSectionId,
  areResourcesLoading,
  blockers,
  clearSyncStatus,
  drafts,
  handleAnalyze,
  handleAddLabel,
  handleBilingualReplySend,
  handleBulkAction,
  handleCreateNote,
  handleCreateTask,
  handleDeleteDraft,
  handleForwardMessage,
  handleMarkMessageRead,
  handleMarkMessageUnread,
  handleDeleteFromProvider,
  handleFolderDeleted,
  handleFolderSelect,
  handleApplyAiReply,
  handleGenerateAiReply,
  handleLoadMoreDrafts,
  handleLoadMoreMessages,
  handleLoadMoreSubscriptions,
  handleLoadMoreThreads,
  handleLoadMoreTopSenders,
  handleMute,
  handleNewMessage,
  handleOpenDraft,
  handleOpenThreadMessage,
  handleReply,
  handleReplyAll,
  handleReplyToThreadMessage,
  handleRedirectMessage,
  handleRemoveLabel,
  handleReviewRecipients,
  handleReviewSecurity,
  handleSavedSearchDeleted,
  handleSavedSearchSelect,
  handleSaveThreadReplyDraft,
  handleSearchQueryUpdate,
  handleSelectMessage,
  handleSelectThread,
  handleSendThreadReply,
  handleSnoozeMessage,
  handleSyncNow,
  handleUpdateSyncSettings,
  handleToggleImportant,
  handleTogglePin,
  handleTranslate,
  handleExportMessage,
  hasMoreDrafts,
  hasMoreOutboxItems,
  hasMoreSubscriptions,
  hasRail,
  hasMoreTopSenders,
  hasThreadNextPage,
  hasVisibleNextPage,
  isAccountSetupOpen,
  isBulkActionRunning,
  isFetchingThreadNextPage,
  isFetchingVisibleNextPage,
  isLoadingMoreDrafts,
  isLoadingMoreOutbox,
  isLoadingMoreSubscriptions,
  isLoadingMoreTopSenders,
  isNavigatorListLoading,
  isOutboxLoading,
  isSelectedThreadLoading,
  isSyncSettingsLoading,
  isSyncSettingsSaving,
  isThreadReplySending,
  isUndoingOutbox,
  loadMoreOutboxItems,
  mailboxHealth,
  messageDetail,
  outboxErrorMessage,
  outboxItems,
  prefetchMoreOutboxItems,
  refetchMailList,
  savedSearchChannelKind,
  selectedBulkCount,
  selectedMailSyncSettings,
  selectedThreadErrorMessage,
  selectedThreadMessages,
  selectSection,
  stateCounts,
  store,
  subscriptions,
  topSenders,
  undoOutbox,
  visibleMailList,
  visibleMailListErrorMessage
} = useCommunicationsPageController()
</script>

<template>
  <section class="communications-page">
    <TelegramCommunicationsPanel v-if="nav.activeCommunicationSection === 'telegram'" />
    <WhatsAppCommunicationsPanel v-else-if="nav.activeCommunicationSection === 'whatsapp'" />
    <template v-else>
    <CommunicationsActionBar
      :search-query="store.messageSearchQuery"
      :section-tabs="communicationSectionTabs"
      :active-section-id="activeSectionId"
      :state-counts="stateCounts"
      :is-sync-busy="store.isMailSyncBusy"
      :sync-status-message="store.mailSyncStatusMessage"
      :sync-error="store.mailSyncError"
      :sync-settings="selectedMailSyncSettings"
      :is-sync-settings-loading="isSyncSettingsLoading"
      :is-sync-settings-saving="isSyncSettingsSaving"
      :health="mailboxHealth"
      :subscriptions="subscriptions"
      :top-senders="topSenders"
      :blockers="blockers"
      :are-resources-loading="areResourcesLoading"
      :has-more-subscriptions="hasMoreSubscriptions"
      :is-loading-more-subscriptions="isLoadingMoreSubscriptions"
      :has-more-top-senders="hasMoreTopSenders"
      :is-loading-more-top-senders="isLoadingMoreTopSenders"
      :drafts="drafts"
      :has-more-drafts="hasMoreDrafts"
      :is-loading-more-drafts="isLoadingMoreDrafts"
      :action-status="store.mailActionStatus"
      :action-error="store.mailActionError"
      :last-message-export="store.lastMessageExport"
      :page-error="store.communicationsError"
      @update:search-query="handleSearchQueryUpdate"
      @search="refetchMailList"
      @open-account-setup="isAccountSetupOpen = true"
      @compose="handleNewMessage"
      @sync-now="handleSyncNow"
      @update-sync-settings="handleUpdateSyncSettings"
      @load-more-subscriptions="handleLoadMoreSubscriptions"
      @load-more-top-senders="handleLoadMoreTopSenders"
      @clear-sync-status="clearSyncStatus"
      @select-section="selectSection"
      @open-draft="handleOpenDraft"
      @delete-draft="handleDeleteDraft"
      @load-more-drafts="handleLoadMoreDrafts"
      @clear-page-error="store.setCommunicationsError('')"
    />

    <CommunicationsWorkbench :is-loading="isNavigatorListLoading" :has-error="Boolean(visibleMailListErrorMessage)" :has-rail="hasRail">
      <template #list>
        <div class="communications-list-stack">
          <BulkActionsBar
            v-if="selectedBulkCount > 0"
            :selected-count="selectedBulkCount"
            :is-running="isBulkActionRunning"
            @action="handleBulkAction"
            @clear="store.clearMessageSelection"
          />
          <OutboxStatusStrip
            :items="outboxItems"
            :is-loading="isOutboxLoading"
            :is-loading-more="isLoadingMoreOutbox"
            :has-more="hasMoreOutboxItems"
            :is-undoing="isUndoingOutbox"
            :error-message="outboxErrorMessage"
            @undo="undoOutbox"
            @prefetch-more="prefetchMoreOutboxItems"
            @load-more="loadMoreOutboxItems"
          />
          <SavedSearchStrip
            :account-id="store.selectedMailAccountId || null"
            :active-id="activeSavedSearchId"
            :current-query="store.messageSearchQuery"
            :current-workflow-state="store.mailStateFilter"
            :current-local-state="store.mailLocalStateFilter"
            :current-channel-kind="savedSearchChannelKind || 'email'"
            @select="handleSavedSearchSelect"
            @deleted="handleSavedSearchDeleted"
          />
          <CommunicationFolderStrip
            :account-id="store.selectedMailAccountId || null"
            :active-id="activeFolderId"
            @select="handleFolderSelect"
            @deleted="handleFolderDeleted"
          />
          <AttachmentSearchPanel :account-id="store.selectedMailAccountId || null" />
          <CommunicationsListPane
            :account-id="store.selectedMailAccountId"
            :messages="visibleMailList"
            :threads="store.threads"
            :selected-index="store.selectedConversationIndex"
            :selected-thread-id="store.selectedThreadId"
            :selected-message-ids="store.selectedMessageIds"
            :navigator-mode="store.communicationsNavigatorMode"
            :is-folder-mode="Boolean(activeFolderId)"
            :is-loading="isNavigatorListLoading"
            :has-next-page="hasVisibleNextPage"
            :is-fetching-next-page="isFetchingVisibleNextPage"
            :has-thread-next-page="hasThreadNextPage"
            :is-fetching-thread-next-page="isFetchingThreadNextPage"
            :error-message="visibleMailListErrorMessage"
            @select="handleSelectMessage"
            @select-thread="handleSelectThread"
            @toggle-selection="store.toggleMessageSelection"
            @select-visible="store.selectVisibleMessages"
            @clear-selection="store.clearMessageSelection"
            @load-more="handleLoadMoreMessages"
            @load-more-threads="handleLoadMoreThreads"
            @update:navigator-mode="store.setNavigatorMode"
          />
        </div>
      </template>

      <template #detail>
        <CommunicationsDetailPane
          :detail="messageDetail"
          :insight="store.mailMessageInsight"
          :active-tab="store.activeMessageContextTab"
          :selected-thread="store.selectedThread"
          :thread-messages="selectedThreadMessages"
          :is-thread-loading="isSelectedThreadLoading"
          :thread-error-message="selectedThreadErrorMessage"
          :is-thread-reply-sending="isThreadReplySending"
          @update:active-tab="store.setActiveMessageContextTab"
          @reply="handleReply"
          @reply-all="handleReplyAll"
          @forward-message="handleForwardMessage"
          @redirect-message="handleRedirectMessage"
          @create-task="handleCreateTask"
          @create-note="handleCreateNote"
          @translate="handleTranslate"
          @generate-ai-reply="handleGenerateAiReply"
          @apply-ai-reply="handleApplyAiReply"
          @review-security="handleReviewSecurity"
          @review-recipients="handleReviewRecipients"
          @analyze="handleAnalyze"
          @send-bilingual-reply="handleBilingualReplySend"
          @mark-message-read="handleMarkMessageRead"
          @mark-message-unread="handleMarkMessageUnread"
          @delete-from-provider="handleDeleteFromProvider"
          @toggle-pin="handleTogglePin"
          @toggle-important="handleToggleImportant"
          @mute="handleMute"
          @export-message="handleExportMessage"
          @add-label="handleAddLabel"
          @remove-label="handleRemoveLabel"
          @snooze-message="handleSnoozeMessage"
          @open-compose="handleNewMessage"
          @open-thread-message="handleOpenThreadMessage"
          @reply-to-thread-message="handleReplyToThreadMessage"
          @save-thread-reply-draft="handleSaveThreadReplyDraft"
          @send-thread-reply="handleSendThreadReply"
        />
      </template>

      <template #rail>
        <CommunicationsRailPane
          :detail="messageDetail"
          :inspector-mode="store.communicationsInspectorMode"
          :projects="store.communicationProjects"
          :tasks="store.communicationTasks"
          @update:inspector-mode="store.setInspectorMode"
        />
      </template>
    </CommunicationsWorkbench>

    <ComposeDrawer v-if="store.isComposeOpen" />
    <AccountSetupModal v-if="isAccountSetupOpen" @close="isAccountSetupOpen = false" />
    </template>
  </section>
</template>

<style scoped>
.communications-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--hh-bg-primary, #ffffff);
}

.communications-list-stack {
  height: 100%; min-height: 0; display: flex; flex-direction: column; overflow: hidden;
}
</style>
