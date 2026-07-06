import {
  useAddMessageLabelMutation,
  useAnalyzeMessageMutation,
  useExportMessageMutation,
  useDeleteMessageFromProviderMutation,
  useMarkMessageReadMutation,
  useMarkMessageSpamMutation,
  useMarkMessageUnreadMutation,
  useExtractMessageNotesMutation,
  useExtractMessageTasksMutation,
  useGenerateAiReplyMutation,
  useRedirectMessageMutation,
  useRemoveMessageLabelMutation,
  useReviewMessageRecipientsMutation,
  useReviewMessageSecurityMutation,
  useSnoozeMessageMutation,
  useToggleMessageImportantMutation,
  useToggleMessageMuteMutation,
  useToggleMessagePinMutation,
  useTranslateMessageMutation
} from '../queries/useCommunicationsQuery'
import { useI18n } from '@/platform/i18n'
import { splitComposeRecipients } from '../forms/composeValidation'
import {
  emptyCommunicationMessageInsight,
  forwardComposeForm,
  newComposeForm,
  replyAllComposeForm,
  replyComposeForm
} from '../helpers/communicationPageModels'
import { useCommunicationActionNotifications } from '../queries/communicationActionNotifications'
import type { useCommunicationsStore } from '../stores/communications'
import type {
  AiReplyResponse,
  CommunicationMessageDetailItem,
  MessageExportFormat,
  TranslationResponse
} from '../types/communications'
import type { BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'

type CommunicationsStore = ReturnType<typeof useCommunicationsStore>
type RefetchHandler = () => Promise<unknown>
type SelectedMessageActionTone = 'success' | 'warning' | 'info'
type SelectedMessageActionOutcome = {
  message: string
  title?: string
  tone?: SelectedMessageActionTone
}
type SelectedMessageActionResult = string | SelectedMessageActionOutcome

type SelectedMessageActionOptions = {
  getMessageDetail: () => CommunicationMessageDetailItem | null
  refetchMessageDetail: RefetchHandler
}

export function useSelectedMessageActions(
  store: CommunicationsStore,
  deps: SelectedMessageActionOptions
) {
  const { locale } = useI18n()
  const notifications = useCommunicationActionNotifications()
  const togglePinMutation = useToggleMessagePinMutation()
  const toggleImportantMutation = useToggleMessageImportantMutation()
  const toggleMuteMutation = useToggleMessageMuteMutation()
  const exportMessageMutation = useExportMessageMutation()
  const markMessageReadMutation = useMarkMessageReadMutation()
  const markMessageSpamMutation = useMarkMessageSpamMutation()
  const markMessageUnreadMutation = useMarkMessageUnreadMutation()
  const deleteMessageFromProviderMutation = useDeleteMessageFromProviderMutation()
  const generateAiReplyMutation = useGenerateAiReplyMutation()
  const reviewSecurityMutation = useReviewMessageSecurityMutation()
  const reviewRecipientsMutation = useReviewMessageRecipientsMutation()
  const redirectMessageMutation = useRedirectMessageMutation()
  const addLabelMutation = useAddMessageLabelMutation()
  const removeLabelMutation = useRemoveMessageLabelMutation()
  const snoozeMessageMutation = useSnoozeMessageMutation()
  const analyzeMessageMutation = useAnalyzeMessageMutation()
  const translateMessageMutation = useTranslateMessageMutation()
  const extractMessageTasksMutation = useExtractMessageTasksMutation()
  const extractMessageNotesMutation = useExtractMessageNotesMutation()

  function handleReply() {
    if (!store.selectedCommunication) return
    store.openCompose(replyComposeForm(store.selectedCommunication, store.selectedMailAccountId, `draft-${Date.now()}`))
  }

  function handleReplyAll() {
    if (!store.selectedCommunication) return
    store.openCompose(replyAllComposeForm(store.selectedCommunication, store.selectedMailAccountId, `draft-${Date.now()}`))
  }

  function handleForwardMessage() {
    if (!store.selectedCommunication) return
    store.openCompose(forwardComposeForm(store.selectedCommunication, store.selectedMailAccountId, `draft-${Date.now()}`))
  }

  async function handleRedirectMessage(recipientsText: string) {
    await runSelectedMessageAction(async (messageId) => {
      const to = splitComposeRecipients(recipientsText)
      if (to.length === 0) {
        throw new Error('Redirect recipient is required')
      }
      const result = await redirectMessageMutation.mutateAsync({
        messageId,
        request: { to, confirmed_provider_write: true }
      })
      return result.status === 'sent' ? 'Redirected' : `Redirect ${result.status}`
    })
  }

  function handleBilingualReplySend(response: BilingualReplyFlowResponse): void {
    const detail = deps.getMessageDetail()
    if (!detail || !response.send_ready) return
    store.openCompose({
      mode: 'reply',
      draftId: `draft-${Date.now()}`,
      accountId: detail.account_id || store.selectedMailAccountId || '',
      toText: detail.sender,
      ccText: '',
      bccText: '',
      subject: response.subject,
      body: response.reply.text,
      bodyHtml: null,
      bodyFormat: 'plain',
      scheduledSendAt: '',
      undoSendSeconds: null,
      inReplyTo: detail.provider_record_id || null
    })
  }

  function handleNewMessage() {
    store.openCompose(newComposeForm(store.selectedMailAccountId || '', `draft-${Date.now()}`))
  }

  async function runSelectedMessageAction(
    action: (messageId: string) => Promise<SelectedMessageActionResult>,
    actionKey = 'selected-message-action'
  ) {
    const messageId = store.selectedCommunicationMessageId
    if (!messageId) return
    const notificationKey = mailActionNotificationKey(actionKey, messageId)
    store.setIsMailActionRunning(true)
    store.setLastMessageExport(null)
    store.setMailActionStatus('')
    store.setMailActionError('')
    try {
      const outcome = selectedMessageActionOutcome(await action(messageId))
      store.setMailActionStatus(outcome.message)
      notifySelectedMessageAction(outcome, messageId, notificationKey)
    } catch (e) {
      const message = selectedMessageActionErrorMessage(e)
      store.setMailActionError(message)
      notifications.error('Mail action failed', message, messageId, notificationKey)
    } finally {
      store.setIsMailActionRunning(false)
    }
  }

  async function handleTogglePin() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await togglePinMutation.mutateAsync(messageId)
      return result.pinned ? 'Pinned' : 'Unpinned'
    })
  }

  async function handleToggleImportant() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await toggleImportantMutation.mutateAsync(messageId)
      return result.important ? 'Marked important' : 'Unmarked important'
    })
  }

  async function handleMute() {
    await runSelectedMessageAction(async (messageId) => {
      await toggleMuteMutation.mutateAsync(messageId)
      return 'Muted'
    })
  }

  async function handleExportMessage(format: MessageExportFormat) {
    await runSelectedMessageAction(async (messageId) => {
      const exported = await exportMessageMutation.mutateAsync({ messageId, format })
      store.setLastMessageExport(exported)
      return `Exported ${exported.filename}`
    })
  }

  async function handleMarkMessageRead() {
    await runSelectedMessageAction(async (messageId) => {
      await markMessageReadMutation.mutateAsync(messageId)
      await deps.refetchMessageDetail()
      return 'Marked as read'
    })
  }

  async function handleMarkMessageUnread() {
    await runSelectedMessageAction(async (messageId) => {
      await markMessageUnreadMutation.mutateAsync(messageId)
      await deps.refetchMessageDetail()
      return 'Marked as unread'
    })
  }

  async function handleMarkMessageSpam() {
    await runSelectedMessageAction(async (messageId) => {
      await markMessageSpamMutation.mutateAsync(messageId)
      await deps.refetchMessageDetail()
      return 'Marked as spam'
    })
  }

  async function handleDeleteFromProvider() {
    await runSelectedMessageAction(async (messageId) => {
      await deleteMessageFromProviderMutation.mutateAsync(messageId)
      await deps.refetchMessageDetail()
      return 'Deleted in provider mode'
    })
  }

  async function handleAddLabel(label: string) {
    await runSelectedMessageAction(async (messageId) => {
      await addLabelMutation.mutateAsync({ messageId, label })
      await deps.refetchMessageDetail()
      return `Added label ${label}`
    })
  }

  async function handleRemoveLabel(label: string) {
    await runSelectedMessageAction(async (messageId) => {
      await removeLabelMutation.mutateAsync({ messageId, label })
      await deps.refetchMessageDetail()
      return `Removed label ${label}`
    })
  }

  async function handleSnoozeMessage(until: string) {
    await runSelectedMessageAction(async (messageId) => {
      await snoozeMessageMutation.mutateAsync({ messageId, until })
      await deps.refetchMessageDetail()
      return 'Snoozed'
    })
  }

  async function handleAnalyze() {
    await runSelectedMessageAction(async (messageId) => {
      await analyzeMessageMutation.mutateAsync(messageId)
      await deps.refetchMessageDetail()
      return 'Analyzed'
    })
  }

  async function handleTranslate() {
    await runSelectedMessageAction(async (messageId) => {
      const targetLanguage = locale.value
      const notificationKey = mailActionNotificationKey('translation', messageId)
      notifications.info('Translation started', `Target: ${targetLanguage}`, messageId, notificationKey)
      const result = await translateMessageMutation.mutateAsync({
        messageId,
        targetLanguage
      })
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        translation: result
      })
      return translationActionOutcome(result, targetLanguage)
    }, 'translation')
  }

  async function handleGenerateAiReply(replyOptions: { tone: string; language: string }) {
    await runSelectedMessageAction(async (messageId) => {
      const result = await generateAiReplyMutation.mutateAsync({
        messageId,
        tone: replyOptions.tone,
        language: replyOptions.language
      })
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        aiReply: result
      })
      return result.generated === false ? result.reason || 'AI reply not generated' : 'AI reply generated'
    })
  }

  function handleApplyAiReply(response: AiReplyResponse) {
    const detail = deps.getMessageDetail()
    if (!detail || !response.body) return
    store.openCompose({
      mode: 'reply',
      draftId: `draft-${Date.now()}`,
      accountId: detail.account_id || store.selectedMailAccountId || '',
      toText: detail.sender,
      ccText: '',
      bccText: '',
      subject: response.subject || (detail.subject.startsWith('Re:') ? detail.subject : `Re: ${detail.subject}`),
      body: response.body,
      bodyHtml: null,
      bodyFormat: 'plain',
      scheduledSendAt: '',
      undoSendSeconds: null,
      inReplyTo: detail.provider_record_id || null
    })
  }

  async function handleReviewSecurity() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await reviewSecurityMutation.mutateAsync(messageId)
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        auth: result.auth,
        signature: result.signature
      })
      return result.auth.risk.risk_summary
    })
  }

  async function handleReviewRecipients() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await reviewRecipientsMutation.mutateAsync(messageId)
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        smartCc: result
      })
      return `${result.suggestions.length} recipient suggestions`
    })
  }

  async function handleCreateTask() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await extractMessageTasksMutation.mutateAsync(messageId)
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        tasks: result.tasks
      })
      return `Extracted ${result.tasks.length} tasks`
    })
  }

  async function handleCreateNote() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await extractMessageNotesMutation.mutateAsync(messageId)
      store.setCommunicationMessageInsight({
        ...(store.mailMessageInsight ?? emptyCommunicationMessageInsight(messageId)),
        notes: result.notes
      })
      return `Extracted ${result.notes.length} notes`
    })
  }

  return {
    handleAddLabel,
    handleDeleteFromProvider,
    handleAnalyze,
    handleApplyAiReply,
    handleBilingualReplySend,
    handleCreateNote,
    handleCreateTask,
    handleExportMessage,
    handleMarkMessageRead,
    handleMarkMessageSpam,
    handleMarkMessageUnread,
    handleForwardMessage,
    handleGenerateAiReply,
    handleMute,
    handleNewMessage,
    handleRedirectMessage,
    handleRemoveLabel,
    handleReply,
    handleReplyAll,
    handleReviewRecipients,
    handleReviewSecurity,
    handleSnoozeMessage,
    handleToggleImportant,
    handleTogglePin,
    handleTranslate
  }

  function notifySelectedMessageAction(
    outcome: SelectedMessageActionOutcome,
    messageId: string,
    notificationKey: string
  ): void {
    const title = outcome.title ?? 'Mail action completed'
    if (outcome.tone === 'warning') {
      notifications.warning(title, outcome.message, messageId, notificationKey)
      return
    }
    if (outcome.tone === 'info') {
      notifications.info(title, outcome.message, messageId, notificationKey)
      return
    }

    notifications.success(title, outcome.message, messageId, notificationKey)
  }
}

function mailActionNotificationKey(actionKey: string, messageId: string): string {
  return `mail:${actionKey}:${messageId}`
}

function selectedMessageActionErrorMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : 'Message action failed'
  if (message.includes('Failed to fetch')) {
    return 'Backend API is unavailable. Check system health and retry.'
  }

  return message
}

function selectedMessageActionOutcome(
  result: SelectedMessageActionResult
): SelectedMessageActionOutcome {
  if (typeof result === 'string') {
    return { message: result, tone: 'success' }
  }

  return { tone: 'success', ...result }
}

function translationActionOutcome(
  result: TranslationResponse,
  targetLanguage: string
): SelectedMessageActionOutcome {
  if (result.translated && result.text?.trim()) {
    return {
      title: 'Translation ready',
      message: `Translated to ${result.target ?? targetLanguage}`,
      tone: 'success',
    }
  }

  if (result.translated) {
    return {
      title: 'Translation ready',
      message: `Backend returned no translated text for ${result.target ?? targetLanguage}`,
      tone: 'warning',
    }
  }

  return {
    title: 'Translation unavailable',
    message: result.reason ?? 'Backend did not return a translation',
    tone: 'warning',
  }
}
