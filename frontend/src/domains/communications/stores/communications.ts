import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { format, formatDistanceToNow } from 'date-fns'
import type {
  CommunicationMessageSummary,
  MailMessageDetailResponse,
  WorkflowState,
  LocalMessageState,
  MailSyncStatus,
  MailboxHealth,
  EmailDraft,
  ComposeFormModel,
  ComposeMode,
  MailAccountOption,
  NavigatorMode,
  InspectorMode,
  MessageContextTab,
  MailMessageInsight,
  CommunicationSectionId,
  ProjectItem,
  TaskItem
} from '../types/communications'

const emptyComposeForm: ComposeFormModel = {
  mode: 'compose',
  draftId: '',
  accountId: '',
  toText: '',
  ccText: '',
  bccText: '',
  subject: '',
  body: '',
  inReplyTo: null
}

export const useCommunicationsStore = defineStore('communications-ui', () => {
  // --- Message list state ---
  const communicationMessages = ref<CommunicationMessageSummary[]>([])
  const selectedCommunicationDetail = ref<MailMessageDetailResponse | null>(null)
  const communicationsError = ref('')
  const isCommunicationsLoading = ref(false)
  const selectedConversationIndex = ref(-1)
  const selectedCommunicationMessageId = ref('')

  // --- Filters ---
  const mailStateFilter = ref<WorkflowState | ''>('')
  const mailLocalStateFilter = ref<LocalMessageState>('active')
  const mailStateCounts = ref<{ state: string; count: number }[]>([])
  const isMailStateTransitioning = ref(false)

  // --- AI ---
  const isAiAnswerSubmitting = ref(false)
  const aiAnalysisResult = ref<Record<string, unknown> | null>(null)

  // --- Drafts ---
  const drafts = ref<EmailDraft[]>([])

  // --- Health ---
  const mailboxHealth = ref<MailboxHealth | null>(null)

  // --- Senders ---
  const topSenders = ref<{ sender: string; message_count: number }[]>([])

  // --- Threads ---
  const threads = ref<{ thread_id: string; subject: string; message_count: number }[]>([])

  // --- Resources ---
  const mailResources = ref<Record<string, unknown>>({})
  const mailResourceSummary = ref<Record<string, number>>({})

  // --- Message insight ---
  const mailMessageInsight = ref<MailMessageInsight | null>(null)

  // --- Action status ---
  const isMailActionRunning = ref(false)
  const mailActionStatus = ref('')
  const mailActionError = ref('')
  const lastMessageExport = ref<{ content_type: string; content: string; filename: string } | null>(null)

  // --- Sync ---
  const mailSyncStatuses = ref<MailSyncStatus[]>([])
  const selectedMailSyncSettings = ref<{ account_id: string; sync_enabled: boolean; batch_size: number; poll_interval_seconds: number } | null>(null)
  const lastMailSyncRuns = ref<Record<string, unknown>[]>([])
  const isMailSyncBusy = ref(false)
  const mailSyncStatusMessage = ref('')
  const mailSyncError = ref('')

  // --- Compose ---
  const isComposeOpen = ref(false)
  const composeForm = ref<ComposeFormModel>({ ...emptyComposeForm })
  const selectedMailAccountId = ref('')
  const isSendReviewOpen = ref(false)
  const isSendingMessage = ref(false)
  const composeSendError = ref('')
  const composeStatusMessage = ref('')
  const lastSendResponse = ref<Record<string, unknown> | null>(null)

  // --- UI state ---
  const messageSearchQuery = ref('')
  const communicationsNavigatorMode = ref<NavigatorMode>('threads')
  const expandedCommunicationContactKey = ref<string | null>(null)
  const communicationsInspectorMode = ref<InspectorMode>('context')
  const activeMessageContextTab = ref<MessageContextTab>('message')
  const communicationProjects = ref<ProjectItem[]>([])
  const communicationTasks = ref<TaskItem[]>([])

  // --- Derived: selected communication ---
  const selectedCommunication = computed(() => {
    const idx = selectedConversationIndex.value
    if (idx >= 0 && idx < communicationMessages.value.length) {
      return communicationMessages.value[idx]
    }
    return null
  })

  // --- Actions ---

  function setMessages(messages: CommunicationMessageSummary[]) {
    communicationMessages.value = messages
  }

  function selectMessage(index: number) {
    selectedConversationIndex.value = index
    if (index >= 0 && index < communicationMessages.value.length) {
      selectedCommunicationMessageId.value = communicationMessages.value[index].message_id
    }
  }

  function setMessageDetail(detail: MailMessageDetailResponse | null) {
    selectedCommunicationDetail.value = detail
  }

  function setCommunicationsError(error: string) {
    communicationsError.value = error
  }

  function setCommunicationsLoading(loading: boolean) {
    isCommunicationsLoading.value = loading
  }

  function setStateFilter(state: WorkflowState | '') {
    mailStateFilter.value = state
  }

  function setLocalStateFilter(state: LocalMessageState) {
    mailLocalStateFilter.value = state
  }

  function setStateCounts(counts: { state: string; count: number }[]) {
    mailStateCounts.value = counts
  }

  function setMailSyncStatuses(statuses: MailSyncStatus[]) {
    mailSyncStatuses.value = statuses
  }

  function setMailSyncStatusMessage(msg: string) {
    mailSyncStatusMessage.value = msg
  }

  function setMailSyncError(err: string) {
    mailSyncError.value = err
  }

  function setIsMailSyncBusy(busy: boolean) {
    isMailSyncBusy.value = busy
  }

  function setDrafts(draftList: EmailDraft[]) {
    drafts.value = draftList
  }

  function setMailboxHealth(health: MailboxHealth | null) {
    mailboxHealth.value = health
  }

  function setTopSenders(senders: { sender: string; message_count: number }[]) {
    topSenders.value = senders
  }

  function setThreads(threadList: { thread_id: string; subject: string; message_count: number }[]) {
    threads.value = threadList
  }

  function setMailMessageInsight(insight: MailMessageInsight | null) {
    mailMessageInsight.value = insight
  }

  function setIsMailActionRunning(running: boolean) {
    isMailActionRunning.value = running
  }

  function setMailActionStatus(status: string) {
    mailActionStatus.value = status
  }

  function setMailActionError(error: string) {
    mailActionError.value = error
  }

  // --- Compose actions ---

  function openCompose(form: ComposeFormModel) {
    composeForm.value = { ...form }
    isComposeOpen.value = true
  }

  function closeCompose() {
    isComposeOpen.value = false
    composeForm.value = { ...emptyComposeForm }
    isSendReviewOpen.value = false
    composeSendError.value = ''
    composeStatusMessage.value = ''
  }

  function updateComposeForm(partial: Partial<ComposeFormModel>) {
    composeForm.value = { ...composeForm.value, ...partial }
  }

  function setComposeStatusMessage(msg: string) {
    composeStatusMessage.value = msg
  }

  function setComposeSendError(err: string) {
    composeSendError.value = err
  }

  function setIsSendingMessage(sending: boolean) {
    isSendingMessage.value = sending
  }

  function openSendReview() {
    isSendReviewOpen.value = true
  }

  function closeSendReview() {
    isSendReviewOpen.value = false
  }

  // --- Search ---

  function setMessageSearchQuery(query: string) {
    messageSearchQuery.value = query
  }

  // --- Navigator ---

  function setNavigatorMode(mode: NavigatorMode) {
    communicationsNavigatorMode.value = mode
  }

  function setExpandedContactKey(key: string | null) {
    expandedCommunicationContactKey.value = key
  }

  // --- Inspector ---

  function setInspectorMode(mode: InspectorMode) {
    communicationsInspectorMode.value = mode
  }

  function setActiveMessageContextTab(tab: MessageContextTab) {
    activeMessageContextTab.value = tab
  }

  // --- Projects & Tasks ---

  function setCommunicationProjects(projects: ProjectItem[]) {
    communicationProjects.value = projects
  }

  function setCommunicationTasks(tasks: TaskItem[]) {
    communicationTasks.value = tasks
  }

  // --- Account ---

  function setSelectedMailAccountId(accountId: string) {
    selectedMailAccountId.value = accountId
  }

  return {
    // State
    communicationMessages, selectedCommunicationDetail, communicationsError, isCommunicationsLoading,
    selectedConversationIndex, selectedCommunicationMessageId,
    mailStateFilter, mailLocalStateFilter, mailStateCounts, isMailStateTransitioning,
    isAiAnswerSubmitting, aiAnalysisResult,
    drafts, mailboxHealth, topSenders, threads,
    mailResources, mailResourceSummary, mailMessageInsight,
    isMailActionRunning, mailActionStatus, mailActionError, lastMessageExport,
    mailSyncStatuses, selectedMailSyncSettings, lastMailSyncRuns,
    isMailSyncBusy, mailSyncStatusMessage, mailSyncError,
    isComposeOpen, composeForm, selectedMailAccountId,
    isSendReviewOpen, isSendingMessage, composeSendError, composeStatusMessage, lastSendResponse,
    messageSearchQuery, communicationsNavigatorMode, expandedCommunicationContactKey,
    communicationsInspectorMode, activeMessageContextTab,
    communicationProjects, communicationTasks,
    // Computed
    selectedCommunication,
    // Setters
    setMessages, selectMessage, setMessageDetail,
    setCommunicationsError, setCommunicationsLoading,
    setStateFilter, setLocalStateFilter, setStateCounts,
    setMailSyncStatuses, setMailSyncStatusMessage, setMailSyncError, setIsMailSyncBusy,
    setDrafts, setMailboxHealth, setTopSenders, setThreads,
    setMailMessageInsight, setIsMailActionRunning, setMailActionStatus, setMailActionError,
    openCompose, closeCompose, updateComposeForm,
    setComposeStatusMessage, setComposeSendError, setIsSendingMessage,
    openSendReview, closeSendReview,
    setMessageSearchQuery,
    setNavigatorMode, setExpandedContactKey,
    setInspectorMode, setActiveMessageContextTab,
    setCommunicationProjects, setCommunicationTasks,
    setSelectedMailAccountId
  }
})

