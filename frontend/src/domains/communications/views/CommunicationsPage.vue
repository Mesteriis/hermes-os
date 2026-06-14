<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'

// Components
import CommunicationsConversationList from '../components/CommunicationsConversationList.vue'
import CommunicationsContextInspector from '../components/CommunicationsContextInspector.vue'
import CommunicationsContextRail from '../components/CommunicationsContextRail.vue'
import MailViewer from '../components/MailViewer.vue'
import MailList from '../components/MailList.vue'
import DraftStrip from '../components/DraftStrip.vue'
import HealthStrip from '../components/HealthStrip.vue'
import ComposeDrawer from '../components/ComposeDrawer.vue'
import AccountSetupModal from '../components/AccountSetupModal.vue'
import CommunicationsEmptyPage from './CommunicationsEmptyPage.vue'

// Store
import { useCommunicationsStore } from '../stores/communications'

// TanStack Query
import {
  useMailListQuery,
  useMessageQuery,
  useStateCountsQuery,
  useSyncStatusesQuery,
  useDraftsQuery,
  useMailboxHealthQuery,
  useConversationsQuery,
  useSendMailMutation,
  useSaveDraftMutation,
  useDeleteDraftMutation
} from '../queries/useCommunicationsQuery'

// API
import {
  transitionMessageWorkflowState,
  fetchMailMessage,
  fetchMailSyncStatus,
  fetchMailSyncSettings,
  runMailSyncNow,
  runMailFullResync,
  fetchMessageStateCounts,
  toggleMessagePin,
  toggleMessageImportant,
  toggleMessageMute,
  exportMessage,
  analyzeMessage,
  detectMessageLanguage,
  translateMessage,
  generateAiReply,
  extractMessageTasks,
  extractMessageNotes,
  fetchMessageExplain,
  fetchMessageSmartCc,
  fetchMessageAuth,
  fetchMessageSignature,
  fetchTopSenders,
  fetchMailboxHealth,
  fetchThreads,
  trashMessage,
  restoreMessage
} from '../api/communications'

// Types
import type {
  CommunicationMessageSummary,
  MailMessageDetailResponse,
  MailMessageInsight,
  WorkflowState,
  LocalMessageState,
  CommunicationSectionId,
  MessageContextTab,
  MailSyncStatus,
  MailboxHealth,
  EmailDraft,
  MessageExplainResponse,
  SmartCcResponse,
  MessageAuthCheckResponse,
  SignatureDetection,
  LanguageDetection,
  AiReplyResponse,
  TranslationResponse
} from '../types/communications'

const { t } = useI18n()
const store = useCommunicationsStore()

// --- State ---
const showAccountSetup = ref(false)
const inspectorVisible = ref(true)
const composeDrawerOpen = ref(false)
const isAccountSetupOpen = ref(false)

// --- TanStack Query ---

// Mail list
const {
  data: mailListData,
  isLoading: isMailListLoading,
  refetch: refetchMailList
} = useMailListQuery(
  store.selectedMailAccountId || undefined,
  store.mailStateFilter as WorkflowState | undefined,
  undefined,
  store.messageSearchQuery || undefined
)

// Message detail
const {
  data: messageDetailData,
  isLoading: isMessageDetailLoading,
  refetch: refetchMessageDetail
} = useMessageQuery(store.selectedCommunicationMessageId || null)

// State counts
const {
  data: stateCountsData,
  refetch: refetchStateCounts
} = useStateCountsQuery(store.selectedMailAccountId || undefined)

// Sync statuses
const {
  data: syncStatusesData,
  refetch: refetchSyncStatuses
} = useSyncStatusesQuery()

// Drafts
const {
  data: draftsData,
  refetch: refetchDrafts
} = useDraftsQuery(store.selectedMailAccountId || undefined)

// Mailbox health
const {
  data: mailboxHealthData,
  refetch: refetchMailboxHealth
} = useMailboxHealthQuery(store.selectedMailAccountId || undefined)

// Conversations/threads
const {
  data: conversationsData,
  refetch: refetchConversations
} = useConversationsQuery(store.selectedMailAccountId || undefined)

// --- Mutations ---
const sendMailMutation = useSendMailMutation()
const saveDraftMutation = useSaveDraftMutation()
const deleteDraftMutation = useDeleteDraftMutation()

// --- Computed ---
const mailList = computed(() => mailListData.value ?? [])
const messageDetail = computed(() => messageDetailData.value ?? null)
const stateCounts = computed(() => stateCountsData.value ?? [])
const syncStatuses = computed(() => syncStatusesData.value ?? [])
const drafts = computed(() => draftsData.value ?? [])
const mailboxHealth = computed(() => mailboxHealthData.value ?? null)
const conversations = computed(() => conversationsData.value ?? [])

