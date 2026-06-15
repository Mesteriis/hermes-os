import {
  useAddMessageLabelMutation,
  useAnalyzeMessageMutation,
  useExportMessageMutation,
  useDeleteMessageFromProviderMutation,
  useMarkMessageReadMutation,
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
import { splitComposeRecipients } from '../forms/composeValidation'
import {
  emptyMailMessageInsight,
  forwardComposeForm,
  newComposeForm,
  replyAllComposeForm,
  replyComposeForm
} from '../helpers/mailPageModels'
import type { useCommunicationsStore } from '../stores/communications'
import type {
  AiReplyResponse,
  MailMessageDetailItem,
  MessageExportFormat
} from '../types/communications'
import type { BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'

type CommunicationsStore = ReturnType<typeof useCommunicationsStore>
type RefetchHandler = () => Promise<unknown>

type SelectedMessageActionOptions = {
  getMessageDetail: () => MailMessageDetailItem | null
  refetchMessageDetail: RefetchHandler
}

export function useSelectedMessageActions(
  store: CommunicationsStore,
  deps: SelectedMessageActionOptions
) {
  const togglePinMutation = useToggleMessagePinMutation()
  const toggleImportantMutation = useToggleMessageImportantMutation()
  const toggleMuteMutation = useToggleMessageMuteMutation()
  const exportMessageMutation = useExportMessageMutation()
  const markMessageReadMutation = useMarkMessageReadMutation()
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

  async function runSelectedMessageAction(action: (messageId: string) => Promise<string>) {
    const messageId = store.selectedCommunicationMessageId
    if (!messageId) return
    store.setIsMailActionRunning(true)
    store.setLastMessageExport(null)
    try {
      store.setMailActionStatus(await action(messageId))
    } catch (e) {
      store.setMailActionError(e instanceof Error ? e.message : 'Message action failed')
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
      const result = await translateMessageMutation.mutateAsync({ messageId, targetLanguage: 'en' })
      store.setMailMessageInsight({
        ...(store.mailMessageInsight ?? emptyMailMessageInsight(messageId)),
        translation: result
      })
      return 'Translated'
    })
  }

  async function handleGenerateAiReply(replyOptions: { tone: string; language: string }) {
    await runSelectedMessageAction(async (messageId) => {
      const result = await generateAiReplyMutation.mutateAsync({
        messageId,
        tone: replyOptions.tone,
        language: replyOptions.language
      })
      store.setMailMessageInsight({
        ...(store.mailMessageInsight ?? emptyMailMessageInsight(messageId)),
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
      store.setMailMessageInsight({
        ...(store.mailMessageInsight ?? emptyMailMessageInsight(messageId)),
        auth: result.auth,
        signature: result.signature
      })
      return result.auth.risk.risk_summary
    })
  }

  async function handleReviewRecipients() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await reviewRecipientsMutation.mutateAsync(messageId)
      store.setMailMessageInsight({
        ...(store.mailMessageInsight ?? emptyMailMessageInsight(messageId)),
        smartCc: result
      })
      return `${result.suggestions.length} recipient suggestions`
    })
  }

  async function handleCreateTask() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await extractMessageTasksMutation.mutateAsync(messageId)
      store.setMailMessageInsight({
        ...(store.mailMessageInsight ?? emptyMailMessageInsight(messageId)),
        tasks: result.tasks
      })
      return `Extracted ${result.tasks.length} tasks`
    })
  }

  async function handleCreateNote() {
    await runSelectedMessageAction(async (messageId) => {
      const result = await extractMessageNotesMutation.mutateAsync(messageId)
      store.setMailMessageInsight({
        ...(store.mailMessageInsight ?? emptyMailMessageInsight(messageId)),
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
}
