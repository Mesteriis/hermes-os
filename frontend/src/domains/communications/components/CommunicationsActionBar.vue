<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import CommunicationsTopbarSlot from './CommunicationsTopbarSlot.vue'
import DraftStrip from './DraftStrip.vue'
import HealthStrip from './HealthStrip.vue'
import MailCertificateStrip from './MailCertificateStrip.vue'
import MailResourceOverviewStrip from './MailResourceOverviewStrip.vue'
import MailSyncSettingsStrip from '../../../shared/mailSync/MailSyncSettingsStrip.vue'
import type {
  CommunicationSectionId,
  CommunicationDraft,
  CommunicationArchitectureBlocker,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailboxHealth,
  MessageExportResponse,
  SenderStats,
  SubscriptionSource,
  WorkflowStateCountItem
} from '../types/communications'

type SectionTab = {
  id: CommunicationSectionId
  label: string
  icon: string
}

const props = defineProps<{
  searchQuery: string
  sectionTabs: SectionTab[]
  activeSectionId: CommunicationSectionId
  stateCounts: WorkflowStateCountItem[]
  isSyncBusy: boolean
  syncStatusMessage: string
  syncError: string
  syncSettings: MailSyncSettings | null
  isSyncSettingsLoading: boolean
  isSyncSettingsSaving: boolean
  health: MailboxHealth | null
  subscriptions: SubscriptionSource[]
  topSenders: SenderStats[]
  blockers: CommunicationArchitectureBlocker[]
  areResourcesLoading: boolean
  hasMoreSubscriptions: boolean
  isLoadingMoreSubscriptions: boolean
  hasMoreTopSenders: boolean
  isLoadingMoreTopSenders: boolean
  drafts: CommunicationDraft[]
  hasMoreDrafts: boolean
  isLoadingMoreDrafts: boolean
  actionStatus: string
  actionError: string
  lastMessageExport: MessageExportResponse | null
  pageError: string
}>()

const emit = defineEmits<{
  'update:searchQuery': [query: string]
  search: []
  openAccountSetup: []
  compose: []
  syncNow: []
  updateSyncSettings: [settings: MailSyncSettingsUpdate]
  clearSyncStatus: []
  loadMoreSubscriptions: []
  loadMoreTopSenders: []
  selectSection: [sectionId: CommunicationSectionId]
  openDraft: [draft: CommunicationDraft]
  deleteDraft: [draftId: string]
  loadMoreDrafts: []
  clearPageError: []
}>()

const messageExportDownloadHref = computed(() => {
  if (!props.lastMessageExport) return ''
  const encoded = encodeURIComponent(props.lastMessageExport.content)
  return `data:${props.lastMessageExport.content_type};charset=utf-8,${encoded}`
})
</script>

<template>
  <Teleport to="#hermes-topbar-slot">
    <CommunicationsTopbarSlot
      :search-query="searchQuery"
      :is-sync-busy="isSyncBusy"
      @update:search-query="emit('update:searchQuery', $event)"
      @search="emit('search')"
      @open-account-setup="emit('openAccountSetup')"
      @compose="emit('compose')"
      @sync-now="emit('syncNow')"
    />
  </Teleport>

  <div class="communications-actionbar">
    <div v-if="syncStatusMessage || syncError" class="sync-status-bar">
      <span v-if="syncStatusMessage" class="sync-status-msg">{{ syncStatusMessage }}</span>
      <span v-if="syncError" class="sync-status-error">{{ syncError }}</span>
      <Button variant="ghost" size="sm" @click="emit('clearSyncStatus')">
        <Icon icon="tabler:x" />
      </Button>
    </div>

    <MailSyncSettingsStrip
      :settings="syncSettings"
      :is-loading="isSyncSettingsLoading"
      :is-saving="isSyncSettingsSaving"
      @update="emit('updateSyncSettings', $event)"
    />
    <HealthStrip :health="health" />
    <MailCertificateStrip />
    <MailResourceOverviewStrip
      :subscriptions="subscriptions"
      :top-senders="topSenders"
      :blockers="blockers"
      :is-loading="areResourcesLoading"
      :has-more-subscriptions="hasMoreSubscriptions"
      :is-loading-more-subscriptions="isLoadingMoreSubscriptions"
      :has-more-top-senders="hasMoreTopSenders"
      :is-loading-more-top-senders="isLoadingMoreTopSenders"
      @load-more-subscriptions="emit('loadMoreSubscriptions')"
      @load-more-top-senders="emit('loadMoreTopSenders')"
    />
    <DraftStrip
      :drafts="drafts"
      :has-more="hasMoreDrafts"
      :is-loading-more="isLoadingMoreDrafts"
      @open-draft="emit('openDraft', $event)"
      @delete-draft="emit('deleteDraft', $event)"
      @load-more="emit('loadMoreDrafts')"
    />
  </div>

  <div v-if="actionStatus" class="action-toast">
    <Icon icon="tabler:check-circle" />
    <span>{{ actionStatus }}</span>
  </div>
  <div v-if="lastMessageExport" class="action-toast export-ready">
    <Icon icon="tabler:download" />
    <span>Export ready</span>
    <a :href="messageExportDownloadHref" :download="lastMessageExport.filename">
      {{ lastMessageExport.filename }}
    </a>
  </div>
  <div v-if="actionError" class="action-toast error">
    <Icon icon="tabler:alert-circle" />
    <span>{{ actionError }}</span>
  </div>

  <div v-if="pageError" class="page-error">
    <Icon icon="tabler:alert-triangle" />
    <span>{{ pageError }}</span>
    <Button variant="ghost" size="sm" @click="emit('clearPageError')">
      <Icon icon="tabler:x" />
    </Button>
  </div>
</template>

<style scoped>
.communications-actionbar {
  flex-shrink: 0;
}

.sync-status-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem;
  background: var(--hh-bg-info-light, #eff6ff);
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.sync-status-msg {
  flex: 1;
  color: var(--hh-accent, #3b82f6);
}

.sync-status-error {
  flex: 1;
  color: var(--hh-text-error, #ef4444);
}

.action-toast,
.page-error {
  position: fixed;
  bottom: 1rem;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  font-size: 0.8125rem;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  z-index: 50;
}

.action-toast {
  background: var(--hh-bg-success-light, #f0fdf4);
  color: var(--hh-text-success, #16a34a);
  animation: toast-in 0.2s ease-out;
}

.export-ready {
  bottom: 3.75rem;
}

.export-ready a {
  color: inherit;
  font-weight: 600;
  text-decoration: underline;
  text-underline-offset: 2px;
}

.action-toast.error,
.page-error {
  background: var(--hh-bg-error-light, #fef2f2);
  color: var(--hh-text-error, #ef4444);
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

</style>