const isViewerVisible = computed(() =>
  store.selectedConversationIndex >= 0 &&
  store.selectedCommunicationMessageId !== ''
)

// --- Watchers: sync data into store ---
watch(mailListData, (items) => {
  store.setMessages(items ?? [])
})

watch(messageDetailData, (detail) => {
  store.setMessageDetail(detail ?? null)
})

watch(stateCountsData, (counts) => {
  store.setStateCounts(counts ?? [])
})

watch(syncStatusesData, (statuses) => {
  store.setMailSyncStatuses(statuses ?? [])
})

watch(draftsData, (items) => {
  store.setDrafts(items ?? [])
})

watch(mailboxHealthData, (health) => {
  store.setMailboxHealth(health ?? null)
})

// --- Message interaction handlers ---

async function handleSelectMessage(index: number) {
  store.selectMessage(index)
  store.setActiveMessageContextTab('message')
  store.setMailMessageInsight(null)
}

async function handleReply() {
  if (!store.selectedCommunication) return
  store.openCompose({
    mode: 'reply',
    draftId: `draft-${Date.now()}`,
    accountId: store.selectedCommunication.account_id || store.selectedMailAccountId,
    toText: store.selectedCommunication.sender,
    ccText: '',
    bccText: '',
    subject: store.selectedCommunication.subject.startsWith('Re:')
      ? store.selectedCommunication.subject
      : `Re: ${store.selectedCommunication.subject}`,
    body: '',
    inReplyTo: store.selectedCommunication.provider_record_id || null
  })
}

async function handleForward() {
  if (!store.selectedCommunication) return
  store.openCompose({
    mode: 'forward',
    draftId: `draft-${Date.now()}`,
    accountId: store.selectedCommunication.account_id || store.selectedMailAccountId,
    toText: '',
    ccText: '',
    bccText: '',
    subject: store.selectedCommunication.subject.startsWith('Fwd:')
      ? store.selectedCommunication.subject
      : `Fwd: ${store.selectedCommunication.subject}`,
    body: `\n\nForwarded message:\nFrom: ${store.selectedCommunication.sender}\nSubject: ${store.selectedCommunication.subject}\n\n${store.selectedCommunication.body_text_preview}`,
    inReplyTo: null
  })
}

async function handleNewMessage() {
  store.openCompose({
    mode: 'compose',
    draftId: `draft-${Date.now()}`,
    accountId: store.selectedMailAccountId || '',
    toText: '',
    ccText: '',
    bccText: '',
    subject: '',
    body: '',
    inReplyTo: null
  })
}

async function handleTogglePin() {
  if (!store.selectedCommunicationMessageId) return
  store.setIsMailActionRunning(true)
  try {
    const result = await toggleMessagePin(store.selectedCommunicationMessageId)
    store.setMailActionStatus(result.pinned ? 'Pinned' : 'Unpinned')
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Toggle pin failed')
  } finally {
    store.setIsMailActionRunning(false)
  }
}

async function handleToggleImportant() {
  if (!store.selectedCommunicationMessageId) return
  store.setIsMailActionRunning(true)
  try {
    const result = await toggleMessageImportant(store.selectedCommunicationMessageId)
    store.setMailActionStatus(result.important ? 'Marked important' : 'Unmarked important')
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Toggle important failed')
  } finally {
    store.setIsMailActionRunning(false)
  }
}

async function handleMute() {
  if (!store.selectedCommunicationMessageId) return
  store.setIsMailActionRunning(true)
  try {
    await toggleMessageMute(store.selectedCommunicationMessageId)
    store.setMailActionStatus('Muted')
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Mute failed')
  } finally {
    store.setIsMailActionRunning(false)
  }
}

async function handleExportMd() {
  if (!store.selectedCommunicationMessageId) return
  store.setIsMailActionRunning(true)
  try {
    const result = await exportMessage(store.selectedCommunicationMessageId, 'md')
    store.setMailActionStatus('Exported')
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Export failed')
  } finally {
    store.setIsMailActionRunning(false)
  }
}

async function handleAnalyze() {
  if (!store.selectedCommunicationMessageId) return
  store.setIsMailActionRunning(true)
  try {
    const result = await analyzeMessage(store.selectedCommunicationMessageId)
    // Refresh detail to get updated AI fields
    await refetchMessageDetail()
    store.setMailActionStatus('Analyzed')
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Analysis failed')
  } finally {
    store.setIsMailActionRunning(false)
  }
}

