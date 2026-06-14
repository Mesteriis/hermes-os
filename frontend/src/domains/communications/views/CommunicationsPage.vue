<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import AccountSetupModal from '../components/AccountSetupModal.vue'
import CommunicationsActionBar from '../components/CommunicationsActionBar.vue'
import CommunicationsDetailPane from '../components/CommunicationsDetailPane.vue'
import CommunicationsListPane from '../components/CommunicationsListPane.vue'
import CommunicationsRailPane from '../components/CommunicationsRailPane.vue'
import CommunicationsWorkbench from '../components/CommunicationsWorkbench.vue'
import ComposeDrawer from '../components/ComposeDrawer.vue'
import {
  analyzeMessage,
  exportMessage,
  extractMessageNotes,
  extractMessageTasks,
  runMailFullResync,
  runMailSyncNow,
  toggleMessageImportant,
  toggleMessageMute,
  toggleMessagePin,
  translateMessage
} from '../api/communications'
import {
  useDeleteDraftMutation,
  useDraftsQuery,
  useConversationsQuery,
  useMailboxHealthQuery,
  useMailListQuery,
  useMessageQuery,
  useStateCountsQuery,
  useSyncStatusesQuery
} from '../queries/useCommunicationsQuery'
import {
  communicationSectionWorkflowState,
  communicationWorkflowStateSectionId,
  useCommunicationsStore
} from '../stores/communications'
import type {
  CommunicationSectionId,
  EmailDraft,
  MailMessageInsight
} from '../types/communications'

const store = useCommunicationsStore()
const isAccountSetupOpen = ref(false)
const inspectorVisible = ref(true)

const {
  data: mailListData,
  error: mailListError,
  isLoading: isMailListLoading,
  refetch: refetchMailList
} = useMailListQuery(
  store.selectedMailAccountId || undefined,
  store.mailStateFilter || undefined,
  undefined,
  store.messageSearchQuery || undefined
)

const {
  data: messageDetailData,
  refetch: refetchMessageDetail
} = useMessageQuery(store.selectedCommunicationMessageId || null)

const {
  data: stateCountsData,
  refetch: refetchStateCounts
} = useStateCountsQuery(store.selectedMailAccountId || undefined)

const {
  data: syncStatusesData,
  refetch: refetchSyncStatuses
} = useSyncStatusesQuery()

const {
  data: draftsData,
  refetch: refetchDrafts
} = useDraftsQuery(store.selectedMailAccountId || undefined)

const {
  data: mailboxHealthData,
  refetch: refetchMailboxHealth
} = useMailboxHealthQuery(store.selectedMailAccountId || undefined)

const { data: conversationsData } = useConversationsQuery(store.selectedMailAccountId || undefined)
const deleteDraftMutation = useDeleteDraftMutation()

const mailList = computed(() => mailListData.value ?? [])
const messageDetail = computed(() => messageDetailData.value ?? null)
const stateCounts = computed(() => stateCountsData.value ?? [])
const drafts = computed(() => draftsData.value ?? [])
const mailboxHealth = computed(() => mailboxHealthData.value ?? null)
const conversations = computed(() => conversationsData.value ?? [])
const hasRail = computed(() => inspectorVisible.value && messageDetail.value !== null)
const activeSectionId = computed<CommunicationSectionId>(() =>
  communicationWorkflowStateSectionId(store.mailStateFilter)
)
const mailListErrorMessage = computed(() => {
  if (!mailListError.value) return ''
  return mailListError.value instanceof Error ? mailListError.value.message : 'Failed to load messages'
})

const sectionTabs: { id: CommunicationSectionId; label: string; icon: string }[] = [
  { id: 'unified', label: 'Unified', icon: 'tabler:inbox' },
  { id: 'inbox', label: 'Inbox', icon: 'tabler:mail' },
  { id: 'needs_reply', label: 'Need Reply', icon: 'tabler:message-reply' },
  { id: 'waiting', label: 'Waiting', icon: 'tabler:clock' },
  { id: 'done', label: 'Done', icon: 'tabler:check' },
  { id: 'archived', label: 'Archived', icon: 'tabler:archive' }
]

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

watch(conversationsData, (threads) => {
  store.setThreads(
    (threads ?? []).map((thread) => ({
      thread_id: thread.thread_id,
      subject: thread.subject,
      message_count: thread.message_count
    }))
  )
})

function emptyInsight(): MailMessageInsight {
  return {
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
  }
}

function handleSearchQueryUpdate(query: string) {
  store.setMessageSearchQuery(query)
}

function handleSearch() {
  void refetchMailList()
}

function handleSelectMessage(index: number) {
  store.selectMessage(index)
  store.setActiveMessageContextTab('message')
  store.setMailMessageInsight(null)
}

