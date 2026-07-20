// Historical pre-clean-room Mail compose orchestration. It is not part of the active client graph.
import { useCommunicationAttachmentImportMutation, useSaveDraftMutation, useSendMailMutation } from '../queries/useCommunicationsQuery'
import { useCommunicationActionNotifications } from '../queries/communicationActionNotifications'
import { buildComposeDraftPayload } from '../forms/composeDraftAutosave'
import {
  composeAttachmentSendError,
  composeFileContentBase64,
  failedComposeAttachment,
  importedComposeAttachment,
  pendingComposeAttachment,
} from '../forms/composeAttachmentUpload'
import { composeFormToSendRequest, draftToComposeForm } from '../helpers/communicationPageModels'
import { useCommunicationsStore } from '../stores/communications'
import type { CommunicationAccountOption, CommunicationDraft, ComposeFormModel } from '../types/communications'

type CommunicationsStore = ReturnType<typeof useCommunicationsStore>

type MailComposeActionsDependencies = {
  getDefaultMailAccountId: () => string
  getMailComposeAccountOptions: () => CommunicationAccountOption[]
  getSendCapableMailComposeAccountOptions: () => CommunicationAccountOption[]
  refetchDrafts: () => Promise<unknown>
  refetchMailList: () => Promise<unknown>
}

export function useMailComposeActions(
  store: CommunicationsStore,
  {
    getDefaultMailAccountId,
    getMailComposeAccountOptions,
    getSendCapableMailComposeAccountOptions,
    refetchDrafts,
    refetchMailList,
  }: MailComposeActionsDependencies
) {
  const notifications = useCommunicationActionNotifications()
  const attachmentImportMutation = useCommunicationAttachmentImportMutation()
  const saveDraftMutation = useSaveDraftMutation()
  const sendMailMutation = useSendMailMutation()

  function handleOpenDraft(draft: CommunicationDraft) {
    store.openCompose(draftToComposeForm(draft))
  }

  async function handleSaveComposeDraft() {
    store.setComposeStatusMessage('')
    store.setComposeSendError('')
    const form = composeFormWithAvailableMailAccount()
    const accountError = composeAccountValidationError(form)
    if (accountError) {
      store.setComposeSendError(accountError)
      notifications.error('Draft save failed', accountError)
      return
    }
    if (form.accountId !== store.composeForm.accountId) {
      store.updateComposeForm({ accountId: form.accountId })
    }
    store.setIsSendingMessage(true)
    try {
      const draft = await saveDraftMutation.mutateAsync(buildComposeDraftPayload(form))
      store.openCompose({ ...draftToComposeForm(draft), attachments: form.attachments })
      store.setComposeStatusMessage('Draft saved')
      notifications.success('Draft saved')
      await refetchDrafts()
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Save draft failed'
      store.setComposeSendError(message)
      notifications.error('Draft save failed', message)
    } finally {
      store.setIsSendingMessage(false)
    }
  }

  async function handleSendCompose() {
    store.setComposeStatusMessage('')
    store.setComposeSendError('')
    const form = composeFormWithAvailableMailAccount()
    const accountError = composeAccountValidationError(form)
    if (accountError) {
      store.setComposeSendError(accountError)
      notifications.error('Send failed', accountError)
      return
    }
    const attachmentError = composeAttachmentSendError(form.attachments)
    if (attachmentError) {
      store.setComposeSendError(attachmentError)
      notifications.error('Send blocked', attachmentError)
      return
    }
    if (form.accountId !== store.composeForm.accountId) {
      store.updateComposeForm({ accountId: form.accountId })
    }
    store.setIsSendingMessage(true)
    try {
      if (form.attachments.length > 0) {
        await saveDraftMutation.mutateAsync(buildComposeDraftPayload(form))
      }
      const result = await sendMailMutation.mutateAsync(composeFormToSendRequest(form))
      store.closeCompose()
      const status = result.status === 'sent' ? `Sent via ${result.transport}` : `Message ${result.status}`
      store.setMailActionStatus(status)
      notifications.success('Message queued', status)
      await Promise.all([refetchMailList(), refetchDrafts()])
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Send failed'
      store.setComposeSendError(message)
      notifications.error('Send failed', message)
    } finally {
      store.setIsSendingMessage(false)
    }
  }

  async function handleComposeFiles(files: File[]) {
    const form = composeFormWithAvailableMailAccount()
    const accountError = composeAccountValidationError(form)
    if (accountError) {
      store.setComposeSendError(accountError)
      notifications.error('Attachment upload failed', accountError)
      return
    }
    for (const file of files) {
      const temporaryId = `pending:${crypto.randomUUID()}`
      const pending = pendingComposeAttachment(file, temporaryId)
      store.updateComposeForm({ attachments: [...store.composeForm.attachments, pending] })
      try {
        const imported = await attachmentImportMutation.mutateAsync({
          account_id: form.accountId,
          channel_kind: 'mail',
          filename: pending.filename,
          content_type: pending.contentType,
          content_base64: await composeFileContentBase64(file),
          source_kind: 'mail_compose',
          metadata: { draft_id: form.draftId },
        })
        replaceComposeAttachment(temporaryId, importedComposeAttachment(imported))
      } catch (error) {
        replaceComposeAttachment(temporaryId, failedComposeAttachment(pending, error))
      }
    }
  }

  function handleRemoveComposeAttachment(attachmentId: string) {
    store.updateComposeForm({
      attachments: store.composeForm.attachments.filter(
        (attachment) => attachment.attachmentId !== attachmentId
      ),
    })
  }

  function replaceComposeAttachment(attachmentId: string, replacement: ComposeFormModel['attachments'][number]) {
    store.updateComposeForm({
      attachments: store.composeForm.attachments.map((attachment) =>
        attachment.attachmentId === attachmentId ? replacement : attachment
      ),
    })
  }

  function composeFormWithAvailableMailAccount(): ComposeFormModel {
    const form = store.composeForm
    const accountId = form.accountId.trim()
    const sendAccountOptions = getSendCapableMailComposeAccountOptions()
    const currentAccountCanSend = Boolean(accountId) && (
      sendAccountOptions.some((option) => option.account_id === accountId)
    )
    if (currentAccountCanSend) return form

    const fallbackAccountId = getDefaultMailAccountId()
    return fallbackAccountId ? { ...form, accountId: fallbackAccountId } : form
  }

  function composeAccountValidationError(form: ComposeFormModel): string {
    const accountId = form.accountId.trim()
    if (!accountId) return 'Select a sender account'
    const selectedAccount = getMailComposeAccountOptions().find((option) => option.account_id === accountId)
    if (selectedAccount && !selectedAccount.can_send) {
      return selectedAccount.send_unavailable_reason
    }
    return ''
  }

  return {
    handleComposeFiles,
    handleOpenDraft,
    handleRemoveComposeAttachment,
    handleSaveComposeDraft,
    handleSendCompose,
  }
}