async function handleTranslate() {
  if (!store.selectedCommunicationMessageId) return
  store.setIsMailActionRunning(true)
  try {
    const result = await translateMessage(store.selectedCommunicationMessageId, 'en')
    const existing = store.mailMessageInsight
    store.setMailMessageInsight({
      ...(existing ?? {
        messageId: store.selectedCommunicationMessageId,
        explain: null,
        smartCc: null,
        auth: null,
        signature: null,
        language: null,
        aiReply: null,
        tasks: [],
        notes: [],
        translation: null
      }),
      translation: result
    })
    store.setMailActionStatus('Translated')
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Translation failed')
  } finally {
    store.setIsMailActionRunning(false)
  }
}

async function handleCreateTask() {
  if (!store.selectedCommunicationMessageId) return
  store.setIsMailActionRunning(true)
  try {
    const result = await extractMessageTasks(store.selectedCommunicationMessageId)
    const existing = store.mailMessageInsight
    store.setMailMessageInsight({
      ...(existing ?? {
        messageId: store.selectedCommunicationMessageId,
        explain: null,
        smartCc: null,
        auth: null,
        signature: null,
        language: null,
        aiReply: null,
        tasks: [],
        notes: [],
        translation: null
      }),
      tasks: result.tasks
    })
    store.setMailActionStatus(`Extracted ${result.tasks.length} tasks`)
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Task extraction failed')
  } finally {
    store.setIsMailActionRunning(false)
  }
}

async function handleCreateNote() {
  if (!store.selectedCommunicationMessageId) return
  store.setIsMailActionRunning(true)
  try {
    const result = await extractMessageNotes(store.selectedCommunicationMessageId)
    const existing = store.mailMessageInsight
    store.setMailMessageInsight({
      ...(existing ?? {
        messageId: store.selectedCommunicationMessageId,
        explain: null,
        smartCc: null,
        auth: null,
        signature: null,
        language: null,
        aiReply: null,
        tasks: [],
        notes: [],
        translation: null
      }),
      notes: result.notes
    })
    store.setMailActionStatus(`Extracted ${result.notes.length} notes`)
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Note extraction failed')
  } finally {
    store.setIsMailActionRunning(false)
  }
}

// --- Section selection ---

const sectionTabs: { id: CommunicationSectionId; label: string; icon: string }[] = [
  { id: 'unified', label: 'Unified', icon: 'tabler:inbox' },
  { id: 'inbox', label: 'Inbox', icon: 'tabler:mail' },
  { id: 'needs_reply', label: 'Need Reply', icon: 'tabler:message-reply' },
  { id: 'waiting', label: 'Waiting', icon: 'tabler:clock' },
  { id: 'done', label: 'Done', icon: 'tabler:check' },
  { id: 'archived', label: 'Archived', icon: 'tabler:archive' }
]

function sectionCount(sectionId: CommunicationSectionId): number {
  const countItem = stateCounts.value.find(c => c.state === sectionId)
  return countItem?.count ?? 0
}

function selectSection(sectionId: CommunicationSectionId) {
  store.setStateFilter(sectionId as WorkflowState | '')
  store.selectMessage(-1)
  store.setMessageDetail(null)
  store.setMailMessageInsight(null)
  refetchMailList()
  refetchStateCounts()
}

// --- Open draft handlers ---

function handleOpenDraft(draft: EmailDraft) {
  store.openCompose({
    mode: 'compose',
    draftId: draft.draft_id,
    accountId: draft.account_id,
    toText: draft.to_recipients.join(', '),
    ccText: draft.cc_recipients.join(', '),
    bccText: draft.bcc_recipients.join(', '),
    subject: draft.subject,
    body: draft.body_text,
    inReplyTo: draft.in_reply_to
  })
}

async function handleDeleteDraft(draftId: string) {
  try {
    await deleteDraftMutation.mutateAsync(draftId)
    refetchDrafts()
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Delete draft failed')
  }
}

// --- Sync handlers ---

async function handleSyncNow() {
  const accountId = store.selectedMailAccountId
  if (!accountId) return
  store.setIsMailSyncBusy(true)
  store.setMailSyncStatusMessage('Syncing...')
  try {
    await runMailSyncNow(accountId)
    store.setMailSyncStatusMessage('Sync completed')
    await Promise.all([
      refetchMailList(),
      refetchStateCounts(),
      refetchSyncStatuses(),
      refetchMailboxHealth()
    ])
  } catch (e) {
    store.setMailSyncError(e instanceof Error ? e.message : 'Sync failed')
  } finally {
    store.setIsMailSyncBusy(false)
  }
}