function handleReply() {
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

function handleNewMessage() {
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

async function runSelectedMessageAction(action: () => Promise<string>) {
  if (!store.selectedCommunicationMessageId) return
  store.setIsMailActionRunning(true)
  try {
    store.setMailActionStatus(await action())
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Message action failed')
  } finally {
    store.setIsMailActionRunning(false)
  }
}

async function handleTogglePin() {
  await runSelectedMessageAction(async () => {
    const result = await toggleMessagePin(store.selectedCommunicationMessageId)
    return result.pinned ? 'Pinned' : 'Unpinned'
  })
}

async function handleToggleImportant() {
  await runSelectedMessageAction(async () => {
    const result = await toggleMessageImportant(store.selectedCommunicationMessageId)
    return result.important ? 'Marked important' : 'Unmarked important'
  })
}

async function handleMute() {
  await runSelectedMessageAction(async () => {
    await toggleMessageMute(store.selectedCommunicationMessageId)
    return 'Muted'
  })
}

async function handleExportMd() {
  await runSelectedMessageAction(async () => {
    await exportMessage(store.selectedCommunicationMessageId, 'md')
    return 'Exported'
  })
}

async function handleAnalyze() {
  await runSelectedMessageAction(async () => {
    await analyzeMessage(store.selectedCommunicationMessageId)
    await refetchMessageDetail()
    return 'Analyzed'
  })
}

async function handleTranslate() {
  await runSelectedMessageAction(async () => {
    const result = await translateMessage(store.selectedCommunicationMessageId, 'en')
    store.setMailMessageInsight({
      ...(store.mailMessageInsight ?? emptyInsight()),
      translation: result
    })
    return 'Translated'
  })
}

async function handleCreateTask() {
  await runSelectedMessageAction(async () => {
    const result = await extractMessageTasks(store.selectedCommunicationMessageId)
    store.setMailMessageInsight({
      ...(store.mailMessageInsight ?? emptyInsight()),
      tasks: result.tasks
    })
    return `Extracted ${result.tasks.length} tasks`
  })
}

async function handleCreateNote() {
  await runSelectedMessageAction(async () => {
    const result = await extractMessageNotes(store.selectedCommunicationMessageId)
    store.setMailMessageInsight({
      ...(store.mailMessageInsight ?? emptyInsight()),
      notes: result.notes
    })
    return `Extracted ${result.notes.length} notes`
  })
}

function selectSection(sectionId: CommunicationSectionId) {
  const workflowState = communicationSectionWorkflowState(sectionId)
  if (workflowState === null) return
  store.setStateFilter(workflowState)
  store.selectMessage(-1)
  store.setMessageDetail(null)
  store.setMailMessageInsight(null)
  void refetchMailList()
  void refetchStateCounts()
}

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
    await refetchDrafts()
  } catch (e) {
    store.setMailActionError(e instanceof Error ? e.message : 'Delete draft failed')
  }
}

async function handleSyncNow() {
  const accountId = store.selectedMailAccountId
  if (!accountId) return
  store.setIsMailSyncBusy(true)
  store.setMailSyncStatusMessage('Syncing...')
  try {
    await runMailSyncNow(accountId)
    store.setMailSyncStatusMessage('Sync completed')
    await Promise.all([refetchMailList(), refetchStateCounts(), refetchSyncStatuses(), refetchMailboxHealth()])
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
    await Promise.all([refetchMailList(), refetchStateCounts(), refetchSyncStatuses(), refetchMailboxHealth()])
  } catch (e) {
    store.setMailSyncError(e instanceof Error ? e.message : 'Full resync failed')
  } finally {
    store.setIsMailSyncBusy(false)
  }
}

function clearSyncStatus() {
  store.setMailSyncStatusMessage('')
  store.setMailSyncError('')
}

async function loadInitialData() {
  await Promise.all([refetchSyncStatuses(), refetchMailboxHealth(), refetchStateCounts()])
}

onMounted(() => {
  void loadInitialData()
})
</script>

<template>
  <section class="communications-page">
    <CommunicationsActionBar
      :search-query="store.messageSearchQuery"
      :section-tabs="sectionTabs"
      :active-section-id="activeSectionId"
      :state-counts="stateCounts"
      :is-sync-busy="store.isMailSyncBusy"
      :sync-status-message="store.mailSyncStatusMessage"
      :sync-error="store.mailSyncError"
      :health="mailboxHealth"
      :drafts="drafts"
      :action-status="store.mailActionStatus"
      :action-error="store.mailActionError"
      :page-error="store.communicationsError"
      @update:search-query="handleSearchQueryUpdate"
      @search="handleSearch"
      @open-account-setup="isAccountSetupOpen = true"
      @compose="handleNewMessage"
      @sync-now="handleSyncNow"
      @clear-sync-status="clearSyncStatus"
      @select-section="selectSection"
      @open-draft="handleOpenDraft"
      @delete-draft="handleDeleteDraft"
      @clear-page-error="store.setCommunicationsError('')"
    />

    <CommunicationsWorkbench :is-loading="isMailListLoading" :has-error="Boolean(mailListError)" :has-rail="hasRail">
      <template #list>
        <CommunicationsListPane
          :messages="mailList"
          :selected-index="store.selectedConversationIndex"
          :navigator-mode="store.communicationsNavigatorMode"
          :is-loading="isMailListLoading"
          :error-message="mailListErrorMessage"
          @select="handleSelectMessage"
          @update:navigator-mode="store.setNavigatorMode"
        />
      </template>

      <template #detail>
        <CommunicationsDetailPane
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
</style>