// --- Utility functions ---

export function messageTime(dateStr: string | null): string {
  if (!dateStr) return ''
  const d = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))
  if (diffDays === 0) return format(d, 'HH:mm')
  if (diffDays === 1) return 'Yesterday'
  if (diffDays < 7) return format(d, 'EEEE')
  return format(d, 'MMM d')
}

export function communicationChannelIcon(channelKind: string): string {
  switch (channelKind) {
    case 'telegram': return 'tabler:brand-telegram'
    case 'whatsapp': return 'tabler:brand-whatsapp'
    case 'slack': return 'tabler:brand-slack'
    case 'signal': return 'tabler:brand-signal'
    default: return 'tabler:mail'
  }
}

export function communicationChannelLabel(channelKind: string): string {
  switch (channelKind) {
    case 'telegram': return 'Telegram'
    case 'whatsapp': return 'WhatsApp'
    case 'slack': return 'Slack'
    case 'signal': return 'Signal'
    default: return 'Email'
  }
}

export function attachmentIcon(contentType: string): string {
  if (contentType.startsWith('image/')) return 'tabler:photo'
  if (contentType.startsWith('video/')) return 'tabler:video'
  if (contentType.startsWith('audio/')) return 'tabler:music'
  if (contentType.includes('pdf')) return 'tabler:file-text'
  if (contentType.includes('spreadsheet') || contentType.includes('excel') || contentType.includes('csv')) return 'tabler:table'
  if (contentType.includes('word') || contentType.includes('document')) return 'tabler:file-description'
  if (contentType.includes('zip') || contentType.includes('rar') || contentType.includes('tar')) return 'tabler:archive'
  return 'tabler:file'
}

export function communicationSectionWorkflowState(sectionId: string): WorkflowState | '' | null {
  switch (sectionId) {
    case 'unified': return ''
    case 'inbox': return 'new'
    case 'needs_reply': return 'needs_action'
    case 'waiting': return 'waiting'
    case 'done': return 'done'
    case 'archived': return 'archived'
    default: return null
  }
}

export function senderLabel(sender: string): string {
  const match = sender.match(/^"?(.+?)"?\s*<(.+?)>$/)
  if (match) return match[1].trim()
  return sender
}

export function senderEmail(sender: string): string {
  const match = sender.match(/<(.+?)>$/)
  if (match) return match[1]
  const atMatch = sender.match(/(\S+@\S+)/)
  if (atMatch) return atMatch[1]
  return sender
}

export function conversationPreview(msg: CommunicationMessageSummary): string {
  if (msg.ai_summary) return msg.ai_summary
  if (msg.body_text_preview) return msg.body_text_preview
  return msg.subject
}

export function formatRelativeTime(dateStr: string): string {
  const d = new Date(dateStr)
  return formatDistanceToNow(d, { addSuffix: true })
}