async function handleFullResync() {
  const accountId = store.selectedMailAccountId
  if (!accountId) return
  store.setIsMailSyncBusy(true)
  store.setMailSyncStatusMessage('Full resync...')
  try {
    await runMailFullResync(accountId)
    store.setMailSyncStatusMessage('Full resync completed')
    await Promise.all([
      refetchMailList(),
      refetchStateCounts(),
      refetchSyncStatuses(),
      refetchMailboxHealth()
    ])
  } catch (e) {
    store.setMailSyncError(e instanceof Error ? e.message : 'Full resync failed')
  } finally {
    store.setIsMailSyncBusy(false)
  }
}

// --- Compose drawer management ---

watch(() => store.isComposeOpen, (open) => {
  composeDrawerOpen.value = open
})

function handleCloseCompose() {
  store.closeCompose()
}

// --- Initial load ---

async function loadInitialData() {
  // Sync statuses and health load on mount
  await Promise.all([
    refetchSyncStatuses(),
    refetchMailboxHealth(),
    refetchStateCounts()
  ])
}

onMounted(() => {
  loadInitialData()
})
</script>

<template>
  <section class="communications-page">
    <!-- Header -->
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">Communications</h1>
        <div class="search-box">
          <Icon icon="tabler:search" class="search-icon" />
          <input
            type="text"
            placeholder="Search messages..."
            :value="store.messageSearchQuery"
            @input="store.setMessageSearchQuery(($event.target as HTMLInputElement).value)"
            @keyup.enter="void refetchMailList()"
          />
        </div>
      </div>
      <div class="header-right">
        <Button variant="ghost" size="sm" @click="isAccountSetupOpen = true">
          <Icon icon="tabler:plus" /> Account
        </Button>
        <Button variant="ghost" size="sm" @click="handleNewMessage">
          <Icon icon="tabler:edit" /> Compose
        </Button>
        <Button variant="ghost" size="sm" @click="handleSyncNow" :disabled="store.isMailSyncBusy">
          <Icon icon="tabler:refresh" :class="store.isMailSyncBusy ? 'spin-icon' : ''" />
        </Button>
      </div>
    </div>

    <!-- Section tabs -->
    <div class="section-tabs">
      <button
        v-for="tab in sectionTabs"
        :key="tab.id"
        class="section-tab"
        :class="{ active: store.mailStateFilter === tab.id || (store.mailStateFilter === '' && tab.id === 'unified') }"
        @click="selectSection(tab.id)"
      >
        <Icon :icon="tab.icon" />
        <span>{{ tab.label }}</span>
        <span v-if="sectionCount(tab.id) > 0" class="section-count">{{ sectionCount(tab.id) }}</span>
      </button>
    </div>

    <!-- Status bar -->
    <div v-if="store.mailSyncStatusMessage || store.mailSyncError" class="sync-status-bar">
      <span v-if="store.mailSyncStatusMessage" class="sync-status-msg">{{ store.mailSyncStatusMessage }}</span>
      <span v-if="store.mailSyncError" class="sync-status-error">{{ store.mailSyncError }}</span>
      <Button variant="ghost" size="sm" @click="store.setMailSyncStatusMessage(''); store.setMailSyncError('')">
        <Icon icon="tabler:x" />
      </Button>
    </div>

    <!-- Health Strip -->
    <HealthStrip :health="mailboxHealth" />

    <!-- Draft Strip -->
    <DraftStrip
      :drafts="drafts"
      @open-draft="handleOpenDraft"
      @delete-draft="handleDeleteDraft"
    />

    <!-- Main 3-pane grid -->
    <div class="mail-grid">
      <!-- Left pane: Conversation list -->
      <nav class="grid-pane pane-conversations">
        <CommunicationsConversationList
          :messages="mailList"
          :selected-index="store.selectedConversationIndex"
          :navigator-mode="store.communicationsNavigatorMode"
          @select="handleSelectMessage"
          @update:navigator-mode="store.setNavigatorMode"
        />
      </nav>

      <!-- Center pane: Message viewer -->
      <main class="grid-pane pane-viewer">
        <MailViewer
          :detail="messageDetail"
          :insight="store.mailMessageInsight"
          :active-tab="store.activeMessageContextTab"
          @update:active-tab="store.setActiveMessageContextTab"
          @reply="handleReply"
          @create-task="handleCreateTask"
          @create-note="handleCreateNote"
          @translate="handleTranslate"
          @analyze="handleAnalyze"
          @toggle-pin="handleTogglePin"
          @toggle-important="handleToggleImportant"
          @mute="handleMute"
          @export-md="handleExportMd"
          @open-compose="handleNewMessage"
        />
      </main>

      <!-- Right pane: Context inspector (optional) -->
      <aside v-if="inspectorVisible && messageDetail" class="grid-pane pane-inspector">
        <CommunicationsContextInspector
          :detail="messageDetail"
          :inspector-mode="store.communicationsInspectorMode"
          @update:inspector-mode="store.setInspectorMode"
        />
      </aside>
    </div>

    <!-- Action status indicator -->
    <div v-if="store.mailActionStatus" class="action-toast">
      <Icon icon="tabler:check-circle" />
      <span>{{ store.mailActionStatus }}</span>
    </div>
    <div v-if="store.mailActionError" class="action-toast error">
      <Icon icon="tabler:alert-circle" />
      <span>{{ store.mailActionError }}</span>
    </div>

    <!-- Error message -->
    <div v-if="store.communicationsError" class="page-error">
      <Icon icon="tabler:alert-triangle" />
      <span>{{ store.communicationsError }}</span>
      <Button variant="ghost" size="sm" @click="store.setCommunicationsError('')">
        <Icon icon="tabler:x" />
      </Button>
    </div>

    <!-- Compose Drawer (overlay) -->
    <ComposeDrawer v-if="composeDrawerOpen" />

    <!-- Account Setup Modal -->
    <AccountSetupModal v-if="isAccountSetupOpen" @close="isAccountSetupOpen = false" />
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

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  gap: 1rem;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 1rem;
  flex: 1;
}

.page-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
  margin: 0;
  white-space: nowrap;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  background: var(--hh-bg-secondary, #f9fafb);
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  padding: 0.3125rem 0.625rem;
  flex: 1;
  max-width: 320px;
}

.search-icon {
  width: 14px;
  height: 14px;
  color: var(--hh-text-tertiary, #9ca3af);
  flex-shrink: 0;
}

.search-box input {
  border: none;
  background: transparent;
  flex: 1;
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  outline: none;
  min-width: 0;
}

.search-box input::placeholder {
  color: var(--hh-text-tertiary, #9ca3af);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  flex-shrink: 0;
}

/* Section tabs */
.section-tabs {
  display: flex;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  overflow-x: auto;
  padding: 0 0.5rem;
  gap: 0;
}

.section-tab {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.75rem;
  border: none;
  background: transparent;
  font-size: 0.8125rem;
  color: var(--hh-text-secondary, #6b7280);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: color 0.1s, border-color 0.1s;
  white-space: nowrap;
}

.section-tab:hover {
  color: var(--hh-text-primary, #1f2937);
  background: var(--hh-bg-hover, #f3f4f6);
}

.section-tab.active {
  color: var(--hh-accent, #3b82f6);
  border-bottom-color: var(--hh-accent, #3b82f6);
  font-weight: 500;
}

.section-count {
  font-size: 0.6875rem;
  background: var(--hh-bg-secondary, #f3f4f6);
  color: var(--hh-text-secondary, #6b7280);
  padding: 0.0625rem 0.375rem;
  border-radius: 0.75rem;
  font-weight: 500;
}

.section-tab.active .section-count {
  background: rgba(59, 130, 246, 0.1);
  color: var(--hh-accent, #3b82f6);
}

/* Sync status bar */
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

/* Main grid */
.mail-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 280px 1fr;
  overflow: hidden;
}

.mail-grid:has(.pane-inspector) {
  grid-template-columns: 240px 1fr 280px;
}

.grid-pane {
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.pane-conversations {
  border-right: 1px solid var(--hh-border, #e5e7eb);
  background: var(--hh-bg-primary, #ffffff);
}

.pane-viewer {
  background: var(--hh-bg-primary, #ffffff);
}

.pane-inspector {
  border-left: 1px solid var(--hh-border, #e5e7eb);
  background: var(--hh-bg-primary, #ffffff);
}

/* Action toasts */
.action-toast {
  position: fixed;
  bottom: 1rem;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: var(--hh-bg-success-light, #f0fdf4);
  color: var(--hh-text-success, #16a34a);
  border-radius: 0.5rem;
  font-size: 0.8125rem;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  z-index: 50;
  animation: toast-in 0.2s ease-out;
}

.action-toast.error {
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

/* Page error */
.page-error {
  position: fixed;
  bottom: 1rem;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: var(--hh-bg-error-light, #fef2f2);
  color: var(--hh-text-error, #ef4444);
  border-radius: 0.5rem;
  font-size: 0.8125rem;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  z-index: 50;
}

.spin-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
